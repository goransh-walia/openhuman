use super::*;

#[test]
fn all_schemas_use_inference_namespace_and_have_descriptions() {
    for s in all_controller_schemas() {
        assert_eq!(s.namespace, "inference", "function {}", s.function);
        assert!(!s.description.is_empty(), "function {} desc", s.function);
        assert!(!s.outputs.is_empty(), "function {} outputs", s.function);
    }
}

#[test]
fn every_registered_key_resolves_to_non_unknown_schema() {
    let keys = ["transcribe", "transcribe_bytes", "tts", "test_connection"];
    for k in keys {
        let s = schemas(k);
        assert_eq!(s.namespace, "inference");
        assert_ne!(s.function, "unknown", "key `{k}` fell through");
    }
}

/// Model download, asset, and Piper-install controllers were removed: the
/// user runs their own local runtime and pulls models there.
#[test]
fn download_and_installer_controllers_are_not_registered() {
    let functions: Vec<&str> = all_controller_schemas()
        .into_iter()
        .map(|schema| schema.function)
        .collect();
    for removed in [
        "assets_status",
        "downloads_progress",
        "download_asset",
        "install_piper",
        "piper_install_status",
    ] {
        assert!(
            !functions.contains(&removed),
            "`{removed}` still registered"
        );
        assert_eq!(schemas(removed).function, "unknown");
    }
}

#[test]
fn registered_controllers_all_in_inference_namespace() {
    for h in all_registered_controllers() {
        assert_eq!(h.schema.namespace, "inference");
        assert!(!h.schema.function.is_empty());
    }
}

#[test]
fn field_builder_helpers_are_correct_shape() {
    let r = required_string("k", "c");
    assert!(r.required);
    assert!(matches!(r.ty, TypeSchema::String));

    let o = optional_string("k", "c");
    assert!(!o.required);

    let j = json_output("result", "c");
    assert!(j.required);
    assert!(matches!(j.ty, TypeSchema::Json));
}

#[test]
fn to_json_wraps_rpc_outcome() {
    let v = to_json(Outcome::single_log(serde_json::json!({"ok": true}), "l")).expect("serialize");
    assert!(v.get("logs").is_some() || v.get("result").is_some() || v.get("ok").is_some());
}
