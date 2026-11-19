//! Retained procedure and resource hold accounts
//! (temper/docs/design/domain/forge/change/repairs.md;
//! temper/docs/design/domain/forge/change/README.md, 3;
//! temper/docs/design/domain/forge/branches.md).
use crate as forge;
use crate::routing::{append, decimal};
use jig_domain_authority as authority;
use jig_domain_core_vocabulary::effects::connector::EffectDescription;
use jig_domain_tasks as tasks;
use skein_lib::List;

pub fn findings(words: &mut List<u8>, findings: &[authority::Finding]) {
    for finding in findings {
        append(words, b"; ");
        append(
            words,
            match finding {
                authority::Finding::Executor { .. } => b"executor outside its delegation authority",
                authority::Finding::Tasks { .. } => b"no remaining task authority",
                authority::Finding::Oversized => b"request exceeds its configured bounds",
                authority::Finding::UnknownProject => b"project policy missing",
                authority::Finding::UnknownRole => b"role missing",
                authority::Finding::Authority { lacks, .. } => {
                    for (missing, name) in [
                        (lacks.tools, b"tool authority".as_slice()),
                        (lacks.grants, b"resource grants"),
                        (lacks.executors, b"executor authority"),
                        (lacks.tasks, b"task capacity"),
                        (lacks.depth, b"delegation depth"),
                        (lacks.spend, b"spend"),
                        (lacks.deadline, b"deadline authority"),
                        (lacks.notes, b"note scopes"),
                        (lacks.models, b"models"),
                    ] {
                        if missing {
                            append(words, b"lacks ");
                            append(words, name);
                            append(words, b"; ");
                        }
                    }
                    continue;
                }
                authority::Finding::Spend { .. } | authority::Finding::RunBudget => {
                    b"insufficient spend for the delegate"
                }
                authority::Finding::Grant { .. } => b"operation has no authority grant",
                authority::Finding::Required { .. } => b"required fact missing or pending",
                authority::Finding::Failed { .. } => b"required fact failed",
                authority::Finding::Unguarded { .. } => b"required fact cannot be guarded",
                authority::Finding::ResourceAccess => b"resource access refused",
                authority::Finding::ResourceUnavailable => b"resource unavailable",
                authority::Finding::Tool => b"tool family not granted",
                authority::Finding::Arithmetic => b"authority accounting overflow",
                authority::Finding::RunCap => b"run budget exceeds its cap",
                authority::Finding::Deadline => b"deadline passed",
                authority::Finding::Account => b"model account unusable",
                authority::Finding::Writer => b"resource has another writer",
                authority::Finding::Reference => b"task reference not granted",
                authority::Finding::Scope { .. } => b"note scope not granted",
                authority::Finding::Price { .. } => b"effect price exceeds its funding",
                authority::Finding::Unpermitted => b"request not permitted",
                authority::Finding::Undecidable => b"proposal decision not permitted",
                authority::Finding::PeriodSpend => b"period spend exceeds its ceiling",
            },
        );
    }
}

pub fn effect(words: &mut List<u8>, description: &EffectDescription) {
    append(
        words,
        match description.effect.kind {
            3 => b"open pull request",
            4 => b"merge pull request",
            5 => b"post review",
            6 => b"set status",
            7 => b"post comment",
            8 => b"create issue",
            9 => b"create branch",
            10 => b"edit pull request",
            11 => b"update pull request",
            12 => b"close or reopen pull request",
            13 => b"edit issue",
            14 => b"set reviewers",
            15 => b"delete branch",
            16 => b"dispatch workflow",
            17 => b"rerun job",
            19 => b"fast-forward branch",
            _ => b"forge effect",
        },
    );
    append(words, b" on ");
    for (index, segment) in description.effect.name.segments.iter().enumerate() {
        if index > 0 {
            append(words, b"/");
        }
        append(words, segment);
    }
    if words.len() > 180 {
        words.truncate(177);
        append(words, b"...");
    }
}

pub fn append_recovery(task: u64, revision: u64, words: &mut List<u8>) {
    let mut tail = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    append(&mut tail, b"; resolve this reason, then task release ");
    append(&mut tail, &decimal(task));
    append(&mut tail, b" --hold ");
    append(&mut tail, &decimal(revision));
    append(&mut tail, b"; or task cancel ");
    append(&mut tail, &decimal(task));
    append(&mut tail, b" REASON");
    let room = tasks::HOLD_ACCOUNT_BYTES.checked_sub(tail.len()).expect("recovery fits the account");
    if words.len() > room {
        words.truncate(room.saturating_sub(3));
        append(words, b"...");
    }
    append(words, tail.as_slice());
}

pub fn drift_change(words: &mut List<u8>, change: &forge::DriftChange) {
    match change {
        forge::DriftChange::Branch { expected, observed } => {
            append(words, b"expected ");
            append(words, &forge::commit_text(expected));
            match observed {
                Some(head) => {
                    append(words, b", observed ");
                    append(words, &forge::commit_text(head));
                }
                None => append(words, b", observed deletion"),
            }
        }
        forge::DriftChange::PullClosed { number } => {
            append(words, b"pull request closed: ");
            append(words, &decimal(*number));
        }
        forge::DriftChange::Retargeted { before, after, .. } => {
            append(words, b"retargeted from ");
            append(words, before);
            append(words, b" to ");
            append(words, after);
        }
    }
}
