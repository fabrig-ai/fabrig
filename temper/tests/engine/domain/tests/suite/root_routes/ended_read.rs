use super::*;
const MARK: Token = Token::new(1201);
const READ: Token = Token::new(1202);

fn ended_conversation() -> Store {
    let (mut driver, assignment) = Driver::chat();
    for number in 1..=2 {
        turn(&mut driver, &assignment, number, u64::from(number));
        driver.settle();
        if number == 1 {
            driver.send(engine::Event::Party(engine::PartyInput::Ask {
                models: Box::new([]),
                reply_to: ReplyTo::new(MARK),
                sign_in: driver.session(),
                key: [32; 16],
                ask: people::Ask::MarkSeen { seen: people::Seen::Conversation { task: assignment.task, through: 1 } },
            }));
            driver.settle();
        }
    }
    driver.answer(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        2,
        tasks::End::Finished { result: tasks::TaskResult::Report { words: REPORT.into() }, cancel_delegates: false },
        engine::Parts::none(),
    );
    driver.settle();
    driver.store
}

#[test]
fn an_ended_conversations_read_position_survives_a_restart() {
    let mut restarted = Driver::new(ended_conversation());
    restarted.settle();
    restarted.sign_in();
    restarted.settle();
    let line = conversation(&mut restarted);
    let position = line.conversation.expect("conversation read position");
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) =
        restarted.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(line.chip.number))))
    else {
        panic!("ended conversation row")
    };
    assert_eq!((position.seen, row.last_reply), (1, 2));
    assert_eq!(Some(position.unread), Some(jig_domain_core::documents::Count::Known(1)));
}

fn conversation(driver: &mut Driver) -> jig_domain_core::documents::TaskLine {
    use jig_domain_core::reads;
    let sign_in = driver.session();
    let mut from = None;
    loop {
        driver.delivered.clear();
        driver.send(engine::Event::Party(engine::PartyInput::Read {
            reply_to: ReplyTo::new(READ),
            sign_in,
            read: reads::Read::Tasks { query: jig_core_world::fixtures::OWN_TASKS, live: None, ended: from },
        }));
        driver.settle();
        let page = driver
            .delivered
            .iter()
            .find_map(|delivery| {
                if let Delivery::Read { answer: reads::Answer::Tasks(page), .. } = delivery { Some(page) } else { None }
            })
            .expect("conversation page");
        let reads::EndedGroup::Rows { rows, next, .. } = &page.ended else { panic!("ended query group") };
        if let Some(line) = rows.iter().next() {
            return line.clone();
        }
        from = Some(next.clone().expect("ended conversation continuation"));
    }
}
