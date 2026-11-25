//! The small store's owner records (protocol/client/small-store.md, 1).
use super::{answers, documents, pages, too_large};
use crate::Limits;
use crate::generated::client;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_domain::{Ask, Family};
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::{List, Reader, RequestRecord, Wall, Writer, bytes};

pub(super) fn text(value: &[u8], limits: &Limits) -> Result<c::Text, w::Problem> {
    match c::Text::new(bytes::copy_of(value), limits.client().text) {
        Ok(value) => Ok(value),
        Err(_) => Err(too_large()),
    }
}

fn person_answer(value: &w::PersonAnswer, limits: &Limits) -> Result<c::PersonAnswer, w::Problem> {
    Ok(match value {
        w::PersonAnswer::Choice(value) => {
            c::PersonAnswer::Choice { option: value.option, reason: text(&value.reason, limits)? }
        }
        w::PersonAnswer::Approved(value) => {
            let words = match &value.words {
                Some(value) => Some(text(value, limits)?),
                None => None,
            };
            c::PersonAnswer::Approved { state: bytes::copy_of(&value.state), words }
        }
        w::PersonAnswer::Words(value) => c::PersonAnswer::Words(text(&value.value, limits)?),
        w::PersonAnswer::Cannot(value) => c::PersonAnswer::Cannot { reason: text(&value.reason, limits)? },
    })
}
fn policy_edit(value: &w::PolicyEdit, limits: &Limits) -> Result<c::PolicyEdit, w::Problem> {
    Ok(match value {
        w::PolicyEdit::Templates(value) => super::operations::restore_templates(&value.overrides),
        w::PolicyEdit::RoleSpend(value) => {
            c::PolicyEdit::RoleSpend { role: value.role, period_spend: value.period_spend }
        }
        w::PolicyEdit::ProjectSpend(value) => c::PolicyEdit::ProjectSpend { period_spend: value.period_spend },
        w::PolicyEdit::Role(value) => {
            let value = &value.value;
            c::PolicyEdit::Role(c::PolicyEditRole {
                number: value.number,
                authority: documents::authority_fields(&value.authority, limits)?,
                period_spend: value.period_spend,
                requests: value.requests,
                decides: value.decides,
            })
        }
        w::PolicyEdit::Requirements(value) => c::PolicyEdit::Requirements {
            requirements: documents::policy_requirement_piece(&value.requirements, limits, limits.client().rows)?,
        },
        w::PolicyEdit::Projections(value) => c::PolicyEdit::Projections {
            grants: documents::grant_piece(&value.grants, limits, limits.client().authority)?,
        },
        w::PolicyEdit::Permissions(value) => {
            let mut mappings = List::with_capacity(value.mappings.len());
            for row in &value.mappings {
                mappings.push(documents::policy_permission_fields(row)).expect("reserved decoded mappings");
            }
            c::PolicyEdit::Permissions { mappings }
        }
    })
}

#[expect(clippy::too_many_lines, reason = "one total match restores every client ask")]
pub(crate) fn restore_ask(value: w::Ask, limits: &Limits) -> Result<c::Ask, w::Problem> {
    Ok(match value {
        w::Ask::ReadLists(value) => c::Ask::ReadLists { provider: value.provider },
        w::Ask::Restore(value) => super::asks::restore_ask_restore(&value, limits)?,
        w::Ask::PauseWorker(value) => {
            c::Ask::PauseWorker { worker: value.worker, mode: client::worker_pause_mode_to_domain(value.mode) }
        }
        w::Ask::MoveToNewest(value) => c::Ask::MoveToNewest(client::move_to_newest_to_domain(value)),
        w::Ask::ChangeModel(value) => c::Ask::ChangeModel(super::settings::restore_change_model(&value, limits)?),
        w::Ask::ChangeSettings(value) => {
            c::Ask::ChangeSettings(super::settings::restore_change_settings(&value, limits)?)
        }
        w::Ask::Propose(value) => super::operations::restore_propose(&value, limits)?,
        w::Ask::Withdraw(value) => c::Ask::Withdraw { project: value.project, proposal: value.proposal },
        w::Ask::Delegate(value) => super::operations::restore_delegate(&value, limits)?,
        w::Ask::Create(value) => super::operations::restore_create(&value, limits)?,
        w::Ask::Compound(value) => super::compound::restore(value, limits)?,
        w::Ask::Talk(value) => {
            c::Ask::Talk { project: value.project, task: value.task, words: pages::long_text(&value.words, limits)? }
        }
        w::Ask::Leave(value) => c::Ask::Leave { project: value.project, task: value.task },
        w::Ask::Message(value) => {
            c::Ask::Message { project: value.project, task: value.task, words: text(&value.words, limits)? }
        }
        w::Ask::Answer(value) => c::Ask::Answer {
            project: value.project,
            on: match &value.on {
                w::AnswerOn::Question(v) => {
                    c::AnswerOn::Question { task: v.task, message: v.message, words: text(&v.words, limits)? }
                }
                w::AnswerOn::Person(v) => c::AnswerOn::Person {
                    task: v.task,
                    revision: answers::revision(&v.revision),
                    answer: person_answer(&v.answer, limits)?,
                },
            },
        },
        w::Ask::Take(value) => c::Ask::Take { project: value.project, task: value.task },
        w::Ask::HandBack(value) => c::Ask::HandBack { project: value.project, task: value.task },
        w::Ask::Decide(value) => {
            c::Ask::Decide { project: value.project, on: client::decide_on_to_domain(value.on, limits)? }
        }
        w::Ask::Lift(value) => c::Ask::Lift {
            hold: value.hold,
            revision: c::Revision { value: value.revision },
            with_tasks: value.with_tasks,
        },
        w::Ask::Release(value) => c::Ask::Release {
            project: value.project,
            task: value.task,
            revision: answers::revision(&value.revision),
            again: value.again,
        },
        w::Ask::Amend(value) => c::Ask::Amend {
            project: value.project,
            task: value.task,
            revision: match &value.revision {
                Some(value) => Some(answers::revision(value)),
                None => None,
            },
            change: Box::new(pages::amendment_fields(&value.change, limits)?),
            reason: text(value.reason.as_ref(), limits)?,
        },
        w::Ask::Stop(value) => {
            c::Ask::Stop { project: value.project, task: value.task, reason: pages::long_text(&value.reason, limits)? }
        }
        w::Ask::Conclude(value) => c::Ask::Conclude { project: value.project, task: value.task },
        w::Ask::Cancel(value) => c::Ask::Cancel {
            project: value.project,
            task: value.task,
            reason: text(&value.reason, limits)?,
            grace: value.grace,
        },
        w::Ask::MoveTask(value) => {
            c::Ask::Move { project: value.project, task: value.task, reason: text(&value.reason, limits)? }
        }
        w::Ask::Seen(value) => c::Ask::Seen { through: answers::seen(&value.through) },
        w::Ask::EditNote(value) => c::Ask::EditNote {
            project: value.project,
            note: value.note,
            scope: answers::note_scope(&value.scope)?,
            revision: answers::revision(&value.revision),
            description: text(&value.description, limits)?,
            body: text(&value.body, limits)?,
            references: value.references.clone(),
        },
        w::Ask::DeleteNote(value) => c::Ask::DeleteNote {
            project: value.project,
            note: value.note,
            scope: answers::note_scope(&value.scope)?,
            revision: answers::revision(&value.revision),
        },
        w::Ask::ChangePolicy(value) => {
            c::Ask::ChangePolicy { project: value.project, edit: policy_edit(&value.edit, limits)? }
        }
        w::Ask::SetRoles(value) => {
            let mut holdings = List::with_capacity(value.holdings.len());
            for row in &value.holdings {
                holdings.push(client::holding_to_domain(row.clone())).expect("reserved decoded holdings");
            }
            c::Ask::SetRoles { project: value.project, holdings }
        }
        w::Ask::ReissueService(value) => c::Ask::ReissueService { party: value.party },
        w::Ask::MakeService(value) => c::Ask::MakeService {
            project: value.project,
            name: text(&value.name, limits)?,
            role: answers::held_role(&value.role),
        },
        w::Ask::AccountChange(value) => {
            c::Ask::AccountChange { holder: account_holder(&value.holder), change: account_change(&value.change)? }
        }
        w::Ask::SignIn(value) => c::Ask::SignIn { holder: account_holder(&value.holder), provider: value.provider },
        w::Ask::SignOut(value) => c::Ask::SignOut { holder: account_holder(&value.holder), source: value.source },
        w::Ask::SetVisibility(value) => c::Ask::SetVisibility { project: value.project, public: value.public },
        w::Ask::RemoveProject(value) => c::Ask::RemoveProject { project: value.project, grace: value.grace },
        w::Ask::Rename(value) => c::Ask::Rename { project: value.project, name: text(&value.name, limits)? },
        w::Ask::MakeProject(value) => c::Ask::MakeProject { name: text(&value.name, limits)?, policy: value.policy },
        w::Ask::SetPool(value) => c::Ask::SetPool { project: value.project, party: value.party, budget: value.budget },
    })
}

fn family_layout(family: u16, version: u32, limits: &Limits) -> Result<[u8; 32], w::Problem> {
    for owner in limits.client().owners {
        if owner.family == family && owner.version == version {
            return Ok(owner.layout);
        }
    }
    Err(super::piece_problem(skein_codec::Reason::Version))
}

pub fn table(value: &RequestRecord<jig_client_domain::Held>, limits: &Limits) -> Result<w::TableEntry, w::Problem> {
    let (owner, version, layout, request) = match &value.request.request {
        Ask::Jig(value) => {
            let record = super::asks::ask(value, limits.wire())?;
            let mut writer = Writer::new(skein_lib::length::usize_of(record.measure(limits.wire())?));
            record.write(limits.wire(), &mut writer).expect("measured ask");
            (w::Owner::Jig, w::VERSION, w::LAYOUT, writer.finish())
        }
        Ask::Family { scope, record: value } => (
            w::Owner::Family(w::Family { number: value.family }),
            value.version,
            family_layout(value.family, value.version, limits)?,
            super::envelope::request(*scope, value, limits.wire())?,
        ),
    };
    let first_sent = match value.first_sent {
        Some(value) => Some(value.as_nanos()),
        None => None,
    };
    Ok(w::TableEntry {
        asker: value.request.key.asker.0,
        number: value.request.key.number,
        key: value.key,
        scope: value.scope,
        first_sent,
        owner,
        version,
        layout,
        request,
    })
}
pub fn restore_table(
    value: w::TableEntry,
    limits: &Limits,
) -> Result<RequestRecord<jig_client_domain::Held>, w::Problem> {
    let request = match &value.owner {
        w::Owner::Jig => {
            if value.version != w::VERSION || value.layout != w::LAYOUT {
                return Err(super::piece_problem(skein_codec::Reason::Version));
            }
            let mut reader = Reader::new(&value.request);
            let ask = w::Ask::decode(limits.wire(), &mut reader)?;
            if reader.remaining() != 0 {
                return Err(super::piece_problem(skein_codec::Reason::Malformed));
            }
            Ask::Jig(Box::new(restore_ask(ask, limits)?))
        }
        w::Owner::Family(owner) => {
            if value.layout != family_layout(owner.number, value.version, limits)? {
                return Err(super::piece_problem(skein_codec::Reason::Version));
            }
            let (scope, bytes) = super::envelope::restore(&value.request, limits.wire())?;
            Ask::Family { scope, record: Family { family: owner.number, version: value.version, bytes } }
        }
    };
    let first_sent = match &value.first_sent {
        Some(value) => Some(Wall::from_nanos(*value)),
        None => None,
    };
    Ok(RequestRecord {
        key: value.key,
        scope: value.scope,
        first_sent,
        request: jig_client_domain::Held {
            key: jig_client_domain::Key { asker: jig_client_domain::Asker(value.asker), number: value.number },
            request,
        },
    })
}

fn account_holder(value: &w::AskAccountHolder) -> c::AskAccountHolder {
    client::ask_account_holder_to_domain(value.clone())
}

fn account_change(value: &w::AccountChange) -> Result<c::AccountChange, w::Problem> {
    match value {
        w::AccountChange::DeclareProgram(row) if !c::valid_program_name(&row.name) => {
            return Err(w::Problem {
                path: w::paths::ACCOUNT_CHANGE_DECLARE_PROGRAM_NAME,
                reason: skein_codec::Reason::Malformed,
            });
        }
        w::AccountChange::LoginProgram(row) if !c::valid_program_name(&row.name) => {
            return Err(w::Problem {
                path: w::paths::ACCOUNT_CHANGE_LOGIN_PROGRAM_NAME,
                reason: skein_codec::Reason::Malformed,
            });
        }
        w::AccountChange::DeclareProgram(_)
        | w::AccountChange::LoginProgram(_)
        | w::AccountChange::SetUses(_)
        | w::AccountChange::AddKey(_)
        | w::AccountChange::ReplaceKey(_)
        | w::AccountChange::RemoveKey(_)
        | w::AccountChange::Resume(_)
        | w::AccountChange::Remove(_)
        | w::AccountChange::MoveAccount(_)
        | w::AccountChange::MoveSource(_)
        | w::AccountChange::CheckProgram(_)
        | w::AccountChange::MoveOut(_)
        | w::AccountChange::Written(_)
        | w::AccountChange::MoveIn(_)
        | w::AccountChange::FinishSignIn(_)
        | w::AccountChange::CancelLogin(_)
        | w::AccountChange::LoginEnded(_) => {}
    }
    Ok(client::account_change_to_domain(value.clone()))
}
