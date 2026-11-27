//! Skein's test table (`skein/docs/foundation/cli.md`, 8) and the process's
//! fuzzy sweep mode (`skein/docs/design/testing/sweeps.md`, 4.3).

use std::sync::OnceLock;

use skein_shell::{Set, TestTable, Variable, Way};

pub const FUZZY_CAMPAIGN: Variable = Variable {
    name: "SKEIN_TEST_FUZZY_CAMPAIGN",
    set: Set::Test,
    form: "SEED",
    line: "Run each adopted sweep's fresh cases for this campaign seed.",
    absent: "With neither fuzzy variable set, each sweep runs its gate seeds.",
    notes: "A campaign sets it. Use sixteen hex digits with no prefix. A malformed value or both variables set fails the test before its first case. tools/tiers/one-process removes it.",
    way: Way::Reads,
};

pub const FUZZY_CASE: Variable = Variable {
    name: "SKEIN_TEST_FUZZY_CASE",
    set: Set::Test,
    form: "SEED",
    line: "Run one case with this seed, in the campaign's configuration.",
    absent: "With neither fuzzy variable set, each sweep runs its gate seeds.",
    notes: "A campaign's replay sets it. Use sixteen hex digits with no prefix. A malformed value or both variables set fails the test before its first case. tools/tiers/one-process removes it.",
    way: Way::Reads,
};

/// The test variables documented in `skein/docs/design/cli/test-env.md`.
pub const TESTS: TestTable = TestTable { tree: "skein", variables: &[FUZZY_CAMPAIGN, FUZZY_CASE] };

pub(crate) type EnvironmentEntry = (Box<[u8]>, Box<[u8]>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Mode {
    Gate,
    Campaign(u64),
    Replay(u64),
}

impl Mode {
    pub(crate) fn gate(self) -> bool {
        matches!(self, Self::Gate)
    }
}

pub(crate) fn mode() -> Mode {
    static MODE: OnceLock<Result<Mode, String>> = OnceLock::new();
    match MODE.get_or_init(|| mode_in(&capture())) {
        Ok(mode) => *mode,
        Err(why) => crate::fail(why),
    }
}

#[cfg(unix)]
#[expect(
    clippy::disallowed_methods,
    reason = "a test harness: fuzzy sweeps capture their process environment once (skein/docs/foundation/cli.md, 6)"
)]
fn capture() -> Box<[EnvironmentEntry]> {
    skein_shell::environment()
}

#[cfg(not(unix))]
fn capture() -> Box<[EnvironmentEntry]> {
    Box::default()
}

pub(crate) fn mode_in(pairs: &[EnvironmentEntry]) -> Result<Mode, String> {
    match (FUZZY_CAMPAIGN.value(pairs), FUZZY_CASE.value(pairs)) {
        (None, None) => Ok(Mode::Gate),
        (Some(bytes), None) => seed(&FUZZY_CAMPAIGN, bytes).map(Mode::Campaign),
        (None, Some(bytes)) => seed(&FUZZY_CASE, bytes).map(Mode::Replay),
        (Some(campaign), Some(case)) => Err(format!(
            "fuzzy setup: {}=\"{}\" and {}=\"{}\" cannot both be set",
            FUZZY_CAMPAIGN.name,
            campaign.escape_ascii(),
            FUZZY_CASE.name,
            case.escape_ascii()
        )),
    }
}

fn seed(variable: &Variable, bytes: &[u8]) -> Result<u64, String> {
    if bytes.len() == 16 && bytes.iter().all(u8::is_ascii_hexdigit) {
        let text = core::str::from_utf8(bytes).expect("ASCII hex is UTF-8");
        return Ok(u64::from_str_radix(text, 16).expect("sixteen hex digits fit u64"));
    }
    Err(format!("fuzzy setup: {}=\"{}\" must be sixteen hex digits", variable.name, bytes.escape_ascii()))
}
