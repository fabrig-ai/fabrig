//! People-child outputs translated to durable replies and one exhaustive party
//! request dispatcher (`domain/people.md`, section 5.1). The former chain's
//! observable precedence remains in `party_decisions`: person-origin goals
//! before live task proposals before history; current escalation revisions
//! before history. Other families select disjoint variants.

use super::note_routes::edit_note_request;
use super::parties::{adopt_request, change_policy_request, make_service_request, pool_route, roles_route};
use super::party_controls::{
    amend_route, answer_question_request, cancel_request, move_request, person_task_request, release_request,
    say_request, stop_request,
};
use super::party_decisions::{escalation_request, proposal_request};
use crate::routing::Work;
use crate::{Core, Event, Held, Key, Limits, Now, Record, Request, Write};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_people as people;
use skein_lib::{Env, Queue, ReplyTo};

pub(super) fn route_people(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    event: people::Event,
) -> Queue<Request> {
    if let people::Event::Forget { task, .. } = &event {
        drop(core.archived_chats.remove(task));
    }
    let mut unused = None;
    let event = match event {
        people::Event::Decided { request, outcome } => {
            if let Some(flight) = core.applications.get_mut(&request)
                && flight.part
            {
                flight.part = false;
                people::application_part_done(&mut core.people, request);
                return super::application_routes::answer(flight.connector, request, outcome);
            }
            if let Some(flight) = core.applications.remove(&request) {
                unused = flight.carried;
                if let people::Outcome::Applied { .. } = &outcome {
                    unused = None;
                }
            }
            let key_unused = super::key_routes::finished(core, request, &outcome);
            assert!(unused.is_none() || key_unused.is_none(), "one carried owner per people flight");
            unused = unused.or(key_unused);
            people::Event::Decided { request, outcome }
        }
        people::Event::SignOut { reply_to, sign_in } => {
            if let Some(person) = core.people.person(sign_in, env.now, env.wall) {
                super::result_reads::clear_person(core, person);
            }
            people::Event::SignOut { reply_to, sign_in }
        }
        other @ (people::Event::ChatState { .. }
        | people::Event::MoveChat { .. }
        | people::Event::ConversationSeen { .. }
        | people::Event::Deleting { .. }
        | people::Event::CloseProject { .. }
        | people::Event::ForgetProject { .. }
        | people::Event::Forget { .. }
        | people::Event::MakeService { .. }
        | people::Event::Seed { .. }
        | people::Event::Waiting { .. }
        | people::Event::ApplyRoles { .. }
        | people::Event::SignedIn { .. }
        | people::Event::Roles { .. }
        | people::Event::Act { .. }
        | people::Event::Whose { .. }
        | people::Event::Ask { .. }
        | people::Event::Restore { .. }) => other,
    };
    let mut child = Queue::uncapped();
    super::step::step_people(core, env, event, &mut child);
    super::application_routes::first(unused, tag_people(core, env, child, work))
}

pub(super) fn sign_in(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    reply_to: ReplyTo,
    identity: people::Identity,
    held: Option<u64>,
) -> Queue<Request> {
    let mut out = Queue::uncapped();
    if !core.people.ready() {
        out.push(Request::Held(Box::new(Held::PeopleReply {
            to: reply_to,
            reply: people::Reply::Again(people::Again::NotReady),
        })));
    } else if core.fill >= env.limits.fill_mark
        || core.counters.deployment().people == u64::MAX
        || core.counters.deployment().sign_ins == u64::MAX
    {
        out.push(Request::Now(Box::new(Now::SignInRefused { to: reply_to })));
    } else {
        let person = crate::fresh(&mut core.counters, crate::Family::Person).expect("person counter available");
        let sign_in = crate::fresh(&mut core.counters, crate::Family::SignIn).expect("sign-in counter available");
        let kind = if identity.key.provider == core.settings.deployment_provider {
            people::Kind::Service
        } else {
            people::Kind::Person
        };
        work.push(Work::Event(Event::People(people::Event::SignedIn {
            reply_to,
            person,
            sign_in,
            identity,
            kind,
            held,
        })));
    }
    out
}
#[expect(clippy::too_many_lines, reason = "one exhaustive party vocabulary routes its bounded child decision")]
pub(crate) fn tag_people(
    core: &mut Core,
    env: &Env<Limits>,
    mut child: Queue<people::Request>,
    work: &mut Queue<Work>,
) -> Queue<Request> {
    let mut pending = Queue::uncapped();
    for _ in 0..child.len() {
        pending.push(child.pop().expect("party output count"));
    }
    let mut routed = super::party::Routed::new(work);
    for _ in 0..env.limits.routes {
        let Some(next) = pending.pop() else { break };
        let request = match next {
            people::Request::Act { request, connector: _, act } => {
                super::deployment_acts::route(core, env, &mut routed.work, &mut routed.out, request, *act);
                continue;
            }
            people::Request::Acted { to, outcome } => {
                if outcome == people::Outcome::Again(people::Again::Busy) {
                    core.touch_deployment();
                }
                Request::Held(Box::new(Held::Acted { to, outcome }))
            }
            people::Request::ServiceMade { request, outcome } => {
                routed.work.push(Work::Event(Event::People(people::Event::Decided { request, outcome })));
                continue;
            }
            people::Request::Save { record } => {
                super::view_routes::touch_people_row(core, &record);
                Request::Write(Write::Save(Record::People(record)))
            }
            people::Request::Erase { key } => {
                match key {
                    people::Key::Answer(_) | people::Key::Acted(_) => core.touch_deployment(),
                    people::Key::Person(_)
                    | people::Key::ReadResults { .. }
                    | people::Key::EndedRead { .. }
                    | people::Key::ReadConversation { .. }
                    | people::Key::SignIn(_)
                    | people::Key::Roles(_)
                    | people::Key::Deletion(_)
                    | people::Key::Policy(_) => {}
                }
                Request::Write(Write::Erase(Key::People(key)))
            }
            people::Request::Reply { to, reply } => match reply {
                reply @ (people::Reply::Full { .. }
                | people::Reply::Again(people::Again::Busy)
                | people::Reply::Outcome(people::Outcome::Again(people::Again::Busy))) => {
                    core.touch_deployment();
                    Request::Held(Box::new(Held::PeopleReply { to, reply }))
                }
                people::Reply::Outcome(people::Outcome::Applied { connector, token }) => {
                    Request::Held(Box::new(Held::Applied { to, connector, token }))
                }
                reply @ (people::Reply::Signer(_)
                | people::Reply::SignedIn { .. }
                | people::Reply::SignedOut
                | people::Reply::Again(..)
                | people::Reply::Refused(..)
                | people::Reply::Outcome(
                    people::Outcome::Compound { .. }
                    | people::Outcome::CompoundRefused { .. }
                    | people::Outcome::Again(..)
                    | people::Outcome::SignedOut
                    | people::Outcome::MarkedSeen { .. }
                    | people::Outcome::NoteEdited { .. }
                    | people::Outcome::ServiceMade { .. }
                    | people::Outcome::Adopted { .. }
                    | people::Outcome::Created { .. }
                    | people::Outcome::Proposed { .. }
                    | people::Outcome::Withdrawn { .. }
                    | people::Outcome::Stopped { .. }
                    | people::Outcome::Cancelled { .. }
                    | people::Outcome::Lifted { .. }
                    | people::Outcome::Released { .. }
                    | people::Outcome::PersonTaken { .. }
                    | people::Outcome::PersonHandedBack { .. }
                    | people::Outcome::PersonAnswered { .. }
                    | people::Outcome::Moved { .. }
                    | people::Outcome::ProposalDecided { .. }
                    | people::Outcome::Said { .. }
                    | people::Outcome::QuestionAnswered { .. }
                    | people::Outcome::Amended { .. }
                    | people::Outcome::RolesSet { .. }
                    | people::Outcome::PolicyChanged { .. }
                    | people::Outcome::Renamed { .. }
                    | people::Outcome::ProjectMade { .. }
                    | people::Outcome::ProjectRemoved { .. }
                    | people::Outcome::SettingsChanged { .. }
                    | people::Outcome::MovedToNewest { .. }
                    | people::Outcome::PoolSet { .. }
                    | people::Outcome::EscalationDecided { .. }
                    | people::Outcome::AccountSignInStarted { .. }
                    | people::Outcome::AccountProgramDeclared { .. }
                    | people::Outcome::AccountKeyKept { .. }
                    | people::Outcome::AccountMovingOut { .. }
                    | people::Outcome::AccountFinishing { .. }
                    | people::Outcome::AccountMovedIn { .. }
                    | people::Outcome::AccountChanged
                    | people::Outcome::AccountRemoved { .. }
                    | people::Outcome::AccountSignedOut { .. }
                    | people::Outcome::Refused(..),
                )) => Request::Held(Box::new(Held::PeopleReply { to, reply })),
            },
            people::Request::RolesApplied { .. } => continue,
            people::Request::RolesRefused { .. } | people::Request::RestoreRefused { .. } => {
                core.start_collect_failure();
                assert!(core.start_failure().is_some(), "people refusal names its fit");
                continue;
            }
            people::Request::Route { request, person, project, role, mut ask, models } => {
                if !crate::model_names::resolve(&mut ask, &models, &core.settings.models) {
                    routed.work.push(Work::Event(Event::People(people::Event::Decided {
                        request,
                        outcome: people::Outcome::Refused(people::Refusal::Unknown),
                    })));
                    continue;
                }
                let admission = if *ask == (people::Ask::RemoveProject { project }) {
                    Ok(())
                } else {
                    crate::acts::request(core, &env.limits, person, project, role, &ask, env.wall)
                };
                if let Err(why) = admission {
                    routed.work.push(Work::Event(Event::People(people::Event::Decided {
                        request,
                        outcome: people::Outcome::Refused(why),
                    })));
                    continue;
                }
                let party = super::party::Party { request, person, project, role };
                match *ask {
                    people::Ask::RemoveProject { project } => {
                        let result = if !core.open_project(project) {
                            Err(people::Refusal::Unknown)
                        } else if role == Some(people::Role::Owner) {
                            crate::deletion::begin(core, env, &mut routed.work, &mut routed.out, request, project)
                        } else {
                            Err(people::Refusal::Role { held: role })
                        };
                        if let Err(why) = result {
                            routed.work.push(Work::Event(Event::People(people::Event::Decided {
                                request,
                                outcome: people::Outcome::Refused(why),
                            })));
                        }
                    }
                    people::Ask::Compound { operations, .. } => {
                        crate::compound::route(core, env, &mut routed.work, party, operations);
                    }
                    people::Ask::Application { scope, connector, .. } => {
                        super::application_routes::admitted(core, env, &mut routed, party, connector, scope);
                    }
                    people::Ask::TakePerson { task, .. } => {
                        person_task_request(core, &mut routed, party, crate::person_task::Action::Take { task });
                    }
                    people::Ask::HandBackPerson { task, .. } => {
                        person_task_request(core, &mut routed, party, crate::person_task::Action::HandBack { task });
                    }
                    people::Ask::AnswerPerson { task, revision, result, .. } => {
                        person_task_request(
                            core,
                            &mut routed,
                            party,
                            crate::person_task::Action::Answer { task, revision, result },
                        );
                    }
                    people::Ask::Stop { task, .. } => stop_request(core, env, &mut routed, party, task),
                    people::Ask::Cancel { task, reason, grace, .. } => {
                        cancel_request(core, env, &mut routed, party, task, reason, grace);
                    }
                    people::Ask::Lift { hold, revision, with_tasks } => {
                        super::party_controls::lift_request(core, &mut routed, party, hold, revision, with_tasks);
                    }
                    people::Ask::Release { task, revision, again, .. } => {
                        release_request(core, env, &mut routed, party, task, revision, again);
                    }
                    people::Ask::MakeService { role: service_role, .. } => {
                        make_service_request(core, env, party, &mut pending, service_role);
                    }
                    people::Ask::EditNote { name, scope, change, .. } => {
                        edit_note_request(core, &mut routed, party, name, scope, *change);
                    }
                    people::Ask::Adopt { adoption, .. } => {
                        adopt_request(core, env, &mut routed, party, adoption);
                    }
                    people::Ask::SetPool { person: beneficiary, budget, .. } => {
                        pool_route(core, env, &mut routed, party, beneficiary, budget);
                    }
                    people::Ask::DecideProposal { proposer, proposal, decision, running, .. } => proposal_request(
                        core,
                        env,
                        &mut routed,
                        party,
                        super::party::Proposal { proposer, number: proposal, decision, running },
                    ),
                    people::Ask::SetRoles { holdings, .. } => {
                        roles_route(core, env, &mut routed, party, holdings);
                    }
                    people::Ask::Amend { task, revision, amendment, release, .. } => {
                        amend_route(
                            core,
                            env,
                            &mut routed,
                            party,
                            super::party::Amendment { task, revision, change: amendment, release },
                            super::amendment_reads::Evidence::new(&env.limits.tasks),
                        );
                    }
                    people::Ask::Rename { name, .. } => {
                        super::projects::rename_request(core, env, &mut routed, party, name);
                    }
                    people::Ask::MoveToNewest { revision } => {
                        let outcome = match super::settings::move_to_newest(core, env, &mut routed.out, revision) {
                            Ok(moved) => people::Outcome::MovedToNewest {
                                newest: core.current.newest(),
                                moved: moved.moved,
                                stayed: moved.stayed,
                            },
                            Err(why) => people::Outcome::Refused(why),
                        };
                        routed.work.push(Work::Event(Event::People(people::Event::Decided { request, outcome })));
                    }
                    people::Ask::ChangeSettings { against, .. } => {
                        super::settings::admitted(core, env, &mut routed, party, against);
                    }
                    people::Ask::MakeProject { name, policy } => {
                        super::projects::make_project_request(core, env, &mut routed, party, name, policy);
                    }
                    people::Ask::ChangePolicy { change, .. } => {
                        change_policy_request(core, env, &mut routed, party, change);
                    }
                    people::Ask::DecideEscalation { task, revision, decision, .. } => {
                        escalation_request(core, env, &mut routed, party, task, revision, decision);
                    }
                    people::Ask::Create { batch, .. } => super::create::create_request(
                        core,
                        env,
                        &mut routed,
                        super::create::Creation { request, person, project, role: role.expect("admitted role"), batch },
                    ),
                    people::Ask::Propose { action, reason, .. } => match action {
                        people::Action::Settings { against, .. } => {
                            super::settings::propose_admitted(core, env, &mut routed, party, against, reason);
                        }
                        people::Action::Batch(batch) => super::propose::propose_request(
                            core,
                            env,
                            &mut routed,
                            super::create::Creation {
                                request,
                                person,
                                project,
                                role: role.expect("admitted role"),
                                batch,
                            },
                            reason,
                        ),
                        people::Action::Amend { task, revision, amendment } => super::propose::amend_request(
                            core,
                            env,
                            &mut routed.work,
                            super::propose::Edit {
                                request,
                                person,
                                project,
                                role,
                                task,
                                revision,
                                amendment: *amendment,
                                reason,
                            },
                        ),
                    },
                    people::Ask::AccountChange { holder, change } => {
                        let holder = super::account_requests::holder(holder, person);
                        match super::key_routes::admitted(core, request, holder, Some(person), change) {
                            Ok(change) => {
                                routed.work.push(Work::Event(Event::Account(accounts::Event::Change {
                                    request,
                                    holder,
                                    change,
                                })));
                            }
                            Err(why) => routed.work.push(Work::Event(Event::People(people::Event::Decided {
                                request,
                                outcome: people::Outcome::Refused(why),
                            }))),
                        }
                    }
                    people::Ask::AccountSignIn { holder, provider } => {
                        let holder = super::account_requests::holder(holder, person);
                        routed.work.push(Work::Event(Event::Account(accounts::Event::SignIn {
                            request,
                            holder,
                            by: person,
                            provider,
                        })));
                    }
                    people::Ask::AccountSignOut { holder, source } => {
                        let holder = super::account_requests::holder(holder, person);
                        routed.work.push(Work::Event(Event::Account(accounts::Event::SignOut {
                            request,
                            holder,
                            source,
                        })));
                    }
                    people::Ask::Withdraw { proposal, .. } => super::propose::withdraw_request(
                        core,
                        &mut routed,
                        super::propose::Withdrawal { request, person, project, proposal },
                    ),
                    people::Ask::MarkSeen { seen } => match seen {
                        people::Seen::Results { through } => {
                            let mut loading = false;
                            for result in core.people.cached_results(person).unwrap_or(&[]) {
                                loading |= result.position <= through
                                    && result.forgettable.is_some()
                                    && core.people.conversation_mark_loading(result.task, request);
                            }
                            if loading {
                                routed.work.push(Work::Event(Event::People(people::Event::Decided {
                                    request,
                                    outcome: people::Outcome::Again(people::Again::Busy),
                                })));
                                continue;
                            }
                            routed.work.push(Work::Event(Event::People(people::Event::Decided {
                                request,
                                outcome: people::Outcome::MarkedSeen { seen },
                            })));
                            for result in core.people.cached_results(person).unwrap_or(&[]) {
                                if result.position <= through
                                    && core.tasks.task(result.task).is_none()
                                    && let Some(owner) = crate::forget::Owner::of_result(person, result)
                                {
                                    let turns = if result.forgettable.expect("forgettable owner").turns_read {
                                        owner.last_reply
                                    } else {
                                        None
                                    };
                                    if crate::forget::forgets(
                                        &owner,
                                        crate::forget::Reading {
                                            results: through,
                                            turns,
                                            named: core.tasks.stub(owner.task).is_some(),
                                        },
                                    )
                                    .is_some()
                                    {
                                        routed.work.push(Work::Forget { owner });
                                    }
                                }
                            }
                        }
                        people::Seen::Conversation { task, .. }
                            if core.people.conversation_mark_loading(task, request) =>
                        {
                            routed.work.push(Work::Event(Event::People(people::Event::Decided {
                                request,
                                outcome: people::Outcome::Again(people::Again::Busy),
                            })));
                        }
                        people::Seen::Conversation { task, .. } => match core.tasks.task(task) {
                            Some(row) => {
                                if core.conversation_requester(row) == Some(person)
                                    && core.people.role(person, row.project).is_some()
                                {
                                    routed.work.push(Work::Event(Event::People(people::Event::ConversationSeen {
                                        project: row.project,
                                        request,
                                        last_reply: row.last_reply,
                                    })));
                                } else {
                                    routed.work.push(Work::Event(Event::People(people::Event::Decided {
                                        request,
                                        outcome: people::Outcome::Refused(people::Refusal::Role {
                                            held: core.people.role(person, row.project),
                                        }),
                                    })));
                                }
                            }
                            None => {
                                routed.out.push(Request::Held(Box::new(Held::Load(Box::new(Now::SeenConversation {
                                    request,
                                    task,
                                })))));
                            }
                        },
                    },
                    people::Ask::Say { task, words, .. } => {
                        say_request(core, env, &mut routed, party, task, words);
                    }
                    people::Ask::AnswerQuestion { task, question, words, .. } => {
                        answer_question_request(core, env, &mut routed, party, task, question, words);
                    }
                    people::Ask::Move { task, reason, .. } => {
                        move_request(core, env, &mut routed, party, task, reason);
                    }
                }
                continue;
            }
        };
        routed.out.push(request);
    }
    assert!(pending.is_empty(), "party_routes_drained");
    routed.finish(work)
}
