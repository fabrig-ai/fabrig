//! The standing task's own scope and required Now (domain/tasks/standing.md, 6).
use crate::{fresh_windows, settings_support as support};
use jig_core_world::{
    client_peer::ClientPeer,
    fixtures,
    reads::{Parties, ask as read, latest},
    world::{Boot, Setup, World},
};
use jig_domain_authority as authority;
use jig_domain_brief::{Core as Section, GatherPlaced};
use jig_domain_core::{self as core, reads};
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{List, ReplyTo, Token, Wall};

fn start(ending: people::EndedBy, resume: bool, kept: Vec<notes::Entry>) -> (World, u64, u64) {
    let (mut config, limits) = support::run_configuration();
    config.core.defaults.charters[0].policy.resume = resume;
    let mut rows = jig_test_profile::fixtures::account_rows(&config.core.settings.llm_providers).into_vec();
    for entry in kept {
        rows.push(root::Record::Core(core::Record::Notes(notes::Record::Line(notes::Line {
            name: entry.name,
            scope: entry.scope.clone(),
            description: entry.description.clone(),
            revision: entry.revision,
        }))));
        rows.push(root::Record::Core(core::Record::Notes(notes::Record::Entry(entry))));
    }
    let mut world = World::new(Setup {
        configuration: Some(config),
        limits,
        rows: Some(rows.into_boxed_slice()),
        boot: Boot::Empty,
        ..Setup::default()
    });
    world.client_peer = Some(ClientPeer::default());
    let signed = support::sign_in(&mut world);
    world.sign_in = Some(signed);
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
    fresh_windows::hello(&mut world);
    let task = create(&mut world, signed, 101, ending);
    (world, signed, task)
}

#[expect(clippy::unnecessary_wraps, reason = "the store's transcript renderer returns optional text")]
fn transcript_text(bytes: &[u8]) -> Option<Box<[u8]>> {
    Some(bytes.into())
}

fn create(world: &mut World, signed: u64, key: u8, ending: people::EndedBy) -> u64 {
    let mut ask = fixtures::chat(world, 1, b"Keep watching".as_slice().into());
    let people::Ask::Create { batch, .. } = &mut ask else { panic!("creation fixture") };
    let member = &mut batch[0];
    member.flags.ended_by = ending;
    member.flags.conversation = ending == people::EndedBy::Requester;
    member.tracked = None;
    member.budget = people::Budget::Named;
    member.authority.spend = 40;
    member.authority.tools = 2047;
    let answer = support::ask(world, signed, key, ask);
    let people::Reply::Outcome(people::Outcome::Created { first, .. }) = answer else { panic!("created: {answer:?}") };
    assert_eq!(
        world.assigned.as_ref().expect("assigned").task,
        first,
        "phase {:?}; trace {:?}",
        fixtures::live_row(world, first).phase,
        world.trace
    );
    first
}

fn wake(world: &mut World, key: u8) -> u64 {
    let task = world.assigned.as_ref().expect("assigned").task;
    let signed = world.sign_in.expect("signed in");
    let answer =
        support::ask(world, signed, key, people::Ask::Say { project: 1, task, words: b"Continue".as_slice().into() });
    let people::Reply::Outcome(people::Outcome::Said { message, .. }) = answer else {
        panic!(
            "wake {key}: {answer:?}; phase {:?}; immediate {:?}",
            fixtures::live_row(world, task).phase,
            world.client_peer.as_ref().expect("peer").now
        )
    };
    message
}

fn section(world: &World, wanted: Section) -> Option<&[u8]> {
    world.assigned.as_ref().expect("assigned").brief.iter().find_map(|section| match section {
        GatherPlaced::Core { kind, text } if *kind == wanted => Some(text.as_ref()),
        _ => None,
    })
}

fn call(world: &mut World, completion: u32, action: core::NamedAction) -> core::CallPart {
    let key = world.call_key(completion);
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(u64::from(completion))),
        key,
        action,
    }));
    let (_, found, answer) = world.answers.last().expect("named answer");
    assert_eq!(*found, key);
    answer.clone()
}

fn note(scope: notes::Scope, name: u64, body: &[u8], recalled: Option<u32>) -> core::NamedAction {
    core::NamedAction::Note {
        entry: notes::New {
            name,
            scope,
            description: b"now".as_slice().into(),
            body: body.into(),
            references: List::with_capacity(0),
            author: notes::Author::Party { party: support::OWNER },
        },
        recalled,
    }
}

#[test]
fn a_standing_task_owns_its_now_scope_and_its_project_member_reads_it() {
    let (mut world, signed, task) = start(people::EndedBy::Standing, false, vec![]);
    let scope = notes::Scope::Goal { project: 1, goal: task };
    let answer = call(&mut world, 1, note(scope.clone(), 0, b"Waiting for the answer", None));
    let core::CallPart::NoteWritten { name, revision: 1, .. } = answer else { panic!("own scope: {answer:?}") };
    let answer = call(&mut world, 2, core::NamedAction::Recall { by: notes::Recall::Name { name }, page: 0 });
    let core::CallPart::NoteRecalled { entries, more: false } = answer else { panic!("own recall: {answer:?}") };
    assert_eq!(entries[0].body.as_ref(), b"Waiting for the answer");
    assert_eq!(entries[0].author, notes::Author::Task { task, attempt: world.call_key(1).attempt });
    world.end(tasks::End::Parked, 0);
    let message = wake(&mut world, 71);
    assert!(section(&world, Section::Now).expect("next Now").ends_with(b"Waiting for the answer"));
    let answer = call(&mut world, 1, note(scope.clone(), name, b"Answer arrived", Some(1)));
    assert!(matches!(answer, core::CallPart::NoteWritten { revision: 2, .. }));
    world.turn(1, 0, Some(message), b"Updated the Now");
    world.end(tasks::End::Parked, 0);
    let message = wake(&mut world, 72);
    let now = std::str::from_utf8(section(&world, Section::Now).expect("updated Now")).expect("Now text");
    assert!(now.contains(&format!("Note {name}, revision 2, written at ")));
    assert!(now.ends_with("Answer arrived"));
    world.turn(1, 0, Some(message), b"Read the updated Now");
    world.end(tasks::End::Parked, 0);

    world.send(fixtures::sign_in(Token::new(900), 1, b"now-reader", b"reader", b"Reader"));
    let (
        _,
        root::Delivery::Core(core::Held::PeopleReply {
            reply: people::Reply::SignedIn { person, sign_in: member, .. },
            ..
        }),
    ) = world.client_peer.as_mut().expect("client peer").delivered.pop().expect("member sign in")
    else {
        panic!("member signs in")
    };
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: 1,
        holdings: Box::new([
            people::Holding { person: support::OWNER, role: people::Role::Owner },
            people::Holding { person, role: people::Role::Member },
        ]),
    })));
    world.sign_in = Some(member);
    let client_peer = world.client_peer.take();
    world.reads = Some(Parties::new(world.store.applied));
    read(
        &mut world,
        1,
        reads::Read::ScopedNotes { scope: scope.clone(), description: Some(b"now".as_slice().into()), from: None },
    );
    let reads::Answer::Page(page) = latest(&world) else { panic!("notes page") };
    let [reads::PageRow::Note(entry)] = page.rows.as_slice() else { panic!("one Now") };
    assert_eq!(entry.number, name);
    assert_eq!(entry.revision, 2);
    assert_eq!(entry.body.bytes.as_ref(), b"Answer arrived");

    world.client_peer = client_peer;
    world.sign_in = Some(signed);
    let other = create(&mut world, signed, 102, people::EndedBy::Result);
    assert_ne!(other, task);
    let before = world.store.rows[&root::Key::Core(core::Key::Notes(notes::Key::Entry { name }))].clone();
    let denied = core::CallPart::ToolDenied {
        answer: authority::Answer::Refuse,
        findings: Box::new([authority::Finding::Scope { source: authority::Source::Task }]),
    };
    assert_eq!(call(&mut world, 1, note(scope, name, b"Not mine", Some(2))), denied);
    assert_eq!(call(&mut world, 2, core::NamedAction::Recall { by: notes::Recall::Name { name }, page: 0 }), denied);
    assert_eq!(world.store.rows[&root::Key::Core(core::Key::Notes(notes::Key::Entry { name }))], before);
    fixtures::finish_chat(&mut world, 0, b"Scope checked");
    world.close();
}

fn kept(name: u64, description: &[u8], body: &[u8]) -> notes::Entry {
    notes::Entry {
        name,
        scope: notes::Scope::Goal { project: 1, goal: 1 },
        description: description.into(),
        body: body.into(),
        references: List::with_capacity(0),
        author: notes::Author::Party { party: support::OWNER },
        revision: 3,
        changed: Wall::from_nanos(123_456_789),
    }
}

#[test]
fn a_standing_brief_places_its_first_now_whole_and_bounds_an_oversized_body() {
    let mut elsewhere = kept(1, b"now", b"Other scope");
    elsewhere.scope = notes::Scope::Project { project: 1 };
    let rows = [
        ("none", vec![], None, 0),
        (
            "lowest exact description",
            vec![
                kept(30, b"now", b"Later"),
                kept(20, b"now", b"Where things stand"),
                kept(10, b"now later", b"Wrong description"),
                elsewhere,
            ],
            Some(b"Where things stand".to_vec()),
            0,
        ),
        ("at bound", vec![kept(20, b"now", &vec![b'x'; 4096])], Some(vec![b'x'; 4096]), 0),
        ("over bound", vec![kept(20, b"now", &vec![b'x'; 5000])], Some(vec![b'x'; 4096]), 5000),
        (
            "character edge",
            vec![kept(20, b"now", "界".repeat(1667).as_bytes())],
            Some("界".repeat(1365).into_bytes()),
            5001,
        ),
    ];
    for (label, kept, body, original) in rows {
        let (mut world, _, task) = start(people::EndedBy::Standing, false, kept);
        assert_eq!(task, 1);
        let brief = &world.assigned.as_ref().expect("assigned").brief;
        assert!(matches!(brief.first(), Some(GatherPlaced::Core { kind: Section::Task, .. })), "{label}");
        assert!(matches!(brief.get(1), Some(GatherPlaced::Core { kind: Section::Now, .. })), "{label}");
        let text = std::str::from_utf8(section(&world, Section::Now).expect("Now")).expect("character edge");
        assert!(text.contains("before you park"), "{label}");
        match body {
            Some(body) => {
                assert!(
                    text.starts_with("Note 20, revision 3, written at 123456789 ns since Unix epoch.\n"),
                    "{label}"
                );
                let (_, carried) = text.split_once("\n\n").expect("body separator");
                let marker = if original == 0 {
                    String::new()
                } else {
                    format!("\n[Now: {original} bytes of 4096; shorten it to the bound.]\n")
                };
                let expected = [body.as_slice(), marker.as_bytes()].concat();
                assert_eq!(carried.as_bytes(), expected, "{label}");
            }
            None => assert!(text.starts_with("None written yet.\n"), "{label}"),
        }
        assert!(
            !section(&world, Section::Task)
                .expect("task text")
                .windows(b"[Report:".len())
                .any(|part| part == b"[Report:")
        );
        world.end(tasks::End::Parked, 0);
        world.close();
    }
}

#[test]
fn a_standing_brief_omits_a_normal_tail_but_keeps_the_attempt_it_retries() {
    for (ending, resume) in
        [(people::EndedBy::Standing, false), (people::EndedBy::Standing, true), (people::EndedBy::Requester, false)]
    {
        let (mut world, _, _) = start(ending, resume, vec![]);
        world.store.transcript_text(transcript_text);
        let standing = ending == people::EndedBy::Standing;
        world.turn(1, 1, None, b"A normal attempt");
        world.end(tasks::End::Parked, 1);
        let read = wake(&mut world, 71);
        assert_eq!(
            section(&world, Section::TranscriptTail),
            if standing { None } else { Some(b"A normal attempt".as_slice()) }
        );
        assert_eq!(section(&world, Section::Now).is_some(), standing);
        world.turn(1, 1, Some(read), b"The attempt to retry");
        world.end(tasks::End::Failed(tasks::Class::Transient), 1);
        world.retry_due();
        let tail = section(&world, Section::TranscriptTail).expect("retry's tail");
        assert_eq!(tail, b"The attempt to retry", "{ending:?}, resume {resume}");
        world.turn(1, 1, None, b"The retry succeeded");
        world.end(tasks::End::Parked, 1);
        let read = wake(&mut world, 72);
        if standing {
            assert_eq!(section(&world, Section::TranscriptTail), None, "past failures do not make a wake a retry");
        }
        world.turn(1, 1, Some(read), b"The next wake");
        world.end(tasks::End::Parked, 1);
        world.close();
    }
}
