//! Raw microphone capture for the always-on loop.
//!
//! The `cpal` stream, its realtime callback and the bounded-queue forwarding
//! live in `tinyvoice::capture`. This file supplies the host's part: the
//! microphone-permission policy. Downmixing and resampling live in the
//! processor, not in the callback.

pub(super) use tinyvoice::capture::{CaptureFormat, RawChunk};

/// Spawn the dedicated cpal capture thread. Blocks until the stream is set up
/// (or fails), mirroring `audio_capture::start_recording`'s readiness handshake.
pub(super) fn spawn_capture_thread(
    tx: tokio::sync::mpsc::Sender<RawChunk>,
) -> Result<CaptureFormat, String> {
    tinyvoice::capture::spawn_capture_thread(tx, || Ok(())).map_err(|e| e.to_string())
}
