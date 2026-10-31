//! Sends root requests through the host link with one token and failure path.
//! Keeps no independent state and never selects a domain outcome.
//! Contract: smith/docs/design/protocol/agent.md, sections 4–6.

use super::up::live_step;
use super::{Failure, Frame, Stages, program, teardown};
use alloc::boxed::Box;
use skein_channel::Lower;
use skein_lib::{Duration, Queue, Time, Token};
use smith_domain as domain;
use smith_protocol_channel::{self as channel, Piece};

// One entrance borrows the channel edge; no borrow survives a service step.
pub(crate) struct HostLink<'a> {
    pub channel: &'a mut channel::Component,
    pub above: &'a mut Queue<channel::OpenEvent>,
    pub below: &'a mut Queue<Lower>,
    pub next: &'a mut u64,
    pub failed: bool,
    pub room: channel::EdgeRoom,
    pub ending: bool,
}

impl HostLink<'_> {
    pub(crate) fn takes(&self, kind: channel::PieceKind<'_>, bytes: u32) -> bool {
        let room = self.room;
        let entrances = if self.ending { 2 } else { 1 };
        self.above.room() >= room.to_domain.saturating_mul(entrances)
            && self.below.room() >= room.below.saturating_mul(entrances)
            && self.channel.takes_live(kind, bytes)
    }

    pub(crate) fn send(&mut self, piece: Piece<'_>) {
        let token = token(self.next);
        if self.channel.send_live(piece, token, self.above, self.below).is_err() {
            self.failed = true;
        }
    }

    pub(crate) fn lose(&mut self, bytes: u64) {
        self.channel.lose_live(bytes);
    }
}

pub(super) fn token(next: &mut u64) -> Token {
    let token = Token::new(*next);
    *next = next.checked_add(1).expect("one run does not exhaust send names");
    token
}

fn account_notice(
    service: &mut Stages,
    now: Time,
    kind: smith_events::NoticeKind,
    account: u32,
    reset: Option<Duration>,
    code: Option<Box<[u8]>>,
) {
    if let Some(events) = &mut service.events {
        events.notice(
            now,
            smith_events::Notice {
                level: smith_events::Level::Warning,
                kind,
                run: None,
                account: Some(account),
                reset_ms: match reset {
                    Some(reset) => Some(reset.as_millis()),
                    None => None,
                },
                code,
                bytes: None,
                exit: None,
                forced: None,
            },
        );
    }
}

#[expect(clippy::too_many_lines, reason = "one exhaustive framed host-request projection")]
pub(super) fn request(service: &mut Stages, frame: &mut Frame, request: domain::Request) {
    match &request {
        domain::Request::Turn { number, turn, .. } => {
            program::settled(service, turn);
            live_step(service, Some(&mut *frame), crate::live::Action::End { turn: *number });
        }
        domain::Request::Complete { .. }
        | domain::Request::Yield { .. }
        | domain::Request::Cancel { .. }
        | domain::Request::Io { .. }
        | domain::Request::Job { .. }
        | domain::Request::CancelIo { .. }
        | domain::Request::Read { .. }
        | domain::Request::Probe { .. }
        | domain::Request::Check { .. }
        | domain::Request::Abort { .. }
        | domain::Request::Waiting { .. }
        | domain::Request::HostCall { .. }
        | domain::Request::WithdrawHost { .. }
        | domain::Request::MessageRefused { .. }
        | domain::Request::Admitted { .. }
        | domain::Request::Long { .. }
        | domain::Request::LongDone { .. }
        | domain::Request::Deliver { .. }
        | domain::Request::Unauthorized { .. }
        | domain::Request::Limited { .. }
        | domain::Request::Refused { .. }
        | domain::Request::Answer { .. } => {}
    }
    let token = service.next_send();
    let mut answered = false;
    let result = match request {
        domain::Request::Waiting { host_run: _, read } => {
            frame.channel.send_waiting(read, token, &mut service.channel_events, &mut service.channel_below)
        }
        domain::Request::Turn { host_run: _, number, position: _, read, spent, opens_window, turn } => {
            frame.channel.send_turn(
                number,
                read,
                spent,
                opens_window,
                &turn,
                token,
                &mut service.channel_events,
                &mut service.channel_below,
            )
        }
        domain::Request::HostCall { host_run: _, relay, name, tool, effect, input, deadline } => {
            frame.channel.send_host_call(
                service.domain_env.now,
                name,
                relay,
                tool,
                effect,
                input,
                deadline,
                token,
                &mut service.channel_events,
                &mut service.channel_below,
            )
        }
        domain::Request::WithdrawHost { relay } => {
            frame.channel.send_withdraw(relay, token, &mut service.channel_events, &mut service.channel_below)
        }
        domain::Request::MessageRefused { host_run: _, name, reason } => frame.channel.send_message_refused(
            name,
            reason,
            token,
            &mut service.channel_events,
            &mut service.channel_below,
        ),
        domain::Request::Admitted { host_run: _, run } => {
            service.admitted = Some(run);
            frame.channel.send_admitted(token, &mut service.channel_events, &mut service.channel_below)
        }
        domain::Request::Answer { to, answer, read, detail } => {
            answered = true;
            crate::terminal::remember(service, &answer, detail, service.domain_env.now);
            assert!(to.into_token() == super::START_REPLY, "one Start reply right per process");
            if let Some(events) = &mut service.events {
                events.answer(service.domain_env.now, service.run_started, &answer, read);
            }
            frame.channel.send_answer(
                service.domain_env.now,
                answer,
                read,
                token,
                &mut service.channel_events,
                &mut service.channel_below,
            )
        }
        domain::Request::Long { host_run: _, operation, until } => {
            let span = until.saturating_since(service.domain_env.now);
            frame.channel.send_long(operation, span, token, &mut service.channel_events, &mut service.channel_below)
        }
        domain::Request::LongDone { host_run: _ } => {
            frame.channel.send_long_done(token, &mut service.channel_events, &mut service.channel_below)
        }
        domain::Request::Deliver { name, deadline, host_run: _, owner, change } => frame.channel.send_delivery(
            service.domain_env.now,
            name,
            owner,
            change,
            deadline,
            token,
            &mut service.channel_events,
            &mut service.channel_below,
        ),
        domain::Request::Unauthorized { grant } => {
            account_notice(
                service,
                service.domain_env.now,
                smith_events::NoticeKind::CredentialUnauthorized,
                grant.account,
                None,
                None,
            );
            frame.channel.send_unauthorized(grant, token, &mut service.channel_events, &mut service.channel_below)
        }
        domain::Request::Limited { account, reset } => {
            account_notice(
                service,
                service.domain_env.now,
                smith_events::NoticeKind::AccountLimited,
                account,
                reset,
                None,
            );
            frame.channel.send_limited(account, reset, token, &mut service.channel_events, &mut service.channel_below)
        }
        domain::Request::Refused { account, code, reason } => {
            account_notice(
                service,
                service.domain_env.now,
                smith_events::NoticeKind::AccountRefused,
                account,
                None,
                Some(code.clone()),
            );
            frame.channel.send_refused(
                account,
                &code,
                &reason,
                token,
                &mut service.channel_events,
                &mut service.channel_below,
            )
        }
        domain::Request::Complete { .. }
        | domain::Request::Yield { .. }
        | domain::Request::Cancel { .. }
        | domain::Request::Io { .. }
        | domain::Request::Job { .. }
        | domain::Request::CancelIo { .. }
        | domain::Request::Read { .. }
        | domain::Request::Probe { .. }
        | domain::Request::Check { .. }
        | domain::Request::Abort { .. } => unreachable!("lower requests are dispatched before the framed face"),
    };
    sent(service, result.is_ok());
    if answered {
        teardown::answered(service);
    }
}

pub(super) fn sent(service: &mut Stages, succeeded: bool) {
    if !succeeded {
        service.mark_failed(Failure::Send);
    }
}
