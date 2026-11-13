//! A named red write joins its forge mutation and answer (domain/forge/reds.md, 3).
use super::{CallAnswer, CallKey, Decision, Domain, Env, Limits, Work, decide_call, forge, forge_route, host_route};
use crate::{RedCall, RedToolAnswer};
use alloc::boxed::Box;
use skein_lib::{List, Queue, ReplyTo};

pub(crate) fn call(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    key: CallKey,
    call: RedCall,
) {
    let goal = key.task;
    if let RedCall::List = call {
        let answer = if domain.forge.holds_goal_branch(goal) {
            let mut reds = List::with_capacity(env.limits.forge.branch_reds);
            for (_, red) in domain.forge.reds(goal) {
                reds.push(red.clone()).expect("goal red bound");
            }
            RedToolAnswer::List { goal, bound: env.limits.forge.branch_reds, reds: reds.into_boxed() }
        } else {
            RedToolAnswer::NoGoal { task: goal }
        };
        host_route::read(domain, to, CallAnswer::Red(Box::new(answer)));
        return;
    }
    let red_key = match &call {
        RedCall::Record { key, .. } | RedCall::End { key, .. } => key.clone(),
        RedCall::List => unreachable!("list answered above"),
    };
    if !forge::reds::key_fits(&red_key, env.limits.forge.red_key_bytes) {
        let mut words = List::uncapped();
        for part in [
            b"red: key exceeds forge.red_key_bytes (".as_slice(),
            skein_lib::Decimal::of(u64::from(env.limits.forge.red_key_bytes)).as_bytes(),
            b")",
        ] {
            for &byte in part {
                words.push(byte).expect("fixed limit feedback");
            }
        }
        host_route::rendered(
            domain,
            to,
            jig_domain_core::SettledAnswer::Host { error: true, body: words.into_boxed() },
        );
        return;
    }
    let mut out = Queue::uncapped();
    let result = match call {
        RedCall::Record { key, at, why } => {
            let recorded = domain.forge.record_red(
                &env.limits.forge,
                forge::RecordRed {
                    goal,
                    key,
                    head: at.clone(),
                    why,
                    by: forge::Recorder::Task(goal),
                    at: env.wall.as_nanos(),
                },
                &mut out,
            );
            match recorded {
                Ok(recorded) => Ok(forge::RedOutcome::Recorded { head: at, recorded }),
                Err(why) => Err(why),
            }
        }
        RedCall::End { key, why: _ } => match domain.forge.end_red(goal, &key, &mut out) {
            Ok(red) => Ok(forge::RedOutcome::Ended(red)),
            Err(why) => Err(why),
        },
        RedCall::List => unreachable!("list answered above"),
    };
    forge_route::outputs(domain, env, decision, &mut out);
    if let Err(why @ forge::RedRefusal::Full { .. }) = result {
        // The refusal itself is the account; its release belongs to the person.
        domain.work.push(Work::Tasks(jig_domain_tasks::Event::HoldFrom {
            task: goal,
            why: jig_domain_tasks::Hold::Procedure,
            source: Box::new(jig_domain_tasks::HoldSource::Account {
                connector: domain.config.forge_connector,
                entry: None,
                words: why.words(goal, &red_key),
            }),
        }));
    }
    let reply = forge::RedReply { goal, key: red_key, result };
    decide_call(domain, &env.limits, decision, to, key, CallAnswer::Red(Box::new(RedToolAnswer::Write(reply))));
}
