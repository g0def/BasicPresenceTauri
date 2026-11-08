//! Known-answer tests pinning the on-disk cryptographic formats.
//!
//! Every value below was produced by the crate versions that wrote the vaults
//! already on users' disks. A dependency upgrade (argon2, chacha20poly1305,
//! sha2, base64…) must keep reproducing them byte for byte, or existing
//! accounts could no longer be opened.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};
use sha2::{Digest, Sha256};

use crate::domain::policy::KdfParams;
use crate::domain::services::key_service::{KeyBinding, KeyRole, KeyService, WrappedKey};
use crate::domain::services::password_hasher::PasswordHasher;
use crate::infrastructure::crypto::argon2_hasher::Argon2PasswordHasher;
use crate::infrastructure::crypto::key_service::Argon2KeyService;

const PARAMS: KdfParams = KdfParams {
    m_cost: 64,
    t_cost: 1,
    p_cost: 1,
};
const SALT: [u8; 16] = [7u8; 16];
const NONCE: [u8; 24] = [3u8; 24];
const DEK: [u8; 32] = [42u8; 32];

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Wrap-format v2 associated data, spelled out on purpose: it pins the format.
fn aad_v2() -> Vec<u8> {
    b"bp-wrap-v2\0dek\0acc".to_vec()
}

/// An Argon2id PHC string written by the current hasher.
const PHC: &str = "$argon2id$v=19$m=64,t=1,p=1$ZNSjmZ4Q87/XqIuWqY3iWw$P9KSyZdl5uN+Bw5M+hxxFBnYDg2Bfxb8r1PkwVXq6l8";
const KEK_HEX: &str = "0aa8d76217ab242c6d14433d5ff105dd539f1ce4be07f1103bc3f7ed6aee0efb";
const WRAP_V2_HEX: &str = "6869b00934aaa5046731df84126d9cf07ead3a2307f969a8b51d54af9aa20d35de12c7cdc3a55d2030ea2e06361cbd5e";
const WRAP_V1_HEX: &str = "6869b00934aaa5046731df84126d9cf07ead3a2307f969a8b51d54af9aa20d352dda59db4585e81fe08982f36a918cde";

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

#[tokio::test]
async fn stored_password_hashes_still_verify() {
    let hasher = Argon2PasswordHasher::new(PARAMS).unwrap();
    assert!(hasher.verify(b"correct horse", PHC).await.unwrap());
    assert!(!hasher.verify(b"wrong horse", PHC).await.unwrap());
    assert!(!hasher.needs_rehash(PHC));
}

#[tokio::test]
async fn key_derivation_is_unchanged() {
    let kek = Argon2KeyService
        .derive_kek(b"correct horse", &SALT, PARAMS)
        .await
        .unwrap();
    assert_eq!(hex(kek.as_slice()), KEK_HEX);
}

#[test]
fn wrapped_keys_still_unwrap() {
    let kek = unhex(KEK_HEX);
    for (version, ciphertext) in [(2, WRAP_V2_HEX), (1, WRAP_V1_HEX)] {
        let wrapped = WrappedKey {
            ciphertext: unhex(ciphertext),
            nonce: NONCE.to_vec(),
        };
        let binding = KeyBinding {
            version,
            role: KeyRole::Dek,
            account_id: "acc",
        };
        let key = Argon2KeyService
            .unwrap_key(&wrapped, &kek, binding)
            .unwrap();
        assert_eq!(key.as_slice(), &DEK, "wrap v{version}");
    }
}

#[test]
fn wrapping_is_unchanged() {
    // Same key, nonce and associated data → same ciphertext as before.
    let cipher = XChaCha20Poly1305::new_from_slice(&unhex(KEK_HEX)).unwrap();
    let payload = Payload {
        msg: &DEK,
        aad: &aad_v2(),
    };
    let v2 = cipher.encrypt(&XNonce::from(NONCE), payload).unwrap();
    assert_eq!(hex(&v2), WRAP_V2_HEX);
}

#[test]
fn session_token_hashing_and_encoding_are_unchanged() {
    assert_eq!(
        hex(&Sha256::digest(b"session-token")),
        "c101e911469c969171040b50d70543313cf968fdef5bacc780776f8fb399ab36"
    );
    assert_eq!(URL_SAFE_NO_PAD.encode([0xfbu8, 0xff, 0x00, 0x10]), "-_8AEA");
}
