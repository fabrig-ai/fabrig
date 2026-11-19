//! Branch names resolve before admission; a file's render is pinned to the answered head.
use super::*;
use skein_forge::api::{Answer, Error, Read};

fn file(branch: &[u8], max_bytes: u32) -> engine::Tool {
    engine::Tool::ReadBranchFile { repository: None, branch: branch.into(), path: b"file".as_slice().into(), max_bytes }
}

fn send(world: &mut World, chat: &engine::Assignment, completion: u32, tool: engine::Tool) {
    let event = world.protocol.call(
        Token::new(7),
        chat.task,
        chat.attempt,
        u64::from(completion),
        engine::Call { completion, position: 1, tool },
    );
    world.send(event);
}

fn no_record(world: &World, chat: &engine::Assignment, completion: u32) {
    let key = engine::CallKey { task: chat.task, attempt: chat.attempt, completion, position: 1 };
    assert!(
        !world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Call(key))))
    );
    let named = forge_top::NamedCall { task: key.task, attempt: key.attempt, completion, position: 1 };
    assert!(!world.store.rows.contains_key(&Key::Forge(forge_top::Key::Call(named))));
    assert!(world.store.pending.is_empty(), "reads need no durable decision");
}

#[test]
fn branch_reads_resolve_defaults_and_render_files_at_the_answered_head() {
    let (mut world, chat) = change_tool::chat_world_with_words(true, b"Read files at a branch");
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"contents", 1)
        .expect("file head");
    let head = translate::commit(head);
    let branch = Read::Branch { branch: b"main".as_slice().into() };
    let protection = Read::Protection { branch: b"main".as_slice().into() };
    let limit = world.env.limits.forge.client.answer_bytes;
    let full = Read::File { head, path: b"file".as_slice().into(), max_bytes: limit };
    let cut = Read::File { head, path: b"file".as_slice().into(), max_bytes: 2 };
    let full_answer =
        Answer::File { head, path: b"file".as_slice().into(), bytes: b"contents".as_slice().into(), truncated: false };
    let cut_answer =
        Answer::File { head, path: b"file".as_slice().into(), bytes: b"co".as_slice().into(), truncated: true };
    let missing = Read::Branch { branch: b"missing".as_slice().into() };
    let cases = [
        (
            engine::Tool::ReadNamed { repository: None, read: Read::Branch { branch: Box::new([]) } },
            vec![branch.clone()],
            branch.clone(),
            Ok(Answer::Commit(head)),
        ),
        (
            engine::Tool::ReadNamed { repository: None, read: Read::Protection { branch: Box::new([]) } },
            vec![protection.clone()],
            protection,
            Ok(Answer::Protection(None)),
        ),
        (file(b"main", 0), vec![branch.clone(), full.clone()], full.clone(), Ok(full_answer.clone())),
        (file(b"", 0), vec![branch.clone(), full.clone()], full, Ok(full_answer)),
        (file(b"main", 2), vec![branch, cut.clone()], cut, Ok(cut_answer)),
        (file(b"missing", 0), vec![missing.clone()], missing, Err(Error::Missing)),
    ];
    for (index, (tool, reads, rendered, answer)) in cases.into_iter().enumerate() {
        let completion = u32::try_from(index + 93).expect("few calls");
        world.answers.clear();
        world.forge_reads = Some(Vec::new());
        send(&mut world, &chat, completion, tool);
        world.until(Until::Read);
        assert_eq!(world.answers, vec![engine::CallAnswer::ForgeRead(Box::new(answer))]);
        let seen: Vec<_> =
            world.forge_reads.as_ref().expect("record reads").iter().map(|(_, read)| read.clone()).collect();
        assert_eq!(seen, reads, "only the requested reads reach the forge");
        let name: Box<[u8]> = u64::from(completion).to_be_bytes().into();
        assert_eq!(world.protocol.read_forms.get(&name), Some(&rendered));
        assert!(!world.protocol.read_jobs.contains_key(&name));
        assert!(!world.protocol.read_pages.contains_key(&name));
        no_record(&world, &chat, completion);
    }
    for (completion, tool) in [
        (
            100,
            engine::Tool::ReadBranchFile {
                repository: Some(b"org/unknown".as_slice().into()),
                branch: Box::new([]),
                path: b"file".as_slice().into(),
                max_bytes: 0,
            },
        ),
        (
            101,
            engine::Tool::ReadNamed {
                repository: Some(b"org/unknown".as_slice().into()),
                read: Read::Branch { branch: Box::new([]) },
            },
        ),
    ] {
        world.answers.clear();
        world.forge_reads = Some(Vec::new());
        send(&mut world, &chat, completion, tool);
        for _ in 0..200 {
            world.tick();
            if !world.answers.is_empty() {
                break;
            }
        }
        assert!(
            matches!(world.answers.as_slice(), [engine::CallAnswer::ReadRepositoryRefused { named: Some(named), repositories }]
            if named.as_ref() == b"org/unknown" && repositories.as_ref() == [Box::from(b"org/repo".as_slice())])
        );
        assert_eq!(world.forge_reads.as_ref().expect("record reads").as_slice(), []);
        no_record(&world, &chat, completion);
    }
    busy_retry(&mut world, &chat, head);
    world.close();
}

fn busy_retry(world: &mut World, chat: &engine::Assignment, head: skein_forge::api::Commit) {
    // Occupy the client's fresh slots: Busy retries the original tool, not a branch answer.
    world.answers.clear();
    world.busy_reads = true;
    for owner in 0..world.env.limits.forge.client.pending {
        world.send(engine::Event::Forge(Box::new(forge_top::Event::Client(skein_forge_client::Event::Read {
            owner: Token::new(8000 + u64::from(owner)),
            repository: forge_world::REPO,
            read: Read::Settings,
        }))));
    }
    for _ in 0..200 {
        world.tick();
        if world
            .held_reads
            .as_ref()
            .is_some_and(|reads| reads.len() == skein_lib::usize_of(world.env.limits.forge.client.calls))
        {
            break;
        }
    }
    assert_eq!(
        world.held_reads.as_ref().expect("occupied slots").len(),
        skein_lib::usize_of(world.env.limits.forge.client.calls)
    );
    let renders = world.protocol.answers.len();
    send(world, chat, 102, file(b"", 0));
    for _ in 0..200 {
        world.tick();
        if world.busy_calls > 0 {
            break;
        }
    }
    assert_eq!(world.busy_calls, 1);
    assert_eq!(world.answers, [] as [engine::CallAnswer; 0]);
    assert_eq!(world.protocol.answers.len(), renders, "Busy renders no partial result");
    world.busy_reads = false;
    for request in world.held_reads.take().expect("held fresh reads") {
        let engine::Request::Forge { call, .. } = request else { panic!("held forge call") };
        world.send(engine::Event::Forge(Box::new(forge_top::Event::Client(skein_forge_client::Event::Answered {
            call,
            cost: 1,
            free: 0,
            rate: None,
            result: Err(Error::Missing),
        }))));
    }
    send(world, chat, 102, file(b"", 0));
    world.until(Until::Read);
    assert_eq!(
        world.answers,
        vec![engine::CallAnswer::ForgeRead(Box::new(Ok(Answer::File {
            head,
            path: b"file".as_slice().into(),
            bytes: b"contents".as_slice().into(),
            truncated: false,
        })))]
    );
    assert_eq!(world.protocol.answers.len(), renders + 1, "the retry renders exactly one file answer");
    no_record(world, chat, 102);
}

#[test]
fn both_phases_of_a_branch_file_read_require_resource_authority() {
    let mut world = World::configured(true, true);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 1, Box::new([])));
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { panic!("chat") };
    batch[0].authority.grants = Box::new([people::Grant {
        connector: 0,
        kind: 1,
        pattern: people::Pattern {
            segments: [b"forge".as_slice(), b"forge.example", b"org", b"repo", b"branch"].map(Box::from).into(),
            last: people::Last::Exact(b"main".as_slice().into()),
        },
    }]);
    let chat = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"contents", 1)
        .expect("file head");
    let head = translate::commit(head);
    send(
        &mut world,
        &chat,
        93,
        engine::Tool::ReadNamed { repository: None, read: Read::Branch { branch: Box::new([]) } },
    );
    world.until(Until::Read);
    assert_eq!(world.answers, vec![engine::CallAnswer::ForgeRead(Box::new(Ok(Answer::Commit(head))))]);
    no_record(&world, &chat, 93);
    for (completion, branch, expected_reads, rendered) in [
        (
            94,
            b"".as_slice(),
            vec![Read::Branch { branch: b"main".as_slice().into() }],
            Read::File { head, path: b"file".as_slice().into(), max_bytes: world.env.limits.forge.client.answer_bytes },
        ),
        (95, b"other".as_slice(), vec![], Read::Branch { branch: b"other".as_slice().into() }),
    ] {
        world.answers.clear();
        world.forge_reads = Some(Vec::new());
        send(&mut world, &chat, completion, file(branch, 0));
        for _ in 0..200 {
            world.tick();
            if !world.answers.is_empty() {
                break;
            }
        }
        assert!(
            matches!(world.answers.as_slice(), [engine::CallAnswer::ToolDenied { findings, .. }] if !findings.is_empty()),
            "{:?}",
            world.answers
        );
        let reads: Vec<_> =
            world.forge_reads.as_ref().expect("record reads").iter().map(|(_, read)| read.clone()).collect();
        assert_eq!(reads, expected_reads, "neither denied phase reaches the forge");
        let name: Box<[u8]> = u64::from(completion).to_be_bytes().into();
        assert_eq!(world.protocol.read_forms.get(&name), Some(&rendered));
        no_record(&world, &chat, completion);
    }
    world.close();
}
