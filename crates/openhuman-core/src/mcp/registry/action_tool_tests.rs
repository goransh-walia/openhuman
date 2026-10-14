use super::*;
use serde_json::json;
use tinytools::{PermissionLevel, ToolExposure};

#[derive(Clone, Default)]
struct RecoveringModulePort {
    available: Arc<std::sync::atomic::AtomicBool>,
    opens: Arc<std::sync::atomic::AtomicUsize>,
    calls: Arc<std::sync::Mutex<Vec<String>>>,
}

#[async_trait::async_trait]
impl crate::mcp::module_client::McpModulePort for RecoveringModulePort {
    async fn open_configured(
        &self,
        _data_dir: &str,
        _config: &tinymcp_bus::McpClientConfig,
    ) -> Result<tinymcp_bus::OpenConfiguredResponse, tinybus::module::ModuleCallError> {
        self.opens.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if !self.available.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(tinybus::module::ModuleCallError::Unavailable);
        }
        Ok(tinymcp_bus::OpenConfiguredResponse {
            object_path: "/test/mcp".into(),
            transition_id: "open-test".into(),
        })
    }

    async fn close(
        &self,
        _data_dir: &str,
    ) -> Result<Option<String>, tinybus::module::ModuleCallError> {
        Ok(None)
    }

    async fn transition_status(
        &self,
        _transition_id: &str,
    ) -> Result<tinymcp_bus::TransitionStatus, tinybus::module::ModuleCallError> {
        Ok(tinymcp_bus::TransitionStatus::Complete)
    }

    async fn call_object(
        &self,
        _object_path: &str,
        member: &str,
        args: serde_json::Value,
        _confidential: bool,
    ) -> Result<serde_json::Value, tinybus::module::ModuleCallError> {
        if !self.available.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(tinybus::module::ModuleCallError::Unavailable);
        }
        self.calls.lock().unwrap().push(member.to_owned());
        match member {
            tinymcp_bus::names::methods::NORMALIZE_TOOL_ARGUMENTS => args
                .get(0)
                .cloned()
                .ok_or(tinybus::module::ModuleCallError::ModuleFault),
            tinymcp_bus::names::methods::TOOL_CALL => {
                serde_json::to_value(tinymcp_bus::ToolCallOutcome {
                    result: json!({"content":[{"type":"text","text":"loaded"}]}),
                    is_error: false,
                    rendered: tinymcp_bus::McpToolResult::success("loaded"),
                })
                .map_err(|_| tinybus::module::ModuleCallError::ModuleFault)
            }
            tinymcp_bus::names::methods::RENDER_TOOL_OUTPUT => {
                serde_json::to_value("lazy-loaded forecast")
                    .map_err(|_| tinybus::module::ModuleCallError::ModuleFault)
            }
            _ => Err(tinybus::module::ModuleCallError::IncompatibleContract),
        }
    }

    async fn prepare_agent_tools(
        &self,
        _request: tinymcp_bus::PrepareAgentToolsRequest,
    ) -> Result<Vec<tinymcp_bus::McpToolPresentation>, tinybus::module::ModuleCallError> {
        Ok(Vec::new())
    }

    async fn validate_server_arguments(
        &self,
        request: tinymcp_bus::ValidateServerArgumentsRequest,
    ) -> Result<tinymcp_bus::ServerArgumentsValidation, tinybus::module::ModuleCallError> {
        Ok(tinymcp_bus::ServerArgumentsValidation::Valid(
            request.arguments.as_object().cloned().unwrap_or_default(),
        ))
    }
}

fn presentation(
    server_id: &str,
    name: &str,
    legacy_name: &str,
) -> tinymcp_bus::McpToolPresentation {
    tinymcp_bus::McpToolPresentation {
        name: name.into(),
        legacy_name: legacy_name.into(),
        server_id: server_id.into(),
        remote_name: "readGoals".into(),
        family: "example/weather".into(),
        description: "MCP server Weather Service: Get the current weather forecast".into(),
        parameters: json!({
            "type": "object",
            "properties": { "city": { "type": "string" } },
            "required": ["city"]
        }),
        deferred: true,
        tags: vec![
            "mcp.server:example/weather".into(),
            format!("mcp.server_id:{server_id}"),
            "mcp.tool:readGoals".into(),
        ],
    }
}

#[test]
fn host_adapter_uses_the_module_prepared_name_schema_and_policy_tags() {
    let tools = deferred_connected_tools(
        Arc::new(Config::default()),
        &[presentation(
            "server-1",
            "mcp_weather_read_goals_abc123",
            "legacy-goals",
        )],
    );
    assert_eq!(tools.len(), 1);
    let tool = &tools[0];
    assert_eq!(tool.name(), "mcp_weather_read_goals_abc123");
    assert_eq!(tool.exposure(), ToolExposure::Deferred);
    assert_eq!(tool.permission_level(), PermissionLevel::Execute);
    assert!(tool.external_effect());
    assert_eq!(tool.family(), Some("example/weather"));
    assert_eq!(tool.tags()[0], "mcp.server:example/weather");
    assert_eq!(tool.parameters_schema()["required"], json!(["city"]));
}

#[test]
fn a_recorded_legacy_name_is_restored_as_an_alias() {
    let legacy = "mcp_read_goals_legacy".to_string();
    let recorded: HashSet<String> = [legacy.clone()].into_iter().collect();
    let tools = deferred_connected_tools_with_legacy(
        Arc::new(Config::default()),
        &[presentation(
            "server-1",
            "mcp_weather_read_goals_abc123",
            &legacy,
        )],
        &recorded,
    );
    let names: Vec<&str> = tools.iter().map(|tool| tool.name()).collect();
    assert_eq!(
        names,
        ["mcp_read_goals_legacy", "mcp_weather_read_goals_abc123"]
    );

    let none = deferred_connected_tools_with_legacy(
        Arc::new(Config::default()),
        &[presentation(
            "server-1",
            "mcp_weather_read_goals_abc123",
            &legacy,
        )],
        &HashSet::new(),
    );
    assert_eq!(none.len(), 1);
}

#[test]
fn same_remote_name_on_distinct_servers_keeps_module_names_distinct() {
    let tools = deferred_connected_tools(
        Arc::new(Config::default()),
        &[
            presentation("server-1", "mcp_weather_read_goals_abc123", "legacy-1"),
            presentation("server-2", "mcp_weather_read_goals_def456", "legacy-2"),
        ],
    );
    assert_eq!(tools.len(), 2);
    assert_ne!(tools[0].name(), tools[1].name());
}

#[tokio::test]
async fn unavailable_module_returns_a_tool_error_without_a_host_fallback() {
    let workspace = tempfile::tempdir().expect("temp workspace");
    let config = Config {
        workspace_dir: workspace.path().join("workspace"),
        action_dir: workspace.path().join("workspace"),
        config_path: workspace.path().join("config.toml"),
        ..Default::default()
    };
    let tools = deferred_connected_tools(
        Arc::new(config),
        &[presentation(
            "server-1",
            "mcp_weather_read_goals_abc123",
            "legacy-goals",
        )],
    );
    let result = tools[0].execute(json!({ "city": "London" })).await.unwrap();
    assert!(result.is_error);
    assert!(!result.text().is_empty());
}

#[tokio::test]
async fn deferred_executor_retries_lazy_module_open_after_unavailability() {
    let workspace = tempfile::tempdir().expect("temp workspace");
    let mut config = Config {
        workspace_dir: workspace.path().join("workspace"),
        action_dir: workspace.path().join("workspace"),
        config_path: workspace.path().join("config.toml"),
        ..Default::default()
    };
    config.modules.enabled = true;
    let port = RecoveringModulePort::default();
    let opens = Arc::clone(&port.opens);
    let calls = Arc::clone(&port.calls);
    crate::mcp::module_client::McpConfiguredClient::install_test_port(
        &config,
        Box::new(port.clone()),
    )
    .expect("install module port");
    let tools = deferred_connected_tools(
        Arc::new(config),
        &[presentation(
            "server-1",
            "mcp_weather_forecast_abc123",
            "legacy-forecast",
        )],
    );
    let tool = &tools[0];

    let unavailable = tool
        .execute(json!({"city":"London"}))
        .await
        .expect("unavailable tool result");
    assert!(unavailable.is_error);
    assert_eq!(opens.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert!(calls.lock().unwrap().is_empty());

    port.available
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let recovered = tool
        .execute(json!({"city":"London"}))
        .await
        .expect("recovered tool result");
    assert!(!recovered.is_error, "{}", recovered.text());
    assert_eq!(recovered.text(), "lazy-loaded forecast");
    assert_eq!(opens.load(std::sync::atomic::Ordering::SeqCst), 2);
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        [
            tinymcp_bus::names::methods::NORMALIZE_TOOL_ARGUMENTS,
            tinymcp_bus::names::methods::TOOL_CALL,
            tinymcp_bus::names::methods::RENDER_TOOL_OUTPUT,
        ]
    );
}
