//! The system prompt a resumed thread is sent.
//!
//! A resumed thread's prefix is frozen: the runtime restores the system
//! messages its transcript recorded and sends them verbatim, which keeps the
//! provider's prefix cache warm across a restart. A turn can still offer a
//! replacement, which the runtime applies only when it differs from what is
//! recorded; a thread-bound session then seals the transcript generation it
//! was on and carries its conversation into the next. Two things offer one:
//!
//! - the permanent-tool catalogue (`permanent.rs`), whenever it changes;
//! - the whole prompt, rendered afresh, when the agent's definition sets
//!   `refresh_prompt_on_resume`, so prompt edits reach existing threads too.
//!
//! Whatever is offered, the turn's transcript snapshot boundary moves with
//! it: the request starts with the offered prefix, not the recorded one.
//!
//! When the whole prompt is replaced, the user's message on that turn opens
//! with a one-time note saying so (`PROMPT_UPDATE_NOTE`). The conversation
//! before it was written under the old prompt, and a model keeps to what it
//! already said there (that something is "coming soon", say) over
//! instructions that changed since. Only a prompt rendered on that turn
//! counts: the catalogue changing under a prompt the turn kept says nothing.

use std::sync::PoisonError;

use tinyagents_definition::PromptSource;
use tinyagents_runtime::{
    PrefixSnapshot, RuntimeError, SessionStateView, SessionTurnRequest, TurnPreparation,
};
use tinyinference_llm::message::Message;

use super::OpenHumanTurnPrelude;
use crate::agent::message_convert::{user_message_from_text, user_text_with_markers};
use crate::agent::tinyagents::middleware::turn_context::TranscriptSnapshotSink;

/// Prepended to the user's message on the turn a resumed thread's prompt is
/// replaced; the turn's record keeps it, so later turns see where the change
/// happened.
pub(super) const PROMPT_UPDATE_NOTE: &str = "[instructions update] Your system instructions changed since the earlier messages in this conversation. Where they differ from anything said before, including what you said you could or could not do, follow your current instructions. This is an update, not a request: respond to the user's message below.";

impl OpenHumanTurnPrelude {
    /// This turn's tool snapshot and, when it changes, its prefix: what
    /// `prepare` builds, plus the refreshes a resumed thread's recorded
    /// prefix may take (see the module docs). `snapshot`'s request boundary
    /// is rebased onto the prefix offered, and `request` carries the
    /// `PROMPT_UPDATE_NOTE` when the prompt is replaced.
    pub(super) async fn prepare_turn(
        &self,
        new_session: bool,
        view: &SessionStateView<'_>,
        snapshot: &TranscriptSnapshotSink,
        request: &mut SessionTurnRequest,
    ) -> Result<TurnPreparation, RuntimeError> {
        let mut preparation = self
            .prepare(new_session)
            .await
            .map_err(|error| RuntimeError::Driver(error.to_string()))?;
        // Whether this turn rendered the current prompt. Only then can a
        // changed prefix mean the prompt was replaced: the permanent-tool
        // catalogue below also changes the prefix of a prompt that was kept.
        let rendered = if view.resumed && self.refreshes_prompt_on_resume() {
            preparation.prefix = self.current_prompt_prefix(view).await;
            preparation.prefix.is_some()
        } else {
            false
        };
        self.refresh_permanent_prefix(&mut preparation, view.prefix);
        if let Some(prefix) = preparation.prefix.as_ref() {
            // The runtime sends `prefix` in place of the recorded one, so the
            // rounds this run produces start that much further on (or back).
            let mut snapshot = snapshot.lock().unwrap_or_else(PoisonError::into_inner);
            snapshot.request_base_len = snapshot
                .request_base_len
                .saturating_sub(view.prefix.messages().len())
                + prefix.messages().len();
        }
        if rendered {
            let changed = preparation
                .prefix
                .as_ref()
                .is_some_and(|prefix| prefix.messages() != view.prefix.messages());
            if changed {
                tracing::info!(
                    thread_id = ?self.thread_id,
                    recorded_segments = view.prefix.messages().len(),
                    "[session] refresh_prompt_on_resume: the resumed thread's prompt is out of date; it continues under the current one"
                );
                let text = user_text_with_markers(&request.input);
                request.input = user_message_from_text(&format!("{PROMPT_UPDATE_NOTE}\n\n{text}"));
            } else {
                tracing::debug!(
                    thread_id = ?self.thread_id,
                    "[session] refresh_prompt_on_resume: the resumed thread keeps its prompt"
                );
            }
        }
        Ok(preparation)
    }

    /// Whether the agent's definition asks for a resumed thread to take up
    /// the prompt rendered now (`refresh_prompt_on_resume`).
    fn refreshes_prompt_on_resume(&self) -> bool {
        self.definition
            .as_ref()
            .is_some_and(|definition| definition.extensions.refresh_prompt_on_resume)
    }

    /// The prompt this agent renders now, offered in place of the prefix the
    /// resumed thread in `view` restored from its transcript; `None` keeps
    /// the recorded prompt.
    ///
    /// It is kept when:
    ///
    /// - that prefix is empty while the history still opens on system rows:
    ///   the extent of the recorded prompt could not be recovered (a
    ///   compacted legacy head), so its old instructions are still part of
    ///   the history, and a new prompt beside them would leave them in place;
    /// - the prompt renders live state (anything but a verbatim prompt) and
    ///   the connected integrations have not been read this session: a prompt
    ///   rendered without them would replace a complete one;
    /// - the prompt cannot be rendered: the thread goes on under the one it
    ///   has rather than failing its turn.
    ///
    /// It is rendered as a new session's is (`cold_prompt_prefix`), so an
    /// unchanged prompt is offered byte for byte as recorded and writes
    /// nothing.
    async fn current_prompt_prefix(&self, view: &SessionStateView<'_>) -> Option<PrefixSnapshot> {
        let unknown_extent = view.prefix.messages().is_empty()
            && matches!(view.history.first(), Some(Message::System(_)));
        if unknown_extent {
            tracing::warn!(
                thread_id = ?self.thread_id,
                "[session] refresh_prompt_on_resume: the resumed thread's recorded prompt has no known extent; keeping it"
            );
            return None;
        }
        let verbatim = self.definition.as_ref().is_some_and(|definition| {
            matches!(definition.system_prompt, PromptSource::Verbatim(_))
        });
        let hydrated = {
            let mutable = self.mutable.lock().unwrap_or_else(PoisonError::into_inner);
            mutable.connected_integrations_initialized
                && mutable.connected_integrations_authoritative
        };
        if !verbatim && !hydrated {
            tracing::debug!(
                thread_id = ?self.thread_id,
                "[session] refresh_prompt_on_resume: integrations not read yet; keeping the recorded prompt"
            );
            return None;
        }
        match self.cold_prompt_prefix().await {
            Ok(prefix) => Some(prefix.refreshing()),
            Err(error) => {
                tracing::warn!(
                    thread_id = ?self.thread_id,
                    %error,
                    "[session] refresh_prompt_on_resume: the current prompt did not render; keeping the recorded one"
                );
                None
            }
        }
    }
}

#[cfg(test)]
#[path = "prompt_refresh_tests.rs"]
mod tests;
