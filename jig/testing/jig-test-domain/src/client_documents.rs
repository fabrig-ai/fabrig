//! Assemble connector-owned documents in the same decision as the core's pacing.
use crate::routes::{route_connector, route_engine, route_request};
use crate::{Delivery, Domain, Limits, Work, Write};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_core::engine::{self, In, Out, Peer};
use jig_domain_views as views;
use jig_test_connector as connector;
use skein_lib::{Decision, Env, Queue, ReplyTo, Token};

pub(crate) fn window(value: core::documents::Window) -> connector::ClientWindow {
    connector::ClientWindow {
        rows: value.rows,
        after: match value.after {
            Some(core::documents::Cursor::Number { number }) => Some(connector::ClientCursor::Number(number)),
            Some(core::documents::Cursor::Inbox { .. }) => Some(connector::ClientCursor::Unknown),
            None => None,
        },
    }
}

pub(crate) fn scope(value: jig_domain_people::ApplicationScope) -> connector::ClientScope {
    match value {
        jig_domain_people::ApplicationScope::Project(project) => connector::ClientScope::Project(project),
        jig_domain_people::ApplicationScope::Deployment => connector::ClientScope::Deployment,
        jig_domain_people::ApplicationScope::Own => connector::ClientScope::Own,
    }
}

#[expect(clippy::too_many_arguments, reason = "the construction envelope is routed unchanged")]
pub(crate) fn document(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Delivery>,
    number: u16,
    watcher: Token,
    subject: Token,
    document: Result<connector::ClientDocument, connector::DocumentMissing>,
    work: &mut Queue<Work>,
) {
    let (outcome, mut value) = match document {
        Ok(document) => (views::Made::Ready, Some(document)),
        Err(why) => (
            views::Made::Gone {
                why: match why {
                    connector::DocumentMissing::Unknown => views::End::Gone,
                    connector::DocumentMissing::NotAllowed => views::End::NotAllowed,
                },
            },
            None,
        ),
    };
    let mut out = Queue::uncapped();
    engine::step(
        &mut domain.engine,
        &env.with(env.limits.core),
        In::Core(core::Event::View(views::Event::Made { watcher, outcome })),
        &mut out,
    );
    for _ in 0..out.len() {
        let request = match out.pop().expect("document completion output") {
            Out::Core(request) => request,
            Out::Held(Peer::Store(load)) => {
                crate::hold(decision, Delivery::Store(load));
                continue;
            }
        };
        match request {
            core::Request::Held(held) => match *held {
                core::Held::Store(load) => crate::hold(decision, Delivery::Store(load)),
                core::Held::ApplicationDelivery { watcher: due, connector: owner, subject: named } => {
                    assert!(owner == number && named == subject, "construction keeps its owner");
                    let document = value.take().expect("ready construction owns its whole document");
                    crate::hold(
                        decision,
                        Delivery::ApplicationDocument { watcher: due, number, document: Box::new(document) },
                    );
                }
                held @ (core::Held::ApplicationWatchEnded { .. }
                | core::Held::Read { .. }
                | core::Held::PublicRead { .. }
                | core::Held::PublicWait { .. }
                | core::Held::Applied { .. }
                | core::Held::Delivery { .. }
                | core::Held::WatchEnded { .. }
                | core::Held::MakeEffect { .. }
                | core::Held::SettledCall { .. }
                | core::Held::Assign { .. }
                | core::Held::Grant { .. }
                | core::Held::Relayed { .. }
                | core::Held::Inbound { .. }
                | core::Held::Relay { .. }
                | core::Held::PeopleReply { .. }
                | core::Held::CarriedUnused { .. }
                | core::Held::CallAnswer { .. }
                | core::Held::NeedArrived { .. }
                | core::Held::Result { .. }
                | core::Held::Acknowledge { .. }
                | core::Held::TaskTerminalAcknowledged { .. }
                | core::Held::TaskPartKept { .. }
                | core::Held::AcknowledgePart { .. }
                | core::Held::PartBusy { .. }
                | core::Held::TaskTurnKept { .. }
                | core::Held::AcknowledgeTurn { .. }
                | core::Held::Drained
                | core::Held::Restarting { .. }
                | core::Held::Cancel { .. }
                | core::Held::ReadUsage { .. }
                | core::Held::ReadList { .. }
                | core::Held::Sources { .. }
                | core::Held::PauseWorker { .. }
                | core::Held::CheckSource { .. }
                | core::Held::SignOutSource { .. }
                | core::Held::SourceSignedOut { .. }
                | core::Held::Refuse { .. }
                | core::Held::TurnBusy { .. }
                | core::Held::StopRun { .. }
                | core::Held::Load(..)
                | core::Held::NotesLoad { .. }
                | core::Held::NotesWritten { .. }
                | core::Held::Acted { .. }
                | core::Held::NotesDeleted { .. }) => {
                    route_request(domain, env, decision, core::Request::Held(Box::new(held)), work);
                }
            },
            request @ (core::Request::Write(_)
            | core::Request::Ask { .. }
            | core::Request::Now(_)
            | core::Request::ProjectMade { .. }) => route_request(domain, env, decision, request, work),
        }
    }
    assert!(value.is_none(), "every ready document was assembled");
}

#[expect(clippy::too_many_arguments, reason = "an admitted read completes synchronously with its reply owned here")]
pub(crate) fn read(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Delivery>,
    number: u16,
    reply: ReplyTo,
    scope: connector::ClientScope,
    read: Token,
    party: u64,
    window: core::documents::Window,
    work: &mut Queue<Work>,
) {
    let mut reply = Some(reply);
    let mut out = Queue::uncapped();
    connector::step(
        crate::numbered(domain, number),
        &env.with(env.limits.connector),
        connector::Event::ReadDocument { scope, read, party, window: self::window(window) },
        &mut out,
    );
    for _ in 0..out.len() {
        let request = out.pop().expect("read completion count");
        match request {
            connector::Request::SettingsRestored { .. }
            | connector::Request::SettingsJudged { .. }
            | connector::Request::SettingsDefaults { .. }
            | connector::Request::SettingValue { .. }
            | connector::Request::RestoreRefused { .. } => unreachable!("settings routing: apps-world"),
            connector::Request::ReadAnswered { read: named, answer } => {
                assert!(read == named, "the connector echoes its read name");
                crate::hold(
                    decision,
                    Delivery::ApplicationRead { reply: reply.take().expect("one read terminal"), number, answer },
                );
            }
            request @ (connector::Request::Document { .. }
            | connector::Request::Marked { .. }
            | connector::Request::ApplicationDone { .. }
            | connector::Request::ApplicationAgain { .. }
            | connector::Request::ApplicationRelease { .. }
            | connector::Request::Resource { .. }
            | connector::Request::ProjectReleased { .. }
            | connector::Request::Closed { .. }
            | connector::Request::Verdict { .. }
            | connector::Request::Step { .. }
            | connector::Request::Waits { .. }
            | connector::Request::Answer { .. }
            | connector::Request::Ready { .. }
            | connector::Request::Section { .. }
            | connector::Request::Workspace { .. }
            | connector::Request::DriftResource { .. }
            | connector::Request::Changed { .. }
            | connector::Request::ReadAfreshDone
            | connector::Request::RestartDone
            | connector::Request::Described { .. }
            | connector::Request::EffectRefused { .. }
            | connector::Request::EffectBusy { .. }
            | connector::Request::Make { .. }
            | connector::Request::Outcome { .. }
            | connector::Request::System(_)
            | connector::Request::Named { .. }
            | connector::Request::Unknown { .. }
            | connector::Request::Refused { .. }
            | connector::Request::Adopted { .. }
            | connector::Request::Slots { .. }
            | connector::Request::Drift { .. }
            | connector::Request::Topic { .. }
            | connector::Request::TopicRefused { .. }
            | connector::Request::News { .. }
            | connector::Request::Save { .. }
            | connector::Request::Erase { .. }) => {
                let mut requests = Queue::uncapped();
                requests.push(request);
                route_connector(domain, env, decision, number, &mut requests, work);
            }
        }
    }
    assert!(reply.is_none(), "the connector answers within the decision");
}

pub(crate) fn role_number(role: Option<jig_domain_people::Role>) -> Option<u32> {
    let role = role?;
    Some(role.number())
}

/// Refuse excess public work immediately (jig/docs/design/domain/people/public-reading.md, 5).
pub(crate) fn public_busy(domain: &mut Domain, reply: ReplyTo) {
    // Refusals bypass the journal's held queue and never count as admitted reads.
    domain.now.push(crate::Request::Deliver(Delivery::Core(core::Held::PublicRead {
        reply,
        answer: core::PublicAnswer::Busy,
    })));
}

pub(crate) fn public_start(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Delivery>,
    reply: ReplyTo,
    read: core::PublicRead,
    work: &mut Queue<Work>,
) {
    if !crate::admission::public_admits(domain, &env.limits) {
        public_busy(domain, reply);
        return;
    }
    domain.public_reads = domain.public_reads.checked_add(1).expect("bounded public reads");
    // The test application's families 1 and 2 are served by connectors 1 and 2.
    let connector = match read.owner.checked_sub(1) {
        Some(1) => Some(1_u16),
        Some(2) => Some(2_u16),
        Some(_) | None => None,
    };
    if read.owner != 0 && connector.is_none() {
        crate::hold(decision, Delivery::Core(core::Held::PublicRead { reply, answer: core::PublicAnswer::Unknown }));
        return;
    }
    let read = core::PublicRead {
        owner: match connector {
            Some(number) => number.checked_add(1).expect("connector owner"),
            None => 0,
        },
        ..read
    };
    let mut out = Queue::uncapped();
    engine::step(
        &mut domain.engine,
        &env.with(env.limits.core),
        In::Core(core::Event::PublicRead { reply, read }),
        &mut out,
    );
    route_engine(domain, env, decision, out, work);
}

pub(crate) fn public(domain: &mut Domain, number: u16, reply: ReplyTo, work: &mut Queue<Work>) {
    let _connector = crate::numbered(domain, number);
    work.push(Work::Core(core::Event::PublicAnswered { reply, answer: None }));
}
