//! Print invocation shared by hosted and executable stories (smith/docs/design/testing.md, 7).

/// The synthetic conversation identity.
pub const SESSION: &str = "12345678-1234-1234-1234-123456789abc";

/// The print face with bounded scripted tools and a fixed session.
#[must_use]
pub fn arguments() -> Vec<String> {
    [
        "-p",
        "--input-format",
        "stream-json",
        "--output-format",
        "stream-json",
        "--verbose",
        "--setting-sources",
        "user",
        "--model",
        "test-model",
        "--append-system-prompt-file",
        "unread",
        "--mcp-config",
        "run/mcp.json",
        "--strict-mcp-config",
        "--permission-prompt-tool",
        "mcp__smith__prompt",
        "--tools",
        "Read,Bash",
        "--session-id",
        SESSION,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}
