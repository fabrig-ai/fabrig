use jig_domain_core::{CoreRecord, Record as CoreRow};
use jig_domain_tasks as tasks;
use jig_fake_parties::{Ask, Reply};
use std::collections::BTreeSet;
use temper_engine_domain::{Key, Record};
use temper_system_world::composed::{World, text_script};

const SEED: u64 = 7;
const WORDS: &[u8] = b"Hello";
const REPLY: &[u8] = b"World answer";
const STEPS: u32 = 10_000;

#[test]
fn two_one_slot_workers_each_run_a_different_chat_to_its_end() {
    let mut world = World::two_workers(SEED, text_script(REPLY), WORDS);
    world.act(Ask::Chat { words: WORDS.into() }, [42; 16]);
    world.release_both_models();
    let [(_, Reply::SignedIn { .. }), (_, Reply::Started { task: first }), (_, Reply::Started { task: second })] =
        world.party.replies.as_slice()
    else {
        panic!("one sign-in and both creation replies: {:?}", world.party.replies);
    };
    let chats = [*first, *second];
    assert_ne!(chats[0], chats[1]);
    for _ in 0..STEPS {
        world.tick();
        if chats.iter().all(|task| committed_reply(&world, *task)) {
            break;
        }
    }
    for (task, key) in chats.into_iter().zip([[43; 16], [44; 16]]) {
        assert!(committed_reply(&world, task), "chat {task} committed its reply");
        world.close_numbered_chat(task, key);
        assert!(matches!(world.party.replies.last(), Some((_, Reply::Changed { task: closed })) if *closed == task));
        assert!(world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)))));
    }
    let mut ran = Vec::new();
    for (position, starts) in world.worker_starts().enumerate() {
        assert_eq!(starts.len(), 1, "worker at position {position} ran one chat");
        ran.extend(starts.iter().map(|(task, _)| *task));
    }
    assert_eq!(ran.len(), chats.len());
    assert_eq!(ran.into_iter().collect::<BTreeSet<_>>(), BTreeSet::from(chats));
}

fn committed_reply(world: &World, task: u64) -> bool {
    world.store.rows.values().any(|row| {
        matches!(
            row,
            Record::Core(CoreRow::Core(CoreRecord::Turn(turn)))
                if turn.task == task && turn.transcript.windows(REPLY.len()).any(|text| text == REPLY)
        )
    })
}
