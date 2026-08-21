use color_eyre::eyre::{Result, bail, eyre};
use skrillax_packet::{IncomingPacket, SecurityBytes, SecurityContext};
use skrillax_security::SilkroadEncryption;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use tracing::{debug, info};

const SECURITY_CAPABILITY_OPCODE: u16 = 0x5000;
const FLAG_NONE: u8 = 0x01;
const FLAG_INIT_BLOWFISH: u8 = 0x02;
const FLAG_SETUP_CHECKS: u8 = 0x04;
const FLAG_START_HANDSHAKE: u8 = 0x08;
const FLAG_FINISH: u8 = 0x10;
const MAX_PRIVATE_KEY: u32 = 0x7fff_ffff;

#[derive(Clone, Copy, Debug)]
struct EncryptionInitialization {
    handshake_seed: u64,
    g: u32,
    p: u32,
    a: u32,
}

#[derive(Clone, Copy, Debug)]
struct ClientChallenge {
    b: u32,
    key: u64,
}

struct RecoveredEncryption {
    encryption: SilkroadEncryption,
    server_challenge: u64,
}

/// Security reconstructed from handshake packets observed in a capture.
///
/// Silkroad applies check bytes only to client-to-server packets, while the
/// negotiated encryption is usable in both directions.
pub struct CaptureSecurity {
    initialization: Option<EncryptionInitialization>,
    expected_server_challenge: Option<u64>,
    encryption: Option<SilkroadEncryption>,
    client_checks: Option<SecurityBytes>,
    brute_force_threads: usize,
}

impl CaptureSecurity {
    pub fn new(brute_force_threads: usize) -> Self {
        Self {
            initialization: None,
            expected_server_challenge: None,
            encryption: None,
            client_checks: None,
            brute_force_threads: brute_force_threads.clamp(1, 256),
        }
    }

    pub fn server_to_client_context(&self) -> SecurityContext<'_> {
        SecurityContext::new(self.encryption.as_ref(), None)
    }

    pub fn client_to_server_context(&self) -> SecurityContext<'_> {
        SecurityContext::new(self.encryption.as_ref(), self.client_checks.as_ref())
    }

    pub fn observe_server_packet(&mut self, packet: &IncomingPacket) -> Result<()> {
        if packet.opcode() != SECURITY_CAPABILITY_OPCODE {
            return Ok(());
        }

        let data = packet.data();
        let Some(&flag) = data.first() else {
            bail!("empty security capability packet");
        };

        if flag == FLAG_FINISH {
            let challenge = read_u64(data, 1, "server handshake challenge")?;
            if let Some(expected) = self.expected_server_challenge.take()
                && challenge != expected
            {
                bail!("server handshake challenge mismatch: captured {challenge:#018x}, expected {expected:#018x}");
            }
            return Ok(());
        }

        let valid_initial_flag = [
            FLAG_NONE,
            FLAG_SETUP_CHECKS,
            FLAG_INIT_BLOWFISH | FLAG_START_HANDSHAKE,
            FLAG_INIT_BLOWFISH | FLAG_SETUP_CHECKS | FLAG_START_HANDSHAKE,
        ]
        .contains(&flag);
        if !valid_initial_flag {
            bail!("unsupported initial security capability flag {flag:#04x}");
        }

        let mut offset = 1;
        if flag & FLAG_INIT_BLOWFISH != 0 {
            // The initial Blowfish seed is part of the wire format but is not
            // used when deriving the final session key.
            let _ = read_u64(data, offset, "initial Blowfish seed")?;
            offset += 8;
        }

        self.client_checks = if flag & FLAG_SETUP_CHECKS != 0 {
            let count_seed = read_u32(data, offset, "count seed")?;
            let crc_seed = read_u32(data, offset + 4, "CRC seed")?;
            offset += 8;
            Some(SecurityBytes::from_seeds(crc_seed, count_seed))
        } else {
            None
        };

        self.initialization = if flag & FLAG_START_HANDSHAKE != 0 {
            let initialization = EncryptionInitialization {
                handshake_seed: read_u64(data, offset, "handshake seed")?,
                g: read_u32(data, offset + 8, "handshake generator")?,
                p: read_u32(data, offset + 12, "handshake modulus")?,
                a: read_u32(data, offset + 16, "server public key")?,
            };
            if initialization.p < 2 {
                bail!("handshake modulus {} is below 2", initialization.p);
            }
            Some(initialization)
        } else {
            None
        };

        self.encryption = None;
        self.expected_server_challenge = None;
        debug!(
            checks = self.client_checks.is_some(),
            encryption = self.initialization.is_some(),
            "Observed security initialization"
        );
        Ok(())
    }

    pub fn observe_client_packet(&mut self, packet: &IncomingPacket) -> Result<()> {
        if packet.opcode() != SECURITY_CAPABILITY_OPCODE || packet.data().len() != 12 {
            return Ok(());
        }

        let Some(initialization) = self.initialization else {
            bail!("client handshake challenge arrived before server initialization");
        };
        let challenge = ClientChallenge {
            b: read_u32(packet.data(), 0, "client public key")?,
            key: read_u64(packet.data(), 4, "client handshake proof")?,
        };

        info!(
            threads = self.brute_force_threads,
            "Recovering captured Silkroad session key"
        );
        let recovered = recover_encryption(initialization, challenge, self.brute_force_threads)?;
        self.encryption = Some(recovered.encryption);
        self.expected_server_challenge = Some(recovered.server_challenge);
        info!("Recovered captured Silkroad session key");
        Ok(())
    }
}

fn read_u32(data: &[u8], offset: usize, field: &str) -> Result<u32> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or_else(|| eyre!("security packet is missing {field}"))?;
    Ok(u32::from_le_bytes(bytes.try_into().expect("slice length was checked")))
}

fn read_u64(data: &[u8], offset: usize, field: &str) -> Result<u64> {
    let bytes = data
        .get(offset..offset + 8)
        .ok_or_else(|| eyre!("security packet is missing {field}"))?;
    Ok(u64::from_le_bytes(bytes.try_into().expect("slice length was checked")))
}

fn recover_encryption(
    initialization: EncryptionInitialization,
    challenge: ClientChallenge,
    thread_count: usize,
) -> Result<RecoveredEncryption> {
    recover_encryption_through(initialization, challenge, thread_count, MAX_PRIVATE_KEY)
}

fn recover_encryption_through(
    initialization: EncryptionInitialization,
    challenge: ClientChallenge,
    thread_count: usize,
    maximum_private_key: u32,
) -> Result<RecoveredEncryption> {
    let total_candidates = u64::from(maximum_private_key) + 1;
    let worker_count = thread_count.min(total_candidates as usize).max(1);
    let chunk_size = total_candidates.div_ceil(worker_count as u64);
    let found = Arc::new(AtomicBool::new(false));

    let recovered = thread::scope(|scope| {
        let mut workers = Vec::with_capacity(worker_count);
        for worker in 0..worker_count {
            let start = worker as u64 * chunk_size;
            let end = (start + chunk_size).min(total_candidates);
            if start >= end {
                continue;
            }

            let found = Arc::clone(&found);
            workers.push(scope.spawn(move || recover_in_range(initialization, challenge, start as u32, end, &found)));
        }

        workers
            .into_iter()
            .filter_map(|worker| worker.join().expect("key recovery worker panicked"))
            .next()
    });

    recovered.ok_or_else(|| eyre!("could not recover the Silkroad session key"))
}

fn recover_in_range(
    initialization: EncryptionInitialization,
    challenge: ClientChallenge,
    start: u32,
    end: u64,
    found: &AtomicBool,
) -> Option<RecoveredEncryption> {
    let modulus = u64::from(initialization.p);
    let multiplier = u64::from(initialization.g) % modulus;
    let mut public = u64::from(modular_pow(initialization.p, start, initialization.g));

    for private_key in u64::from(start)..end {
        if private_key & 0x0fff == 0 && found.load(Ordering::Relaxed) {
            return None;
        }

        if public as u32 == initialization.a
            && let Some(recovered) = derive_candidate(initialization, challenge, private_key as u32)
        {
            if !found.swap(true, Ordering::Relaxed) {
                return Some(recovered);
            }
            return None;
        }

        public = (public * multiplier) % modulus;
    }

    None
}

fn derive_candidate(
    initialization: EncryptionInitialization,
    challenge: ClientChallenge,
    private_key: u32,
) -> Option<RecoveredEncryption> {
    let shared_secret = modular_pow(initialization.p, private_key, challenge.b);
    let proof_key = transform_key(
        to_u64(initialization.a, challenge.b),
        shared_secret,
        shared_secret as u8 & 0x03,
    );
    let proof_cipher = SilkroadEncryption::from_key(proof_key);
    let decrypted_proof = proof_cipher.decrypt(&challenge.key.to_le_bytes()).ok()?;
    let decrypted_proof = u64::from_le_bytes(decrypted_proof.as_ref().try_into().ok()?);
    let expected_proof = transform_key(
        to_u64(challenge.b, initialization.a),
        shared_secret,
        challenge.b as u8 & 0x07,
    );
    if decrypted_proof != expected_proof {
        return None;
    }

    let challenge_value = transform_key(
        to_u64(initialization.a, challenge.b),
        shared_secret,
        initialization.a as u8 & 0x07,
    );
    let encrypted_challenge = proof_cipher.encrypt(&challenge_value.to_le_bytes()).ok()?;
    let server_challenge = u64::from_le_bytes(encrypted_challenge.as_ref().try_into().ok()?);
    let final_key = transform_key(initialization.handshake_seed, shared_secret, 3);

    Some(RecoveredEncryption {
        encryption: SilkroadEncryption::from_key(final_key),
        server_challenge,
    })
}

fn modular_pow(p: u32, mut x: u32, g: u32) -> u32 {
    let modulus = u128::from(p);
    let mut current = 1_u128;
    let mut multiplier = u128::from(g) % modulus;

    while x != 0 {
        if x & 1 == 1 {
            current = (current * multiplier) % modulus;
        }
        x >>= 1;
        multiplier = (multiplier * multiplier) % modulus;
    }

    current as u32
}

fn transform_key(value: u64, key: u32, key_byte: u8) -> u64 {
    let mut bytes = value.to_le_bytes();
    let key_bytes = key.to_le_bytes();

    for (index, byte) in bytes.iter_mut().enumerate() {
        let key_part = key_bytes[index % key_bytes.len()];
        *byte ^= byte.wrapping_add(key_part).wrapping_add(key_byte);
    }

    u64::from_le_bytes(bytes)
}

fn to_u64(low: u32, high: u32) -> u64 {
    (u64::from(high) << 32) | u64::from(low)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BufMut, Bytes, BytesMut};
    use skrillax_codec::SilkroadFrame;
    use skrillax_packet::{IncomingPacketReframer, ReframingLimits};

    fn captured_handshake(private_key: u32) -> (EncryptionInitialization, ClientChallenge, u64) {
        let initialization = EncryptionInitialization {
            handshake_seed: 0x1112_1314_1516_1718,
            g: 5,
            p: 23,
            a: modular_pow(23, private_key, 5),
        };
        let client_private = 15;
        let b = modular_pow(initialization.p, client_private, initialization.g);
        let shared_secret = modular_pow(initialization.p, private_key, b);
        let proof_key = transform_key(to_u64(initialization.a, b), shared_secret, shared_secret as u8 & 0x03);
        let proof = transform_key(to_u64(b, initialization.a), shared_secret, b as u8 & 0x07);
        let encrypted_proof = SilkroadEncryption::from_key(proof_key)
            .encrypt(&proof.to_le_bytes())
            .unwrap();
        let challenge = ClientChallenge {
            b,
            key: u64::from_le_bytes(encrypted_proof.as_ref().try_into().unwrap()),
        };
        let final_key = transform_key(initialization.handshake_seed, shared_secret, 3);
        (initialization, challenge, final_key)
    }

    #[test]
    fn recovers_encryption_from_both_handshake_sides() {
        let (initialization, challenge, final_key) = captured_handshake(6);
        let recovered = recover_encryption_through(initialization, challenge, 2, 100).unwrap();
        let plaintext = b"12345678";
        let encrypted = SilkroadEncryption::from_key(final_key).encrypt(plaintext).unwrap();

        assert_eq!(recovered.encryption.decrypt(&encrypted).unwrap(), &plaintext[..]);
    }

    #[test]
    fn rejects_a_client_proof_that_does_not_match() {
        let (initialization, mut challenge, _) = captured_handshake(6);
        challenge.key ^= 1;

        assert!(recover_encryption_through(initialization, challenge, 2, 100).is_err());
    }

    #[test]
    fn captured_security_enables_decryption_for_both_directions() {
        let private_key = 6;
        let (initialization, challenge, final_key) = captured_handshake(private_key);
        let recovered = derive_candidate(initialization, challenge, private_key).unwrap();
        let mut initial_data = BytesMut::new();
        initial_data.put_u8(FLAG_INIT_BLOWFISH | FLAG_SETUP_CHECKS | FLAG_START_HANDSHAKE);
        initial_data.put_u64_le(0x0102_0304_0506_0708);
        initial_data.put_u32_le(0x1122_3344);
        initial_data.put_u32_le(0x5566_7788);
        initial_data.put_u64_le(initialization.handshake_seed);
        initial_data.put_u32_le(initialization.g);
        initial_data.put_u32_le(initialization.p);
        initial_data.put_u32_le(initialization.a);

        let mut security = CaptureSecurity::new(2);
        security
            .observe_server_packet(&IncomingPacket::new(SECURITY_CAPABILITY_OPCODE, initial_data.freeze()))
            .unwrap();
        assert!(security.client_to_server_context().checkers().is_some());
        security
            .observe_client_packet(&IncomingPacket::new(
                SECURITY_CAPABILITY_OPCODE,
                Bytes::from(
                    [
                        challenge.b.to_le_bytes().as_slice(),
                        challenge.key.to_le_bytes().as_slice(),
                    ]
                    .concat(),
                ),
            ))
            .unwrap();
        assert!(security.server_to_client_context().encryption().is_some());
        assert!(security.client_to_server_context().encryption().is_some());

        let mut final_data = BytesMut::new();
        final_data.put_u8(FLAG_FINISH);
        final_data.put_u64_le(recovered.server_challenge);
        security
            .observe_server_packet(&IncomingPacket::new(SECURITY_CAPABILITY_OPCODE, final_data.freeze()))
            .unwrap();

        let payload = b"decrypted";
        let mut plaintext_frame = BytesMut::new();
        plaintext_frame.put_u16_le(0x1234);
        plaintext_frame.put_u8(0);
        plaintext_frame.put_u8(0);
        plaintext_frame.extend_from_slice(payload);
        let encrypted_data = SilkroadEncryption::from_key(final_key)
            .encrypt(&plaintext_frame)
            .unwrap();
        let encrypted_frame = SilkroadFrame::Encrypted {
            content_size: payload.len(),
            encrypted_data,
        };
        let mut reframer = IncomingPacketReframer::new(ReframingLimits::recommended());
        let packet = reframer
            .push(&encrypted_frame, security.server_to_client_context())
            .unwrap()
            .unwrap();

        assert_eq!(packet.opcode(), 0x1234);
        assert_eq!(packet.data(), payload);
    }

    #[test]
    fn transform_key_matches_the_protocol_byte_layout() {
        assert_eq!(
            transform_key(0x0807_0605_0403_0201, 0x0c0b_0a09, 3),
            0x1f12_1514_1712_0d0c
        );
    }
}
