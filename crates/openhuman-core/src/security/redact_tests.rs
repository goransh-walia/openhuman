use super::*;

/// A config whose module is the published archive, downloaded into a scratch
/// install under `target/tmp` (a `TMPDIR` under `/tmp` breaks the policy tests).
fn released_config() -> (tempfile::TempDir, crate::config::Config) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp");
    std::fs::create_dir_all(&root).unwrap();
    let dir = tempfile::tempdir_in(&root).unwrap();
    let base = dir.path().canonicalize().unwrap();
    let mut config = crate::config::Config::default();
    config.modules.enabled = true;
    config.modules.allow_download = true;
    config.modules.install_dir = Some(base.join("modules").to_string_lossy().into_owned());
    config.action_dir = base.join("action");
    config.workspace_dir = base.join("state");
    (dir, config)
}

/// Load the module and arm the log bridge through an ordinary async call.
async fn arm(config: &crate::config::Config) {
    crate::modules::security::text::redact(
        config,
        crate::modules::security::text::RedactMode::Text,
        "Bearer abcdefghijklmnopqrstuvwxyz0123456789",
    )
    .await
    .expect("the released module must answer a redaction");
}

fn corpus(name: &str) -> serde_json::Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// Every URL in both shared corpora through the released module: credentials,
/// path, query and fragment are gone from the origin and host forms, and the
/// forms are the ones the removed host helpers produced.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key (downloads the published archive)"]
async fn endpoint_helpers_match_the_url_corpus_on_the_released_archive() {
    let (_dir, config) = released_config();
    arm(&config).await;
    for name in [
        "security-redaction-corpus.json",
        "security-redaction-extended-corpus.json",
    ] {
        for case in corpus(name)["urls"].as_array().unwrap() {
            let input = case["input"].as_str().unwrap();
            let origin = endpoint_origin(input);
            let host = endpoint_host(input);
            for secret in ["alice", "hunter2", "key=secret", "private", "s3cr3t", "/v1"] {
                assert!(
                    !origin.contains(secret) && !host.contains(secret),
                    "case {} leaked {secret}: {origin} / {host}",
                    case["case"]
                );
            }
            if let Some(expected) = case.get("endpoint").and_then(|v| v.as_str()) {
                assert_eq!(origin, expected, "origin case {}", case["case"]);
            }
            if let Some(expected) = case.get("inference").and_then(|v| v.as_str()) {
                assert_eq!(host, expected, "host case {}", case["case"]);
            }
        }
    }
    assert_eq!(
        endpoint_origin("https://user:pass@example.com/path/secret?apiKey=123"),
        "https://example.com"
    );
    assert_eq!(
        endpoint_origin("http://example.com:8080/path?token=secret"),
        "http://example.com:8080"
    );
}

/// The sync text pass removes every secret shape of the shared text corpora and
/// keeps ordinary text, from a plain thread and from a runtime worker.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key (downloads the published archive)"]
async fn text_helper_redacts_the_text_corpus_on_the_released_archive() {
    let (_dir, config) = released_config();
    arm(&config).await;
    for name in [
        "security-redaction-corpus.json",
        "security-redaction-extended-corpus.json",
    ] {
        for case in corpus(name)["text"].as_array().unwrap() {
            let removed_by: Vec<&str> = case["removed_by"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v.as_str())
                .collect();
            let input: String = match case["input_parts"].as_array() {
                Some(parts) => parts.iter().map(|p| p.as_str().unwrap()).collect(),
                None => case["input"].as_str().unwrap().to_string(),
            };
            let needle = case["needle"].as_str().unwrap();
            let out = std::thread::spawn({
                let input = input.clone();
                move || text(&input)
            })
            .join()
            .unwrap();
            if case["case"] == "ordinary_text" {
                assert_eq!(out, input, "ordinary text must pass through");
            } else if removed_by.iter().any(|sink| *sink == "host_scrub") {
                assert!(!out.contains(needle), "case {} leaked: {out}", case["case"]);
            }
            let inside = text(&input);
            assert_eq!(inside, out, "case {} differs by thread", case["case"]);
        }
    }
}

/// A timestamp and a bare digit run survive (corroborated card policy), and a
/// card with an issuer prefix does not.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key (downloads the published archive)"]
async fn text_helper_keeps_timestamps_and_removes_corroborated_cards() {
    let (_dir, config) = released_config();
    arm(&config).await;
    assert_eq!(text("at 1700000000000 ms"), "at 1700000000000 ms");
    let card = text("card 4111 1111 1111 1111 on file");
    assert!(!card.contains("4111 1111 1111 1111"), "{card}");
}
