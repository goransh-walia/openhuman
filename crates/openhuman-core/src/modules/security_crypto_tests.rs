use super::*;
use tinysecurity_bus::{CryptoConfig2, KeyringBackend2, KeyringConfig2, Origin};

/// Bytes written by the pre-bus host (`security::encryption::EncryptionKey`)
/// for password `FIXTURE_PASSWORD`. Never regenerate to make a test pass.
const LEGACY_PAYLOAD: &str = include_str!("fixtures/legacy_password_payload.json");
const FIXTURE_PASSWORD: &str = "fixture-password-7328";
const FIXTURE_PLAINTEXT: &[u8] = b"legacy host payload 7328";

fn consent_request(access: KeyringAccess2) -> KeyringConsentRequest2 {
    KeyringConsentRequest2 {
        caller: caller(),
        purpose: "test".into(),
        access,
        name: "secret-name".into(),
        timeout_ms: 1000,
    }
}

fn caller() -> CallerContext {
    CallerContext {
        agent: "agent".into(),
        thread: None,
        flow: None,
        call_id: "call-1".into(),
        origin: Origin::Operator,
        tier: tinysecurity_bus::AccessTier::Full,
    }
}

#[test]
fn consent_is_granted_only_when_the_host_policy_proceeds() {
    for access in [
        KeyringAccess2::Read,
        KeyringAccess2::Write,
        KeyringAccess2::Delete,
    ] {
        let request = consent_request(access);
        assert!(keyring_consent_response(&request, PolicyDecision::Proceed).allowed);
        assert!(!keyring_consent_response(&request, PolicyDecision::Declined).allowed);
        assert!(!keyring_consent_response(&request, PolicyDecision::ConsentRequired).allowed);
    }
}

#[test]
fn legacy_host_payload_shape_is_the_bus_payload_shape() {
    let payload: LegacyPayload = serde_json::from_str(LEGACY_PAYLOAD).unwrap();
    assert_eq!(payload.nonce.len(), 12);
    assert_eq!(payload.salt.len(), 16);
    let legacy: crate::security::encryption::EncryptedPayload =
        serde_json::from_str(LEGACY_PAYLOAD).unwrap();
    assert_eq!(legacy.ciphertext, payload.ciphertext);
    assert_eq!(legacy.salt, payload.salt);
}

#[test]
fn secret_material_is_redacted_from_debug_output() {
    let request = PasswordRequest2 {
        operation: PasswordOperation2::Encrypt {
            password: "hunter2-secret".into(),
            plaintext: b"plain-secret".to_vec(),
            salt: None,
        },
    };
    let debug = format!("{request:?}");
    assert!(!debug.contains("hunter2"));
    assert!(!debug.contains("plain-secret"));
}

fn crypto_config(dir: &std::path::Path) -> CryptoConfig2 {
    CryptoConfig2 {
        max_payload_bytes: 65_536,
        operations_per_minute: 256,
        keyring: Some(KeyringConfig2 {
            service: "openhuman-test-7328".into(),
            backend: KeyringBackend2::EncryptedFile {
                directory: dir.to_string_lossy().into_owned(),
            },
            consent_timeout_ms: 2000,
        }),
        pairing: None,
    }
}

/// The module decrypts the frozen host fixture, and host and module payloads
/// are interchangeable in both directions.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn password_crypt_opens_legacy_host_payloads_and_interoperates() {
    let fx =
        super::super::native_tests::native_fixture_with(Some(crypto_config(fx_dir().path()))).await;
    let payload: LegacyPayload = serde_json::from_str(LEGACY_PAYLOAD).unwrap();
    let opened = password_decrypt(&fx.proxy, FIXTURE_PASSWORD.into(), payload)
        .await
        .unwrap();
    assert_eq!(opened, FIXTURE_PLAINTEXT);

    let sealed = password_encrypt(&fx.proxy, "pw".into(), b"from module".to_vec(), None)
        .await
        .unwrap();
    let key = crate::security::encryption::EncryptionKey::derive("pw", &sealed.salt).unwrap();
    let host_view = crate::security::encryption::EncryptedPayload {
        ciphertext: sealed.ciphertext.clone(),
        nonce: sealed.nonce.clone(),
        salt: sealed.salt.clone(),
    };
    assert_eq!(key.decrypt(&host_view).unwrap(), b"from module");

    let wrong = password_decrypt(&fx.proxy, "other".into(), sealed).await;
    assert!(wrong.is_err());
}

fn fx_dir() -> tempfile::TempDir {
    let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
    tempfile::tempdir_in(target.canonicalize().unwrap()).unwrap()
}

/// Keyring round trip through the module's encrypted-file backend, with the
/// host consent callback served for real (the unit test above pins the
/// declined mapping), the secret is stored, read and deleted.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key or an explicitly selected local native fixture"]
async fn keyring_round_trip_through_the_served_consent_callback() {
    let fx =
        super::super::native_tests::native_fixture_with(Some(crypto_config(fx_dir().path()))).await;
    crate::modules::security_host::install(&fx.client)
        .await
        .unwrap();
    let name = "user:token-7328";

    keyring_set(
        &fx.proxy,
        caller(),
        "store a token",
        name,
        b"value-1".to_vec(),
    )
    .await
    .unwrap();
    assert_eq!(
        keyring_get(&fx.proxy, caller(), "read a token", name)
            .await
            .unwrap(),
        Some(b"value-1".to_vec())
    );
    assert!(keyring_delete(&fx.proxy, caller(), "remove a token", name)
        .await
        .unwrap());
    assert_eq!(
        keyring_get(&fx.proxy, caller(), "read a token", name)
            .await
            .unwrap(),
        None
    );
}
