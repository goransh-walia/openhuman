#[cfg(not(feature = "security-module"))]
use tinytools::status::POLICY_BLOCKED_MARKER;

use super::types::SecurityPolicy;

impl SecurityPolicy {
    /// Expand a leading `~/` to the user's home directory. Delegates to
    /// [`crate::config::expand_tilde`] — the single source of truth —
    /// so policy and config expand paths byte-for-byte identically (and both
    /// produce platform-native separators; see issue #3353).
    pub(super) fn expand_tilde(&self, path: &str) -> String {
        crate::config::expand_tilde(path)
    }

    /// Validate a path for file I/O: string checks, canonicalize, workspace containment,
    /// and forbidden-path check on the resolved path.
    /// Returns the canonical `PathBuf` on success.
    #[cfg(feature = "security-module")]
    pub async fn validate_path(&self, path: &str) -> Result<PathBuf, String> {
        self.validate_native_path(path, false).await
    }

    /// Without the module there is nothing to ask: file access is denied.
    #[cfg(not(feature = "security-module"))]
    pub async fn validate_path(&self, path: &str) -> Result<PathBuf, String> {
        log::debug!(
            "[security][path] validate_path denied: module feature disabled path_len={}",
            path.len()
        );
        Err(format!(
            "{POLICY_BLOCKED_MARKER} TinySecurity path authorization unavailable; file access denied"
        ))
    }

    /// Like `validate_path` but canonicalizes the parent directory.
    /// Use for write operations where the target file may not yet exist.
    /// Does NOT require the parent directory to exist — walks up to the deepest
    /// existing ancestor and checks that for symlink escapes.
    /// Returns the canonical full path (parent resolved + filename appended).
    #[cfg(feature = "security-module")]
    pub async fn validate_parent_path(&self, path: &str) -> Result<PathBuf, String> {
        self.validate_native_path(path, true).await
    }

    /// Without the module there is nothing to ask: file access is denied.
    #[cfg(not(feature = "security-module"))]
    pub async fn validate_parent_path(&self, path: &str) -> Result<PathBuf, String> {
        log::debug!(
            "[security][path] validate_parent_path denied: module feature disabled path_len={}",
            path.len()
        );
        Err(format!(
            "{POLICY_BLOCKED_MARKER} TinySecurity path authorization unavailable; file access denied"
        ))
    }
}

#[cfg(all(test, feature = "security-module"))]
#[path = "path_checks_native_tests.rs"]
mod native_tests;
