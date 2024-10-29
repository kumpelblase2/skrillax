use blowfish::cipher::{Block, BlockDecrypt, KeyInit};
use blowfish::BlowfishLE;
use bytes::BytesMut;
use once_cell::sync::Lazy;
use std::string::FromUtf8Error;

static INSTANCE: Lazy<PasscodeDecoder> = Lazy::new(PasscodeDecoder::default);

type BlockLE = Block<BlowfishLE>;

/// Decode/Decrypt the password input received by a server from a client when they are logging in. This is a
/// convenience wrapper around [BlowfishCompat] which automatically uses the correct key. One can use the instance
/// provided through [get()][PasscodeDecoder::get()] as there's no state to be kept. Use [decode_passcode][Self::decode_passcode()]
/// to decrypt passcodes.
///
/// ```
/// # use std::string::FromUtf8Error;
/// # use silkroad_security::passcode::PasscodeDecoder;
/// # fn test() -> Result<(), FromUtf8Error> {
/// let input = [113, 42, 1, 64, 127, 104, 60, 94];
/// let decoder = PasscodeDecoder::get();
/// let decoded = decoder.decode_passcode(4, &input)?;
/// assert_eq!(decoded, "1234");
/// # Ok(())
/// # }
/// # fn main() {
/// #     test().unwrap();
/// # }
/// ```
pub struct PasscodeDecoder {
    blowfish: BlowfishLE,
}

impl PasscodeDecoder {
    /// Decodes/Decrypts a passed in passcode.
    ///
    /// Decrypts a given passcode and returns it as a string. Slice length must be exactly one block, which is 8.
    /// Returns a string that is `passcode_length` characters long containing the passcode.
    ///
    /// This can result in an error if the input could not be properly decoded into a string. This can happen if the
    /// input was not an encrypted block, it was altered, or encrypted using a different key.
    pub fn decode_passcode(&self, passcode_length: u16, encrypted: &[u8]) -> Result<String, FromUtf8Error> {
        let mut data = BytesMut::from(encrypted);
        self.blowfish.decrypt_block(BlockLE::from_mut_slice(&mut data));
        String::from_utf8(data[0..(passcode_length as usize)].to_vec())
    }

    /// Returns a globally shared instance.
    pub fn get() -> &'static Self {
        &INSTANCE
    }
}

impl Default for PasscodeDecoder {
    fn default() -> Self {
        let blowfish_key: [u8; 8] = [0x0f, 0x07, 0x3d, 0x20, 0x56, 0x62, 0xc9, 0xeb];
        let blowfish = BlowfishLE::new_from_slice(&blowfish_key).expect("Could not create blowfish key");
        PasscodeDecoder { blowfish }
    }
}
