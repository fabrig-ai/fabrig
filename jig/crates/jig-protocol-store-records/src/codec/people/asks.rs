//! Stored asks and their note and resource pieces (skein/docs/design/data/codec/maps.md, 3.3).
use super::compound::{decode_grace, encode_grace};
use super::{decode_numbers, decode_segments, domain, encode_numbers, encode_segments, wire};
use crate::generated::people as mapped;
use alloc::boxed::Box;
use skein_lib::List;

pub(crate) fn note_change_to_record(
    value: &domain::NoteChange,
    limits: &wire::Limits,
) -> Result<wire::NoteChange, wire::Problem> {
    Ok(match value {
        domain::NoteChange::Correct { description, body, references, recalled } => {
            wire::NoteChange::Correct(wire::NoteChangeCorrect {
                description: description.clone(),
                body: body.clone(),
                references: encode_numbers(
                    references,
                    limits,
                    limits.parameter_bytes_value,
                    limits.note_change_correct_references,
                )?,
                recalled: *recalled,
            })
        }
        domain::NoteChange::Delete { recalled } => {
            wire::NoteChange::Delete(wire::NoteChangeDelete { recalled: *recalled })
        }
    })
}

pub(crate) fn note_change_to_domain(
    value: wire::NoteChange,
    limits: &wire::Limits,
) -> Result<domain::NoteChange, wire::Problem> {
    Ok(match value {
        wire::NoteChange::Correct(value) => {
            let wire::NoteChangeCorrect { description, body, references, recalled } = value;
            domain::NoteChange::Correct {
                description,
                body,
                references: decode_numbers(references, limits, limits.parameter_bytes_value)?,
                recalled,
            }
        }
        wire::NoteChange::Delete(value) => {
            let wire::NoteChangeDelete { recalled } = value;
            domain::NoteChange::Delete { recalled }
        }
    })
}

pub(crate) fn resource_name_to_record(
    value: &domain::ResourceName,
    limits: &wire::Limits,
) -> Result<wire::ResourceName, wire::Problem> {
    let domain::ResourceName { connector, path } = value;
    Ok(wire::ResourceName {
        connector: *connector,
        path: encode_segments(path, limits, limits.parameter_bytes_value, limits.resource_name_path)?,
    })
}

pub(crate) fn resource_name_to_domain(
    value: wire::ResourceName,
    limits: &wire::Limits,
) -> Result<domain::ResourceName, wire::Problem> {
    let wire::ResourceName { connector, path } = value;
    Ok(domain::ResourceName { connector, path: decode_segments(path, limits, limits.parameter_bytes_value)? })
}

pub(crate) fn adoption_to_record(
    value: &domain::Adoption,
    limits: &wire::Limits,
) -> Result<wire::Adoption, wire::Problem> {
    let domain::Adoption { resource, role, options } = value;
    Ok(wire::Adoption {
        resource: resource_name_to_record(resource, limits)?,
        role: mapped::resource_role_to_record(role),
        options: encode_segments(options, limits, limits.parameter_bytes_value, limits.adoption_options)?,
    })
}

pub(crate) fn adoption_to_domain(
    value: wire::Adoption,
    limits: &wire::Limits,
) -> Result<domain::Adoption, wire::Problem> {
    let wire::Adoption { resource, role, options } = value;
    Ok(domain::Adoption {
        resource: resource_name_to_domain(resource, limits)?,
        role: mapped::resource_role_to_domain(role),
        options: decode_segments(options, limits, limits.parameter_bytes_value)?,
    })
}

// Stop and Leave share their stored payload. Leave has no domain reason.
#[expect(clippy::too_many_lines, reason = "an exhaustive translation of stored asks")]
pub fn ask_to_record(value: &domain::Ask, limits: &wire::Limits) -> Result<wire::Ask, wire::Problem> {
    Ok(match value {
        domain::Ask::Rename { project, name } => {
            wire::Ask::Rename(wire::AskRename { project: *project, name: skein_codec::convert::text_to_record(name) })
        }
        domain::Ask::MakeProject { name, policy } => wire::Ask::MakeProject(wire::AskMakeProject {
            name: skein_codec::convert::text_to_record(name),
            policy: *policy,
        }),
        domain::Ask::Compound { project, operations } => wire::Ask::Compound(wire::AskCompound {
            project: *project,
            operations: {
                let mut translated = List::with_capacity(operations.len());
                for item in operations {
                    let converted = mapped::operation_to_record(item, limits)?;
                    assert!(translated.push(converted).is_ok(), "one output per source item");
                }
                translated
            },
        }),
        domain::Ask::Create { project, batch } => wire::Ask::Create(wire::AskCreate {
            project: *project,
            batch: batch_to_record(batch, wire::paths::ASK_CREATE_BATCH, limits)?,
        }),
        domain::Ask::MarkSeen { seen } => wire::Ask::MarkSeen(wire::AskMarkSeen { seen: mapped::seen_to_record(seen) }),
        domain::Ask::Application { scope, connector, digest } => wire::Ask::Application(wire::AskApplication {
            scope: mapped::application_scope_to_record(scope),
            connector: *connector,
            digest: *digest,
        }),
        domain::Ask::EditNote { project, name, scope, change } => wire::Ask::EditNote(wire::AskEditNote {
            project: *project,
            name: *name,
            scope: mapped::note_scope_to_record(scope, limits)?,
            change: note_change_to_record(change, limits)?,
        }),
        domain::Ask::MakeService { project, name, role } => wire::Ask::MakeService(wire::AskMakeService {
            project: *project,
            name: name.clone(),
            role: mapped::role_to_record(role),
        }),
        domain::Ask::Adopt { project, adoption } => {
            wire::Ask::Adopt(wire::AskAdopt { project: *project, adoption: adoption_to_record(adoption, limits)? })
        }
        domain::Ask::Stop { project, task, reason } => {
            wire::Ask::Stop(wire::AskStop { project: *project, task: *task, reason: reason.clone() })
        }
        domain::Ask::Cancel { project, task, reason, grace } => wire::Ask::Cancel(wire::AskCancel {
            project: *project,
            task: *task,
            reason: reason.clone(),

            grace: encode_grace(*grace),
        }),
        domain::Ask::Release { project, task, revision, again } => wire::Ask::Release(wire::AskRelease {
            project: *project,
            task: *task,
            revision: *revision,
            again: match again {
                Some(item) => {
                    let converted = *item;
                    Some(converted)
                }
                None => None,
            },
        }),
        domain::Ask::TakePerson { project, task } => {
            wire::Ask::TakePerson(wire::AskTakePerson { project: *project, task: *task })
        }
        domain::Ask::HandBackPerson { project, task } => {
            wire::Ask::HandBackPerson(wire::AskHandBackPerson { project: *project, task: *task })
        }
        domain::Ask::AnswerPerson { project, task, revision, result } => {
            wire::Ask::AnswerPerson(wire::AskAnswerPerson {
                project: *project,
                task: *task,
                revision: *revision,
                result: mapped::person_result_to_record(result),
            })
        }
        domain::Ask::Move { project, task, reason } => {
            wire::Ask::MoveTask(wire::AskMove { project: *project, task: *task, reason: reason.clone() })
        }
        domain::Ask::DecideProposal { project, proposer, proposal, decision, running } => {
            wire::Ask::DecideProposal(wire::AskDecideProposal {
                project: *project,
                proposer: mapped::proposer_to_record(proposer),
                proposal: *proposal,
                decision: mapped::proposal_decision_to_record(decision),
                running: match running {
                    Some(item) => {
                        let converted = mapped::running_to_record(item);
                        Some(converted)
                    }
                    None => None,
                },
            })
        }
        domain::Ask::Say { project, task, words } => {
            wire::Ask::Say(wire::AskSay { project: *project, task: *task, words: words.clone() })
        }
        domain::Ask::AnswerQuestion { project, task, question, words } => {
            wire::Ask::AnswerQuestion(wire::AskAnswerQuestion {
                project: *project,
                task: *task,
                question: *question,
                words: words.clone(),
            })
        }
        domain::Ask::Amend { project, task, revision, amendment, release } => {
            wire::Ask::Amend(Box::new(wire::AskAmend {
                project: *project,
                task: *task,
                revision: *revision,
                amendment: super::amendment_to_record(amendment, limits)?,
                release: match release {
                    Some(item) => {
                        let converted = *item;
                        Some(converted)
                    }
                    None => None,
                },
            }))
        }
        domain::Ask::SetRoles { project, holdings } => wire::Ask::SetRoles(wire::AskSetRoles {
            project: *project,
            holdings: {
                let source = holdings;
                let count = match u32::try_from(source.len()) {
                    Ok(count) => count,
                    Err(_error) => return Err(wire::paths::ASK_SET_ROLES_HOLDINGS.problem(skein_codec::Reason::Bound)),
                };
                let mut translated = List::with_capacity(count);
                for item in source.as_ref() {
                    let converted = mapped::holding_to_record(item);
                    assert!(translated.push(converted).is_ok(), "one output per source item");
                }
                translated
            },
        }),
        domain::Ask::ChangePolicy { project, change } => wire::Ask::ChangePolicy(wire::AskChangePolicy {
            project: *project,
            change: super::policy_change_to_record(change, limits)?,
        }),
        domain::Ask::SetPool { project, person, budget } => {
            wire::Ask::SetPool(wire::AskSetPool { project: *project, person: *person, budget: *budget })
        }
        domain::Ask::DecideEscalation { project, task, revision, decision } => {
            wire::Ask::DecideEscalation(wire::AskDecideEscalation {
                project: *project,
                task: *task,
                revision: *revision,
                decision: mapped::escalation_decision_to_record(decision),
            })
        }
        domain::Ask::Propose { project, action, reason } => wire::Ask::Propose(wire::AskPropose {
            project: *project,
            action: super::action_to_record(action, limits)?,
            reason: reason.clone(),
        }),
        domain::Ask::Withdraw { project, proposal } => {
            wire::Ask::Withdraw(wire::AskWithdraw { project: *project, proposal: *proposal })
        }
        domain::Ask::AccountSignIn { holder, provider } => wire::Ask::AccountSignIn(wire::AskAccountSignIn {
            holder: mapped::account_holder_to_record(holder),
            provider: *provider,
        }),
        domain::Ask::AccountSignOut { holder, source } => wire::Ask::AccountSignOut(wire::AskAccountSignOut {
            holder: mapped::account_holder_to_record(holder),
            source: *source,
        }),
        domain::Ask::AccountChange { holder, change } => wire::Ask::AccountChange(wire::AskAccountChange {
            holder: mapped::account_holder_to_record(holder),
            change: mapped::account_change_to_record(change)?,
        }),
        domain::Ask::ChangeSettings { against, digest, projects } => {
            wire::Ask::ChangeSettings(wire::AskChangeSettings {
                against: *against,
                digest: *digest,
                projects: {
                    count(projects.len(), wire::paths::ASK_CHANGE_SETTINGS_PROJECTS)?;
                    List::from_boxed(projects.clone())
                },
            })
        }
        domain::Ask::MoveToNewest { revision } => {
            wire::Ask::MoveToNewest(wire::AskMoveToNewest { revision: *revision })
        }
        domain::Ask::Lift { hold, revision, with_tasks } => {
            wire::Ask::Lift(wire::AskLift { hold: *hold, revision: *revision, with_tasks: *with_tasks })
        }
        domain::Ask::RemoveProject { project, grace } => {
            wire::Ask::RemoveProject(wire::AskRemoveProject { project: *project, grace: encode_grace(*grace) })
        }
        domain::Ask::Restore { revision, reason } => {
            wire::Ask::Restore(wire::AskRestore { revision: *revision, reason: reason.clone() })
        }
        domain::Ask::SetVisibility { project, public } => {
            wire::Ask::SetVisibility(wire::AskSetVisibility { project: *project, public: *public })
        }
        domain::Ask::Talk { project, task, words } => {
            wire::Ask::Talk(wire::AskSay { project: *project, task: *task, words: words.clone() })
        }
        domain::Ask::Leave { project, task } => {
            wire::Ask::Leave(wire::AskStop { project: *project, task: *task, reason: Box::new([]) })
        }
        domain::Ask::Conclude { project, task } => {
            wire::Ask::Conclude(wire::AskConclude { project: *project, task: *task })
        }
        domain::Ask::Delegate { project, task, batch } => wire::Ask::Delegate(wire::AskDelegate {
            project: *project,
            task: *task,
            batch: batch_to_record(batch, wire::paths::ASK_DELEGATE_BATCH, limits)?,
        }),
        domain::Ask::PauseWorker { worker, mode } => wire::Ask::PauseWorker(wire::AskPauseWorker {
            worker: *worker,
            mode: mapped::worker_pause_mode_to_record(mode),
        }),
        domain::Ask::ReissueService { party } => wire::Ask::ReissueService(wire::ServiceParty { party: *party }),
    })
}

#[expect(clippy::too_many_lines, reason = "an exhaustive translation of stored asks")]
pub fn ask_to_domain(record: wire::Ask, limits: &wire::Limits) -> Result<domain::Ask, wire::Problem> {
    Ok(match record {
        wire::Ask::Rename(wire::AskRename { project, name }) => {
            domain::Ask::Rename { project, name: name_from_record(name, wire::paths::ASK_RENAME_NAME)? }
        }
        wire::Ask::MakeProject(wire::AskMakeProject { name, policy }) => {
            domain::Ask::MakeProject { name: name_from_record(name, wire::paths::ASK_MAKE_PROJECT_NAME)?, policy }
        }
        wire::Ask::Compound(wire::AskCompound { project, operations }) => domain::Ask::Compound {
            project,
            operations: {
                let mut translated = List::with_capacity(operations.len());
                for item in operations.into_boxed() {
                    let converted = mapped::operation_to_domain(item, limits)?;
                    assert!(translated.push(converted).is_ok(), "one output per source item");
                }
                translated
            },
        },
        wire::Ask::Create(wire::AskCreate { project, batch }) => {
            domain::Ask::Create { project, batch: batch_to_domain(batch, limits)? }
        }
        wire::Ask::MarkSeen(wire::AskMarkSeen { seen }) => domain::Ask::MarkSeen { seen: mapped::seen_to_domain(seen) },
        wire::Ask::Application(wire::AskApplication { scope, connector, digest }) => {
            domain::Ask::Application { scope: mapped::application_scope_to_domain(scope), connector, digest }
        }
        wire::Ask::EditNote(wire::AskEditNote { project, name, scope, change }) => domain::Ask::EditNote {
            project,
            name,
            scope: Box::new(mapped::note_scope_to_domain(scope, limits)?),
            change: Box::new(note_change_to_domain(change, limits)?),
        },
        wire::Ask::MakeService(wire::AskMakeService { project, name, role }) => {
            domain::Ask::MakeService { project, name, role: mapped::role_to_domain(role) }
        }
        wire::Ask::Adopt(wire::AskAdopt { project, adoption }) => {
            domain::Ask::Adopt { project, adoption: adoption_to_domain(adoption, limits)? }
        }
        wire::Ask::Stop(wire::AskStop { project, task, reason }) => domain::Ask::Stop { project, task, reason },
        wire::Ask::Cancel(wire::AskCancel { project, task, reason, grace }) => {
            domain::Ask::Cancel { project, task, reason, grace: decode_grace(grace) }
        }
        wire::Ask::Release(wire::AskRelease { project, task, revision, again }) => domain::Ask::Release {
            project,
            task,
            revision,
            again: match again {
                Some(item) => {
                    let converted = item;
                    Some(converted)
                }
                None => None,
            },
        },
        wire::Ask::TakePerson(wire::AskTakePerson { project, task }) => domain::Ask::TakePerson { project, task },
        wire::Ask::HandBackPerson(wire::AskHandBackPerson { project, task }) => {
            domain::Ask::HandBackPerson { project, task }
        }
        wire::Ask::AnswerPerson(wire::AskAnswerPerson { project, task, revision, result }) => {
            domain::Ask::AnswerPerson { project, task, revision, result: mapped::person_result_to_domain(result) }
        }
        wire::Ask::MoveTask(wire::AskMove { project, task, reason }) => domain::Ask::Move { project, task, reason },
        wire::Ask::DecideProposal(wire::AskDecideProposal { project, proposer, proposal, decision, running }) => {
            domain::Ask::DecideProposal {
                project,
                proposer: mapped::proposer_to_domain(proposer),
                proposal,
                decision: mapped::proposal_decision_to_domain(decision),
                running: match running {
                    Some(item) => {
                        let converted = mapped::running_to_domain(item);
                        Some(converted)
                    }
                    None => None,
                },
            }
        }
        wire::Ask::Say(wire::AskSay { project, task, words }) => domain::Ask::Say { project, task, words },
        wire::Ask::AnswerQuestion(wire::AskAnswerQuestion { project, task, question, words }) => {
            domain::Ask::AnswerQuestion { project, task, question, words }
        }
        wire::Ask::Amend(held) => {
            let wire::AskAmend { project, task, revision, amendment, release } = *held;
            domain::Ask::Amend {
                project,
                task,
                revision,
                amendment: super::amendment_to_domain(amendment, limits)?,
                release: match release {
                    Some(item) => {
                        let converted = item;
                        Some(converted)
                    }
                    None => None,
                },
            }
        }
        wire::Ask::SetRoles(wire::AskSetRoles { project, holdings }) => domain::Ask::SetRoles {
            project,
            holdings: {
                let mut translated = List::with_capacity(holdings.len());
                for item in holdings.into_boxed() {
                    let converted = mapped::holding_to_domain(item);
                    assert!(translated.push(converted).is_ok(), "one output per source item");
                }
                Box::from(translated)
            },
        },
        wire::Ask::ChangePolicy(wire::AskChangePolicy { project, change }) => {
            domain::Ask::ChangePolicy { project, change: super::policy_change_to_domain(change, limits)? }
        }
        wire::Ask::SetPool(wire::AskSetPool { project, person, budget }) => {
            domain::Ask::SetPool { project, person, budget }
        }
        wire::Ask::DecideEscalation(wire::AskDecideEscalation { project, task, revision, decision }) => {
            domain::Ask::DecideEscalation {
                project,
                task,
                revision,
                decision: mapped::escalation_decision_to_domain(decision),
            }
        }
        wire::Ask::Propose(wire::AskPropose { project, action, reason }) => {
            domain::Ask::Propose { project, action: super::action_to_domain(action, limits)?, reason }
        }
        wire::Ask::Withdraw(wire::AskWithdraw { project, proposal }) => domain::Ask::Withdraw { project, proposal },
        wire::Ask::AccountSignIn(wire::AskAccountSignIn { holder, provider }) => {
            domain::Ask::AccountSignIn { holder: mapped::account_holder_to_domain(holder), provider }
        }
        wire::Ask::AccountSignOut(wire::AskAccountSignOut { holder, source }) => {
            domain::Ask::AccountSignOut { holder: mapped::account_holder_to_domain(holder), source }
        }
        wire::Ask::AccountChange(wire::AskAccountChange { holder, change }) => domain::Ask::AccountChange {
            holder: mapped::account_holder_to_domain(holder),
            change: mapped::account_change_to_domain(change)?,
        },
        wire::Ask::ChangeSettings(wire::AskChangeSettings { against, digest, projects }) => {
            domain::Ask::ChangeSettings { against, digest, projects: projects.into_boxed() }
        }
        wire::Ask::MoveToNewest(wire::AskMoveToNewest { revision }) => domain::Ask::MoveToNewest { revision },
        wire::Ask::Lift(wire::AskLift { hold, revision, with_tasks }) => {
            domain::Ask::Lift { hold, revision, with_tasks }
        }
        wire::Ask::RemoveProject(wire::AskRemoveProject { project, grace }) => {
            domain::Ask::RemoveProject { project, grace: decode_grace(grace) }
        }
        wire::Ask::Restore(wire::AskRestore { revision, reason }) => domain::Ask::Restore { revision, reason },
        wire::Ask::SetVisibility(wire::AskSetVisibility { project, public }) => {
            domain::Ask::SetVisibility { project, public }
        }
        wire::Ask::Talk(wire::AskSay { project, task, words }) => domain::Ask::Talk { project, task, words },
        wire::Ask::Leave(wire::AskStop { project, task, reason: _ }) => domain::Ask::Leave { project, task },
        wire::Ask::Conclude(wire::AskConclude { project, task }) => domain::Ask::Conclude { project, task },
        wire::Ask::Delegate(wire::AskDelegate { project, task, batch }) => {
            domain::Ask::Delegate { project, task, batch: batch_to_domain(batch, limits)? }
        }
        wire::Ask::PauseWorker(wire::AskPauseWorker { worker, mode }) => {
            domain::Ask::PauseWorker { worker, mode: mapped::worker_pause_mode_to_domain(mode) }
        }
        wire::Ask::ReissueService(wire::ServiceParty { party }) => domain::Ask::ReissueService { party },
    })
}

fn count(len: usize, path: wire::Path) -> Result<u32, wire::Problem> {
    match u32::try_from(len) {
        Ok(count) => Ok(count),
        Err(_) => Err(path.problem(skein_codec::Reason::Bound)),
    }
}

fn batch_to_record(
    batch: &[domain::Member],
    path: wire::Path,
    limits: &wire::Limits,
) -> Result<List<wire::Member>, wire::Problem> {
    let mut rows = List::with_capacity(count(batch.len(), path)?);
    for member in batch {
        rows.push(super::member_to_record(member, limits)?).expect("one row per member");
    }
    Ok(rows)
}

fn batch_to_domain(batch: List<wire::Member>, limits: &wire::Limits) -> Result<Box<[domain::Member]>, wire::Problem> {
    let mut members = List::with_capacity(batch.len());
    for row in batch.into_boxed() {
        members.push(super::member_to_domain(row, limits)?).expect("one member per row");
    }
    Ok(members.into_boxed())
}

fn name_from_record(name: Box<[u8]>, path: wire::Path) -> Result<Box<str>, wire::Problem> {
    match skein_codec::convert::text_to_domain(name) {
        Ok(name) => Ok(name),
        Err(reason) => Err(path.problem(reason)),
    }
}
