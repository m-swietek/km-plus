//! On-disk envelope of the vault file: a fixed 59-byte header followed by the
//! XChaCha20-Poly1305 ciphertext. The whole header is authenticated as AAD.
//!
//! | field            | size    | value                              |
//! | ---------------- | ------- | ---------------------------------- |
//! | magic            | 4 B     | `KMPV`                             |
//! | envelope_version | u16 LE  | `1`                                |
//! | kdf_id           | u8      | `1` = Argon2id v0x13               |
//! | m_cost (KiB)     | u32 LE  | default `65536`                    |
//! | t_cost           | u32 LE  | default `3`                        |
//! | p_cost           | u32 LE  | default `1`                        |
//! | salt             | 16 B    | random, fixed at vault creation    |
//! | nonce            | 24 B    | random, fresh on every save        |
//! | ciphertext       | rest    | includes the 16-byte Poly1305 tag  |

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use zeroize::Zeroizing;

pub const MAGIC: [u8; 4] = *b"KMPV";
pub const ENVELOPE_VERSION: u16 = 1;
pub const KDF_ARGON2ID: u8 = 1;

pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 24;
pub const KEY_LEN: usize = 32;
pub const TAG_LEN: usize = 16;
pub const HEADER_LEN: usize = 4 + 2 + 1 + 4 + 4 + 4 + SALT_LEN + NONCE_LEN;

// Byte offsets inside the header.
const OFF_ENVELOPE: usize = 4;
const OFF_KDF_ID: usize = 6;
const OFF_M_COST: usize = 7;
const OFF_T_COST: usize = 11;
const OFF_P_COST: usize = 15;
const OFF_SALT: usize = 19;
const OFF_NONCE: usize = OFF_SALT + SALT_LEN;

// Sanity bounds: anything outside is treated as a damaged header so that a
// corrupted file cannot force a huge allocation or an endless derivation.
const MAX_M_COST_KIB: u32 = 1024 * 1024; // 1 GiB
const MAX_T_COST: u32 = 10;
const MAX_P_COST: u32 = 8;

/// Argon2id cost parameters, stored in the header of every vault file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    /// Memory cost in KiB.
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            m_cost: 65536,
            t_cost: 3,
            p_cost: 1,
        }
    }
}

impl KdfParams {
    /// Within the sanity bounds and accepted by Argon2 (`m_cost >= 8 * p_cost`).
    pub fn is_valid(&self) -> bool {
        (1..=MAX_P_COST).contains(&self.p_cost)
            && (1..=MAX_T_COST).contains(&self.t_cost)
            && self.m_cost >= 8 * self.p_cost
            && self.m_cost <= MAX_M_COST_KIB
    }
}

/// Header fields that stay fixed for the lifetime of a vault (the nonce is
/// generated on every seal).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub params: KdfParams,
    pub salt: [u8; SALT_LEN],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenError {
    /// The header cannot be parsed: the file is certainly damaged.
    Malformed,
    /// The file was written by a newer envelope format.
    UnsupportedEnvelope,
    /// The header is fine but the AEAD tag does not match: wrong password or
    /// damaged ciphertext/header.
    AuthFailed,
}

#[derive(Debug, thiserror::Error)]
#[error("nieprawidłowe parametry wyprowadzania klucza")]
pub struct KdfError;

pub fn encode_header(header: &Header, nonce: &[u8; NONCE_LEN]) -> [u8; HEADER_LEN] {
    let mut out = [0u8; HEADER_LEN];
    out[..OFF_ENVELOPE].copy_from_slice(&MAGIC);
    out[OFF_ENVELOPE..OFF_KDF_ID].copy_from_slice(&ENVELOPE_VERSION.to_le_bytes());
    out[OFF_KDF_ID] = KDF_ARGON2ID;
    out[OFF_M_COST..OFF_T_COST].copy_from_slice(&header.params.m_cost.to_le_bytes());
    out[OFF_T_COST..OFF_P_COST].copy_from_slice(&header.params.t_cost.to_le_bytes());
    out[OFF_P_COST..OFF_SALT].copy_from_slice(&header.params.p_cost.to_le_bytes());
    out[OFF_SALT..OFF_NONCE].copy_from_slice(&header.salt);
    out[OFF_NONCE..HEADER_LEN].copy_from_slice(nonce);
    out
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&bytes[offset..offset + 4]);
    u32::from_le_bytes(buf)
}

/// Parses and sanity-checks the header. Does not touch the ciphertext beyond
/// requiring room for the authentication tag.
pub fn parse_header(bytes: &[u8]) -> Result<(Header, [u8; NONCE_LEN]), OpenError> {
    if bytes.len() < OFF_KDF_ID || bytes[..OFF_ENVELOPE] != MAGIC {
        return Err(OpenError::Malformed);
    }
    let envelope = u16::from_le_bytes([bytes[OFF_ENVELOPE], bytes[OFF_ENVELOPE + 1]]);
    if envelope > ENVELOPE_VERSION {
        return Err(OpenError::UnsupportedEnvelope);
    }
    if envelope != ENVELOPE_VERSION || bytes.len() < HEADER_LEN + TAG_LEN {
        return Err(OpenError::Malformed);
    }
    if bytes[OFF_KDF_ID] != KDF_ARGON2ID {
        return Err(OpenError::Malformed);
    }
    let params = KdfParams {
        m_cost: read_u32(bytes, OFF_M_COST),
        t_cost: read_u32(bytes, OFF_T_COST),
        p_cost: read_u32(bytes, OFF_P_COST),
    };
    if !params.is_valid() {
        return Err(OpenError::Malformed);
    }
    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&bytes[OFF_SALT..OFF_NONCE]);
    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(&bytes[OFF_NONCE..HEADER_LEN]);
    Ok((Header { params, salt }, nonce))
}

/// Argon2id (v0x13) over the UTF-8 password bytes, exactly as typed.
pub fn derive_key(
    password: &str,
    salt: &[u8; SALT_LEN],
    params: &KdfParams,
) -> Result<Zeroizing<[u8; KEY_LEN]>, KdfError> {
    if !params.is_valid() {
        return Err(KdfError);
    }
    let argon_params = Params::new(params.m_cost, params.t_cost, params.p_cost, Some(KEY_LEN))
        .map_err(|_| KdfError)?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);
    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    argon
        .hash_password_into(password.as_bytes(), salt, key.as_mut())
        .map_err(|_| KdfError)?;
    Ok(key)
}

/// Encrypts `plaintext` under a freshly generated nonce and returns the full
/// file contents (header + ciphertext).
pub fn seal(key: &[u8; KEY_LEN], header: &Header, plaintext: &[u8]) -> Vec<u8> {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key));
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let mut nonce_bytes = [0u8; NONCE_LEN];
    nonce_bytes.copy_from_slice(&nonce);
    let head = encode_header(header, &nonce_bytes);
    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: plaintext,
                aad: &head,
            },
        )
        // Only fails for plaintexts of hundreds of GiB.
        .expect("XChaCha20-Poly1305 encryption cannot fail for vault-sized payloads");
    let mut out = Vec::with_capacity(HEADER_LEN + ciphertext.len());
    out.extend_from_slice(&head);
    out.extend_from_slice(&ciphertext);
    out
}

/// Authenticates and decrypts full file contents with an already derived key.
pub fn open(key: &[u8; KEY_LEN], bytes: &[u8]) -> Result<Zeroizing<Vec<u8>>, OpenError> {
    let (_, nonce) = parse_header(bytes)?;
    let (head, ciphertext) = bytes.split_at(HEADER_LEN);
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key));
    cipher
        .decrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: ciphertext,
                aad: head,
            },
        )
        .map(Zeroizing::new)
        .map_err(|_| OpenError::AuthFailed)
}
