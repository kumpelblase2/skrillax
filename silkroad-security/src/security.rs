use blowfish_compat::{Block, BlockDecrypt, BlockEncrypt, BlowfishCompat, NewBlockCipher, BLOCK_SIZE};
use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, Bytes};
use rand::random;
use thiserror::Error;
use tracing::{span, Level};

#[derive(Error, Debug)]
pub enum SilkroadSecurityError {
    /// The handshake hasn't been started or hasn't completed, yet the operation required it.
    #[error("Security has not been initialized")]
    SecurityUninitialized,
    /// The handshake has already completed. The security would need to be reset before continuing.
    #[error("Security is already initialized")]
    AlreadyInitialized,
    /// Finalizing the handshake requires the handshake to have exchanged public key data, which hasn't happened yet.
    #[error("Security has not completed the initialization")]
    InitializationUnfinished,
    /// The given encrypted data is not the correct block length, as required for decryption.
    #[error("{0} is an invalid block length")]
    InvalidBlockLength(usize),
    /// We calculated a different secret than the client, something went wrong in the handshake.
    #[error("Local calculated key was {calculated} but received {received}")]
    KeyExchangeMismatch { received: u64, calculated: u64 },
}

/// Initialization data for the handshake. These are transmitted to the client side for a Diffie-Hellman style key
/// exchange.
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

/// [SilkroadSecurity] handles the handshake and continuous encryption/decryption of a connection to a silkroad client.
///
///  A general
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
    /// Starts the handshake process. This generates the private key parts and returns [InitializationData], which
    /// should be transferred to the client. This should later be followed by calling [start_challenge()][Self::start_challenge()]
    /// with the client response.
    ///
    /// If a handshake has already been started or completed, will return [SilkroadSecurityError::AlreadyInitialized].
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
        let value_a = g_pow_x_mod_p(value_p.into(), value_x, value_g);

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

    /// Initialize the security with a predefined set of values.
    /// These are the same values that would be generated randomly in [initialize()].
    /// This effectively does the initialization, just with the predefined values,
    /// resulting in a deterministic handshake.
    pub fn initialize_with(&mut self, count_seed: u32, crc_seed: u32, handshake_seed: u64, x: u32, p: u32, a: u32) {
        self.state = SecurityState::HandshakeStarted {
            count_seed,
            crc_seed,
            handshake_seed,
            value_x: x,
            value_a: a,
            value_p: p,
        }
    }

    /// Resets the security to a fresh state. Is equivalent to creating a new object using [default()][Self::default()]
    pub fn restart(&mut self) {
        self.state = SecurityState::Uninitialized;
    }

    /// Create a challenge to the client.
    ///
    /// This creates a challenge for the client, signaling a switch to an encrypted channel using the exchanged key
    /// material. We also check if the key, that the client sent us, matches was we would expect given what we've
    /// witnessed in the key exchange.
    ///
    /// If successful, returns the challenge for the client. If [initialize][Self::initialize()] hasn't been called,
    /// returns [SilkroadSecurityError::SecurityUninitialized]. If the passed key does not match the key we expect,
    /// will return [SilkroadSecurityError::KeyExchangeMismatch].
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
                let value_k = g_pow_x_mod_p(value_p.into(), value_x, value_b);
                let new_key = to_u64(value_a, value_b);
                let new_key = transform_key(new_key, value_k, LOBYTE(LOWORD(value_k)) & 0x03);
                let blowfish = blowfish_from_int(new_key);

                let mut key_bytes: [u8; 8] = client_key.to_le_bytes();
                blowfish.decrypt_block(Block::from_mut_slice(&mut key_bytes));

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
                blowfish.encrypt_block(Block::from_mut_slice(&mut key_bytes));
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

    /// Finish the handshake.
    ///
    /// Client has confirmed the challenge and the handshake is complete. This will initialize the `count_seed` with the
    /// values generated in the handshake. After this is completed, encryption/decryption is possible.
    ///
    /// Will return [SilkroadSecurityError::InitializationUnfinished] if [start_challenge][Self::start_challenge()]
    /// hasn't been successfully executed.
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

    /// Decrypt an encrypted message sent by the client.
    ///
    /// Decrypts the given input by splitting it into the individual encrypted blocks. The output is all decrypted data,
    /// which may contain padding that was added before encryption. Bytes are copied before performing decryption.
    /// To decrypt in place, use [decrypt_mut][Self::decrypt_mut()]
    ///
    /// If handshake hasn't been completed yet, will result in [SilkroadSecurityError::SecurityUninitialized].
    /// If the input doesn't match the required block length it will return [SilkroadSecurityError::InvalidBlockLength].
    pub fn decrypt(&self, data: &[u8]) -> Result<Bytes, SilkroadSecurityError> {
        let mut result = bytes::BytesMut::from(data);
        self.decrypt_mut(&mut result)?;
        Ok(result.freeze())
    }

    /// Decrypt an encrypted message sent by the client.
    ///
    /// Decrypts the given input by splitting it into the individual encrypted blocks in place. The decrypted data may
    /// still be padded to match block length (8 bytes).
    ///
    /// If handshake hasn't been completed yet, will result in [SilkroadSecurityError::SecurityUninitialized].
    /// If the input doesn't match the required block length it will return [SilkroadSecurityError::InvalidBlockLength].
    pub fn decrypt_mut(&self, data: &mut [u8]) -> Result<(), SilkroadSecurityError> {
        match &self.state {
            SecurityState::Established {
                blowfish,
                crc_seed: _,
                count_seed: _,
            } => {
                if data.len() % BLOWFISH_BLOCK_SIZE != 0 {
                    return Err(SilkroadSecurityError::InvalidBlockLength(data.len()));
                }

                let span = span!(Level::TRACE, "security decryption");
                let _enter = span.enter();
                for chunk in data.chunks_mut(BLOWFISH_BLOCK_SIZE) {
                    let block = Block::from_mut_slice(chunk);
                    blowfish.decrypt_block(block);
                }
                Ok(())
            },
            _ => Err(SilkroadSecurityError::SecurityUninitialized),
        }
    }

    /// Encrypt a message to be sent to the client.
    ///
    /// Encrypts the given bytes using the previously established secret. Requires that the handshake has been completed.
    /// It will copy the bytes and return the encrypted bytes as an owned reference. Bytes will be padded automatically
    /// to the necessary block length. Use [encrypt_mut][Self::encrypt_mut()] for encryption in place.
    ///
    /// If handshake hasn't been completed yet, will result in [SilkroadSecurityError::SecurityUninitialized].
    pub fn encrypt(&self, data: &[u8]) -> Result<Bytes, SilkroadSecurityError> {
        if !matches!(&self.state, SecurityState::Established { .. }) {
            return Err(SilkroadSecurityError::SecurityUninitialized);
        }

        let target_buffer_size = Self::find_encrypted_length(data.len());
        let mut result = bytes::BytesMut::with_capacity(target_buffer_size);
        result.extend_from_slice(data);
        for _ in 0..(target_buffer_size - data.len()) {
            result.put_u8(0);
        }
        self.encrypt_mut(&mut result)?;
        Ok(result.freeze())
    }

    /// Encrypt a message to be sent to the client.
    ///
    /// Encrypts the given bytes using the previously established secret. Requires that the handshake has been completed
    /// and that `data` is a multiple of the block length.
    ///
    /// If handshake hasn't been completed yet, will result in [SilkroadSecurityError::SecurityUninitialized].
    /// If the data is not block-aligned, will result in [SilkroadSecurityError::InvalidBlockLength]
    pub fn encrypt_mut(&self, data: &mut [u8]) -> Result<(), SilkroadSecurityError> {
        match &self.state {
            SecurityState::Established {
                blowfish,
                crc_seed: _,
                count_seed: _,
            } => {
                if data.len() % BLOCK_SIZE != 0 {
                    return Err(SilkroadSecurityError::InvalidBlockLength(data.len()));
                }

                let span = span!(Level::TRACE, "security encryption");
                let _enter = span.enter();
                for chunk in data.chunks_mut(BLOWFISH_BLOCK_SIZE) {
                    let block = Block::from_mut_slice(chunk);
                    blowfish.encrypt_block(block);
                }
                Ok(())
            },
            _ => Err(SilkroadSecurityError::SecurityUninitialized),
        }
    }

    /// Find the nearest block-aligned length.
    ///
    /// Given the current length of data to encrypt, calculates the length of the encrypted output, which includes
    /// padding. Can at most increase by `BLOWFISH_BLOCK_SIZE - 1`, which is `7`.
    pub fn find_encrypted_length(given_length: usize) -> usize {
        let aligned_length = given_length % BLOWFISH_BLOCK_SIZE;
        if aligned_length == 0 {
            // Already block-aligned, no need to pad
            return given_length;
        }

        given_length + (8 - aligned_length) // Add padding
    }

    /// Generate the next count byte.
    ///
    /// A count byte is used to avoid replay attacks, used to determine a continuous flow of the data. If a packet is
    /// dropped, or another injected, this will no longer match. It is essentially a seeded RNG number.
    ///
    /// If handshake hasn't been completed yet, will result in [SilkroadSecurityError::SecurityUninitialized].
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
    use super::*;
    use byteorder::{ByteOrder, LittleEndian};

    #[test]
    fn finishes_encoding() {
        let handshake_seed = LittleEndian::read_u64(&[0xbf, 0x89, 0x96, 0x76, 0xae, 0x97, 0x5e, 0x17]);
        let _value_g = LittleEndian::read_u32(&[0x95, 0x0b, 0xf5, 0x20]);
        let value_p = LittleEndian::read_u32(&[0x0d, 0xf4, 0x13, 0x52]);
        let value_x = 189993144; // brute forced
        let value_a = LittleEndian::read_u32(&[0x36, 0x44, 0x96, 0x24]);

        let mut security = SilkroadSecurity::default();
        security.initialize_with(0, 0, handshake_seed, value_x, value_p, value_a);

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
        let mut security = SilkroadSecurity::default();
        assert!(matches!(
            security.encrypt(&[]),
            Err(SilkroadSecurityError::SecurityUninitialized)
        ));
        assert!(matches!(
            security.decrypt(&[]),
            Err(SilkroadSecurityError::SecurityUninitialized)
        ));

        assert!(matches!(
            security.accept_challenge(),
            Err(SilkroadSecurityError::InitializationUnfinished)
        ));

        assert!(matches!(
            security.start_challenge(0, 0),
            Err(SilkroadSecurityError::SecurityUninitialized)
        ));
    }
}
