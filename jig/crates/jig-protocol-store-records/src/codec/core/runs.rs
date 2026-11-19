//! Core durable rows translated exhaustively (protocol/store/records.md, 1).
use super::{View, domain, task_problem, too_large, wire};
use alloc::boxed::Box;
use skein_lib::{List, Wall};
pub(super) fn encode_deployment(source: &domain::Deployment, _limits: &View) -> wire::Deployment {
    let domain::Deployment {
        id,
        tasks,
        people,
        sign_ins,
        messages,
        runs,
        calls,
        connector_rows,
        applications,
        commits,
        projects,
        restart_wait_ms,
    } = source;
    wire::Deployment {
        id: *id,
        tasks: *tasks,
        people: *people,
        sign_ins: *sign_ins,
        messages: *messages,
        runs: *runs,
        calls: *calls,
        connector_rows: *connector_rows,
        applications: *applications,
        commits: *commits,
        projects: *projects,
        restart_wait_ms: *restart_wait_ms,
    }
}
pub(super) fn decode_deployment(source: wire::Deployment) -> domain::Deployment {
    let wire::Deployment {
        id,
        tasks,
        people,
        sign_ins,
        messages,
        runs,
        calls,
        connector_rows,
        applications,
        commits,
        projects,
        restart_wait_ms,
    } = source;
    domain::Deployment {
        id,
        tasks,
        people,
        sign_ins,
        messages,
        runs,
        calls,
        connector_rows,
        applications,
        commits,
        projects,
        restart_wait_ms,
    }
}
pub(super) fn encode_terminal_record(
    source: &domain::TerminalRecord,
    limits: &View,
) -> Result<wire::TerminalRecord, wire::Problem> {
    let domain::TerminalRecord { task, attempt, cumulative, end } = source;
    Ok(wire::TerminalRecord {
        task: *task,
        attempt: *attempt,
        cumulative: *cumulative,
        end: wire::EndBytes {
            value: match crate::codec::tasks::carried::end(end, &limits.tasks) {
                Ok(value) => value,
                Err(error) => return Err(task_problem(error)),
            },
        },
    })
}
pub(super) fn decode_terminal_record(
    source: wire::TerminalRecord,
    limits: &View,
) -> Result<domain::TerminalRecord, wire::Problem> {
    let wire::TerminalRecord { task, attempt, cumulative, end } = source;
    Ok(domain::TerminalRecord {
        task,
        attempt,
        cumulative,
        end: match crate::codec::tasks::carried::read_end(&end.value, &limits.tasks) {
            Ok(value) => value,
            Err(error) => return Err(task_problem(error)),
        },
    })
}
pub(super) fn encode_turn_proof(source: &domain::TurnProof, _limits: &View) -> wire::TurnProof {
    let domain::TurnProof { turn, cumulative, read } = source;
    wire::TurnProof { turn: *turn, cumulative: *cumulative, read: *read }
}
pub(super) fn decode_turn_proof(source: wire::TurnProof) -> domain::TurnProof {
    let wire::TurnProof { turn, cumulative, read } = source;
    domain::TurnProof { turn, cumulative, read }
}
pub(super) fn encode_run_proof(source: &domain::RunProof, limits: &View) -> Result<wire::RunProof, wire::Problem> {
    let domain::RunProof {
        task,
        attempt,
        conversation,
        endpoint,
        accounts,
        placement,
        account_failure,
        account_terminal,
        credentials,
        window,
        crossed,
        host,
        offered,
        outstanding,
        turn,
        terminal,
    } = source;
    Ok(wire::RunProof {
        task: *task,
        attempt: *attempt,
        conversation: *conversation,
        endpoint: *endpoint,
        accounts: encode_accounts(accounts, limits)?,
        placement: match placement {
            Some(channel) => Some(channel.raw()),
            None => None,
        },
        account_failure: *account_failure,
        account_terminal: *account_terminal,
        credentials: encode_credentials(credentials, limits)?,
        window: encode_window(*window, limits),
        crossed: *crossed,
        host: super::encode_host_kind(host),
        offered: *offered,
        outstanding: encode_outstanding(outstanding, limits)?,
        turn: match turn {
            Some(value) => Some(encode_turn_proof(value, limits)),
            None => None,
        },
        terminal: match terminal {
            Some(value) => Some(encode_terminal_record(value, limits)?),
            None => None,
        },
    })
}
pub(super) fn decode_run_proof(source: wire::RunProof, limits: &View) -> Result<domain::RunProof, wire::Problem> {
    let wire::RunProof {
        task,
        attempt,
        conversation,
        endpoint,
        accounts,
        placement,
        account_failure,
        account_terminal,
        credentials,
        window,
        crossed,
        host,
        offered,
        outstanding,
        turn,
        terminal,
    } = source;
    Ok(domain::RunProof {
        task,
        attempt,
        conversation,
        endpoint,
        accounts: decode_accounts(accounts),
        placement: match placement {
            Some(channel) => Some(skein_lib::Token::new(channel)),
            None => None,
        },
        account_failure,
        account_terminal,
        credentials: decode_credentials(credentials),
        window: match window {
            Some(value) => {
                let result = read_window(value);
                Some(result)
            }
            None => None,
        },
        crossed,
        host: super::decode_host_kind(host),
        offered,
        outstanding: decode_outstanding(outstanding),
        turn: match turn {
            Some(value) => {
                let result = decode_turn_proof(value);
                Some(result)
            }
            None => None,
        },
        terminal: match terminal {
            Some(value) => Some(decode_terminal_record(value, limits)?),
            None => None,
        },
    })
}
pub(super) fn encode_turn_record(source: &domain::TurnRecord, _limits: &View) -> wire::TurnRecord {
    let domain::TurnRecord { opens_window, task, attempt, turn, spent, read, at, transcript } = source;
    wire::TurnRecord {
        opens_window: *opens_window,
        task: *task,
        attempt: *attempt,
        turn: *turn,
        spent: *spent,
        read: *read,
        at: at.as_nanos(),
        transcript: transcript.clone(),
    }
}
pub(super) fn decode_turn_record(source: wire::TurnRecord) -> domain::TurnRecord {
    let wire::TurnRecord { opens_window, task, attempt, turn, spent, read, at, transcript } = source;
    domain::TurnRecord { opens_window, task, attempt, turn, spent, read, at: Wall::from_nanos(at), transcript }
}
pub(super) fn encode_proposal_decision_record(
    source: &domain::ProposalDecisionRecord,
    limits: &View,
) -> Result<wire::ProposalDecisionRecord, wire::Problem> {
    let domain::ProposalDecisionRecord { project, proposer, proposal, kind, by, at, choice, created } = source;
    Ok(wire::ProposalDecisionRecord {
        project: *project,
        proposer: super::encode_proposer(proposer, limits)?,
        proposal: *proposal,
        kind: super::encode_proposal_kind(kind),
        by: *by,
        at: at.as_nanos(),
        choice: super::encode_proposal_choice(choice),
        created: encode_created(*created, limits),
    })
}
pub(super) fn decode_proposal_decision_record(source: wire::ProposalDecisionRecord) -> domain::ProposalDecisionRecord {
    let wire::ProposalDecisionRecord { project, proposer, proposal, kind, by, at, choice, created } = source;
    domain::ProposalDecisionRecord {
        project,
        proposer: super::decode_proposer(proposer),
        proposal,
        kind: super::decode_proposal_kind(kind),
        by,
        at: Wall::from_nanos(at),
        choice: super::decode_proposal_choice(choice),
        created: match created {
            Some(value) => {
                let result = read_created(value);
                Some(result)
            }
            None => None,
        },
    }
}
pub(super) fn encode_escalation_decision_record(
    source: &domain::EscalationDecisionRecord,
    limits: &View,
) -> Result<wire::EscalationDecisionRecord, wire::Problem> {
    let domain::EscalationDecisionRecord { project, requester, task, revision, by, at, decision } = source;
    Ok(wire::EscalationDecisionRecord {
        project: *project,
        requester: *requester,
        task: *task,
        revision: *revision,
        by: *by,
        at: at.as_nanos(),
        decision: super::encode_escalation_decision(decision, limits)?,
    })
}
pub(super) fn decode_escalation_decision_record(
    source: wire::EscalationDecisionRecord,
) -> domain::EscalationDecisionRecord {
    let wire::EscalationDecisionRecord { project, requester, task, revision, by, at, decision } = source;
    domain::EscalationDecisionRecord {
        project,
        requester,
        task,
        revision,
        by,
        at: Wall::from_nanos(at),
        decision: super::decode_escalation_decision(decision),
    }
}
fn encode_window(value: Option<domain::CurrentWindow>, _limits: &View) -> Option<wire::Window> {
    match value {
        Some(domain::CurrentWindow { attempt, turn, closed }) => Some(wire::Window { attempt, turn, closed }),
        None => None,
    }
}
fn read_window(value: wire::Window) -> domain::CurrentWindow {
    let wire::Window { attempt, turn, closed } = value;
    domain::CurrentWindow { attempt, turn, closed }
}
fn encode_created(value: Option<(u64, u64)>, _limits: &View) -> Option<wire::Created> {
    match value {
        Some((first, last)) => Some(wire::Created { first, last }),
        None => None,
    }
}
fn read_created(value: wire::Created) -> (u64, u64) {
    let wire::Created { first, last } = value;
    (first, last)
}
fn encode_credentials(values: &[u32], limits: &View) -> Result<List<wire::Credential>, wire::Problem> {
    let count = match u32::try_from(values.len()) {
        Ok(value) => value,
        Err(error) => return Err(too_large(error)),
    };
    if count > limits.core.run_proof_credentials {
        return Err(super::problem(skein_codec::Reason::Bound));
    }
    let mut result = List::with_capacity(count);
    for value in values {
        result.push(wire::Credential { value: *value }).expect("same number of credentials");
    }
    Ok(result)
}
fn decode_credentials(values: List<wire::Credential>) -> Box<[u32]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(value.value).expect("same number of credentials");
    }
    result.into_boxed()
}

fn encode_accounts(
    values: &[jig_domain_authority::AccountBinding],
    limits: &View,
) -> Result<List<wire::AccountBinding>, wire::Problem> {
    let count = match u32::try_from(values.len()) {
        Ok(count) => count,
        Err(error) => return Err(too_large(error)),
    };
    if count > limits.core.run_proof_accounts {
        return Err(super::problem(skein_codec::Reason::Bound));
    }
    let mut result = List::with_capacity(count);
    for value in values {
        result
            .push(wire::AccountBinding { provider: value.provider, account: value.account, source: value.source })
            .expect("same binding count");
    }
    Ok(result)
}
fn decode_accounts(values: List<wire::AccountBinding>) -> Box<[jig_domain_authority::AccountBinding]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        let wire::AccountBinding { provider, account, source } = value;
        result.push(jig_domain_authority::AccountBinding { provider, account, source }).expect("same binding count");
    }
    result.into_boxed()
}

fn encode_outstanding(values: &[u64], limits: &View) -> Result<List<wire::Number>, wire::Problem> {
    let count = match u32::try_from(values.len()) {
        Ok(count) => count,
        Err(error) => return Err(too_large(error)),
    };
    if count > limits.core.run_proof_outstanding {
        return Err(super::problem(skein_codec::Reason::Bound));
    }
    let mut result = List::with_capacity(count);
    for &value in values {
        result.push(wire::Number { value }).expect("same offer count");
    }
    Ok(result)
}
fn decode_outstanding(values: List<wire::Number>) -> Box<[u64]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(value.value).expect("same offer count");
    }
    result.into_boxed()
}
