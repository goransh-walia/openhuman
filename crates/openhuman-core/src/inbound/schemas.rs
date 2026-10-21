//! The `inbound` RPC namespace: `inbound.stats`, `inbound.dead`,
//! `inbound.redrive`. Each answers `COMPONENT_DISABLED:eventqueues` when the
//! component is off.

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::core::all::{ControllerFuture, RegisteredController};
use crate::core::runtime::DispatchCtx;
use crate::core::Outcome;
use crate::core::{ControllerSchema, FieldSchema, TypeSchema};

#[derive(Debug, Deserialize)]
struct DeadParams {
    limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct RedriveParams {
    id: String,
}

/// Every `inbound.*` schema.
pub fn all_controller_schemas() -> Vec<ControllerSchema> {
    vec![schemas("stats"), schemas("dead"), schemas("redrive")]
}

/// Every `inbound.*` controller.
pub fn all_registered_controllers() -> Vec<RegisteredController> {
    vec![
        RegisteredController {
            schema: schemas("stats"),
            handler: handle_stats,
        },
        RegisteredController {
            schema: schemas("dead"),
            handler: handle_dead,
        },
        RegisteredController {
            schema: schemas("redrive"),
            handler: handle_redrive,
        },
    ]
}

/// The schema of `inbound.<function>`.
pub fn schemas(function: &str) -> ControllerSchema {
    match function {
        "stats" => ControllerSchema {
            namespace: "inbound",
            function: "stats",
            description: "Inbound event queue depths (ready, delayed, leased, dead) and \
                          outcome counts (delivered, deferred, dropped, dead, ...).",
            inputs: vec![],
            outputs: vec![json_output("result", "Queue depths and outcome counts.")],
        },
        "dead" => ControllerSchema {
            namespace: "inbound",
            function: "dead",
            description: "List inbound events parked in the dead-letter queue.",
            inputs: vec![FieldSchema {
                name: "limit",
                ty: TypeSchema::Option(Box::new(TypeSchema::BoundedU64 {
                    min: 0,
                    max: u64::from(u32::MAX),
                })),
                comment: "Maximum number of items (default 50, at most 500).",
                required: false,
            }],
            outputs: vec![json_output("result", "Dead-lettered items.")],
        },
        "redrive" => ControllerSchema {
            namespace: "inbound",
            function: "redrive",
            description: "Put one dead-lettered inbound event back into the pipeline.",
            inputs: vec![FieldSchema {
                name: "id",
                ty: TypeSchema::String,
                comment: "The dead item's id, as `inbound.dead` lists it.",
                required: true,
            }],
            outputs: vec![json_output("result", "Whether the item was redriven.")],
        },
        _ => ControllerSchema {
            namespace: "inbound",
            function: "unknown",
            description: "Unknown inbound function.",
            inputs: vec![],
            outputs: vec![json_output("error", "Lookup error details.")],
        },
    }
}

async fn config_and_client(
    ctx: &DispatchCtx,
) -> Result<(crate::config::Config, tinybus::module::ModuleClient), String> {
    let config = ctx.load_config().await?;
    let client = crate::modules::ops::client(config.clone());
    Ok((config, client))
}

fn handle_stats(ctx: DispatchCtx, _params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let (config, client) = config_and_client(&ctx).await?;
        to_json(super::ops::stats(&config, &client).await?)
    })
}

fn handle_dead(ctx: DispatchCtx, params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let params = deserialize_params::<DeadParams>(params)?;
        let (config, client) = config_and_client(&ctx).await?;
        to_json(super::ops::dead(&config, &client, params.limit).await?)
    })
}

fn handle_redrive(ctx: DispatchCtx, params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let params = deserialize_params::<RedriveParams>(params)?;
        let (config, client) = config_and_client(&ctx).await?;
        to_json(super::ops::redrive(&config, &client, &params.id).await?)
    })
}

fn deserialize_params<T: DeserializeOwned>(params: Map<String, Value>) -> Result<T, String> {
    serde_json::from_value(Value::Object(params)).map_err(|e| format!("invalid params: {e}"))
}

fn to_json<T: serde::Serialize>(outcome: Outcome<T>) -> Result<Value, String> {
    outcome.into_cli_compatible_json()
}

fn json_output(name: &'static str, comment: &'static str) -> FieldSchema {
    FieldSchema {
        name,
        ty: TypeSchema::Json,
        comment,
        required: true,
    }
}

#[cfg(test)]
#[path = "schemas_tests.rs"]
mod tests;
