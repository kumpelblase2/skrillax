use blowfish_compat::cipher::generic_array::GenericArray;
use blowfish_compat::cipher::{BlockDecrypt, NewBlockCipher};
use blowfish_compat::BlowfishCompat;
use bytes::BytesMut;
use std::string::FromUtf8Error;

pub struct PassCodeDecoder {
    blowfish: BlowfishCompat,
}

impl PassCodeDecoder {
    pub fn decode_passcode(&self, passcode_length: u16, encrypted: &[u8]) -> Result<String, FromUtf8Error> {
        let mut data = BytesMut::from(encrypted);
        self.blowfish.decrypt_block(GenericArray::from_mut_slice(&mut data));
        String::from_utf8(data[0..(passcode_length as usize)].to_vec())
    }
}

impl Default for PassCodeDecoder {
    fn default() -> Self {
        let blowfish_key: [u8; 8] = [0x0f, 0x07, 0x3d, 0x20, 0x56, 0x62, 0xc9, 0xeb];
        let blowfish = BlowfishCompat::new_from_slice(&blowfish_key).expect("Could not create blowfish key");
        PassCodeDecoder { blowfish }
    }
}
