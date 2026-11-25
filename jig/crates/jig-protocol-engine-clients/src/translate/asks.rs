//! Wire asks into the people vocabulary (client/domain/contract/requests.md).
use super::Untranslatable;
use crate::generated::client as mapped;
use alloc::boxed::Box;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_domain_people as p;
use skein_lib::{List, Reader, length::len_u64};
#[derive(Debug)]
pub(crate) struct NamedAsk {
    pub ask: p::Ask,
    pub models: Box<[Box<[u8]>]>,
}

pub(crate) fn named(record: w::Ask, models: &[crate::Model], limits: &w::Limits) -> Result<NamedAsk, Untranslatable> {
    let mut names = List::with_capacity(p::MODELS);
    let mut ask = decode(record, models, limits, &mut names)?;
    let original = names.into_boxed();
    let mut canonical = original.clone();
    canonical.sort_unstable();
    let mut moved = [0_u64; 64];
    for (at, name) in original.iter().enumerate() {
        for (to, other) in canonical.iter().enumerate() {
            if name == other {
                *moved.get_mut(at).expect("bounded names") =
                    1_u64.checked_shl(u32::try_from(to).expect("bounded model index")).unwrap_or(0);
                break;
            }
        }
    }
    if !original.is_empty() {
        p::remap_models(&mut ask, &moved);
    }
    Ok(NamedAsk { ask, models: canonical })
}

#[cfg(test)]
pub(crate) fn ask(record: w::Ask, models: &[crate::Model], limits: &w::Limits) -> Result<p::Ask, Untranslatable> {
    Ok(named(record, models, limits)?.ask)
}

#[expect(clippy::too_many_lines, reason = "one total match names every wire ask")]
fn decode(
    record: w::Ask,
    models: &[crate::Model],
    limits: &w::Limits,
    names: &mut List<Box<[u8]>>,
) -> Result<p::Ask, Untranslatable> {
    Ok(match record {
        w::Ask::PauseWorker(value) => {
            p::Ask::PauseWorker { worker: value.worker, mode: mapped::worker_pause_mode_to_domain(value.mode) }
        }
        w::Ask::ReadLists(value) => p::Ask::AccountChange {
            holder: p::AccountHolder::Own,
            change: p::AccountChange::ReadLists { provider: value.provider },
        },
        w::Ask::Restore(value) => p::Ask::Restore { revision: value.revision, reason: value.reason },
        w::Ask::MoveToNewest(value) => p::Ask::MoveToNewest { revision: value.revision },
        w::Ask::ChangeModel(_) => return Err(Untranslatable { path: w::paths::CHANGE_MODEL_ACT }),
        w::Ask::ChangeSettings(value) => {
            let against = value.against;
            let record = w::Ask::ChangeSettings(value);
            let mut bytes = skein_lib::Writer::new(skein_lib::length::usize_of(
                record.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
            ));
            record.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("measured ask");
            p::Ask::ChangeSettings {
                projects: Box::default(),
                against,
                digest: crate::sign_in::digest(&bytes.finish()).expose().try_into().expect("32-byte digest"),
            }
        }
        w::Ask::Propose(value) => super::operations::propose(value, models, limits, names)?,
        w::Ask::Withdraw(value) => p::Ask::Withdraw { project: value.project, proposal: value.proposal },
        w::Ask::Delegate(value) => super::operations::delegate(value, models, limits, names)?,
        w::Ask::Create(value) => super::operations::create(value, models, limits, names)?,
        w::Ask::Compound(value) => super::operations::compound(value, models, limits, names)?,
        w::Ask::Talk(value) => {
            let v = value;
            p::Ask::Talk { project: v.project, task: v.task, words: v.words }
        }
        w::Ask::Leave(value) => p::Ask::Leave { project: value.project, task: value.task },
        w::Ask::Message(value) => {
            let v = value;
            p::Ask::Say { project: v.project, task: v.task, words: v.words }
        }
        w::Ask::Answer(value) => {
            let v = value;
            super::operations::answer(v.project, v.on)
        }
        w::Ask::Take(value) => {
            let v = value;
            p::Ask::TakePerson { project: v.project, task: v.task }
        }
        w::Ask::HandBack(value) => {
            let v = value;
            p::Ask::HandBackPerson { project: v.project, task: v.task }
        }
        w::Ask::Decide(value) => {
            let v = value;
            super::operations::decide(v.project, v.on)?
        }
        w::Ask::Lift(value) => {
            let v = value;
            p::Ask::Lift { hold: v.hold, revision: v.revision, with_tasks: v.with_tasks }
        }
        w::Ask::Release(value) => {
            let v = value;
            p::Ask::Release { project: v.project, task: v.task, revision: v.revision.value, again: v.again }
        }
        w::Ask::Amend(value) => {
            let v = value;
            p::Ask::Amend {
                release: None,
                project: v.project,
                task: v.task,
                revision: match v.revision {
                    Some(value) => revision(value),
                    None => 0,
                },
                amendment: amendment(v.change, v.reason, models, limits, names)?,
            }
        }
        w::Ask::Stop(value) => {
            let v = value;
            p::Ask::Stop { project: v.project, task: v.task, reason: v.reason }
        }
        w::Ask::Conclude(value) => p::Ask::Conclude { project: value.project, task: value.task },
        w::Ask::Cancel(value) => {
            let v = value;
            p::Ask::Cancel { project: v.project, task: v.task, reason: v.reason, grace: v.grace }
        }
        w::Ask::MoveTask(value) => {
            let v = value;
            p::Ask::Move { project: v.project, task: v.task, reason: v.reason }
        }
        w::Ask::Seen(value) => {
            let v = value;
            p::Ask::MarkSeen { seen: mapped::seen_through_to_domain(v.through) }
        }
        w::Ask::ReissueService(value) => p::Ask::ReissueService { party: value.party },
        w::Ask::MakeService(value) => {
            let v = value;
            p::Ask::MakeService { project: v.project, name: v.name, role: mapped::held_role_to_domain(v.role) }
        }
        w::Ask::AccountChange(value) => p::Ask::AccountChange {
            holder: mapped::ask_account_holder_to_domain(value.holder),
            change: account_change(value.change)?,
        },
        w::Ask::SignIn(value) => p::Ask::AccountSignIn {
            holder: mapped::ask_account_holder_to_domain(value.holder),
            provider: value.provider,
        },
        w::Ask::SignOut(value) => {
            p::Ask::AccountSignOut { holder: mapped::ask_account_holder_to_domain(value.holder), source: value.source }
        }
        w::Ask::SetVisibility(value) => p::Ask::SetVisibility { project: value.project, public: value.public },
        w::Ask::RemoveProject(value) => p::Ask::RemoveProject { project: value.project, grace: value.grace },
        w::Ask::Rename(value) => {
            let v = value;
            p::Ask::Rename { project: v.project, name: project_name(v.name, w::paths::RENAME_NAME)? }
        }
        w::Ask::MakeProject(value) => {
            let v = value;
            p::Ask::MakeProject { name: project_name(v.name, w::paths::MAKE_PROJECT_NAME)?, policy: v.policy }
        }
        w::Ask::SetPool(value) => {
            let v = value;
            p::Ask::SetPool { project: v.project, person: v.party, budget: v.budget }
        }
        w::Ask::EditNote(value) => {
            let v = value;
            p::Ask::EditNote {
                project: v.project,
                name: v.note,
                scope: Box::new(note_scope(v.scope, v.project, w::paths::EDIT_NOTE_SCOPE)?),
                change: Box::new(p::NoteChange::Correct {
                    description: v.description,
                    body: v.body,
                    references: v.references.into_boxed(),
                    recalled: recalled(v.revision, w::paths::EDIT_NOTE_REVISION)?,
                }),
            }
        }
        w::Ask::DeleteNote(value) => {
            let v = value;
            p::Ask::EditNote {
                project: v.project,
                name: v.note,
                scope: Box::new(note_scope(v.scope, v.project, w::paths::DELETE_NOTE_SCOPE)?),
                change: Box::new(p::NoteChange::Delete {
                    recalled: recalled(v.revision, w::paths::DELETE_NOTE_REVISION)?,
                }),
            }
        }
        w::Ask::ChangePolicy(value) => {
            let v = value;
            p::Ask::ChangePolicy { project: v.project, change: policy(v.edit, limits, names)? }
        }
        w::Ask::SetRoles(value) => {
            let v = value;
            let mut holdings = List::with_capacity(v.holdings.capacity());
            for item in v.holdings.into_boxed() {
                holdings.push(mapped::holding_to_domain(item)).expect("same capacity");
            }
            p::Ask::SetRoles { project: v.project, holdings: holdings.into_boxed() }
        }
    })
}
fn revision(value: w::Revision) -> u64 {
    value.value
}
fn recalled(value: w::Revision, path: w::Path) -> Result<u32, Untranslatable> {
    match u32::try_from(value.value) {
        Ok(value) => Ok(value),
        Err(_) => Err(Untranslatable { path }),
    }
}
pub(super) fn proposer(value: &w::Proposer) -> p::Proposer {
    mapped::proposer_to_domain(value.clone())
}
pub(super) fn proposal_decision(value: w::Decision) -> Result<p::ProposalDecision, Untranslatable> {
    match value {
        w::Decision::Accepted => Ok(p::ProposalDecision::Accept),
        w::Decision::Rejected(v) => Ok(p::ProposalDecision::Reject { reason: v.reason }),
        w::Decision::PassedUp => Ok(p::ProposalDecision::Pass),
        w::Decision::Released(_) | w::Decision::LeftHeld(_) => {
            Err(Untranslatable { path: w::paths::DECIDE_PROPOSAL_DECISION })
        }
    }
}
pub(super) fn escalation_decision(value: w::Decision) -> Result<p::EscalationDecision, Untranslatable> {
    match value {
        w::Decision::Released(v) => match &v.again {
            None => Ok(p::EscalationDecision::Release),
            Some(_) => Err(Untranslatable { path: w::paths::DECIDE_ESCALATION_DECISION }),
        },
        w::Decision::Rejected(v) => Ok(p::EscalationDecision::Reject { reason: v.reason }),
        w::Decision::LeftHeld(v) => Ok(p::EscalationDecision::Reject { reason: v.reason }),
        w::Decision::PassedUp => Ok(p::EscalationDecision::Pass),
        w::Decision::Accepted => Err(Untranslatable { path: w::paths::DECIDE_ESCALATION_DECISION }),
    }
}
pub(super) fn person_answer(value: w::PersonAnswer) -> p::PersonResult {
    mapped::person_answer_to_domain(value)
}
fn pattern(value: w::Pattern) -> p::Pattern {
    mapped::pattern_to_domain(value)
}
fn note_scope(value: w::NoteScope, project: u32, path: w::Path) -> Result<p::NoteScope, Untranslatable> {
    Ok(match value {
        w::NoteScope::Tasks(_) => return Err(Untranslatable { path }),
        w::NoteScope::Party(value) => p::NoteScope::Party { party: value.party },
        w::NoteScope::Deployment => p::NoteScope::Deployment,
        w::NoteScope::Project(v) => {
            if v.value != project {
                return Err(Untranslatable { path });
            }
            p::NoteScope::Project
        }
        w::NoteScope::Task(v) => {
            if v.project != project {
                return Err(Untranslatable { path });
            }
            p::NoteScope::Goal { goal: v.task }
        }
        w::NoteScope::Resource(v) => {
            if v.project != project {
                return Err(Untranslatable { path });
            }
            p::NoteScope::Resources { connector: v.connector, pattern: pattern(v.pattern) }
        }
    })
}
pub(super) fn spec(value: w::Specification) -> p::Spec {
    mapped::specification_to_domain(value)
}
pub(super) fn wake(value: w::WakePolicy) -> p::WakePolicy {
    mapped::wake_policy_to_domain(value)
}
pub(super) fn amendment(
    value: w::Amendment,
    reason: Box<[u8]>,
    models: &[crate::Model],
    limits: &w::Limits,
    names: &mut List<Box<[u8]>>,
) -> Result<p::Amendment, Untranslatable> {
    let v = value;
    let authority = match v.authority {
        Some(value) => Some(authority(value, limits, names)?),
        None => None,
    };
    Ok(p::Amendment {
        tracked: match v.tracked {
            Some(value) => Some(super::operations::tracking(value)),
            None => None,
        },
        addressed: match v.addressed {
            Some(value) => Some(mapped::address_change_to_domain(value)),
            None => None,
        },
        instructions: v.instructions,
        spec: match v.spec {
            Some(value) => Some(spec(value)),
            None => None,
        },
        wake: match v.wake {
            Some(value) => Some(wake(value)),
            None => None,
        },
        dependencies: match v.dependencies {
            Some(value) => Some(List::into_boxed(value)),
            None => None,
        },
        authority,
        llm: super::llm::people(v.llm, models, w::paths::AMENDMENT_LLM)?,
        reason,
    })
}
pub(super) fn count(bytes: &[u8], path: w::Path) -> Result<u32, Untranslatable> {
    match skein_codec::piece::count(bytes) {
        Ok(count) => Ok(count),
        Err(_) => Err(Untranslatable { path }),
    }
}
pub(super) fn item(reader: &mut Reader<'_>, path: w::Path) -> Result<Box<[u8]>, Untranslatable> {
    match skein_codec::piece::split(reader) {
        Ok(Some(bytes)) => Ok(bytes),
        Ok(None) | Err(_) => Err(Untranslatable { path }),
    }
}
fn sum(total: &mut u64, bytes: Option<u64>, ceiling: u32, path: w::Path) -> Result<(), Untranslatable> {
    *total = total.checked_add(bytes.ok_or(Untranslatable { path })?).ok_or(Untranslatable { path })?;
    if *total > u64::from(ceiling) {
        return Err(Untranslatable { path });
    }
    Ok(())
}
pub(super) fn grant(value: w::Grant) -> p::Grant {
    mapped::grant_to_domain(value)
}
pub(super) fn grants(
    bytes: &[u8],
    ceiling: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[p::Grant]>, Untranslatable> {
    let count = count(bytes, path)?;
    let mut values = List::with_capacity(count);
    let mut reader = Reader::new(bytes);
    let mut total = 0;
    for _ in 0..count {
        let bytes = skein_codec::piece::counted(&mut reader);
        let value = match w::Grant::decode(limits, &mut Reader::new(&bytes)) {
            Ok(value) => grant(value),
            Err(_) => return Err(Untranslatable { path }),
        };
        sum(&mut total, pattern_bytes(&value.pattern, size_of::<p::Grant>()), ceiling, path)?;
        values.push(value).expect("counted pieces");
    }
    Ok(values.into_boxed())
}
pub(super) fn authority(
    value: w::Authority,
    limits: &w::Limits,
    names: &mut List<Box<[u8]>>,
) -> Result<p::Authority, Untranslatable> {
    let v = value;
    let delegation = mapped::delegation_to_domain(v.delegation);
    let path = w::paths::AUTHORITY_NOTE_RESOURCES;
    let count = count(&v.note_resources, path)?;
    let mut resources = List::with_capacity(count);
    let mut reader = Reader::new(&v.note_resources);
    let mut total = 0;
    for _ in 0..count {
        let bytes = skein_codec::piece::counted(&mut reader);
        let Ok(value) = w::ResourceScope::decode(limits, &mut Reader::new(&bytes)) else {
            return Err(Untranslatable { path });
        };
        let value = mapped::resource_scope_to_domain(value);
        let size = pattern_bytes(&value.pattern, size_of::<p::ResourceScope>());
        sum(&mut total, size, limits.authority_note_resources, path)?;
        resources.push(value).expect("counted pieces");
    }
    Ok(p::Authority {
        tools: v.tools,
        models: super::llm::requested(v.models, names)?,
        grants: grants(&v.grants, limits.authority_grants, w::paths::AUTHORITY_GRANTS, limits)?,
        delegation,
        spend: v.spend,
        deadline: match v.deadline {
            Some(value) => Some(skein_codec::convert::wall_to_domain(value)),
            None => None,
        },
        notes: v.notes,
        note_resources: resources.into_boxed(),
    })
}
pub(super) fn requirement(value: w::PolicyRequirement) -> p::Requirement {
    mapped::policy_requirement_to_domain(value)
}
fn policy(
    value: w::PolicyEdit,
    limits: &w::Limits,
    names: &mut List<Box<[u8]>>,
) -> Result<p::PolicyChange, Untranslatable> {
    Ok(match value {
        w::PolicyEdit::Templates(v) => super::operations::templates(v),
        w::PolicyEdit::RoleSpend(v) => p::PolicyChange::RoleSpend { role: v.role, period_spend: v.period_spend },
        w::PolicyEdit::ProjectSpend(v) => p::PolicyChange::ProjectSpend { period_spend: v.period_spend },
        w::PolicyEdit::Role(v) => {
            let v = v.value;
            p::PolicyChange::Role(p::PolicyRole {
                number: v.number,
                authority: authority(v.authority, limits, names)?,
                period_spend: v.period_spend,
                requests: v.requests,
                decides: v.decides,
            })
        }
        w::PolicyEdit::Requirements(v) => {
            let path = w::paths::POLICY_EDIT_REQUIREMENTS_REQUIREMENTS;
            let count = count(v.requirements.as_ref(), path)?;
            let mut values = List::with_capacity(count);
            let mut reader = Reader::new(v.requirements.as_ref());
            let mut total = 0;
            for _ in 0..count {
                let bytes = skein_codec::piece::counted(&mut reader);
                let value = match w::PolicyRequirement::decode(limits, &mut Reader::new(&bytes)) {
                    Ok(value) => requirement(value),
                    Err(_) => return Err(Untranslatable { path }),
                };
                sum(
                    &mut total,
                    p::requirements_bytes(core::slice::from_ref(&value)),
                    limits.policy_edit_requirements_requirements,
                    path,
                )?;
                values.push(value).expect("counted pieces");
            }
            p::PolicyChange::Requirements { requirements: values.into_boxed() }
        }
        w::PolicyEdit::Projections(v) => p::PolicyChange::Projections {
            grants: grants(
                v.grants.as_ref(),
                limits.policy_edit_projections_grants,
                w::paths::POLICY_EDIT_PROJECTIONS_GRANTS,
                limits,
            )?,
        },
        w::PolicyEdit::Permissions(v) => {
            let mut mappings = List::with_capacity(v.mappings.len());
            for value in v.mappings.into_boxed() {
                mappings.push(mapped::policy_permission_to_domain(value)).expect("same capacity");
            }
            p::PolicyChange::Permissions { mappings: mappings.into_boxed() }
        }
    })
}

fn pattern_bytes(value: &p::Pattern, fixed: usize) -> Option<u64> {
    let mut bytes = u64::try_from(fixed).ok()?;
    bytes =
        bytes.checked_add(len_u64(value.segments.len()).checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?)?;
    for segment in &value.segments {
        bytes = bytes.checked_add(len_u64(segment.len()))?;
    }
    bytes.checked_add(match &value.last {
        p::Last::Exact(bytes) | p::Last::Open(bytes) => len_u64(bytes.len()),
    })
}

fn account_change(value: w::AccountChange) -> Result<p::AccountChange, Untranslatable> {
    Ok(match value {
        // These receive typed interim refusals before core translation (domain/terminal-login.md, 5).
        w::AccountChange::LoginProgram(row) => {
            p::AccountChange::LoginProgram { provider: row.provider, name: row.name }
        }
        w::AccountChange::CancelLogin(row) => p::AccountChange::CancelLogin { number: row.number },
        w::AccountChange::LoginEnded(row) => {
            p::AccountChange::LoginEnded { number: row.number, exit: mapped::login_exit_to_domain(row.exit) }
        }
        w::AccountChange::DeclareProgram(row) => {
            if jig_domain_link::SourceName::new(row.name.as_ref()).is_none() {
                return Err(Untranslatable { path: w::paths::ACCOUNT_CHANGE_DECLARE_PROGRAM_NAME });
            }
            p::AccountChange::DeclareProgram { provider: row.provider, join: row.join, name: row.name }
        }
        w::AccountChange::CheckProgram(row) => p::AccountChange::CheckProgram { source: row.source },
        w::AccountChange::MoveOut(row) => p::AccountChange::MoveOut { account: row.account },
        w::AccountChange::Written(row) => p::AccountChange::Written { number: row.number },
        w::AccountChange::FinishSignIn(row) => {
            p::AccountChange::FinishSignIn { source: row.source, digest: row.digest }
        }
        w::AccountChange::MoveIn(row) => p::AccountChange::MoveIn { digest: row.digest },
        w::AccountChange::AddKey(row) => {
            p::AccountChange::AddKey { provider: row.provider, join: row.join, digest: row.digest }
        }
        w::AccountChange::ReplaceKey(row) => p::AccountChange::ReplaceKey { source: row.source, digest: row.digest },
        w::AccountChange::RemoveKey(row) => p::AccountChange::RemoveKey { source: row.source },
        w::AccountChange::Resume(row) => p::AccountChange::Resume { source: row.source },
        w::AccountChange::Remove(row) => p::AccountChange::Remove { account: row.account },
        w::AccountChange::MoveAccount(row) => p::AccountChange::MoveAccount { account: row.account, place: row.place },
        w::AccountChange::MoveSource(row) => {
            p::AccountChange::MoveSource { account: row.account, source: row.source, place: row.place }
        }
        w::AccountChange::SetUses(row) => p::AccountChange::SetUses {
            account: row.account,
            uses: match row.uses {
                None => None,
                Some(uses) => {
                    let mut rows = List::with_capacity(uses.len());
                    for use_ in uses.into_boxed() {
                        rows.push(mapped::account_use_to_domain(use_)).expect("reserved uses");
                    }
                    Some(rows.into_boxed())
                }
            },
        },
    })
}

fn project_name(bytes: Box<[u8]>, path: w::Path) -> Result<Box<str>, Untranslatable> {
    match skein_codec::convert::text_to_domain(bytes) {
        Ok(name) => Ok(name),
        Err(_) => Err(Untranslatable { path }),
    }
}
