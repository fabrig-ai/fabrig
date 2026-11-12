//! The owner's tool reaches the ctl act and keeps its answer in that decision (follows.md, 5).
use super::{CallAnswer, CallKey, Decision, Domain, Env, Limits, decide_call, forge, forge_route, host_route};
use crate::{FollowCall, FollowToolAnswer};
use alloc::boxed::Box;
use skein_lib::{Decimal, Duration, Queue, ReplyTo, Token, bytes};

pub(crate) fn call(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    key: CallKey,
    call: FollowCall,
) {
    let Some(context) = domain.core.tasks.delegation(key.task) else { return };
    let project = context.project;
    let (follow, act) = match call {
        FollowCall::List | FollowCall::Show { .. } => {
            let view = domain.forge.document(
                forge::follows_subject(project),
                jig_domain_people::ApplicationScope::Project(project),
                0,
                env.wall,
                &env.limits.forge,
            );
            let view = match view {
                Some(forge::Document::Follows(view)) => view,
                Some(_) | None => forge::FollowsView { project, follows: Box::new([]), dependencies: Box::new([]) },
            };
            let answer = match call {
                FollowCall::List => FollowToolAnswer::List(view),
                FollowCall::Show { follow } => FollowToolAnswer::Show {
                    follow,
                    view: {
                        let mut found = None;
                        for row in view.follows {
                            if row.number == follow { found = Some(row); break; }
                        }
                        found
                    },
                },
                FollowCall::Act { .. } => unreachable!("read call"),
            };
            host_route::read(domain, to, CallAnswer::Follow(Box::new(answer)));
            return;
        }
        FollowCall::Act { follow, act } => (follow, act),
    };
    let refusal = match domain.forge.follow(project, follow) {
        None => Some(forge::FollowRefusal::UnknownFollow.words().into()),
        Some(setting) if setting.owner != forge::settings::FollowOwner::Task(key.task) => {
            let (kind, owner): (&[u8], u64) = match setting.owner {
                forge::settings::FollowOwner::Task(task) => (b"task ", task),
                forge::settings::FollowOwner::Person(person) => (b"person ", person),
            };
            Some(bytes::concat(&[
                b"follow ", Decimal::of(u64::from(follow)).as_bytes(), b" is not yours: its owner is ",
                kind, Decimal::of(owner).as_bytes(),
            ]))
        }
        Some(_) => {
            let Some(number) = crate::fresh(&mut domain.core.counters, jig_domain_core::Family::ConnectorRow) else {
                host_route::rendered(domain, to, jig_domain_core::SettledAnswer::Host { error: true, body: b"busy".as_slice().into() });
                return;
            };
            let token = Token::new(number);
            let mut out = Queue::uncapped();
            // The named call owns retention; the ctl outcome is needed only for this decision.
            forge::step(&mut domain.forge, &env.with(env.limits.forge), forge::Event::FollowAct {
                request: token, token, retention: Duration::ZERO, project, follow, act, authorized: true,
            }, &mut out);
            let mut kept = false;
            let mut remaining = Queue::uncapped();
            while let Some(output) = out.pop() {
                if let forge::Request::ApplicationDone { request, .. } = &output && *request == token {
                    kept = true;
                } else if let forge::Request::ApplicationAgain { request, .. } = &output && *request == token {
                } else {
                    remaining.push(output);
                }
            }
            forge_route::outputs(domain, env, decision, &mut remaining);
            if !kept {
                host_route::rendered(domain, to, jig_domain_core::SettledAnswer::Host { error: true, body: b"busy".as_slice().into() });
                return;
            }
            match domain.forge.application_answer(token) {
                Some(forge::Answered::Follow(refusal)) => refusal,
                Some(_) | None => unreachable!("follow act answered"),
            }
        }
    };
    decide_call(domain, &env.limits, decision, to, key, CallAnswer::Follow(Box::new(FollowToolAnswer::Write { follow, refusal })));
}
