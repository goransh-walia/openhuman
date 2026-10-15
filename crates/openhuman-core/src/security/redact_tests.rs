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
            } else {
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

#[test]
fn text_falls_back_to_the_floor_never_the_original_or_a_blank() {
    let out = answer_or_floor(
        None,
        "retry failed: Authorization: Bearer abcdefghijklmnop\nok",
    );
    assert!(!out.contains("abcdefghijklmnop"), "{out}");
    assert!(
        out.starts_with("retry failed:") && out.ends_with("\nok"),
        "{out}"
    );
}

#[test]
fn text_prefers_the_module_answer() {
    assert_eq!(
        answer_or_floor(Some("module said".into()), "raw"),
        "module said"
    );
}

/// The moved callers end to end on the released archive: the artifact redactor
/// reports a secret-bearing body as rewritten without the secret, leaves a clean
/// body unchanged, and the halt-summary error line loses the secret.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a released host key (downloads the published archive)"]
async fn moved_text_callers_redact_through_the_released_archive() {
    use tinyagents_harness::artifacts::ArtifactRedactor;

    let (_dir, config) = released_config();
    arm(&config).await;
    let redactor = crate::agent::harness::artifact_offload::SanitizingRedactor;

    let secret_body = "result ok\nAuthorization: Bearer abcdefghijklmnopqrstuvwxyz0123456789\n";
    let rewritten = redactor.redact(secret_body);
    assert!(rewritten.changed, "a secret-bearing body must be rewritten");
    assert!(!rewritten.text.contains("abcdefghijklmnopqrstuvwxyz"));
    assert!(rewritten.text.contains("result ok"), "{}", rewritten.text);

    let clean = redactor.redact("plain output with no secrets\n");
    assert!(!clean.changed);
    assert_eq!(clean.text, "plain output with no secrets\n");

    let failure = format!(
        "request failed: api_key={}-{}",
        "sk", "live-abcdefghijklmnopqrstuvwxyz0123456789"
    );
    let line = crate::agent::session_host::stop_summary::short_error_line(&failure);
    assert!(!line.contains("abcdefghijklmnopqrstuvwxyz"), "{line}");
    assert!(line.contains("request failed"), "{line}");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "exploration"]
async fn explore_arguments() {
    let (_dir, config) = released_config();
    arm(&config).await;
    let cases = vec![
        serde_json::json!({"action":"send","body":"hello world","to":"a@b.com","token":"abc","integration":"gmail"}),
        serde_json::json!({"path":"/home/alice/x.txt","note":"c:\\Users\\bob\\y","list":[1,2,{"subject":"s"}],"n":5,"b":true,"nested":{"password":"hunter2","k":null}}),
        serde_json::json!({"payload":"Authorization: Bearer abcdefghijklmnopqrstuvwxyz0123","url":"https://u:p@h.com/a?k=v"}),
        serde_json::json!({"attachments":[{"url":"https://s3/x"}],"user_id":"u-1","code":123456}),
        serde_json::json!("plain string /Users/zed/x"),
        serde_json::json!([1,2,3]),
    ];
    for c in cases {
        println!("IN : {c}");
        println!("OLD: {}", crate::security::approval::redact_args(&c));
        println!("NEW: {}", arguments(&c));
    }
}
