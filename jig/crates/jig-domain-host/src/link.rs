//! The engine link's vocabulary (domain/hosts/README.md, section 2, and domain/hosts/hub.md, section 1).
use crate::{Assignment, Grant, Login, RunAnswer, SignOutResult, SourceName, Turn};
use alloc::boxed::Box;
use skein_lib::{Duration, Token};

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum LinkDown {
    Asked { run: Token, attempt: Token, number: u64, answer: crate::AskAnswer },
    ReadList { read: u64, endpoint: u32, grant: Grant },
    Sources { sources: Box<[SourceName]> },
    Check { name: SourceName },
    SignOut { name: SourceName },
    Assign { assignment: Assignment },
    AssignJob { job: crate::JobAssignment },
    Message { run: Token, attempt: Token, name: Token, sender: Box<[u8]>, words: Box<[u8]>, finish_requested: bool },
    Relayed { run: Token, attempt: Token, call: Box<[u8]>, answer: Relayed },
    TurnKept { run: Token, attempt: Token, turn: u32 },
    Acknowledge { run: Token, attempt: Token },
    Cancel { run: Token, attempt: Token },
    Grant { run: Token, attempt: Token, grant: Grant },
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum LinkUp {
    Ask {
        run: Token,
        attempt: Token,
        number: u64,
        bytes: Box<[u8]>,
    },
    /// The host has started to shut down (domain/hosts/hub.md, 7).
    Draining,
    ListRead {
        read: u64,
        outcome: jig_domain_link::ListOutcome,
    },
    JobAnswer {
        run: Token,
        attempt: Token,
        answer: crate::JobEnding,
    },
    /// A piece of live content (domain/hosts/hub.md, 5).
    Piece {
        run: Token,
        attempt: Token,
        piece: crate::Piece,
    },
    Logins {
        logins: Box<[Login]>,
    },
    SignedOut {
        name: SourceName,
        result: SignOutResult,
    },
    Activity {
        run: Token,
        attempt: Token,
        activity: crate::AgentActivity,
    },
    Unauthorized {
        run: Token,
        attempt: Token,
        account: u32,
        generation: u64,
    },
    Limited {
        run: Token,
        attempt: Token,
        account: u32,
        reset: Option<Duration>,
    },
    AccountRefused {
        run: Token,
        attempt: Token,
        account: u32,
        code: Box<[u8]>,
        reason: Box<[u8]>,
    },
    Turn {
        run: Token,
        attempt: Token,
        turn: Turn,
    },
    Call {
        contract: u16,
        run: Token,
        attempt: Token,
        call: Box<[u8]>,
        tool: Box<[u8]>,
        writes: bool,
        input: Box<[u8]>,
        deadline: Duration,
    },
    Answer {
        run: Token,
        attempt: Token,
        answer: RunAnswer,
    },
}

impl LinkUp {
    /// Calls start new work; other reports give notices or settle hosted work (domain/hosts/hub.md).
    #[must_use]
    pub const fn new_work(&self) -> bool {
        match self {
            Self::Call { .. } | Self::Ask { .. } => true,
            Self::Draining
            | Self::ListRead { .. }
            | Self::JobAnswer { .. }
            | Self::Piece { .. }
            | Self::Logins { .. }
            | Self::SignedOut { .. }
            | Self::Activity { .. }
            | Self::Unauthorized { .. }
            | Self::Limited { .. }
            | Self::AccountRefused { .. }
            | Self::Turn { .. }
            | Self::Answer { .. } => false,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Relayed {
    Answer { error: bool, text: Box<[u8]> },
    Busy,
}

#[cfg(test)]
mod tests {
    use super::LinkUp;
    use crate::{
        AgentActivity, Ending, JobEnding, LiveKind, Phase, Piece, RunAnswer, SignOutResult, SourceName, Turn, Work,
    };
    use alloc::boxed::Box;
    use skein_lib::{Duration, Token};

    #[test]
    fn calls_and_asks_start_new_work() {
        let run = Token::new(1);
        let attempt = Token::new(2);
        for writes in [false, true] {
            assert!(
                LinkUp::Call {
                    contract: 1,
                    run,
                    attempt,
                    call: Box::new([]),
                    tool: Box::new([]),
                    writes,
                    input: Box::new([]),
                    deadline: Duration::ZERO,
                }
                .new_work()
            );
        }
        assert!(LinkUp::Ask { run, attempt, number: 1, bytes: Box::new([]) }.new_work());
        let work = Work { left: None, saved: None };
        for record in [
            LinkUp::Draining,
            LinkUp::JobAnswer {
                run,
                attempt,
                answer: JobEnding::Finished { result: Box::new([]), work: work.clone() },
            },
            LinkUp::Piece {
                run,
                attempt,
                piece: Piece { turn: 1, retry: 0, kind: LiveKind::Text, offset: 0, bytes: Box::new([]) },
            },
            LinkUp::ListRead { read: 7, outcome: jig_domain_link::ListOutcome::Listed(Box::new([])) },
            LinkUp::Logins { logins: Box::new([]) },
            LinkUp::SignedOut {
                name: SourceName::new(b"source").expect("source name"),
                result: SignOutResult::SignedOut,
            },
            LinkUp::Activity { run, attempt, activity: AgentActivity::Phase(Phase::Active) },
            LinkUp::Unauthorized { run, attempt, account: 1, generation: 1 },
            LinkUp::Limited { run, attempt, account: 1, reset: None },
            LinkUp::AccountRefused { run, attempt, account: 1, code: Box::new([]), reason: Box::new([]) },
            LinkUp::Turn {
                run,
                attempt,
                turn: Turn {
                    opens_window: true,
                    refused: Box::new([]),
                    turn: 1,
                    spent: 0,
                    read: None,
                    body: Box::new([]),
                },
            },
            LinkUp::Answer {
                run,
                attempt,
                answer: RunAnswer {
                    read: None,
                    refused: Box::new([]),
                    turns: 1,
                    spent: 0,
                    ending: Ending::Parked { work },
                },
            },
        ] {
            assert!(!record.new_work(), "hosted work is admitted: {record:?}");
        }
    }
}
