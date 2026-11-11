//! A standing role keeps a person after theirs leaves (domain/agent/standing.md, 3).
use super::*;
use jig_domain_core::{self as core, documents, reads};

const CHANNEL: Token = Token::new(7);
const ROLE_SPEND: u64 = 100;
const BLOCKER_SPEND: u64 = 450; // Leaves 50 of the owner's 500, less than the role needs.

struct Cast {
    driver: Driver,
    people: [u64; 4],
    sessions: [u64; 4],
}

fn cast() -> Cast {
    let [(mut first, mut limits), (mut second, _)] = [people_roles_config(124), people_roles_config(124)];
    for config in [&mut first, &mut second] {
        config.core.defaults.rules.period_spend = 2000;
        config.core.seed_projects[0].policy.policy.period_spend = 2000;
    }
    limits.tasks.tasks = 4;
    limits.tasks.project_tasks = 4;
    limits.tasks.project_goals = 4;
    limits.tasks.funders = 12; // Two periods and their four possible person pools.
    limits.fleet.slots = 4;
    let mut driver = Driver::configured(Store::new(), [first, second], &limits);
    driver.send(protocol::hello_slots(CHANNEL, 4, Box::new([])));
    driver.settle();
    let signed = [7, 8, 9, 10].map(|user| sign_in_person(&mut driver, user, 1200 + user));
    let people = signed.map(|(person, _)| person);
    let sessions = signed.map(|(_, session)| session);
    roles(
        &mut driver,
        sessions[0],
        20,
        &[
            (people[1], people::Role::Owner),
            (people[0], people::Role::Owner),
            (people[2], people::Role::Member),
            (people[3], people::Role::Member),
        ],
    );
    driver.settle();
    Cast { driver, people, sessions }
}

fn ask(driver: &mut Driver, session: u64, key: u8, ask: people::Ask) {
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: session,
        key: [key; 16],
        ask,
    }));
}

fn roles(driver: &mut Driver, session: u64, key: u8, roster: &[(u64, people::Role)]) {
    ask(
        driver,
        session,
        key,
        people::Ask::SetRoles {
            project: 1,
            holdings: roster.iter().map(|(person, role)| people::Holding { person: *person, role: *role }).collect(),
        },
    );
}

fn start(driver: &mut Driver, session: u64, key: u8, ended_by: people::EndedBy, spend: u64) -> engine::Assignment {
    let people::Ask::Create { project, mut batch } = driver.chat_ask(1, b"Keep the project in order") else {
        panic!("task fixture")
    };
    batch[0].flags.ended_by = ended_by;
    batch[0].flags.conversation = ended_by == people::EndedBy::Requester;
    batch[0].budget = people::Budget::Named;
    batch[0].authority.spend = spend;
    ask(driver, session, key, people::Ask::Create { project, batch });
    driver.settle();
    driver.last_assignment()
}

fn question(driver: &mut Driver, run: &engine::Assignment, person: u64) -> u64 {
    driver.call(
        CHANNEL,
        run.task,
        run.attempt,
        90,
        tool_call(engine::Tool::Message {
            target: engine::MessageTarget::Requester,
            form: tasks::MessageForm::Question,
            words: b"Should I keep this rule?".as_slice().into(),
        }),
    );
    driver.settle();
    let engine::CallAnswer::Message { number } = driver.answered(90).expect("question answered") else {
        panic!("question accepted: {:?}", driver.answered(90))
    };
    let number = *number;
    assert_eq!(live(driver, run.task).questions[0].asked, tasks::Party::Person(person));
    number
}

fn inbox(driver: &mut Driver, session: u64) -> Vec<reads::PageRow> {
    driver.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to: ReplyTo::new(Token::new(1500)),
        sign_in: session,
        read: reads::Read::Inbox {
            group: reads::Group::Questions,
            filter: reads::Filter { project: Some(1), goal: None },
            from: None,
        },
    }));
    driver.settle();
    driver.delivered.iter().rev().find_map(|delivery| {
        if let Delivery::Read { answer: reads::Answer::Page(page), .. } = delivery {
            Some(page.rows.to_vec())
        } else { None }
    }).expect("person's questions inbox")
}

fn has_question(rows: &[reads::PageRow], task: u64, question: u64) -> bool {
    rows.iter().any(|row| matches!(row, reads::PageRow::Inbox(entry)
        if matches!(&entry.task, documents::TaskLink::Live(chip) if chip.number == task)
        && entry.object == documents::InboxObject::Question { message: question }))
}

fn funding(driver: &Driver, person: u64, period: u64) -> tasks::Numbers {
    let key = Key::Core(core::Key::Tasks(tasks::Key::Ledger(tasks::Funder::Pool { project: 1, person, period })));
    let Some(Record::Core(core::Record::Tasks(tasks::Stored::Ledger(row)))) = driver.store.rows.get(&key) else {
        panic!("person's pool")
    };
    row.numbers
}

#[test]
fn a_roles_save_moves_only_the_leavers_standing_role_after_commit() {
    let Cast { mut driver, people, sessions } = cast();
    let role = start(&mut driver, sessions[2], 21, people::EndedBy::Standing, ROLE_SPEND);
    let question = question(&mut driver, &role, people[2]);
    assert!(has_question(&inbox(&mut driver, sessions[2]), role.task, question));
    let chat = start(&mut driver, sessions[2], 22, people::EndedBy::Requester, ROLE_SPEND);
    let stays = start(&mut driver, sessions[3], 23, people::EndedBy::Standing, ROLE_SPEND);
    let original_chat = live(&driver, chat.task).clone();
    let original_stays = live(&driver, stays.task).clone();
    let before = driver.transactions.len();
    roles(
        &mut driver,
        sessions[1],
        24,
        &[(people[1], people::Role::Owner), (people[3], people::Role::Member), (people[0], people::Role::Owner)],
    );
    for _ in 0..32 {
        driver.advance(false);
        if !driver.store.pending.is_empty() { break; }
    }
    assert!(!driver.store.pending.is_empty(), "roles save has a commit");
    for _ in 0..8 { driver.advance(false); }
    assert_eq!(live(&driver, role.task).requester, tasks::Party::Person(people[2]), "no move before commit");
    assert!(!driver.transactions[before..].iter().flatten().any(|write| matches!(write,
        Write::Save(Record::Core(core::Record::Tasks(tasks::Stored::Live(row))))
        if row.number == role.task && row.requester == tasks::Party::Person(people[1]))));
    driver.settle();
    let saves = &driver.transactions[before..];
    let roster = saves.iter().position(|writes| writes.iter().any(|write| matches!(write,
        Write::Save(Record::Core(core::Record::People(people::Stored::Roles { .. })))))).expect("roles decision");
    let moved = saves.iter().position(|writes| writes.iter().any(|write| matches!(write,
        Write::Save(Record::Core(core::Record::Tasks(tasks::Stored::Live(row))))
        if row.number == role.task && row.requester == tasks::Party::Person(people[1])))).expect("move decision");
    assert_eq!(moved, roster + 1, "the move is the decision after the roles save");
    assert!(people[0] < people[1], "the saver is not the lowest owner");
    let row = live(&driver, role.task);
    assert_eq!(row.requester, tasks::Party::Person(people[1]));
    assert_eq!(row.funder, tasks::Funder::Pool { project: 1, person: people[1], period: 1 });
    assert_eq!(funding(&driver, people[1], 1).reserved, ROLE_SPEND);
    assert_eq!(funding(&driver, people[2], 1).reserved, ROLE_SPEND, "only the chat remains funded by M");
    assert!(has_question(&inbox(&mut driver, sessions[1]), role.task, question));
    assert!(!has_question(&inbox(&mut driver, sessions[0]), role.task, question));
    assert_eq!(live(&driver, chat.task), &original_chat, "M's chat is not moved");
    assert_eq!(live(&driver, stays.task), &original_stays, "a remaining member's role is not moved");
}

#[test]
fn a_refused_move_waits_unchanged_and_retries_at_the_next_period() {
    let Cast { mut driver, people, sessions } = cast();
    let role = start(&mut driver, sessions[2], 21, people::EndedBy::Standing, ROLE_SPEND);
    let question = question(&mut driver, &role, people[2]);
    start(&mut driver, sessions[0], 22, people::EndedBy::Result, BLOCKER_SPEND);
    let original = live(&driver, role.task).clone();
    let old_pool = funding(&driver, people[2], 1);
    let owner_pool = funding(&driver, people[0], 1);
    roles(&mut driver, sessions[0], 23, &[(people[0], people::Role::Owner)]);
    driver.settle();
    assert_eq!(live(&driver, role.task), &original, "a funding refusal changes nothing");
    assert_eq!(funding(&driver, people[2], 1), old_pool);
    assert_eq!(funding(&driver, people[0], 1), owner_pool);
    assert!(has_question(&inbox(&mut driver, sessions[0]), role.task, question) == false);
    driver.fire_at(temper_engine_domain_world::walking::PERIOD_SPAN);
    driver.settle();
    let row = live(&driver, role.task);
    assert_eq!(row.requester, tasks::Party::Person(people[0]));
    assert_eq!(row.funder, tasks::Funder::Pool { project: 1, person: people[0], period: 2 });
    assert_eq!(funding(&driver, people[0], 2).reserved, ROLE_SPEND);
    assert_eq!(funding(&driver, people[2], 1).reserved, 0);
    // The original sign-in expires before the calendar's next period.
    let (_, session) = sign_in_person(&mut driver, 7, 1501);
    assert!(has_question(&inbox(&mut driver, session), role.task, question));
}
