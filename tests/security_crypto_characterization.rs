//! Phase-0 crypto characterization (issue #7328, plan Task 1). Committed
//! fixtures written by the current implementations must keep decrypting after
//! the port into the tinysecurity module: Argon2id + AES-256-GCM payloads
//! (`security/encryption/core.rs`) and the X25519/HKDF/XChaCha20 device tunnel
//! (`security/devices/crypto.rs`). The encrypted-file keyring (`secrets_enc.bin`,
//! `enc2_values.txt`) is decrypted by the test-only master-key seam in
//! `secret_storage/encrypted_file_backend_fixture_tests.rs`; this file pins the
//! digests of those committed bytes so they cannot be regenerated silently.
//! Never regenerate a fixture to make a test pass.

use openhuman_core::security::devices::crypto::{
    base64url_decode, base64url_encode, derive_bootstrap_key, derive_session_keys, DeviceKeypair,
    SessionKeys, TunnelCipher, TunnelRole, HKDF_INFO_C2S, HKDF_INFO_S2C,
};
use openhuman_core::security::encryption::{EncryptedPayload, EncryptionKey};
use serde_json::Value;
use sha2::{Digest, Sha256};

const CORPUS: &str = include_str!("fixtures/security-crypto-corpus.json");
const SECRETS_ENC: &[u8] =
    include_bytes!("../crates/openhuman-core/src/security/secret_storage/fixtures/secrets_enc.bin");
const ENC2_VALUES: &[u8] =
    include_bytes!("../crates/openhuman-core/src/security/secret_storage/fixtures/enc2_values.txt");

fn corpus() -> Value {
    serde_json::from_str(CORPUS).unwrap()
}

fn unhex(v: &Value) -> Vec<u8> {
    hex::decode(v.as_str().unwrap()).unwrap()
}

fn arr32(v: &Value) -> [u8; 32] {
    unhex(v).try_into().unwrap()
}

fn payload(c: &Value) -> EncryptedPayload {
    EncryptedPayload {
        ciphertext: unhex(&c["ciphertext"]),
        nonce: unhex(&c["nonce"]),
        salt: Vec::new(),
    }
}

#[test]
fn argon2id_aes_gcm_fixture_decrypts() {
    let c = corpus();
    let a = &c["argon2_aes_gcm"];
    let key = EncryptionKey::derive(a["password"].as_str().unwrap(), &unhex(&a["salt"])).unwrap();
    let plain = key.decrypt(&payload(a)).unwrap();
    assert_eq!(plain, a["plaintext"].as_str().unwrap().as_bytes());
}

#[test]
fn argon2id_aes_gcm_rejects_wrong_password_tamper_and_bad_nonce() {
    let c = corpus();
    let a = &c["argon2_aes_gcm"];
    let salt = unhex(&a["salt"]);
    let wrong = EncryptionKey::derive("not-the-password", &salt).unwrap();
    assert!(wrong.decrypt(&payload(a)).unwrap_err().contains("Decryption failed"));

    let key = EncryptionKey::derive(a["password"].as_str().unwrap(), &salt).unwrap();
    let mut flipped = payload(a);
    flipped.ciphertext[0] ^= 1;
    assert!(key.decrypt(&flipped).is_err(), "tampered ciphertext");
    let mut truncated_tag = payload(a);
    truncated_tag.ciphertext.pop();
    assert!(key.decrypt(&truncated_tag).is_err(), "truncated tag");
    let mut other_salt = salt.clone();
    other_salt[0] ^= 1;
    let rederived = EncryptionKey::derive(a["password"].as_str().unwrap(), &other_salt).unwrap();
    assert!(rederived.decrypt(&payload(a)).is_err(), "different salt derives a different key");
}

#[test]
fn argon2id_aes_gcm_round_trips_and_never_reuses_a_nonce() {
    let salt = EncryptionKey::generate_salt();
    assert_eq!(salt.len(), 16);
    let key = EncryptionKey::derive("pw", &salt).unwrap();
    let a = key.encrypt(b"same plaintext").unwrap();
    let b = key.encrypt(b"same plaintext").unwrap();
    assert_eq!(a.nonce.len(), 12);
    assert_ne!(a.nonce, b.nonce);
    assert_ne!(a.ciphertext, b.ciphertext);
    assert_eq!(key.decrypt(&a).unwrap(), b"same plaintext");
    assert!(a.salt.is_empty(), "the salt lives beside the payload, not inside it");
    assert_eq!(a.ciphertext.len(), "same plaintext".len() + 16, "GCM tag is 16 bytes");
}

#[test]
fn hkdf_bootstrap_and_session_key_vectors_are_pinned() {
    let c = corpus();
    let d = &c["devices"];
    let boot = derive_bootstrap_key(&arr32(&d["dh"]), &unhex(&d["salt"]), HKDF_INFO_C2S);
    assert_eq!(boot.to_vec(), unhex(&d["bootstrap_c2s"]));
    let other = derive_bootstrap_key(&arr32(&d["dh"]), &unhex(&d["salt"]), HKDF_INFO_S2C);
    assert_ne!(boot, other, "info tags separate directions");

    let keys = derive_session_keys(
        &arr32(&d["static_dh"]),
        &arr32(&d["eph_dh"]),
        &arr32(&d["client_eph_pub"]),
        &arr32(&d["server_eph_pub"]),
    );
    assert_eq!(keys.c2s.to_vec(), unhex(&d["c2s"]));
    assert_eq!(keys.s2c.to_vec(), unhex(&d["s2c"]));
    assert_eq!(HKDF_INFO_C2S, b"openhuman-tunnel/v1/c2s");
    assert_eq!(HKDF_INFO_S2C, b"openhuman-tunnel/v1/s2c");
}

fn fixture_keys() -> SessionKeys {
    let c = corpus();
    SessionKeys {
        c2s: arr32(&c["devices"]["c2s"]),
        s2c: arr32(&c["devices"]["s2c"]),
    }
}

#[test]
fn committed_tunnel_frames_open_once_and_replays_are_rejected() {
    let c = corpus();
    let frame = unhex(&c["devices"]["server_frame"]);
    assert_eq!(frame[0], 0x02, "current frame version");
    let mut client = TunnelCipher::for_role(TunnelRole::Client, &fixture_keys());
    assert_eq!(
        client.open(&frame).unwrap(),
        c["devices"]["server_frame_plaintext"].as_str().unwrap().as_bytes()
    );
    assert!(client.open(&frame).unwrap_err().contains("replayed nonce"));

    // The sealing side cannot open its own direction.
    let mut server = TunnelCipher::for_role(TunnelRole::Server, &fixture_keys());
    assert!(server.open(&frame).unwrap_err().contains("authentication failed"));
}

#[test]
fn malformed_tunnel_frames_are_rejected_with_distinct_errors() {
    let c = corpus();
    let good = unhex(&c["devices"]["server_frame"]);
    let mut client = TunnelCipher::for_role(TunnelRole::Client, &fixture_keys());
    assert!(client.open(&[]).unwrap_err().contains("empty frame"));
    let mut legacy = good.clone();
    legacy[0] = 0x01;
    assert!(client.open(&legacy).unwrap_err().contains("UnsupportedFrameVersion"));
    let mut unknown = good.clone();
    unknown[0] = 0x7f;
    assert!(client.open(&unknown).unwrap_err().contains("unsupported frame version"));
    assert!(client.open(&good[..10]).unwrap_err().contains("too short"));
    let mut tampered = good.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 1;
    assert!(client.open(&tampered).unwrap_err().contains("authentication failed"));
    // None of the failures consumed the nonce: the intact frame still opens.
    assert!(client.open(&good).is_ok());
}

#[test]
fn x25519_agreement_matches_and_rejects_bad_peers() {
    let a = DeviceKeypair::generate();
    let b = DeviceKeypair::generate();
    assert_eq!(
        a.derive_shared_secret(&b.pubkey_b64).unwrap(),
        b.derive_shared_secret(&a.pubkey_b64).unwrap()
    );
    assert!(a.derive_shared_secret("!!not-base64!!").is_err());
    assert!(a
        .derive_shared_secret(&base64url_encode(&[7u8; 31]))
        .unwrap_err()
        .contains("32 bytes"));
    // A low-order point yields an all-zero secret, which must be refused.
    assert!(a.derive_shared_secret(&base64url_encode(&[0u8; 32])).is_err());
    let raw = base64url_decode(&a.pubkey_b64).unwrap();
    assert_eq!(raw.len(), 32);
    assert!(!a.pubkey_b64.contains('='), "unpadded base64url");
}

#[test]
fn encrypted_file_keyring_fixture_bytes_are_unchanged() {
    // Decryption of these bytes is asserted beside the backend (the master-key
    // seam is test-only). Pinning the digests stops a silent regeneration.
    let c = corpus();
    let digest = |bytes: &[u8]| hex::encode(Sha256::digest(bytes));
    assert_eq!(digest(SECRETS_ENC), c["keyring"]["secrets_enc_sha256"].as_str().unwrap());
    assert_eq!(digest(ENC2_VALUES), c["keyring"]["enc2_values_sha256"].as_str().unwrap());
    for line in std::str::from_utf8(ENC2_VALUES).unwrap().lines() {
        assert!(line.starts_with("enc2:"));
    }
}

/// DESIRED (plan Task 10): a payload larger than the module's binary size limit
/// is refused up front. No limit exists today. Red until the crypto module
/// defines one.
#[test]
#[should_panic(expected = "DESIRED-RED")]
fn desired_encrypt_refuses_payloads_over_the_binary_limit() {
    let key = EncryptionKey::derive("pw", &EncryptionKey::generate_salt()).unwrap();
    let oversized = vec![0u8; 64 * 1024 * 1024 + 1];
    assert!(
        key.encrypt(&oversized).is_err(),
        "DESIRED-RED: encrypt accepts an unbounded payload"
    );
}

#[test]
#[ignore = "one-off fixture generator; never run to make a test pass"]
fn zz_generate_fixtures_once() {
    let salt = [0x11u8; 16];
    let key = EncryptionKey::derive("correct horse battery staple", &salt).unwrap();
    let p = key.encrypt("characterized \u{1f512} secret".as_bytes()).unwrap();
    let dh = [0x21u8; 32];
    let hsalt = [0x31u8; 64];
    let (sdh, edh, cp, sp) = ([0x41u8; 32], [0x42u8; 32], [0x43u8; 32], [0x44u8; 32]);
    let keys = derive_session_keys(&sdh, &edh, &cp, &sp);
    let server = TunnelCipher::for_role(TunnelRole::Server, &keys);
    let frame = server.seal(b"hello from the core").unwrap();
    let out = serde_json::json!({
        "argon2_aes_gcm": {
            "password": "correct horse battery staple",
            "salt": hex::encode(salt),
            "nonce": hex::encode(&p.nonce),
            "ciphertext": hex::encode(&p.ciphertext),
            "plaintext": "characterized \u{1f512} secret",
        },
        "devices": {
            "dh": hex::encode(dh), "salt": hex::encode(hsalt),
            "bootstrap_c2s": hex::encode(derive_bootstrap_key(&dh, &hsalt, HKDF_INFO_C2S)),
            "static_dh": hex::encode(sdh), "eph_dh": hex::encode(edh),
            "client_eph_pub": hex::encode(cp), "server_eph_pub": hex::encode(sp),
            "c2s": hex::encode(keys.c2s), "s2c": hex::encode(keys.s2c),
            "server_frame": hex::encode(frame),
            "server_frame_plaintext": "hello from the core",
        },
        "keyring": {
            "secrets_enc_sha256": hex::encode(Sha256::digest(SECRETS_ENC)),
            "enc2_values_sha256": hex::encode(Sha256::digest(ENC2_VALUES)),
        }
    });
    println!("FIXTURE-JSON {}", serde_json::to_string_pretty(&out).unwrap());
}
