//! Outside observations for the live outcome policy, tested with simulated
//! captures and edited records (skein/docs/foundation/testing-strategy.md, 10).
//! No process, clock, credential or filesystem is inspected by this referee.
use skein_fake_checkout::git::Tree;
use skein_io::kernel::Exit;
use skein_lib::Time;
use skein_world::domain::{Expectations, Judge, Referee};
use smith_events::{Delivered, Event, Record, Verdict};

/// The expected report or committed change, supplied by the story's author.
pub struct Outcome {
    pub before: Option<Vec<u8>>,
    pub files: Tree,
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
                && (self.before.is_none() || tool.delivery == Some(Delivered::Delivered)))
            }),
            "finish received its accepted or delivered terminal",
        );
        if let Some(before) = &self.before {
            judge.check(seen.head.as_ref().is_some_and(|head| head != before), "a new live commit exists");
            judge.check(
                seen.message.windows(b"Smith-Delivery:".len()).any(|bytes| bytes == b"Smith-Delivery:"),
                "the live commit carries its delivery identity",
            );
            for (path, expected) in &self.files {
                judge
                    .check(seen.committed.get(path) == Some(expected), "the committed file has its prescribed content");
                judge.check(seen.working.get(path) == Some(expected), "the working file has its prescribed content");
            }
            judge.check(
                seen.shown.windows(b"Change delivered".len()).any(|bytes| bytes == b"Change delivered"),
                "the person saw delivery",
            );
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
        || ["codex", "anthropic"].iter().any(|account| {
            line == format!("smith: warning: {}", smith_local_shell::credentials::Credential::NotSignedIn.line(account))
        })
}

fn trust_notice(line: &str) -> bool {
    line.strip_prefix("smith: trust roots: ").is_some_and(|counts| {
        let fields: Vec<_> = counts.split(", ").collect();
        const ROOT_FIELDS: usize = 4; // taken, skipped, malformed blocks, other blocks.
        fields.len() == ROOT_FIELDS
            && fields
                .iter()
                .zip([" taken", " skipped", " malformed blocks", " other blocks"])
                .all(|(field, suffix)| field.strip_suffix(suffix).is_some_and(|count| count.parse::<u32>().is_ok()))
    })
}
