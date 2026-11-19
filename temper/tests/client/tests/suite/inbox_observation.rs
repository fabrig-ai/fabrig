use jig_client_kit::{harness::App, observe};
use skein_lib::Token;
use temper_client_world::app::Temper;
const TASK: u64 = 9;
const PROJECT: u32 = 1;
const REVISION: jig_client::contract::Revision = jig_client::contract::Revision { value: 7 };
const ATTEMPT: Token = Token::new(1);
const REQUEST: [u8; 16] = [3; 16];

#[test]
fn an_owner_withdraws_their_amendment_of_another_partys_task_from_its_card() {
    use jig_client_kit::harness::Until;
    use jig_domain_people as p;
    use skein_fake_person::{Find, Step};
    use skein_ui::Role;
    use temper_client_world::{fixtures, front::account_acts};
    use temper_engine_domain as engine;

    let (front, task) = amendment_front();
    let mut world = fixtures::world(fixtures::SEED);
    *world.front() = front;
    let person = world.client_with(fixtures::PARTY, fixtures::inbox(), fixtures::domain_limits(), proposal_browser);
    assert_eq!(
        world.run(Until::Time(world.now().saturating_add(fixtures::WITHIN))).verdict,
        skein_world::domain::Verdict::Passed
    );
    let front = world.front();
    let revision = front.engine.task(task).unwrap().revisions.spec;
    let p::Ask::Create { batch, .. } = front.engine.chat_ask(PROJECT, b"work") else { panic!("create fixture") };
    let mut authority = batch[0].authority.clone();
    authority.spend = 600;
    let amendment = p::Amendment {
        tracked: None,
        addressed: None,
        instructions: None,
        spec: Some(p::Spec {
            words: b"More room for this task".as_slice().into(),
            parameters: Box::new([]),
            inputs: Box::new([]),
            asks: false,
        }),
        wake: None,
        dependencies: None,
        authority: Some(authority),
        llm: None,
        reason: b"Widen the task".as_slice().into(),
    };
    let refused = account_acts::party_ask(
        front,
        82,
        p::Ask::Amend { project: PROJECT, task, revision, amendment: amendment.clone(), release: None },
    );
    assert!(
        matches!(refused, p::Outcome::Refused(p::Refusal::Authority { proposable: Some(p::Accepter::Role(0)), .. })),
        "{refused:?}"
    );
    let proposed = account_acts::party_ask(
        front,
        83,
        p::Ask::Propose {
            project: PROJECT,
            action: p::Action::Amend { task, revision, amendment: Box::new(amendment) },
            reason: b"Widen the task".as_slice().into(),
        },
    );
    let p::Outcome::Proposed { proposal, to: 0 } = proposed else { panic!("amendment proposed: {proposed:?}") };
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Button, b"a proposal"), within: fixtures::WITHIN },
            Step::Press { find: Find::named(Role::Button, b"a proposal") },
            Step::See { find: Find::text(format!("Amend task {task}").as_bytes()), within: fixtures::WITHIN },
            Step::See { find: Find::text(b"More room for this task"), within: fixtures::WITHIN },
            Step::See { find: Find::text(b"Authority before"), within: fixtures::WITHIN },
            Step::See { find: Find::text(b"Authority after"), within: fixtures::WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Withdraw") },
            Step::See { find: Find::named(Role::Dialog, b"Withdraw this proposal?"), within: fixtures::WITHIN },
            Step::Press {
                find: Find::named(Role::Button, b"Withdraw").within(Box::new([skein_fake_person::Target::ExactRole {
                    role: Role::Dialog,
                    name: b"Withdraw this proposal?".as_slice().into(),
                }])),
            },
            Step::Gone { find: Find::text(format!("Amend task {task}").as_bytes()), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, skein_world::domain::Verdict::Passed);
    let key = jig_client::contract::ObjectKey::Proposal(jig_client::contract::ProposalKey {
        proposer: jig_client::contract::Proposer::Party(fixtures::PARTY),
        number: proposal,
    });
    let inbox = world.root(person).person().pages().inbox().unwrap();
    assert!(!inbox.rows.iter().any(|row| row.card_key() == key));
    assert!(!inbox.cards.iter().any(|card| card.object == key));
    let front = world.front();
    assert!(front.engine.store.rows.values().any(|row| matches!(row,
        engine::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::DecidedProposal(value))) if value.number == proposal && matches!(value.state, jig_domain_tasks::PersonProposalState::Withdrawn { .. })
    )), "withdrawn in the engine");
}

fn amendment_front() -> (temper_client_world::front::Joint, u64) {
    use jig_domain_people as p;
    use skein_lib::ReplyTo;
    use temper_client_world::{fixtures, front::Joint};
    use temper_engine_domain as engine;
    use temper_engine_domain_world::{store::Store, walking};
    let mut bounds = temper_client_world::front::engine_limits();
    bounds.people.initial_owners = 2;
    // Two creations, the move, the refused amendment, its proposal and withdrawal.
    bounds.people.requests = 6;
    // Two task-fixture sign-ins, the other owner and the browser.
    bounds.people.sign_ins = 4;
    let configs = std::array::from_fn(|_| {
        let mut config = walking::config(fixtures::SEED);
        let owner = config.core.owners[0].clone();
        config.core.owners = Box::new([
            owner.clone(),
            p::InitialOwner { identity: temper_engine_domain_forge::user_key(1, 8_u64), ..owner },
        ]);
        config
    });
    let mut front = Joint::with_engine(
        fixtures::SEED,
        &fixtures::domain_limits(),
        Store::with_limits(&bounds),
        configs,
        &bounds,
        2,
    );
    let (task, _) = fixtures::amend_tasks(&mut front);
    front.engine.send(engine::Event::Party(engine::PartyInput::SignedIn {
        until: None,
        held: None,
        reply_to: ReplyTo::new(Token::new(800)),
        identity: p::Identity {
            key: temper_engine_domain_forge::user_key(1, 8_u64),
            login: b"other".as_slice().into(),
            name: b"Other".as_slice().into(),
        },
    }));
    front.engine.settle();
    let (other, sign_in) = front
        .engine
        .delivered
        .iter()
        .find_map(|delivery| {
            if let engine::Delivery::WebReply { to, reply: p::Reply::SignedIn { sign_in, person, .. } } = delivery
                && *to == ReplyTo::new(Token::new(800))
            {
                Some((*person, *sign_in))
            } else {
                None
            }
        })
        .expect("other owner signed in");
    front.engine.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(801)),
        sign_in,
        key: [81; 16],
        ask: p::Ask::Move { project: PROJECT, task, reason: b"Take this task".as_slice().into() },
    }));
    front.engine.settle();
    assert_eq!(front.engine.task(task).expect("moved task").requester, jig_domain_tasks::Party::Person(other));
    (front, task)
}

fn proposal_browser(seed: u64) -> (temper_client_world::app::Root, temper_client_world::app::View) {
    use temper_client_world::{
        app::{Root, View},
        fixtures,
    };
    let limits = fixtures::domain_limits();
    let mut words = fixtures::phrasebooks(&limits);
    words.shell = temper_client_view::words::Shell::Browser;
    let bounds = fixtures::view_bounds(&limits, &words);
    (
        Root::Person(Box::new(temper_client_domain::unlinked(&limits, seed))),
        View::Person(Box::new(temper_client_view::View::new(&bounds, &limits, words, fixtures::FORGE))),
    )
}
#[test]
fn a_release_is_observed_against_the_inbox_escalation_the_person_saw() {
    let request = temper_client_domain::Request::Unlinked {
        request: jig_client::UnlinkedRequest::Down {
            down: jig_client::Down::Send {
                attempt: ATTEMPT,
                key: REQUEST,
                request: jig_client::Ask::Jig(Box::new(jig_client::contract::Ask::Release {
                    project: PROJECT,
                    task: TASK,
                    revision: REVISION,
                    again: None,
                })),
                value: None,
            },
        },
    };
    let sent = Temper::sent(&request).expect("keyed request");
    assert_eq!(sent.card, Some(observe::key(jig_client::contract::ObjectKey::Escalation(TASK))));
    assert_eq!(sent.revision, Some((2, REVISION.value)));
}

#[test]
fn confirming_preserves_the_inbox_cards_visible_revision() {
    let event = temper_client_world::app::Event::Root(temper_client_domain::Event::Act {
        action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Confirm),
    });
    assert!(!Temper::window_changed(&event));
}

#[test]
fn a_temper_hold_card_reports_the_revision_it_prints() {
    use jig_client_kit::harness::{Seen, Until};
    use skein_fake_person::{Find, Step};
    use skein_ui::Role;
    use temper_client_world::fixtures;
    let mut world = fixtures::world(fixtures::SEED);
    let (_, held) = fixtures::goal_with_held_delegate(world.front());
    let person = world.client(fixtures::PARTY, fixtures::task_page(held));
    world
        .person(person, vec![Step::See { find: Find::named(Role::Button, b"Release task"), within: fixtures::WITHIN }]);
    assert_eq!(world.run(Until::Done).verdict, skein_world::domain::Verdict::Passed);
    let visible = Temper::seen(world.tree(person));
    assert!(visible.iter().any(|value| matches!(value,
        Seen::Card { key, revision: Some(_), .. } if *key == observe::key(jig_client::contract::ObjectKey::Escalation(held))
    )));
}
