//! Sub-agent admission and its immutable opening.
//! Calls retain their child until its terminal; admission never interprets provider
//! syntax. Contract: smith/docs/design/domain/run.md, section 5.3.

use alloc::boxed::Box;

use skein_lib::bytes::copy_of;
use skein_lib::{Deadlines, Env, Id, Queue, Slab};

use super::{Alarm, Asking, Conversation, Phase, Run, begin_call, jobs};
use crate::agent::{self, Child, Means};
use crate::call::{Call, Calls, Work};
use crate::charter::{self, Families};
use crate::{AskRefusal, Limits, Opening, Request, Returned, Share, Spend, prompt};

/// What a conversation asks for when it asks for a sub-agent.
pub(super) struct Wanted {
    pub(super) brief: Box<[u8]>,
    pub(super) families: Families,
    pub(super) llm: Option<Box<[u8]>>,
    pub(super) share: Share,
}

/// Working: the conversation `asker` asked for a sub-agent as `made`. Open
/// it, with no more time than to the call's deadline, or return why not.
#[expect(clippy::too_many_arguments, reason = "a cell handler takes the fields it touches")]
pub(super) fn sub_agent(
    run: &mut Run,
    run_id: Id<Run>,
    conversations: &mut Slab<Conversation>,
    calls: &mut Calls,
    alarms: &mut Deadlines<Alarm>,
    asker: Id<Conversation>,
    made: Asking,
    wanted: Wanted,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) {
    let Asking { call, deadline, name: _ } = made;
    if u64::try_from(wanted.brief.len()).unwrap_or(u64::MAX) > env.limits.host_part {
        out.push(Request::Return {
            spent: 0,
            call,
            result: Returned::Refused { refusal: AskRefusal::TooLarge { bound: env.limits.host_part } },
        });
        return;
    }
    let asking = conversations.get(asker).expect("a conversation lives until it has ended");
    let left = run.deadline.saturating_since(env.now);
    let means = Means {
        spent: run.spent,
        conversations: run.conversations,
        left,
        call_left: deadline.saturating_since(env.now),
    };
    if run.charter.budget.exhausted(run.spent).is_some() {
        out.push(Request::Return { spent: 0, call, result: Returned::Refused { refusal: AskRefusal::Over } });
        return;
    }
    let planned = agent::plan(
        &run.charter,
        means,
        asking.families,
        wanted.families,
        wanted.llm.as_deref(),
        wanted.share,
        &env.limits,
    );
    let plan = match planned {
        Ok(plan) => plan,
        Err(refusal) => {
            out.push(Request::Return { spent: 0, call, result: Returned::Refused { refusal } });
            return;
        }
    };
    assert!(
        plan.budget == crate::Budget { turns: plan.granted.turns, time: plan.granted.time, spend: plan.granted.spend },
        "a child's opening budget is exactly its effective grant"
    );
    match prompt::child_host_bytes(&run.charter, run.workspace.as_ref(), &run.found, &wanted.brief, plan.families) {
        Some(bytes) if bytes <= env.limits.host_part => {}
        Some(_) | None => {
            out.push(Request::Return {
                spent: 0,
                call,
                result: Returned::Refused { refusal: AskRefusal::TooLarge { bound: env.limits.host_part } },
            });
            return;
        }
    }
    if conversations.is_full() || calls.is_full() {
        out.push(Request::Return { spent: 0, call, result: Returned::Busy });
        return;
    }
    let ordinal = run.next_ordinal;
    let Some(next_ordinal) = ordinal.checked_add(1) else {
        out.push(Request::Return { spent: 0, call, result: Returned::Refused { refusal: AskRefusal::TooMany } });
        return;
    };
    // A call is stored before its child, which names it, and holds the child
    // once the child has a name too.
    let unnamed = Call { run: run_id, conversation: asker, owner: call, work: Work::Child(Child::Closed) };
    let id = begin_call(calls, alarms, unnamed, deadline);
    if plan.families.tools.modify || plan.families.tools.shell {
        run.lease.child(id);
    } else {
        run.lease.assert_free();
    }
    let child = Conversation {
        run: run_id,
        kind: crate::facts::Kind::Child { parent: asker.token(), call, asked: wanted.share, granted: plan.granted },
        opened_at: env.now,
        asker: Some(id),
        families: plan.families,
        spent: Spend::ZERO,
        subtree_spent: 0,
        reservation: None,
        budget_close: None,
        calls: 0,
        phase: Phase::Opening,
    };
    let child = conversations.insert(child).expect("checked for room above");
    run.children.insert(child).expect("admitted child count fits run slots");
    calls.get_mut(id).expect("inserted above").work = Work::Child(agent::working(child));
    let asking = conversations.get_mut(asker).expect("a conversation lives until it has ended");
    asking.calls = asking.calls.saturating_add(1);
    run.conversations = run.conversations.saturating_add(1);
    run.next_ordinal = next_ordinal;
    let opening = child_opening(run, &wanted.brief, plan, ordinal, &env.limits);
    if plan_writing(conversations, child) {
        calls.get_mut(id).expect("admitted child call").work =
            Work::Child(Child::Cancelling { child, opening: Box::new(opening), deadline, why: None });
        out.push(Request::CancelJobs { peer: jobs::peer(conversations, asker) });
    } else {
        out.push(Request::Open { conversation: child.token(), opening });
    }
}

fn plan_writing(conversations: &Slab<Conversation>, child: Id<Conversation>) -> bool {
    let families = conversations.get(child).expect("admitted child").families;
    families.tools.modify || families.tools.shell
}

/// Render an admitted child's immutable opening from its effective grant.
fn child_opening(run: &Run, brief: &[u8], plan: agent::Plan, ordinal: u32, limits: &Limits) -> Opening {
    Opening {
        final_turn: Some(crate::FinalTurn {
            note: prompt::child_final_note(),
            allowance: agent::final_allowance(plan.budget.time, limits.wrap_up),
        }),
        generated_schema_bytes: 0,
        identity: run.identity,
        ordinal,
        activation: run.activation,
        transcript: None,
        wait: false,
        host_tools: Box::new([]),
        deliver: false,
        system: prompt::child(
            &run.charter,
            run.workspace.as_ref(),
            &run.found,
            brief,
            plan.families,
            plan.granted,
            limits.answer_bytes,
        ),
        prompt: copy_of(prompt::BEGIN),
        summary: copy_of(prompt::SUMMARY),
        preamble: copy_of(prompt::PREAMBLE),
        tools: plan.families.tools,
        workspace: run.workspace.clone(),
        budget: plan.budget,
        finish: false,
        families: plan.families,
        line: charter::line(&plan.llm, limits.compaction_material),
        llm: plan.llm,
    }
}
