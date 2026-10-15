use super::*;

#[test]
fn defaults_to_the_classic_layout() {
    assert_eq!(StorageConfig::default().url, None);
    let parsed: StorageConfig = toml::from_str("").unwrap();
    assert_eq!(parsed, StorageConfig::default());
}

#[test]
fn reads_a_url_from_toml() {
    let parsed: StorageConfig = toml::from_str("url = \"memory\"").unwrap();
    assert_eq!(parsed.url.as_deref(), Some("memory"));
}

#[test]
fn debug_never_prints_credentials() {
    let config = StorageConfig {
        url: Some("mongodb://app:hunter2@db.internal/openhuman".into()),
    };
    let shown = format!("{config:?}");
    assert!(!shown.contains("hunter2"), "{shown}");
    assert!(shown.contains("db.internal/openhuman"), "{shown}");
}

#[test]
fn debug_drops_query_and_fragment_tokens() {
    let config = StorageConfig {
        url: Some(
            "mongodb://db.internal/openhuman?authMechanismProperties=AWS_SESSION_TOKEN%3Asecret"
                .into(),
        ),
    };
    let shown = format!("{config:?}");
    assert!(!shown.contains("AWS_SESSION_TOKEN"), "{shown}");
    assert!(!shown.contains("secret"), "{shown}");
}

#[test]
fn debug_leaves_credential_free_urls_readable() {
    let config = StorageConfig {
        url: Some("sqlite:/tmp/x".into()),
    };
    assert!(format!("{config:?}").contains("sqlite:/tmp/x"));
}
