//! One path for agent-created agent jobs, shared by the `cron` and `schedule`
//! tools: origin-aware defaults, delivery validation, persistence.
//!
//! A job created inside a conversation returns its output to that
//! conversation: `session_target: current`, `delivery.mode: origin`, with the
//! conversation snapshot stored on the job. A job created outside one (CLI,
//! automation) keeps the old defaults, `isolated` + `proactive`. A target or
//! delivery the caller states explicitly always wins.

use crate::config::Config;
use {
    crate::cron, tinyflows_schedule::types::delivery_mode, tinyflows_schedule::types::CronJob,
    tinyflows_schedule::types::DeliveryConfig, tinyflows_schedule::types::JobOrigin,
    tinyflows_schedule::types::Schedule, tinyflows_schedule::types::SessionTarget,
    tinyflows_schedule::validation, tinyflows_schedule::AgentJobSpec,
};

/// What an agent job creation request carries; everything defaults.
pub struct AgentJobInput {
    pub name: Option<String>,
    pub schedule: Schedule,
    pub prompt: String,
    pub session_target: Option<SessionTarget>,
    pub model: Option<String>,
    pub delivery: Option<DeliveryConfig>,
    pub delete_after_run: bool,
}

/// The `allowed_users` list configured for a channel, by name. `None` when the
/// channel is unknown or unconfigured; an empty list means any sender.
fn allowed_users_for_channel<'a>(config: &'a Config, channel: &str) -> Option<&'a [String]> {
    let ch = channel.trim().to_ascii_lowercase();
    let cc = &config.channels_config;
    match ch.as_str() {
        "telegram" => cc.telegram.as_ref().map(|c| c.allowed_users.as_slice()),
        "discord" => cc.discord.as_ref().map(|c| c.allowed_users.as_slice()),
        "slack" => cc.slack.as_ref().map(|c| c.allowed_users.as_slice()),
        "mattermost" => cc.mattermost.as_ref().map(|c| c.allowed_users.as_slice()),
        "matrix" => cc.matrix.as_ref().map(|c| c.allowed_users.as_slice()),
        "irc" => cc.irc.as_ref().map(|c| c.allowed_users.as_slice()),
        "lark" => cc.lark.as_ref().map(|c| c.allowed_users.as_slice()),
        "dingtalk" => cc.dingtalk.as_ref().map(|c| c.allowed_users.as_slice()),
        "qq" => cc.qq.as_ref().map(|c| c.allowed_users.as_slice()),
        _ => None,
    }
}

/// Validate `delivery` at creation time: the module rules
/// (`tinyflows_schedule::validation::validate_delivery`) plus the channel's
/// configured `allowed_users`, which only the host knows.
fn validate_delivery_for_config(
    config: &Config,
    delivery: &DeliveryConfig,
    origin: Option<&JobOrigin>,
) -> Result<(), String> {
    let allowed = delivery
        .channel
        .as_deref()
        .and_then(|channel| allowed_users_for_channel(config, channel));
    validation::validate_delivery(delivery, origin, allowed)
}

/// Create an agent job from the current turn.
///
/// `origin` is the conversation to bind the job to (see
/// [`cron::origin::current_job_origin`]). It is stored on the job only when
/// something uses it (`session_target: current` or `delivery: origin`), so an
/// explicitly non-origin job keeps its old trust and routing.
pub fn create_agent_job(
    config: &Config,
    input: AgentJobInput,
    origin: Option<JobOrigin>,
) -> Result<CronJob, String> {
    if cron::limits::enforced(config) && origin.is_none() {
        return Err("cron jobs can only be created from a conversation".to_string());
    }
    let session_target = input.session_target.unwrap_or(if origin.is_some() {
        SessionTarget::Current
    } else {
        SessionTarget::Isolated
    });
    let delivery = input
        .delivery
        .unwrap_or_else(|| validation::default_delivery(origin.as_ref()));

    validate_delivery_for_config(config, &delivery, origin.as_ref())?;
    validation::check_origin_requirements(&session_target, &delivery, origin.as_ref())?;

    let uses_origin = matches!(session_target, SessionTarget::Current)
        || delivery
            .mode
            .trim()
            .eq_ignore_ascii_case(delivery_mode::ORIGIN);
    let origin = origin.filter(|_| uses_origin);

    tracing::debug!(
        session_target = session_target.as_str(),
        delivery_mode = %delivery.mode,
        origin = origin.as_ref().map(JobOrigin::kind_str).unwrap_or("none"),
        "[cron] creating agent job"
    );

    let mut spec = AgentJobSpec::new(input.schedule, input.prompt);
    spec.name = input.name;
    spec.session_target = session_target;
    spec.model = input.model;
    spec.delivery = Some(delivery);
    spec.delete_after_run = input.delete_after_run;
    spec.origin = origin;
    cron::add_agent_job_from_spec(config, spec).map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "job_builder_tests.rs"]
mod tests;
