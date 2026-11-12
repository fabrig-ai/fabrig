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
            let view = view(domain, env, project);
            let answer = match call {
                FollowCall::List => FollowToolAnswer::List(view),
                FollowCall::Show { follow } => {
                    let mut found = None;
                    for row in view.follows {
                        if row.number == follow {
                            found = Some(Box::new(row));
                            break;
                        }
                    }
                    FollowToolAnswer::Show { follow, view: found }
                }
                FollowCall::Act { .. } => unreachable!("read call"),
            };
            host_route::read(domain, to, CallAnswer::Follow(Box::new(answer)));
            return;
        }
        FollowCall::Act { follow, act } => (follow, act),
    };
    let refusal = match owner_refusal(domain, project, follow, key.task) {
        Some(words) => Some(words),
        None => {
            let Ok(refusal) = act_kept(domain, env, decision, project, follow, act) else {
                host_route::rendered(
                    domain,
                    to,
                    jig_domain_core::SettledAnswer::Host { error: true, body: b"busy".as_slice().into() },
                );
                return;
            };
            refusal
        }
    };
    decide_call(
        domain,
        &env.limits,
        decision,
        to,
        key,
        CallAnswer::Follow(Box::new(FollowToolAnswer::Write { follow, refusal })),
    );
}
fn view(domain: &Domain, env: &Env<Limits>, project: u32) -> forge::FollowsView {
    if let Some(forge::Document::Follows(view)) = domain.forge.document(
        forge::follows_subject(project),
        jig_domain_people::ApplicationScope::Project(project),
        0,
        env.wall,
        &env.limits.forge,
    ) {
        view
    } else {
        forge::FollowsView { project, follows: Box::new([]), dependencies: Box::new([]) }
    }
}
fn owner_refusal(domain: &Domain, project: u32, follow: u32, caller: u64) -> Option<Box<[u8]>> {
    let Some(setting) = domain.forge.follow(project, follow) else {
        return Some(forge::FollowRefusal::UnknownFollow.words().into());
    };
    if setting.owner == forge::settings::FollowOwner::Task(caller) {
        return None;
    }
    let (kind, owner): (&[u8], u64) = match setting.owner {
        forge::settings::FollowOwner::Task(task) => (b"task ", task),
        forge::settings::FollowOwner::Person(person) => (b"person ", person),
    };
    Some(bytes::concat(&[
        b"follow ",
        Decimal::of(u64::from(follow)).as_bytes(),
        b" is not yours: its owner is ",
        kind,
        Decimal::of(owner).as_bytes(),
    ]))
}
fn act_kept(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    project: u32,
    follow: u32,
    act: forge::follows::FollowAct,
) -> Result<Option<Box<[u8]>>, ()> {
    let number = crate::fresh(&mut domain.core.counters, jig_domain_core::Family::Application).ok_or(())?;
    let token = Token::new(number);
    let mut out = Queue::uncapped();
    // The named call owns retention; the ctl outcome is needed only for this decision.
    forge::step(
        &mut domain.forge,
        &env.with(env.limits.forge),
        forge::Event::FollowAct {
            request: token,
            token,
            retention: Duration::ZERO,
            project,
            follow,
            act,
            authorized: true,
        },
        &mut out,
    );
    let mut kept = false;
    let mut remaining = Queue::uncapped();
    while let Some(output) = out.pop() {
        if let forge::Request::ApplicationDone { request, .. } = &output
            && *request == token
        {
            kept = true;
        } else if let forge::Request::ApplicationAgain { request, .. } = &output
            && *request == token
        {
            // Admission refused before the act; leave this call undecided.
        } else {
            remaining.push(output);
        }
    }
    forge_route::outputs(domain, env, decision, &mut remaining);
    if !kept {
        return Err(());
    }
    let Some(forge::Answered::Follow(refusal)) = domain.forge.application_answer(token) else {
        unreachable!("follow act answered");
    };
    Ok(refusal)
}
