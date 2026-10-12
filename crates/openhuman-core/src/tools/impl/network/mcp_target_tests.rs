use super::*;

use std::{
    result::Result,
    sync::{Arc, Mutex},
};

use crate::mcp::module_client::{McpConfiguredClient, McpModulePort};
use async_trait::async_trait;
use serde_json::{json, Value};
use tinymcp_bus::{
    McpToolPresentation, OpenConfiguredResponse, PrepareAgentToolsRequest,
    ServerArgumentsValidation, TransitionStatus, ValidateServerArgumentsRequest,
};

#[derive(Clone, Default)]
struct PreparePort {
    requests: Arc<Mutex<Vec<PrepareAgentToolsRequest>>>,
    transform_error: Option<crate::modules::client::ModuleCallError>,
}

#[async_trait]
impl McpModulePort for PreparePort {
    async fn open_configured(
        &self,
        _: &str,
        _: &tinymcp_bus::McpClientConfig,
    ) -> Result<OpenConfiguredResponse, crate::modules::client::ModuleCallError> {
        Ok(OpenConfiguredResponse {
            object_path: "/test/mcp".to_owned(),
            transition_id: "open-test".to_owned(),
        })
    }

    async fn close(
        &self,
        _: &str,
    ) -> Result<Option<String>, crate::modules::client::ModuleCallError> {
        Ok(None)
    }

    async fn transition_status(
        &self,
        _: &str,
    ) -> Result<TransitionStatus, crate::modules::client::ModuleCallError> {
        Ok(TransitionStatus::Complete)
    }

    async fn call_object(
        &self,
        _: &str,
        _: &str,
        _: Value,
        _: bool,
    ) -> Result<Value, crate::modules::client::ModuleCallError> {
        Err(crate::modules::client::ModuleCallError::IncompatibleContract)
    }

    async fn prepare_agent_tools(
        &self,
        request: PrepareAgentToolsRequest,
    ) -> Result<Vec<McpToolPresentation>, crate::modules::client::ModuleCallError> {
        self.requests.lock().unwrap().push(request.clone());
        Ok(request
            .sources
            .into_iter()
            .flat_map(|source| {
                source
                    .tools
                    .into_iter()
                    .map(move |tool| McpToolPresentation {
                        name: format!("owner:{}:{}", source.server_id, tool.name),
                        legacy_name: format!("legacy:{}:{}", source.server_id, tool.name),
                        server_id: source.server_id.clone(),
                        remote_name: tool.name.clone(),
                        family: source.family.clone(),
                        description: String::new(),
                        parameters: Value::Null,
                        deferred: false,
                        tags: vec![
                            format!("mcp.server:{}", source.family),
                            format!("mcp.server_id:{}", source.server_id),
                            format!("mcp.tool:{}", tool.name),
                        ],
                    })
            })
            .collect())
    }

    async fn validate_server_arguments(
        &self,
        _: ValidateServerArgumentsRequest,
    ) -> Result<ServerArgumentsValidation, crate::modules::client::ModuleCallError> {
        std::future::ready(Err(
            crate::modules::client::ModuleCallError::IncompatibleContract,
        ))
        .await
    }

    async fn normalize_tool_arguments(
        &self,
        arguments: Option<Value>,
    ) -> Result<serde_json::Map<String, Value>, crate::modules::client::ModuleCallError> {
        match arguments.unwrap_or(Value::Null) {
            Value::Null => Ok(serde_json::Map::new()),
            Value::Object(arguments) => Ok(arguments),
            _ => Err(crate::modules::client::ModuleCallError::ModuleFault),
        }
    }

    async fn transform_text(
        &self,
        request: tinymcp_bus::TransformTextRequest,
    ) -> Result<String, crate::modules::client::ModuleCallError> {
        match self.transform_error {
            Some(error) => Err(error),
            None => Ok(request.text),
        }
    }
}

#[tokio::test]
async fn valid_mcp_identifier_reports_module_unavailable_without_invalid_input_label() {
    let workspace = tempfile::tempdir().unwrap();
    let mut config = Config::default();
    config.workspace_dir = workspace.path().to_path_buf();
    config.modules.enabled = true;
    McpConfiguredClient::install_test_port(
        &config,
        Box::new(PreparePort {
            transform_error: Some(crate::modules::client::ModuleCallError::Unavailable),
            ..PreparePort::default()
        }),
    )
    .unwrap();
    let tool = mcp_call_tool(Arc::new(config), Arc::new(SecurityPolicy::default()));

    let result = tool
        .execute_with_options(
            json!({"server": "docs", "tool": "search", "arguments": {}}),
            ToolCallOptions::default(),
        )
        .await
        .unwrap();

    assert!(result.is_error);
    assert!(result.output().contains("TinyMCP module is unavailable"));
    assert!(!result.output().contains("invalid"));
    let outcome = tinymcp_bus::McpCallOutcome::from_metadata(result.metadata.as_ref().unwrap())
        .expect("sanitized outcome remains available");
    assert_eq!(outcome.error.unwrap().code, tinymcp_bus::errors::BUS);
}

#[tokio::test]
async fn indirect_mcp_target_uses_module_prepared_identity_and_facts() {
    let workspace = tempfile::tempdir().unwrap();
    let mut config = Config::default();
    config.workspace_dir = workspace.path().to_path_buf();
    config.modules.enabled = true;
    let port = PreparePort::default();
    McpConfiguredClient::install_test_port(&config, Box::new(port.clone())).unwrap();
    let tool = mcp_call_tool(Arc::new(config), Arc::new(SecurityPolicy::default()));

    let call = tool
        .indirect_target_async(&json!({
            "server": "docs",
            "tool": "whoami",
            "arguments": {"query": "status"}
        }))
        .await
        .unwrap()
        .expect("module returns the target presentation");

    assert_eq!(call.target.name, "owner:docs:whoami");
    assert_eq!(call.target.family.as_deref(), Some("docs"));
    assert_eq!(
        call.target.tags,
        ["mcp.server:docs", "mcp.server_id:docs", "mcp.tool:whoami"]
    );
    assert_eq!(call.arguments, Some(json!({"query": "status"})));
    let requests = port.requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    let source = &requests[0].sources[0];
    assert_eq!(source.server_id, "docs");
    assert_eq!(source.label, "docs");
    assert_eq!(source.family, "docs");
    assert_eq!(source.tools[0].name, "whoami");
}

#[tokio::test]
async fn indirect_mcp_target_fails_closed_when_modules_are_disabled() {
    let workspace = tempfile::tempdir().unwrap();
    let mut config = Config::default();
    config.workspace_dir = workspace.path().to_path_buf();
    config.modules.enabled = false;
    let tool = mcp_call_tool(Arc::new(config), Arc::new(SecurityPolicy::default()));

    let result = tool
        .indirect_target_async(&json!({
            "server": "docs",
            "tool": "whoami",
            "arguments": {}
        }))
        .await;

    assert!(
        result.is_err(),
        "unavailable owner must not omit target facts"
    );
}
