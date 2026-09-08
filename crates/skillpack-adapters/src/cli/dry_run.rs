//! DryRun guard — wraps side-effectful operations.
//!
//! When `DryRun(true)`, operations are logged but not executed.
//! When `DryRun(false)`, the closure is invoked and its result is returned.

/// Guard type for dry-run mode.
///
/// Wraps a `bool` where `true` means "simulate only".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DryRun(pub bool);

impl DryRun {
    /// Execute `f` only when not in dry-run mode.
    ///
    /// In dry-run (`DryRun(true)`): logs the intent via `tracing::info!` and
    /// returns `R::default()` without invoking `f`.
    ///
    /// In live mode (`DryRun(false)`): executes `f` and returns its result.
    pub fn perform<F: FnOnce() -> R, R: Default>(&self, description: &str, f: F) -> R {
        if self.0 {
            tracing::info!(target: "dry_run", "would: {}", description);
            R::default()
        } else {
            f()
        }
    }

    /// Execute a fallible side effect only when not in dry-run mode.
    ///
    /// In dry-run mode: logs intent and returns `Ok(R::default())`.
    /// In live mode: calls `f` and returns its `Result`.
    pub fn perform_fallible<F, R, E>(&self, description: &str, f: F) -> Result<R, E>
    where
        F: FnOnce() -> Result<R, E>,
        R: Default,
    {
        if self.0 {
            tracing::info!(target: "dry_run", "would: {}", description);
            Ok(R::default())
        } else {
            f()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perform_dry_run_skips_closure_and_returns_default() {
        let dr = DryRun(true);
        let mut called = false;
        let result: i32 = dr.perform("test op", || {
            called = true;
            42
        });
        assert!(!called, "closure must not be called in dry-run mode");
        assert_eq!(result, 0, "must return i32::default() in dry-run mode");
    }

    #[test]
    fn perform_live_mode_calls_closure() {
        let dr = DryRun(false);
        let result: i32 = dr.perform("test op", || 99);
        assert_eq!(result, 99);
    }

    #[test]
    fn perform_fallible_dry_run_returns_ok_without_calling_closure() {
        let dr = DryRun(true);
        let mut called = false;
        let result: Result<(), String> = dr.perform_fallible("test op", || {
            called = true;
            Err("should not be called".to_string())
        });
        assert!(!called, "closure must not be called in dry-run mode");
        assert!(result.is_ok());
    }

    #[test]
    fn perform_fallible_live_mode_propagates_closure_result() {
        let dr = DryRun(false);

        let ok_result: Result<(), String> = dr.perform_fallible("test op", || Ok(()));
        assert!(ok_result.is_ok());

        let err_result: Result<(), String> =
            dr.perform_fallible("test op", || Err("boom".to_string()));
        assert_eq!(err_result.unwrap_err(), "boom");
    }
}
