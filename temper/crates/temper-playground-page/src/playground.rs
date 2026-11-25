//! The playground's frame and words (temper/docs/design/playground.md, 2 and 6).
use skein_lib::{Duration, Time};
use skein_ui_web::bridge::{self, Speed};
use temper_client_shell_web::Control;

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

const MIB: u64 = 1024 * 1024;
const PACES: [(&str, Speed); 4] = [(PAUSE, Speed::PAUSED), (HALF, Speed(500)), (ONE, Speed::ONE), (TEN, Speed(10_000))];

fn elapsed_millis(now: Time) -> u64 {
    now.as_nanos() / 1_000_000
}

fn memory_tenths(bytes: u64) -> u64 {
    bytes / MIB * 10 + bytes % MIB * 10 / MIB
}

fn slice_micros(slice: Duration) -> u64 {
    slice.as_nanos() / 1000
}

/// The figure values in their displayed units, initialized to index.html's figures.
struct Figures {
    millis: u64,
    tenths: u64,
    micros: u64,
    pace: &'static str,
    speed: Speed,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct FigureWrites {
    millis: Option<u64>,
    tenths: Option<u64>,
    micros: Option<u64>,
    pace: Option<&'static str>,
    pressed: [Option<bool>; PACES.len()],
}

impl Figures {
    fn new() -> Self {
        Self { millis: 0, tenths: 0, micros: 0, pace: speed_words(Speed::ONE), speed: Speed::ONE }
    }

    fn update(&mut self, reading: Option<(Time, u64, Duration)>, speed: Speed, hidden: bool) -> FigureWrites {
        let mut writes = FigureWrites::default();
        if let Some((now, memory, slice)) = reading {
            let millis = elapsed_millis(now);
            let tenths = memory_tenths(memory);
            let micros = slice_micros(slice);
            writes.millis = (self.millis != millis).then_some(millis);
            writes.tenths = (self.tenths != tenths).then_some(tenths);
            writes.micros = (self.micros != micros).then_some(micros);
            self.millis = millis;
            self.tenths = tenths;
            self.micros = micros;
        }
        let pace = if hidden { "Paused (tab hidden)" } else { speed_words(speed) };
        writes.pace = (self.pace != pace).then_some(pace);
        writes.pressed = PACES.map(|(_, control)| {
            let pressed = speed == control;
            (pressed != (self.speed == control)).then_some(pressed)
        });
        self.pace = pace;
        self.speed = speed;
        writes
    }
}

fn time_words(millis: u64) -> String {
    let seconds = millis / 1000;
    format!("World time {:02}:{:02}:{:02}.{:03}", seconds / 3600, seconds / 60 % 60, seconds % 60, millis % 1000)
}

fn memory_words(tenths: u64) -> String {
    format!("Memory high water {}.{} MiB", tenths / 10, tenths % 10)
}

fn slice_words(micros: u64) -> String {
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
    let page = std::str::from_utf8(temper_client_domain_forge::sign_in::LINK_PAGE).expect("ASCII link page");
    let path = rest
        .strip_prefix("jig/v1/sign-in/link?secret=")
        .or_else(|| rest.strip_prefix(page.strip_prefix('/').expect("absolute link page"))?.strip_prefix("?secret="))?;
    if path.is_empty() || path.contains(['#', '\\']) || line.bytes().any(|byte| byte.is_ascii_control()) {
        return None;
    }
    // `rest` is a suffix of `line`; keep the slash before it, without copying the secret.
    line.get(line.len().checked_sub(rest.len())?.checked_sub(1)?..)
}
