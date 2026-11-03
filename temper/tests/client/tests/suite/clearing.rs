//! The lowest joined world: retained forge verdicts, task documents, translations and ctl.
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain as engine;
use temper_engine_domain_world::{forge_routes::World, protocol, tier_runs as tiers};

#[path = "../../../engine/domain/tests/suite/forge_routes/tier_task_show.rs"]
mod task_show;

fn cleared_change() -> (World, u64) {
    let mut world = task_show::world();
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let people::Ask::Create { project, mut batch } = world.chat(1, b"Small fix") else { panic!("chat") };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(400) };
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in: world.signed_in.expect("owner"),
        key: [92; 16],
        ask: people::Ask::Create { project, batch },
    }));
    tiers::until(&mut world, tiers::LANDING_TICKS, "chat assigned", |world| !world.assigned.is_empty());
    let chat = world.assigned[0].clone();
    tiers::until(&mut world, tiers::RUN_TICKS, "baseline judged", |world| tiers::tier(world).record.sequence == 1);
    let answer = world.scripted_call(
        &chat,
        93,
        engine::Tool::Change { changes: Box::new([engine::ChangeCall::words(b"Small fix".as_slice().into(), None)]) },
    );
    let number = tiers::produce_delegated(&mut world, answer, b"main", b"file", b"small fix");
    tiers::until(&mut world, tiers::RUN_TICKS * 3, "change cleared", |world| {
        matches!(tiers::task(world, number).phase, tasks::Phase::Ended(_))
    });
    (world, number)
}

fn compare_result(world: &mut World, number: u64) {
    let row = tiers::task(world, number);
    let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { words, .. })) = &row.phase else {
        panic!("landed result")
    };
    let kept = words.clone();
    let tasks::Party::Task(requester) = row.requester else { panic!("requester") };
    let delivered = tiers::task(world, requester).inbox.iter().find(|word| word.from == tasks::Party::Task(number));
    assert_eq!(delivered.expect("requester's whole result").words, kept);
    assert!(kept.len() > 128, "the document window cuts stored words");
    let answer = world.party_read(jig_domain_core::reads::Read::Task { task: number });
    let jig_domain_core::reads::Answer::Ended(ended) = answer else { panic!("ended task read") };
    let jig_domain_core::reads::EndedValue::Task { document, .. } = ended.value else { panic!("task document") };
    let Some(jig_domain_core::documents::Ending::Done(jig_domain_core::documents::ResultValue::Change {
        words, ..
    })) = document.result
    else {
        panic!("change document")
    };
    assert!(words.cut);
    assert_eq!(words.bytes.as_ref(), &kept[..128], "loss occurs at the document window");
    for printed in [task_show::show(world, number), task_show::show_json(world, number)] {
        assert_eq!(printed.status, 0, "{printed:?}");
        let text = std::str::from_utf8(&printed.stdout).expect("ctl UTF-8");
        let merge = String::from_utf8(temper_engine_domain_forge::commit_text(&tiers::merge(world, number)).into_vec())
            .expect("merge ID");
        assert!(text.contains(&merge), "{text}");
        assert!(text.contains("Cleared by slow run 2."), "stored: {kept:?}\nstdout: {text}");
    }
}

#[test]
fn task_show_reads_a_cleared_change_before_and_after_restart() {
    let (mut world, number) = cleared_change();
    assert_eq!(number, 3);
    tiers::until(&mut world, tiers::LANDING_TICKS, "requester result", |world| {
        tiers::task(world, 1).inbox.iter().any(|word| word.from == tasks::Party::Task(number))
    });
    task_show::assert_words(&mut world, number, "Small fix");
    compare_result(&mut world, number);
    tiers::until(&mut world, tiers::LANDING_TICKS, "restart cut", tiers::restart_cut);
    world.restart_same();
    tiers::until(&mut world, tiers::LANDING_TICKS, "restarted", |world| world.root.ready());
    world.signed_in = None;
    world.sign_in();
    compare_result(&mut world, number);
}
