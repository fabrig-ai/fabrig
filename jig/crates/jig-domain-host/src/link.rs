//! The engine link's vocabulary (domain/hosts.md, sections 2 and 6.1).
use crate::{Answer, Assignment, Grant, Login, SignOutResult, SourceName, Turn};
use alloc::boxed::Box;
use skein_lib::{Duration, Token};

#[derive(PartialEq, Eq, Debug)]
pub enum LinkDown {
    Sources { sources: Box<[SourceName]> },
    Check { name: SourceName },
    SignOut { name: SourceName },
    Assign { assignment: Assignment },
    Message { run: Token, attempt: Token, name: Token, sender: Box<[u8]>, words: Box<[u8]>, finish_requested: bool },
    Relayed { run: Token, attempt: Token, call: Box<[u8]>, answer: Relayed },
    TurnKept { run: Token, attempt: Token, turn: u32 },
    Acknowledge { run: Token, attempt: Token },
    Cancel { run: Token, attempt: Token },
    Grant { run: Token, attempt: Token, grant: Grant },
}

#[derive(PartialEq, Eq, Debug)]
pub enum LinkUp {
    /// A piece of live content (hosts.md, 6.5).
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
        activity: crate::Activity,
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
        answer: Answer,
    },
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Relayed {
    Answer { error: bool, text: Box<[u8]> },
    Busy,
}
