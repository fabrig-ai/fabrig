//! A task's LLM choice on the wire reaches the core by configuration's indices, and a refused
//! model comes back by its name (client/domain/contract/requests.md, 2, and client/domain/contract/answers.md, 1; domain/tasks/README.md, 3).
use super::{amendment_setup, final_answer, fixtures, keyed, member_setup, owner_authority, route_real_core, w};
use jig_client::contract as c;
use jig_client_protocol_wire::Record as _;
use jig_core_kit::fixtures::live_row;
use jig_domain_tasks as tasks;
use skein_lib::{List, Writer};

/// A choice by name: a main model and effort, then each sub-agent's model and effort.
fn choice(main: (&[u8], Option<&[u8]>), subagents: &[(&[u8], Option<&[u8]>)]) -> w::LlmChoice {
    let named = |(model, effort): (&[u8], Option<&[u8]>)| w::ModelEffort {
        model: model.into(),
        effort: effort.map(Box::from),
        speed: None,
    };
    let mut items = List::with_capacity(u32::try_from(subagents.len()).expect("few sub-agents"));
    for item in subagents {
        items.push(named(*item)).expect("same capacity");
    }
    w::LlmChoice { later: None, account: None, main: named(main), subagents: items }
}
fn with_later(first: w::LlmChoice, later: w::LlmChoice) -> w::LlmChoice {
    let mut first = first;
    first.later = Some(w::LlmLater { main: later.main, subagents: later.subagents });
    first
}
/// A Create of one agent task in the member's project, run by the configured charter.
fn create(llm: w::LlmChoice) -> w::Ask {
    let task = w::NewTask {
        symbols: List::with_capacity(0),
        deadline: w::NewDeadline::Named,
        executor: w::Executor::Agent(w::ExecutorAgent { charter: 1 }),
        spec: w::Specification {
            asks: false,
            words: b"chosen".as_slice().into(),
            parameters: List::with_capacity(0),
            inputs: List::with_capacity(0),
        },
        contract: w::NewContract::Report(w::ContractReport { words: 16 }),
        authority: owner_authority(20),
        budget: w::NewBudget::Named,
        wake: None,
        dependencies: List::with_capacity(0),
        named: Box::new([]),
        tracked: None,
        label: 0,
        flags: w::TaskFlags {
            ended_by: w::EndedBy::Result,
            conversation: false,
            listed: false,
            retention: w::Retention::Kept,
        },
        llm: Some(llm),
    };
    let size = skein_lib::length::usize_of(task.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"));
    let mut out = Writer::new(size.checked_add(4).expect("piece size"));
    out.put(&u32::try_from(size).expect("task size").to_be_bytes()).expect("piece length");
    task.write(&jig_client_protocol_wire::CEILINGS, &mut out).expect("task bytes");
    w::Ask::Create(w::Create { project: 1, tasks: out.finish() })
}
/// An Amend of task 2 that changes only its LLM choice.
fn amend(revision: u64, llm: w::LlmChoice) -> w::Ask {
    let change = w::Amendment {
        addressed: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        instructions: None,
        tracked: None,
        llm: Some(llm),
    };
    w::Ask::Amend(Box::new(w::Amend {
        project: 1,
        task: 2,
        revision: Some(w::Revision { value: revision }),
        change,
        reason: b"think harder".as_slice().into(),
    }))
}
fn refused_llm(answer: &w::Final) -> (&[u8], w::LlmWhy) {
    let w::Final::Refused(refused) = answer else { panic!("refused: {answer:?}") };
    let w::Refusal::Llm(row) = &refused.refusal else { panic!("an LLM refusal: {:?}", refused.refusal) };
    (row.model.as_ref(), row.why.clone())
}
const SCRIPTED: u8 = 0;
const LOW: u8 = 0;
const HIGH: u8 = 1;

fn speed(name: &[u8]) -> skein_lib::Name<32> {
    skein_lib::Name::from_checked(name).expect("whole speed name")
}

#[test]
fn a_create_naming_a_model_and_its_effort_keeps_them_by_configuration_s_indices() {
    let (mut peer, mut world, bearer) = member_setup(360);
    let first = choice((b"scripted", Some(b"high")), &[(b"scripted", Some(b"low"))]);
    let expected = tasks::LlmChoice {
        later: None,
        account: None,
        main: tasks::ModelEffort { model: SCRIPTED, effort: Some(HIGH), speed: None },
        subagents: Box::new([tasks::ModelEffort { model: SCRIPTED, effort: Some(LOW), speed: None }]),
    };
    let socket = peer.post(b"/jig/v1/request", keyed([62; 16], &create(first.clone())), Some(&bearer));
    assert_eq!(route_real_core(&mut peer, &mut world), [[62; 16]], "core reply: {:?}", world.people_answers.last());
    let w::Final::Done(done) = final_answer(&peer, socket) else { panic!("done") };
    let w::Outcome::Document(owner) = &done.outcome else { panic!("owner outcome") };
    let w::DocumentOutcome::Created(created) = &owner.value else { panic!("created: {:?}", owner.value) };
    assert_eq!(live_row(&world, created.first).llm, Some(expected.clone()));

    // Both translations meet at the core boundary; the core owns retaining a party's choice.
    let models = super::real_core_settings().models;
    let later = with_later(first.clone(), choice((b"scripted", Some(b"low")), &[(b"scripted", Some(b"high"))]));
    let effort_only = tasks::LlmChoice {
        later: Some(tasks::LlmLater {
            main: tasks::ModelEffort { model: SCRIPTED, effort: Some(LOW), speed: None },
            subagents: Box::new([tasks::ModelEffort { model: SCRIPTED, effort: Some(HIGH), speed: None }]),
        }),
        ..expected.clone()
    };
    let mut faster = later.clone();
    faster.main.speed = Some(b"ultrafast".as_slice().into());
    faster.subagents.get_mut(0).unwrap().speed = Some(b"fast".as_slice().into());
    faster.later.as_mut().unwrap().main.speed = Some(b"standard".as_slice().into());
    let mut named_speeds = effort_only.clone();
    named_speeds.main.speed = Some(speed(b"ultrafast"));
    named_speeds.subagents[0].speed = Some(speed(b"fast"));
    named_speeds.later.as_mut().unwrap().main.speed = Some(speed(b"standard"));
    for (key, named, expected) in
        [([66; 16], first, expected), ([67; 16], faster, named_speeds), ([70; 16], later, effort_only)]
    {
        assert_eq!(
            jig_protocol_engine_clients::translate::llm::choice(Some(&expected), &models).expect("named choice"),
            Some(named.clone())
        );
        peer.post(b"/jig/v1/request", keyed(key, &create(named)), Some(&bearer));
        let Some(jig_domain_core::Event::People(jig_domain_people::Event::Ask { ask, reply_to, .. })) =
            peer.take_core()
        else {
            panic!("core ask")
        };
        let jig_domain_people::Ask::Create { batch, .. } = ask else { panic!("create") };
        peer.answer_request(
            reply_to,
            jig_domain_people::Reply::Outcome(jig_domain_people::Outcome::Created { first: 8, last: 8, given: None }),
        );
        assert_choice_indices(batch[0].llm.as_ref().expect("choice"), &expected);
    }
}
fn assert_choice_indices(actual: &jig_domain_people::LlmChoice, expected: &tasks::LlmChoice) {
    assert_eq!(actual.account, expected.account);
    assert_eq!(
        (actual.main.model, actual.main.effort, actual.main.speed),
        (expected.main.model, expected.main.effort, expected.main.speed)
    );
    assert_eq!(actual.subagents.len(), expected.subagents.len());
    for (actual, expected) in actual.subagents.iter().zip(expected.subagents.iter()) {
        assert_eq!((actual.model, actual.effort, actual.speed), (expected.model, expected.effort, expected.speed));
    }
    assert_eq!(
        actual.later.as_ref().map(|later| (later.main.model, later.main.effort, later.main.speed)),
        expected.later.as_ref().map(|later| (later.main.model, later.main.effort, later.main.speed))
    );
    assert_eq!(
        actual.later.as_ref().map(|later| later
            .subagents
            .iter()
            .map(|item| (item.model, item.effort, item.speed))
            .collect::<Vec<_>>()),
        expected.later.as_ref().map(|later| later
            .subagents
            .iter()
            .map(|item| (item.model, item.effort, item.speed))
            .collect::<Vec<_>>())
    );
}

#[test]
fn a_model_or_an_effort_configuration_lacks_is_refused_by_its_name_before_the_core() {
    let (mut peer, mut world, bearer) = member_setup(361);
    for (llm, model, why) in [
        (choice((b"unheard", None), &[]), b"unheard".as_slice(), w::LlmWhy::Unconfigured),
        (choice((b"scripted", Some(b"medium")), &[]), b"scripted".as_slice(), w::LlmWhy::Effort),
        (
            with_later(choice((b"scripted", Some(b"low")), &[]), choice((b"unheard", None), &[])),
            b"unheard".as_slice(),
            w::LlmWhy::Unconfigured,
        ),
        (
            with_later(choice((b"scripted", Some(b"low")), &[]), choice((b"scripted", Some(b"medium")), &[])),
            b"scripted".as_slice(),
            w::LlmWhy::Effort,
        ),
        (
            with_later(
                choice((b"scripted", Some(b"low")), &[]),
                choice((b"scripted", Some(b"low")), &[(b"unheard", None)]),
            ),
            b"unheard".as_slice(),
            w::LlmWhy::Unconfigured,
        ),
        (
            with_later(
                choice((b"scripted", Some(b"low")), &[]),
                choice((b"scripted", Some(b"low")), &[(b"scripted", Some(b"medium"))]),
            ),
            b"scripted".as_slice(),
            w::LlmWhy::Effort,
        ),
    ] {
        let socket = peer.post(b"/jig/v1/request", keyed([63; 16], &create(llm)), Some(&bearer));
        assert_eq!(route_real_core(&mut peer, &mut world), Vec::<[u8; 16]>::new(), "no ask reaches the core");
        assert_eq!(refused_llm(&final_answer(&peer, socket)), (model, why));
    }
}
#[test]
fn a_model_the_core_refuses_comes_back_by_its_name() {
    let (mut peer, mut world, bearer) = member_setup(362);
    let llm = choice((b"scripted", Some(b"low")), &[(b"scripted", Some(b"low")), (b"scripted", Some(b"high"))]);
    let socket = peer.post(b"/jig/v1/request", keyed([64; 16], &create(llm)), Some(&bearer));
    assert_eq!(route_real_core(&mut peer, &mut world), [[64; 16]]);
    assert_eq!(refused_llm(&final_answer(&peer, socket)), (b"scripted".as_slice(), w::LlmWhy::Repeated));
}
#[test]
fn an_undeclared_speed_reaches_the_tasks_child_and_its_refusal_names_the_model() {
    let (mut peer, mut world, bearer) = member_setup(362);
    let mut llm = choice((b"scripted", Some(b"low")), &[]);
    llm.main.speed = Some(b"ultrafast".as_slice().into());
    let socket = peer.post(b"/jig/v1/request", keyed([68; 16], &create(llm)), Some(&bearer));
    assert_eq!(route_real_core(&mut peer, &mut world), [[68; 16]]);
    let answer = w::KeyedAnswer::decode(
        &fixtures::limits().wire,
        &mut skein_lib::Reader::new(peer.answered(socket).expect("http answer")),
    )
    .unwrap();
    let limits = jig_client_kit::test::limits().client;
    let link = jig_client_protocol_link::Limits::new(limits.domain, limits.exchanges).unwrap();
    let c::Answer::Refused { refusal: c::Refusal::Llm { model, why }, at: None } =
        jig_client_protocol_link::translate::answers::answer(answer, &link).unwrap()
    else {
        panic!("a refusal naming its model")
    };
    assert_eq!((model.bytes(), why), (b"scripted".as_slice(), c::LlmWhy::Speed));
}
#[test]
fn an_amend_naming_a_choice_changes_the_one_the_task_s_next_run_takes() {
    let (mut peer, mut world, bearer) = amendment_setup(363);
    let before = live_row(&world, 2);
    assert_eq!(before.llm, None, "an omitted choice is resolved at the run check");
    for (key, named, kept) in
        [([65; 16], Some(b"standard".as_slice()), Some(speed(b"standard"))), ([69; 16], None, None)]
    {
        let revision = live_row(&world, 2).revisions.spec;
        let mut llm = choice((b"scripted", Some(b"high")), &[]);
        llm.main.speed = named.map(Box::from);
        let socket = peer.post(b"/jig/v1/request", keyed(key, &amend(revision, llm.clone())), Some(&bearer));
        assert_eq!(route_real_core(&mut peer, &mut world), [key], "core reply: {:?}", world.people_answers.last());
        let w::Final::Done(_) = final_answer(&peer, socket) else { panic!("amended") };
        let row = live_row(&world, 2);
        assert_eq!(
            row.llm,
            Some(tasks::LlmChoice {
                later: None,
                account: None,
                main: tasks::ModelEffort { model: SCRIPTED, effort: Some(HIGH), speed: kept },
                subagents: Box::new([]),
            })
        );
        assert_eq!(
            jig_protocol_engine_clients::translate::llm::choice(row.llm.as_ref(), &super::real_core_settings().models)
                .unwrap(),
            Some(llm),
        );
    }
}
