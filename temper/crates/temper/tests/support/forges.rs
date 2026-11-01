//! Runtime forge calls for process stories (temper/docs/design/draft/playground-reset.md, 8).
use super::{Ended, ctl::Call};

#[must_use]
pub fn make_here() -> Call {
    Call::new(&["forge", "make", "here"])
}

#[must_use]
pub fn make_github() -> Call {
    Call::new(&["forge", "make", "github"])
}

#[must_use]
pub fn make_forgejo(api: &str, git: &str) -> Call {
    Call::new(&["forge", "make", "forgejo", api, git])
}

#[must_use]
pub fn remove(forge: u16) -> Call {
    Call::new(&["forge", "remove", &forge.to_string()])
}

#[must_use]
pub fn playground_start() -> Call {
    Call::new(&["playground", "start"])
}

#[must_use]
pub fn playground_reset() -> Call {
    Call::new(&["playground", "reset"])
}

/// Read a completed make's text or JSON answer, including a kept answer.
#[must_use]
pub fn made(call: &Ended) -> u16 {
    assert_eq!(call.exit, Some(skein_io::kernel::Exit::Code(0)), "forge make succeeded");
    let forge = if let Ok(document) = serde_json::from_slice::<serde_json::Value>(&call.stdout) {
        let outcome = document.get("outcome").unwrap_or(&document);
        u16::try_from(outcome["made"]["forge"].as_u64().expect("made forge number")).expect("u16 forge")
    } else {
        let text = std::str::from_utf8(&call.stdout).expect("forge make text");
        let text = text.trim().strip_prefix("kept: forge-make ").unwrap_or(text.trim());
        text.strip_prefix("made: forge ").expect("made forge answer").parse().expect("u16 forge")
    };
    assert_ne!(forge, 0, "forge numbers start at one");
    forge
}
