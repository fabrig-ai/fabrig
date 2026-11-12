//! A named finding write joins the forge's mutation and its answer (domain/agent/tools.md).
use super::{CallAnswer, CallKey, Decision, Domain, Env, Limits, decide_call, findings, forge, host_route};
use crate::{FindingCall, FindingToolAnswer};
use alloc::boxed::Box;
use skein_lib::{List, ReplyTo};

pub(crate) fn call(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    key: CallKey,
    call: FindingCall,
) {
    let Some(context) = domain.core.tasks.delegation(key.task) else { return };
    let project = context.project;
    let by = forge::Filer::Task(key.task);
    let reply = match call {
        FindingCall::File { kind, severity, area, title, body } => {
            let result = findings::file_finding(
                domain,
                env,
                decision,
                by,
                forge::FileFinding { project, kind, severity, area: area.clone(), title, body },
            );
            let (number, result) = match result {
                forge::FileFindingAnswer::Filed { number } => {
                    (number, Ok(forge::FindingDone::Filed(findings::holder(domain, project, area.as_deref()))))
                }
                forge::FileFindingAnswer::Refused(why) => (0, Err(why)),
            };
            forge::FindingReply { project, number, area, result }
        }
        FindingCall::Hand { number, area, why } => {
            let result = findings::hand_finding(
                domain,
                env,
                decision,
                by,
                forge::HandFinding { project, number, area: area.clone(), why },
            );
            let result = match result {
                forge::HandFindingAnswer::Handed => {
                    Ok(forge::FindingDone::Handed(findings::holder(domain, project, Some(&area))))
                }
                forge::HandFindingAnswer::Refused(why) => Err(why),
            };
            forge::FindingReply { project, number, area: Some(area), result }
        }
        FindingCall::Triage { number, act } => {
            let done = match &act {
                forge::Triage::Route(goal) => forge::FindingDone::Routed(*goal),
                forge::Triage::Backlog => forge::FindingDone::Backlog,
                forge::Triage::Drop(_) => forge::FindingDone::Dropped,
            };
            let result = match findings::triage_finding(
                domain,
                env,
                decision,
                by,
                forge::TriageFinding { project, number, act },
            ) {
                forge::TriageFindingAnswer::Triaged => Ok(done),
                forge::TriageFindingAnswer::Refused(why) => Err(why),
            };
            forge::FindingReply { project, number, area: None, result }
        }
        FindingCall::List { state, mine } => {
            let mut document = findings::document(domain, project, 0, None, false);
            // The party document is row-bounded; this tool counts every omitted row in its answer.
            let mut rows = List::with_capacity(env.limits.forge.findings_per_project);
            for (_, row) in domain.forge.findings(project).rev() {
                let holder = findings::holder(domain, project, row.area.as_deref());
                if (match state {
                    Some(state) => state != row.state,
                    None => false,
                }) || (mine && holder != forge::Holder::Task(key.task))
                {
                    continue;
                }
                rows.push(row.row(holder)).expect("project finding bound");
            }
            document.rows = rows.into_boxed();
            document.cut = false;
            host_route::read(domain, to, CallAnswer::Finding(Box::new(FindingToolAnswer::List(document))));
            return;
        }
        FindingCall::Show { number } => {
            let finding = findings::read(domain, project, number);
            host_route::read(
                domain,
                to,
                CallAnswer::Finding(Box::new(FindingToolAnswer::Show { project, number, finding })),
            );
            return;
        }
    };
    decide_call(domain, &env.limits, decision, to, key, CallAnswer::Finding(Box::new(FindingToolAnswer::Write(reply))));
}
