//! The playground's frame and words (temper/docs/design/playground.md, 2 and 6).
use skein_lib::{Duration, Time};
use skein_ui_web::bridge::{self, Speed};

#[cfg(test)]
mod tests;
#[cfg(target_arch = "wasm32")]
pub mod web;

pub const APP: &str = "app";
pub const FRAME: &str = "playground-frame";
pub const LOADING: &str = "playground-loading";
pub const DESCRIPTION: &str = "playground-description";
pub const STORY: &str = "playground-story";
pub const PAUSE: &str = "playground-pause";
pub const HALF: &str = "playground-half";
pub const ONE: &str = "playground-one";
pub const TEN: &str = "playground-ten";
pub const RESET: &str = "playground-reset";
pub const LOG: &str = "playground-log";
pub const TIME: &str = "playground-time";
pub const MEMORY: &str = "playground-memory";
pub const SLICE: &str = "playground-slice";
pub const PACE: &str = "playground-pace";
pub const VERSION: &str = "playground-version";
pub const FAILURE: &str = "playground-failure";
pub const FAILURE_WORDS: &str = "playground-failure-words";
pub const FAILURE_LOG: &str = "playground-failure-log";
pub const START_OVER: &str = "playground-start-over";

/// Descriptions are the stories' own data; the page links no world.
pub const STORIES: &[(&str, &str)] = &[("a", include_str!("../../temper-playground/stories/a/description.txt"))];

/// The frame's reports (temper/docs/design/playground.md, 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    Pause,
    Speed(Speed),
    Reset,
    Story(&'static str),
    Log,
}

/// The report for a frame element; a story choice must name an offered story.
#[must_use]
pub fn control(id: &str, story: &str) -> Option<Control> {
    match id {
        PAUSE => Some(Control::Pause),
        HALF => Some(Control::Speed(Speed(500))),
        ONE => Some(Control::Speed(Speed::ONE)),
        TEN => Some(Control::Speed(Speed(10_000))),
        RESET | START_OVER => Some(Control::Reset),
        LOG => Some(Control::Log),
        STORY => STORIES.iter().find(|(name, _)| *name == story).map(|(name, _)| Control::Story(name)),
        _ => None,
    }
}

#[must_use]
pub fn time_words(now: Time) -> String {
    let millis = now.as_nanos() / 1_000_000;
    let seconds = millis / 1000;
    format!("World time {:02}:{:02}:{:02}.{:03}", seconds / 3600, seconds / 60 % 60, seconds % 60, millis % 1000)
}

#[must_use]
pub fn memory_words(bytes: u64) -> String {
    const MIB: u64 = 1024 * 1024;
    format!("Memory high water {}.{} MiB", bytes / MIB, bytes % MIB * 10 / MIB)
}

#[must_use]
pub fn slice_words(slice: Duration) -> String {
    let micros = slice.as_nanos() / 1000;
    format!("Longest slice {}.{:03} ms", micros / 1000, micros % 1000)
}

#[must_use]
pub fn speed_words(speed: Speed) -> &'static str {
    match speed {
        Speed::PAUSED => "Paused",
        Speed(500) => "½×",
        Speed::ONE => "1×",
        Speed(10_000) => "10×",
        Speed(_) => "Custom speed",
    }
}

/// Only a sign-in link printed by `temper serve` on standard output can start the client.
/// The origin belongs to the world; only the path and query enter the page's address.
#[must_use]
pub fn printed_link(printed: &bridge::Printed) -> Option<&str> {
    if printed.process.as_ref() != b"temper serve" || printed.stream != bridge::Stream::Output {
        return None;
    }
    let line = std::str::from_utf8(&printed.line).ok()?;
    let address = line.strip_prefix("http://").or_else(|| line.strip_prefix("https://"))?;
    let (authority, rest) = address.split_once('/')?;
    if authority.is_empty() || authority.contains(['?', '#', '@', '\\']) || line.chars().any(char::is_whitespace) {
        return None;
    }
    let path = rest.strip_prefix("jig/v1/sign-in/link?secret=")?;
    if path.is_empty() || path.contains(['#', '\\']) || line.bytes().any(|byte| byte.is_ascii_control()) {
        return None;
    }
    // `rest` is a suffix of `line`; keep the slash before it, without copying the secret.
    line.get(line.len().checked_sub(rest.len())?.checked_sub(1)?..)
}
