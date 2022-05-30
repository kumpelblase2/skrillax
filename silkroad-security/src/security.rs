use blowfish_compat::cipher::generic_array::GenericArray;
use blowfish_compat::cipher::{BlockDecrypt, BlockEncrypt, NewBlockCipher};
use blowfish_compat::BlowfishCompat;
use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, Bytes};
use rand::random;
use thiserror::Error;
use tracing::{span, Level};

#[derive(Error, Debug)]
pub enum SilkroadSecurityError {
    #[error("Security has not been initialized")]
    SecurityUninitialized,
    #[error("Security is already initialized")]
    AlreadyInitialized,
    #[error("Security has not completed the initialization")]
    InitializationUnfinished,
    #[error("{0} is an invalid block length")]
    InvalidBlockLength(usize),
    #[error("Local calculated key was {calculated} but received {received}")]
    KeyExchangeMismatch { received: u64, calculated: u64 },
}

pub struct InitializationData {
    pub seed: u64,
    pub count_seed: u32,
    pub crc_seed: u32,
    pub handshake_seed: u64,
    pub additional_seeds: [u32; 3],
}

enum SecurityState {
    Uninitialized,
    HandshakeStarted {
        count_seed: u32,
        crc_seed: u32,
        handshake_seed: u64,
        value_x: u32,
        value_p: u32,
        value_a: u32,
    },
    Challenged {
        blowfish: BlowfishCompat,
        count_seed: u32,
        crc_seed: u32,
    },
    Established {
        blowfish: BlowfishCompat,
        count_seed: [u8; 3],
        #[allow(unused)]
        crc_seed: u32,
    },
}

const BLOWFISH_BLOCK_SIZE: usize = 8;

pub struct SilkroadSecurity {
    state: SecurityState,
}

impl Default for SilkroadSecurity {
    fn default() -> Self {
        SilkroadSecurity {
            state: SecurityState::Uninitialized,
        }
    }
}

impl SilkroadSecurity {
    pub fn initialize(&mut self) -> Result<InitializationData, SilkroadSecurityError> {
        match self.state {
            SecurityState::Uninitialized => {},
            _ => return Err(SilkroadSecurityError::AlreadyInitialized),
        }

        let span = span!(Level::TRACE, "security initialization");
        let _enter = span.enter();
        let seed = random::<u64>();
        let count_seed = random::<u32>();
        let crc_seed = random::<u32>();
        let handshake_seed = random::<u64>();
        let value_x = random::<u32>() & 0x7FFFFFFF;
        let value_g = random::<u32>() & 0x7FFFFFFF;
        let value_p = random::<u32>() & 0x7FFFFFFF;
        let value_a = g_pow_x_mod_p(value_p as i64, value_x, value_g);

        self.state = SecurityState::HandshakeStarted {
            count_seed,
            crc_seed,
            handshake_seed,
            value_x,
            value_p,
            value_a,
        };

        Ok(InitializationData {
            seed,
            count_seed,
            crc_seed,
            handshake_seed,
            additional_seeds: [value_g, value_p, value_a],
        })
    }

    pub fn initialize_with(&mut self, handshake_seed: u64, x: u32, p: u32, a: u32) {
        self.state = SecurityState::HandshakeStarted {
            count_seed: 0,
            crc_seed: 0,
            handshake_seed,
            value_x: x,
            value_a: a,
            value_p: p,
        }
    }

    pub fn restart(&mut self) {
        self.state = SecurityState::Uninitialized;
    }

    pub fn start_challenge(&mut self, value_b: u32, client_key: u64) -> Result<u64, SilkroadSecurityError> {
        match self.state {
            SecurityState::HandshakeStarted {
                count_seed,
                crc_seed,
                handshake_seed,
                value_x,
                value_p,
                value_a,
            } => {
                let span = span!(Level::TRACE, "security challenge start");
                let _enter = span.enter();
                let value_k = g_pow_x_mod_p(value_p as i64, value_x, value_b);
                let new_key = to_u64(value_a, value_b);
                let new_key = transform_key(new_key, value_k, LOBYTE(LOWORD(value_k)) & 0x03);
                let blowfish = blowfish_from_int(new_key);

                let mut key_bytes: [u8; 8] = client_key.to_le_bytes();
                blowfish.decrypt_block(GenericArray::from_mut_slice(&mut key_bytes));

                let client_key = LittleEndian::read_u64(&key_bytes);
                let new_key = to_u64(value_b, value_a);
                let new_key = transform_key(new_key, value_k, LOBYTE(LOWORD(value_b)) & 0x07);
                if new_key != client_key {
                    return Err(SilkroadSecurityError::KeyExchangeMismatch {
                        received: client_key,
                        calculated: new_key,
                    });
                }

                let new_key = to_u64(value_a, value_b);
                let new_key = transform_key(new_key, value_k, LOBYTE(LOWORD(value_k)) & 0x03);
                let blowfish = blowfish_from_int(new_key);

                let challenge_key = to_u64(value_a, value_b);
                let challenge_key = transform_key(challenge_key, value_k, LOBYTE(LOWORD(value_a)) & 0x07);
                let mut key_bytes: [u8; 8] = challenge_key.to_le_bytes();
                blowfish.encrypt_block(GenericArray::from_mut_slice(&mut key_bytes));
                let encrypted_challenge = LittleEndian::read_u64(&key_bytes);

                let handshake_seed = transform_key(handshake_seed, value_k, 0x03);
                self.state = SecurityState::Challenged {
                    blowfish: blowfish_from_int(handshake_seed),
                    crc_seed,
                    count_seed,
                };

                Ok(encrypted_challenge)
            },
            _ => Err(SilkroadSecurityError::SecurityUninitialized),
        }
    }

    pub fn accept_challenge(&mut self) -> Result<(), SilkroadSecurityError> {
        match self.state {
            SecurityState::Challenged {
                blowfish,
                crc_seed,
                count_seed,
            } => {
                self.state = SecurityState::Established {
                    blowfish,
                    count_seed: Self::generate_count_seed(count_seed),
                    crc_seed,
                };
                Ok(())
            },
            _ => Err(SilkroadSecurityError::InitializationUnfinished),
        }
    }

    fn generate_count_seed(seed: u32) -> [u8; 3] {
        let round1 = Self::cycle_value(seed);
        let round2 = Self::cycle_value(round1);
        let round3 = Self::cycle_value(round2);
        let round4 = Self::cycle_value(round3);
        let mut byte1 = ((round4 & 0xFF) ^ (round3 & 0xFF)) as u8;
        let mut byte2 = ((round1 & 0xFF) ^ (round2 & 0xFF)) as u8;
        if byte1 == 0 {
            byte1 = 1;
        }

        if byte2 == 0 {
            byte2 = 1;
        }

        [(byte1 ^ byte2) as u8, byte2, byte1]
    }

    fn cycle_value(seed: u32) -> u32 {
        let mut val = seed;
        for _ in 0..32 {
            val = (((((((((((val >> 2) ^ val) >> 2) ^ val) >> 1) ^ val) >> 1) ^ val) >> 1) ^ val) & 1)
                | ((((val & 1) << 31) | (val >> 1)) & 0xFFFFFFFE);
        }
        val
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Bytes, SilkroadSecurityError> {
        match &self.state {
            SecurityState::Established {
                blowfish,
                crc_seed: _,
                count_seed: _,
            } => {
                let span = span!(Level::TRACE, "security decryption");
                let _enter = span.enter();
                if data.len() % BLOWFISH_BLOCK_SIZE != 0 {
                    return Err(SilkroadSecurityError::InvalidBlockLength(data.len()));
                }
                let mut result = bytes::BytesMut::from(data);
                for chunk in result.chunks_mut(BLOWFISH_BLOCK_SIZE) {
                    let block = GenericArray::from_mut_slice(chunk);
                    blowfish.decrypt_block(block);
                }
                Ok(result.freeze())
            },
            _ => Err(SilkroadSecurityError::SecurityUninitialized),
        }
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Bytes, SilkroadSecurityError> {
        match &self.state {
            SecurityState::Established {
                blowfish,
                crc_seed: _,
                count_seed: _,
            } => {
                let span = span!(Level::TRACE, "security encryption");
                let _enter = span.enter();
                let target_buffer_size = Self::find_encrypted_length(data.len());
                let mut result = bytes::BytesMut::with_capacity(target_buffer_size);
                result.extend_from_slice(data);
                for _ in 0..(target_buffer_size - data.len()) {
                    result.put_u8(0);
                }
                for chunk in result.chunks_mut(BLOWFISH_BLOCK_SIZE) {
                    let block = GenericArray::from_mut_slice(chunk);
                    blowfish.encrypt_block(block);
                }
                Ok(result.freeze())
            },
            _ => Err(SilkroadSecurityError::SecurityUninitialized),
        }
    }

    pub fn find_encrypted_length(given_length: usize) -> usize {
        let aligned_length = given_length % BLOWFISH_BLOCK_SIZE;
        if aligned_length == 0 {
            // Already block-aligned, no need to pad
            return given_length;
        }

        given_length + (8 - aligned_length) // Add padding
    }

    pub fn generate_count_byte(&mut self) -> Result<u8, SilkroadSecurityError> {
        match &self.state {
            SecurityState::Established { mut count_seed, .. } => {
                let result = count_seed[2] as u32 * (!count_seed[0] as u32 + count_seed[1] as u32) as u32;
                let result = (result ^ (result >> 4)) as u8;
                count_seed[0] = result;
                Ok(result)
            },
            _ => Err(SilkroadSecurityError::SecurityUninitialized),
        }
    }

    // pub fn generate_crc_byte(&self, stream: &[u8], offset: usize, length: usize) -> Result<u8, SilkroadSecurityError> {
    //     match &self.state {
    //         SecurityState::Established { crc_seed, .. } => {
    //             let mut checksum = 0xFFFFFFFF as u32;
    //             let start_seed = *crc_seed << 8;
    //             for i in offset..(offset + length) {
    //                 checksum = (checksum >> 8) ^ global_security_table[moddedseed + (((uint)stream[x] ^ checksum) & 0xFF)];
    //             }
    //
    //             Ok(0)
    //         }
    //         _ => Err(SilkroadSecurityError::SecurityUninitialized)
    //     }
    // }
}

#[allow(non_snake_case)]
fn LOWORD(a: u32) -> u16 {
    (a & 0xFFFF) as u16
}

#[allow(non_snake_case)]
fn HIWORD(a: u32) -> u16 {
    ((a >> 16) & 0xFFFF) as u16
}

#[allow(non_snake_case)]
fn LOBYTE(a: u16) -> u8 {
    (a & 0xFF) as u8
}

#[allow(non_snake_case)]
fn HIBYTE(a: u16) -> u8 {
    ((a >> 8) & 0xFF) as u8
}

fn g_pow_x_mod_p(p: i64, mut x: u32, g: u32) -> u32 {
    let mut current: i64 = 1;
    let mut mult: i64 = g as i64;

    while x != 0 {
        if (x & 1) > 0 {
            current = (mult * current) % p;
        }
        x >>= 1;
        mult = (mult * mult) % p;
    }
    current as u32
}

#[allow(unused_parens)]
fn transform_key(val: u64, key: u32, key_byte: u8) -> u64 {
    let mut stream = val.to_le_bytes();

    stream[0] ^= (stream[0].wrapping_add(LOBYTE(LOWORD(key))).wrapping_add(key_byte));
    stream[1] ^= (stream[1].wrapping_add(HIBYTE(LOWORD(key))).wrapping_add(key_byte));
    stream[2] ^= (stream[2].wrapping_add(LOBYTE(HIWORD(key))).wrapping_add(key_byte));
    stream[3] ^= (stream[3].wrapping_add(HIBYTE(HIWORD(key))).wrapping_add(key_byte));
    stream[4] ^= (stream[4].wrapping_add(LOBYTE(LOWORD(key))).wrapping_add(key_byte));
    stream[5] ^= (stream[5].wrapping_add(HIBYTE(LOWORD(key))).wrapping_add(key_byte));
    stream[6] ^= (stream[6].wrapping_add(LOBYTE(HIWORD(key))).wrapping_add(key_byte));
    stream[7] ^= (stream[7].wrapping_add(HIBYTE(HIWORD(key))).wrapping_add(key_byte));

    LittleEndian::read_u64(&stream)
}

fn blowfish_from_int(key: u64) -> BlowfishCompat {
    BlowfishCompat::new_from_slice(&key.to_le_bytes()).expect("Could not create blowfish key")
}

fn to_u64(low: u32, high: u32) -> u64 {
    ((high as u64) << 32) | low as u64
}

#[cfg(test)]
mod tests {
    use byteorder::{ByteOrder, LittleEndian};

    use crate::security::{SilkroadSecurity, SilkroadSecurityError};

    #[test]
    fn finishes_encoding() {
        let handshake_seed = LittleEndian::read_u64(&[0xbf, 0x89, 0x96, 0x76, 0xae, 0x97, 0x5e, 0x17]);
        let _value_g = LittleEndian::read_u32(&[0x95, 0x0b, 0xf5, 0x20]);
        let value_p = LittleEndian::read_u32(&[0x0d, 0xf4, 0x13, 0x52]);
        let value_x = 189993144; // brute forced
        let value_a = LittleEndian::read_u32(&[0x36, 0x44, 0x96, 0x24]);

        let mut security = SilkroadSecurity::default();
        security.initialize_with(handshake_seed, value_x, value_p, value_a);

        let value_b = LittleEndian::read_u32(&[0x7a, 0x04, 0x39, 0x43]);
        let key = LittleEndian::read_u64(&[0x69, 0x02, 0xec, 0x3f, 0x16, 0xbb, 0x18, 0x64]);

        let result = security.start_challenge(value_b, key).unwrap();

        let result_expected_bytes = &[0xbe, 0x6f, 0x5e, 0xd4, 0x19, 0x79, 0x7d, 0x26];
        let result_expected = LittleEndian::read_u64(result_expected_bytes);

        assert_eq!(result, result_expected);
        assert!(security.accept_challenge().is_ok());
    }

    #[test]
    fn cannot_encrypt_uninitialized() {
        let security = SilkroadSecurity::default();
        assert!(matches!(
            security.encrypt(&[]),
            Err(SilkroadSecurityError::SecurityUninitialized)
        ));
        assert!(matches!(
            security.decrypt(&[]),
            Err(SilkroadSecurityError::SecurityUninitialized)
        ));
    }
}
