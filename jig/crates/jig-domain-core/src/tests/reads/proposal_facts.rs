//! Decided cards read the same reason and scope as their live proposal.
use crate::tests::*;
use crate::{
    reads,
    routing::{Work, settings::dispatch},
    settings as s,
};

const PROJECT: u32 = 1;
const KIND: u32 = 7;
const REASON: &[u8] = b"keep the project settings reason";

#[test]
fn a_decided_settings_proposal_keeps_its_reason_and_projects() {
    for state in [
        tasks::ProposalState::Accepted { by: tasks::Party::Person(1) },
        tasks::ProposalState::Rejected { by: tasks::Party::Person(1), reason: b"no".as_slice().into() },
    ] {
        let (mut core, mut limits) = watch_core();
        limits.documents.text_bytes = 8;
        let identity = core.people.identity(1).unwrap().clone();
        core.people =
            people::Domain::new(&limits.people, Box::new([]), core.settings.deployment_provider, Box::new([]));
        core.people.start_restore(Wall::EPOCH);
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
        for record in [
            people::Stored::Person { number: 1, identity },
            people::Stored::SignIn {
                number: 10,
                person: 1,
                expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()),
            },
            people::Stored::Roles {
                project: 0,
                holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
            },
            people::Stored::Roles {
                project: PROJECT,
                holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
            },
        ] {
            people::step(&mut core.people, &env, people::Event::Restore { record }, &mut Queue::uncapped());
        }
        finish_people(&mut core.people, &env);
        core.kinds = Box::new([s::Kind {
            number: KIND,
            scope: s::KindScope::Project,
            connector: 1,
            value_bytes: 64,
            per_scope: 4,
            charters: 0,
        }]);
        let mut work = Queue::uncapped();
        let output = dispatch::propose_run(
            &mut core,
            &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            &mut work,
            ReplyTo::new(Token::new(91)),
            CallKey { task: 1, attempt: 7, completion: 1, position: 1 },
            s::Proposed {
                against: 1,
                reason: REASON.into(),
                items: Box::new([s::Item {
                    setting: s::Setting::Application { kind: KIND, scope: PROJECT, number: 1 },
                    edit: s::Edit::Set(s::Value::Application(b"value".as_slice().into())),
                }]),
            },
        );
        let Some(Work::Event(Event::Tasks(tasks::Event::Propose { mut proposal, .. }))) = work.pop() else {
            panic!("task proposal admitted: {output:?}")
        };
        proposal.state = state.clone();
        let number = proposal.number;
        let change = match state {
            tasks::ProposalState::Accepted { .. } => tasks::Change::ProposalAccepted,
            tasks::ProposalState::Rejected { .. } => tasks::Change::ProposalRejected,
            tasks::ProposalState::Pending { .. } | tasks::ProposalState::Withdrawn => unreachable!(),
        };
        let row = stored_row(
            Record::Tasks(tasks::Stored::History(tasks::History {
                task: 1,
                revision: 1,
                by: tasks::Party::Person(1),
                change,
                reason: b"outer history".as_slice().into(),
                proposal: Some(Box::new(proposal)),
                amendment: None,
                at: Wall::EPOCH,
            })),
            Some(core.tasks.task(1).unwrap().clone()),
        );
        let Held::ReadLoad { load, .. } =
            read_start(&mut core, &limits, 10, reads::Read::Proposal { proposer: Proposer::Task(1), proposal: number })
        else {
            panic!("retained proposal load")
        };
        let reads::Answer::Ended(ended) =
            loaded_answer(&mut core, &limits, load, reads::Loaded::Page { rows: Box::new([row]), next: None }).unwrap()
        else {
            panic!("ended proposal")
        };
        let reads::EndedValue::Proposal(proposal) = ended.value else { panic!("proposal facts") };
        assert_eq!(proposal.reason, Text { bytes: REASON.into(), cut: false });
        let RetainedAction::Settings { projects, .. } = proposal.action else { panic!("settings facts") };
        assert_eq!(projects.as_slice(), &[PROJECT]);
    }
}
