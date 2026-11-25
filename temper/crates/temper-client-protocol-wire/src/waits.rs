//! Stable procedure wait codes (temper/docs/design/domain/forge/waits.md, 3).

pub const CHANGE_TAKING: u32 = 101;
pub const CHANGE_PRODUCING: u32 = 102;
pub const CHANGE_OPENING: u32 = 103;
pub const CHANGE_RECREATING: u32 = 104;
pub const CHANGE_REOPENING: u32 = 105;
pub const CHANGE_CHECKING: u32 = 106;
pub const CHANGE_GATING: u32 = 107;
pub const CHANGE_READY: u32 = 108;
pub const CHANGE_PREPARING: u32 = 109;
pub const CHANGE_CANCELLING_GATES: u32 = 110;
pub const CHANGE_QUEUED: u32 = 111;
pub const CHANGE_FIRST: u32 = 112;
pub const CHANGE_UPDATING: u32 = 113;
pub const CHANGE_WAITING_PRODUCER: u32 = 114;
pub const CHANGE_RESOLVING: u32 = 115;
pub const CHANGE_REPAIRING: u32 = 116;
pub const CHANGE_LANDING: u32 = 117;
pub const CHANGE_LANDED: u32 = 118;
pub const CHANGE_RED: u32 = 119;
pub const CHANGE_CLEARED: u32 = 120;
pub const CHANGE_UNCLEARED: u32 = 121;
pub const CHANGE_JUDGING: u32 = 122;

pub const LANDING_PREFIXES: u32 = 201;
pub const LANDING_OPEN: u32 = 202;
pub const LANDING_CLOSE: u32 = 203;
pub const LANDING_DELETE: u32 = 204;
pub const LANDING_POST: u32 = 205;
pub const LANDING_RUN: u32 = 206;
pub const LANDING_RECEIPT: u32 = 207;
pub const LANDING_MERGE: u32 = 208;
pub const LANDING_RECORD: u32 = 209;
pub const LANDING_ANCESTRY: u32 = 210;
pub const LANDING_READ: u32 = 211;
pub const LANDING_READY: u32 = 212;
pub const LANDING_JUDGEMENT: u32 = 213;
pub const LANDING_ROOM: u32 = 214;

pub const CLOSER_CLOSING: u32 = 301;
pub const CLOSER_LANDING: u32 = 302;
pub const CLOSER_REPORTING: u32 = 303;
pub const CLOSER_MAKING: u32 = 304;

pub const THING_ENTRY: u32 = 1;
pub const THING_TASK: u32 = 2;
pub const THING_PULL: u32 = 3;
pub const THING_RUN: u32 = 4;
pub const THING_GATE: u32 = 5;
pub const THING_ROLE: u32 = 6;
pub const THING_RULE: u32 = 7;
pub const THING_FORGE: u32 = 8;

#[must_use]
pub const fn step_name(code: u32) -> Option<&'static [u8]> {
    match code {
        CHANGE_TAKING => Some(b"taking"),
        CHANGE_PRODUCING => Some(b"producing"),
        CHANGE_OPENING => Some(b"opening"),
        CHANGE_RECREATING => Some(b"recreating"),
        CHANGE_REOPENING => Some(b"reopening"),
        CHANGE_CHECKING => Some(b"checking"),
        CHANGE_GATING => Some(b"gating"),
        CHANGE_READY | LANDING_READY => Some(b"ready"),
        CHANGE_PREPARING => Some(b"preparing"),
        CHANGE_CANCELLING_GATES => Some(b"cancelling gates"),
        CHANGE_QUEUED => Some(b"queued"),
        CHANGE_FIRST => Some(b"first"),
        CHANGE_UPDATING => Some(b"updating"),
        CHANGE_WAITING_PRODUCER => Some(b"waiting producer"),
        CHANGE_RESOLVING => Some(b"resolving"),
        CHANGE_REPAIRING => Some(b"repairing"),
        CHANGE_LANDING | CLOSER_LANDING => Some(b"landing"),
        CHANGE_LANDED => Some(b"landed"),
        CHANGE_RED => Some(b"red"),
        CHANGE_CLEARED => Some(b"cleared"),
        CHANGE_UNCLEARED => Some(b"uncleared"),
        CHANGE_JUDGING => Some(b"judging"),
        LANDING_PREFIXES => Some(b"prefixes"),
        LANDING_OPEN => Some(b"open"),
        LANDING_CLOSE => Some(b"close"),
        LANDING_DELETE => Some(b"delete"),
        LANDING_POST => Some(b"post"),
        LANDING_RUN => Some(b"run"),
        LANDING_RECEIPT => Some(b"receipt"),
        LANDING_MERGE => Some(b"merge"),
        LANDING_RECORD => Some(b"record"),
        LANDING_ANCESTRY => Some(b"ancestry"),
        LANDING_READ => Some(b"read"),
        LANDING_JUDGEMENT => Some(b"judgement"),
        LANDING_ROOM => Some(b"room"),
        CLOSER_CLOSING => Some(b"closing"),
        CLOSER_REPORTING => Some(b"reporting"),
        CLOSER_MAKING => Some(b"its goal branch's making"),
        _ => None,
    }
}

#[must_use]
pub const fn thing_name(code: u32) -> Option<&'static [u8]> {
    match code {
        THING_ENTRY => Some(b"entry"),
        THING_TASK => Some(b"task"),
        THING_PULL => Some(b"pull request"),
        THING_RUN => Some(b"run"),
        THING_GATE => Some(b"gate"),
        THING_ROLE => Some(b"role"),
        THING_RULE => Some(b"rule"),
        THING_FORGE => Some(b"forge"),
        _ => None,
    }
}
