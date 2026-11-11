//! Remembered preferences across chats, person reads and private turns
//! (temper/docs/design/domain/agent/ask-temper.md, 4; jig/docs/design/domain/engine/briefs.md, 2.1).
use super::*;
use jig_client::contract as c;
use jig_domain_core::reads;
use jig_domain_notes as notes;
use serde_json::json;
use skein_lib::{List, Writer};
use temper_engine_domain_world::protocol;

const LINE: &[u8] = b"I want short answers.";
const WITHHELD: &str = "Withheld: shown to its person alone.";

fn remember(world: &mut World, run: &engine::Assignment, action: &str) -> (Box<[u8]>, Box<[u8]>) {
    let mut input = json!({"action":action,"text":std::str::from_utf8(LINE).expect("fixture line is UTF-8")});
    if action == "change" {
        input["line"] = json!(1);
    }
    let input: Box<[u8]> = input.to_string().into_bytes().into();
    let (body, error) = host_tool(&mut world.driver, run, 1, b"remember", &input);
    assert!(!error, "{}", String::from_utf8_lossy(&body));
    let did = if action == "add" { "kept" } else { "changed" };
    assert_eq!(body.as_ref(), format!("remember: line 1 {did}; 1 of 12 lines.\n1. I want short answers.").as_bytes());
    (input, body)
}

fn preferences(run: &engine::Assignment) -> Box<[u8]> {
    let section = run
        .sections
        .iter()
        .find(|section| section.kind == engine::BriefKind::Core(jig_domain_brief::Core::Preferences))
        .expect("the assigned brief includes preferences");
    let section = temper_engine_smith::section(section);
    assert_eq!(section.title.as_ref(), b"Kept about this person");
    section.text
}

fn kept(world: &World) -> notes::Entry {
    let entries: Vec<_> = world
        .driver
        .store
        .rows
        .values()
        .filter_map(|row| {
            if let engine::Record::Core(core::Record::Notes(notes::Record::Entry(entry))) = row
                && entry.scope == (notes::Scope::Party { party: OWNER_PERSON })
            {
                Some(entry.clone())
            } else {
                None
            }
        })
        .collect();
    let [entry] = entries.as_slice() else { panic!("one preference: {entries:?}") };
    assert_eq!(entry.description.as_ref(), LINE);
    assert!(entry.body.is_empty());
    entry.clone()
}

fn delete(entry: &notes::Entry) -> people::Ask {
    people::Ask::EditNote {
        project: 0,
        name: entry.name,
        scope: Box::new(people::NoteScope::Party { party: OWNER_PERSON }),
        change: Box::new(people::NoteChange::Delete { recalled: entry.revision }),
    }
}

fn read(world: &mut World, session: u64, query: reads::Read) -> reads::Answer {
    let reply_to = ReplyTo::new(Token::new(100));
    let start = world.driver.delivered.len();
    world.driver.send(Event::Party(PartyInput::Read { reply_to, sign_in: session, read: query }));
    world.driver.settle();
    world.driver.delivered[start..]
        .iter()
        .find_map(|delivery| {
            if let Delivery::Read { reply, answer } = delivery
                && *reply == ReplyTo::new(Token::new(100))
            {
                Some(answer.clone())
            } else {
                None
            }
        })
        .expect("person's read answered")
}

#[test]
fn a_kept_line_reaches_the_next_chat_and_leaves_with_its_person() {
    let mut world = remembering_world();
    world.member();
    let first = world.chat(ASK_TEMPLATE, world.owner, 20);
    remember(&mut world, &first, "add");
    let entry = kept(&world);
    assert_eq!(entry.references.as_slice(), &[first.task]);
    let ordinary = world.chat(CHAT, world.owner, 24);
    assert_ne!(first.task, ordinary.task);
    assert!(
        ordinary
            .sections
            .iter()
            .all(|section| section.kind != engine::BriefKind::Core(jig_domain_brief::Core::Preferences)),
        "a charter-1 chat reads no preferences"
    );
    world.ask(
        world.owner,
        25,
        people::Ask::Cancel {
            project: PROJECT,
            task: ordinary.task,
            reason: b"Close the ordinary chat".as_slice().into(),
            grace: None,
        },
    );
    world.driver.answer(CHANNEL, ordinary.task, ordinary.attempt, 0, tasks::End::Parked, engine::Parts::none());
    world.driver.settle();
    let second = world.chat(ASK_TEMPLATE, world.owner, 21);
    assert_ne!(ordinary.task, second.task);
    assert_eq!(preferences(&second).as_ref(), b"1. I want short answers.\n");
    world.ask(world.owner, 22, delete(&entry));
    assert_eq!(world.reply(22), &people::Reply::Outcome(people::Outcome::NoteEdited { name: entry.name }));
    let third = world.chat(ASK_TEMPLATE, world.owner, 23);
    assert_ne!(second.task, third.task);
    assert_eq!(preferences(&third).as_ref(), b"None kept.\n");
}

#[test]
fn nobody_reads_another_persons_lines() {
    let mut world = remembering_world();
    let q = world.member();
    let people::Reply::SignedIn { person: member, .. } = *world.reply(11) else { panic!("Q signed in") };
    let first = world.chat(ASK_TEMPLATE, world.owner, 20);
    remember(&mut world, &first, "add");
    let entry = kept(&world);
    assert_eq!(
        read(
            &mut world,
            q,
            reads::Read::ScopedNotes {
                scope: notes::Scope::Party { party: OWNER_PERSON },
                description: None,
                from: None,
            }
        ),
        reads::Answer::None(reads::Why::Unknown)
    );
    let correction = people::Ask::EditNote {
        project: 0,
        name: entry.name,
        scope: Box::new(people::NoteScope::Party { party: OWNER_PERSON }),
        change: Box::new(people::NoteChange::Correct {
            description: b"Q's correction".as_slice().into(),
            body: Box::new([]),
            references: Box::new([first.task]),
            recalled: entry.revision,
        }),
    };
    for (key, ask) in [(21, correction), (22, delete(&entry))] {
        world.ask(q, key, ask);
        assert_eq!(world.reply(key), &people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Unknown)));
        assert_eq!(kept(&world), entry, "Q changes none of P's line");
    }
    let other = world.chat(ASK_TEMPLATE, q, 23);
    assert_ne!(other.task, first.task);
    assert_eq!(world.driver.task(other.task).unwrap().requester, tasks::Party::Person(member));
    assert_eq!(preferences(&other).as_ref(), b"None kept.\n");
    let (body, error) = host_tool(
        &mut world.driver,
        &other,
        1,
        b"remember",
        br#"{"action":"change","line":1,"text":"Q wants long answers."}"#,
    );
    assert!(error);
    assert_eq!(body.as_ref(), b"remember: no line 1; the list is empty. Nothing changed.");
    assert_eq!(kept(&world), entry);
}

fn projected(answer: &reads::Answer, query: &reads::Read) -> c::ReadValue {
    use jig_protocol_engine_clients::translate;
    let wire = match answer {
        reads::Answer::Page(page) => translate::pages::page(
            &[],
            query,
            page,
            &temper_engine_smith::private(),
            &jig_client_protocol_wire::CEILINGS,
        ),
        _ => translate::objects::document(
            answer,
            &[],
            &temper_engine_smith::private(),
            &jig_client_protocol_wire::CEILINGS,
        ),
    }
    .expect("client projection");
    let mut bytes = Writer::new(usize::try_from(wire.measure()).expect("bounded client document"));
    wire.encode(&mut bytes).expect("measured client document");
    let preset = temper_client_tools::limits::client().client;
    let limits =
        jig_client_protocol_link::Limits::new(preset.domain, preset.exchanges).expect("client receiving limits");
    jig_client_protocol_link::translate::reads::value(&bytes.finish(), &limits).expect("client reads the turn")
}

fn assert_private(turn: &c::ArchivedTurn, whole: bool, input: &[u8]) {
    let turn = turn.turn.decoded().expect("decoded turn");
    assert!(!turn.cut);
    let blocks: Vec<_> = turn.messages.iter().flat_map(|message| message.blocks.iter()).collect();
    let texts: Vec<_> = blocks
        .iter()
        .filter_map(|block| if let c::Block::Text(text) = block { Some(text.bytes()) } else { None })
        .collect();
    let calls: Vec<_> =
        blocks.iter().filter_map(|block| if let c::Block::Call(call) = block { Some(call) } else { None }).collect();
    let [call] = calls.as_slice() else { panic!("one remember call: {calls:?}") };
    assert_eq!(call.tool.bytes(), b"remember");
    assert_eq!(call.input.bytes(), if whole { input } else { b"{}" });
    let text = texts.concat();
    let text = std::str::from_utf8(&text).expect("turn text is UTF-8");
    assert!(text.contains("## Kept about this person"), "{text}");
    assert_eq!(text.contains("1. I want short answers."), whole, "{text}");
    assert_eq!(text.contains(WITHHELD), !whole, "{text}");
}

#[test]
fn a_turn_shows_its_persons_list_to_them_alone() {
    let mut world = remembering_world();
    let q = world.member();
    let first = world.chat(ASK_TEMPLATE, world.owner, 20);
    remember(&mut world, &first, "add");
    let run = world.chat(ASK_TEMPLATE, world.owner, 21);
    assert_eq!(preferences(&run).as_ref(), b"1. I want short answers.\n");
    let (input, answer) = remember(&mut world, &run, "change");
    let bytes = transcript(&run, &input, &answer);
    world.driver.send(protocol::turn(CHANNEL, run.task, run.attempt, 1, 0, None, &bytes));
    world.driver.settle();
    for (session, whole) in [(world.owner, true), (q, false)] {
        for query in [
            reads::Read::Turn { task: run.task, attempt: run.attempt, turn: 1 },
            reads::Read::Activity { task: run.task, before: None },
        ] {
            let answer = read(&mut world, session, query.clone());
            match projected(&answer, &query) {
                c::ReadValue::Turn(turn) => assert_private(&turn, whole, &input),
                c::ReadValue::Activity(page) => {
                    assert!(!page.cut);
                    let turns: Vec<_> = page.rows.iter().filter_map(|row| row.decoded.as_ref()).collect();
                    let [turn] = turns.as_slice() else { panic!("one activity turn: {turns:?}") };
                    assert_private(turn, whole, &input);
                }
                other => panic!("turn or activity: {other:?}"),
            }
        }
    }
    let later = world.chat(ASK_TEMPLATE, world.owner, 22);
    let (text, _) = see_whole(&mut world.driver, &later, 10, &format!("task {} turn {}.1", run.task, run.attempt));
    assert!(text.contains(WITHHELD), "{text}");
    assert!(!text.contains(std::str::from_utf8(LINE).expect("fixture line is UTF-8")), "{text}");
}

fn transcript(run: &engine::Assignment, input: &[u8], answer: &[u8]) -> Box<[u8]> {
    use smith_transcript::v7 as s;
    let bounds = &s::CEILINGS;
    let section = preferences(run);
    let text =
        format!("## Kept about this person\n\n{}", std::str::from_utf8(&section).expect("preferences are UTF-8"));
    let blocks = [
        s::Block::Text(
            s::Text::new(bounds, s::TextParts { text: text.into_bytes().into(), replay: None }).expect("brief text"),
        ),
        s::Block::Call(
            s::Call::new(
                bounds,
                s::CallParts {
                    id: b"remember-1".as_slice().into(),
                    name: b"remember".as_slice().into(),
                    input: input.into(),
                    replay: None,
                },
            )
            .expect("remember call"),
        ),
        s::Block::Result(
            s::ToolResult::new(
                bounds,
                s::ToolResultParts {
                    id: b"remember-1".as_slice().into(),
                    returned: s::Returned::Said(
                        s::Said::new(bounds, s::SaidParts { text: answer.into(), error: false })
                            .expect("remember answer"),
                    ),
                },
            )
            .expect("remember result"),
        ),
    ];
    let message = s::Message::new(
        bounds,
        s::MessageParts { role: s::Role::Assistant, blocks: List::from_boxed(Box::new(blocks)) },
    )
    .expect("assistant message");
    let turn = s::Turn::new(
        bounds,
        s::TurnParts {
            system: None,
            model: b"scripted".as_slice().into(),
            effort: None,
            kind: s::TurnKind::Ordinary,
            opening: None,
            endpoint: b"scripted".as_slice().into(),
            dialect: 1,
            place: 1,
            usage: s::Usage::new(
                bounds,
                s::UsageParts { input: None, output: None, cache_read: None, cache_write: None, reasoning: None },
            )
            .expect("unused token counts"),
            spent: 0,
            jobs: 0,
            messages: List::from_boxed(Box::new([message])),
        },
    )
    .expect("scripted turn");
    let mut out = Writer::new(usize::try_from(turn.measure()).expect("bounded turn"));
    turn.encode(&mut out).expect("measured transcript");
    out.finish()
}
