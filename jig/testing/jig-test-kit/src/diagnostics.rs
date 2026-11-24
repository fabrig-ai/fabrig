//! Diagnostics for the reference binary's stories (jig/docs/design/protocol/service-testing.md, 1).
use skein_io::kernel::Exit;

pub const STDERR_TAIL: usize = 4096;

/// Never repeat a printed link or an authorization URL in a failure. Scan before
/// cutting the tail so a URL crossing its boundary cannot leave its query behind.
#[must_use]
pub fn premature_exit(status: Exit, milestone: &str, stderr: &[u8]) -> String {
    let start = stderr.len().saturating_sub(STDERR_TAIL);
    let mut safe = Vec::new();
    let mut at = 0;
    while at < stderr.len() {
        let rest = &stderr[at..];
        if rest.starts_with(b"http://") || rest.starts_with(b"https://") {
            let length = rest
                .iter()
                .position(|byte| byte.is_ascii_whitespace() || b"\"'<>".contains(byte))
                .unwrap_or(rest.len());
            at += length;
            if at > start {
                safe.extend_from_slice(b"[address omitted]");
            }
        } else {
            if at >= start {
                safe.push(stderr[at]);
            }
            at += 1;
        }
    }
    let tail = &safe[safe.len().saturating_sub(STDERR_TAIL)..];
    format!(
        "binary exited before SIGTERM: status={status:?}; last milestone={milestone}; stderr tail (up to {STDERR_TAIL} bytes):\n{}",
        String::from_utf8_lossy(tail)
    )
}
