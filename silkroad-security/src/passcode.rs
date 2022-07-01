use blowfish_compat::{Block, BlockDecrypt, BlowfishCompat, NewBlockCipher};
use bytes::BytesMut;
use once_cell::sync::Lazy;
use std::string::FromUtf8Error;

static INSTANCE: Lazy<PassCodeDecoder> = Lazy::new(|| PassCodeDecoder::default());

/// Decode/Decrypt the password input received by a server from a client when they are logging in. This is a
/// convenience wrapper around [BlowfishCompat] which automatically uses the correct key. One can use the instance
/// provided through [get()][PassCodeDecoder::get()] as there's no state to be kept. Use [decode_passcode][Self::decode_passcode()]
/// to decrypt passcodes.
///
/// ```
/// # use silkroad_security::passcode::PassCodeDecoder;
/// let input = [113, 42, 1, 64, 127, 104, 60, 94];
/// let decoder = PassCodeDecoder::get();
/// let decoded = decoder.decode_passcode(4, &input)?;
/// assert_eq!(decoded, "1234");
/// ```
pub struct PassCodeDecoder {
    blowfish: BlowfishCompat,
}

impl PassCodeDecoder {
    pub fn decode_passcode(&self, passcode_length: u16, encrypted: &[u8]) -> Result<String, FromUtf8Error> {
        let mut data = BytesMut::from(encrypted);
        self.blowfish.decrypt_block(Block::from_mut_slice(&mut data));
        String::from_utf8(data[0..(passcode_length as usize)].to_vec())
    }

    /// Returns a globally shared instance.
    pub fn get() -> &Self {
        &INSTANCE
    }
}

impl Default for PassCodeDecoder {
    fn default() -> Self {
        let blowfish_key: [u8; 8] = [0x0f, 0x07, 0x3d, 0x20, 0x56, 0x62, 0xc9, 0xeb];
        let blowfish = BlowfishCompat::new_from_slice(&blowfish_key).expect("Could not create blowfish key");
        PassCodeDecoder { blowfish }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple_decrypt() -> Result<(), FromUtf8Error> {
        let input = [113, 42, 1, 64, 127, 104, 60, 94];
        let decoder = PassCodeDecoder::get();
        let decoded = decoder.decode_passcode(4, &input)?;
        assert_eq!(decoded, "1234");
        Ok(())
    }
}
