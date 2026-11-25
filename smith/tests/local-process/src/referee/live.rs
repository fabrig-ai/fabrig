//! Shared live outcome judgments over outside observations (smith/docs/design/testing-tiers.md, 3).
//! Simulated terminal captures and live exec records use the same delivery
//! checks. No process, clock, credential or filesystem is inspected here.

use crate::fixture::contains;
use skein_fake_checkout::git::Tree;
use skein_io::kernel::Exit;
use skein_lib::Time;
use skein_world::domain::{Expectations, Judge, Referee};
use smith_events::{Delivered, Event, Record, Verdict};

/// The expected report or committed change, supplied by the story's author.
pub struct Outcome {
    pub before: Option<Vec<u8>>,
    pub files: Vec<(Vec<u8>, Vec<u8>)>,
}

impl Outcome {
    /// A change reached a committed head other than the recorded starting head.
    #[must_use]
    pub fn has_new_head(&self, head: Option<&[u8]>) -> bool {
        self.before.as_deref().is_some_and(|before| head.is_some_and(|head| head != before))
    }
}

/// A typed finish terminal records delivery of the change.
#[must_use]
pub fn delivered_finish(record: &Record) -> bool {
    matches!(&record.event, Event::ToolCompleted(tool)
        if tool.tool.as_ref() == b"finish" && tool.delivery == Some(Delivered::Delivered))
}

/// The delivered commit records Smith's delivery identity.
#[must_use]
pub fn carries_delivery_identity(message: &[u8]) -> bool {
    contains(message, b"Smith-Delivery:")
}

/// A prescribed file was observed with precisely the expected bytes.
#[must_use]
pub fn matches_file(found: Option<&[u8]>, expected: &[u8]) -> bool {
    found == Some(expected)
}

/// A settled terminal, trace, commit and working tree observed from outside.
#[derive(Clone)]
pub struct Seen {
    pub exit: Option<Exit>,
    pub errors: Vec<u8>,
    pub shown: Vec<u8>,
    pub records: Vec<Record>,
    pub head: Option<Vec<u8>>,
    pub message: Vec<u8>,
    pub committed: Tree,
    pub working: Tree,
}

impl Expectations for Outcome {
    type Seen = Seen;
    type Name = &'static str;
    type Stimulus = ();

    fn observe(&mut self, seen: Seen, judge: &mut Judge<Self::Name, Self::Stimulus>) {
        judge.check(seen.exit == Some(Exit::Code(0)), "the live outcome exits successfully");
        judge.check(
            std::str::from_utf8(&seen.errors).is_ok_and(|text| text.lines().all(operator_line)),
            "only expected live operator diagnostics",
        );
        judge.check(
            seen.records.iter().any(|record| {
                matches!(&record.event,
            Event::ToolStarted(tool) if tool.tool.as_ref() == b"finish")
            }),
            "finish started",
        );
        judge.check(
            seen.records.iter().any(|record| {
                matches!(&record.event,
            Event::ToolCompleted(tool) if tool.tool.as_ref() == b"finish"
                && tool.verdict == Verdict::Result
                && (self.before.is_none() || delivered_finish(record)))
            }),
            "finish received its accepted or delivered terminal",
        );
        if self.before.is_some() {
            judge.check(self.has_new_head(seen.head.as_deref()), "a new live commit exists");
            judge.check(carries_delivery_identity(&seen.message), "the live commit carries its delivery identity");
            for (path, expected) in &self.files {
                judge.check(
                    matches_file(seen.committed.get(path).map(Vec::as_slice), expected),
                    "the committed file has its prescribed content",
                );
                judge.check(
                    matches_file(seen.working.get(path).map(Vec::as_slice), expected),
                    "the working file has its prescribed content",
                );
            }
            judge.check(contains(&seen.shown, b"Change delivered"), "the person saw delivery");
        }
    }
}

/// Apply live outcome expectations to one immutable outside capture.
#[must_use]
pub fn review(seen: Seen, outcome: Outcome) -> Referee<Outcome> {
    let mut referee = Referee::new(outcome);
    referee.observe(Time::ZERO, seen, &mut Vec::new());
    referee
}

fn operator_line(line: &str) -> bool {
    matches!(
        line,
        "smith: local host started"
            | "smith: agent started"
            | "smith: commands run in process groups and are not contained"
            | "smith: chat chat"
    ) || trust_notice(line)
}

fn trust_notice(line: &str) -> bool {
    line.strip_prefix("smith: trust roots: ").is_some_and(|counts| {
        let fields: Vec<_> = counts.split(", ").collect();
        const ROOT_FIELDS: usize = 4; // taken, skipped, malformed blocks, other blocks.
        fields.len() == ROOT_FIELDS
            && fields.iter().zip([" taken", " skipped", " malformed blocks", " other blocks"]).all(|(field, suffix)| {
                field.strip_suffix(suffix).is_some_and(|count| {
                    count.parse::<u32>().is_ok_and(|count| suffix != " malformed blocks" || count > 0)
                })
            })
    })
}
