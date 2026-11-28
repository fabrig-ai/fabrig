//! The session edge of a park: close in flight and reopen from told parts.
//! Contract: smith/docs/design/parks.md, section 4.

use skein_lib::Token;
use smith_domain_session::{self as session, llm, record};
use smith_domain_tools::{self as tools, Done, Op};
use smith_session_world::bytes;
use smith_session_world::recorded::{self, Scripted};

fn opening() -> record::Opening {
    let mut opening = recorded::opening(None, 100);
    opening.spec.tell_parts = true;
    let mount = tools::Name::new(bytes(b"repo")).expect("fixture mount");
    opening.spec.authority = tools::Authority {
        cwd: Some(mount.clone()),
        repos: Box::new([tools::Directory { host: Box::default(), mount, root: Token::new(7), writable: false }]),
        grants: tools::Grants { inspect: true, modify: false, shell: false },
    };
    opening.spec.delegated = Box::default();
    opening
}

fn read(path: &str) -> llm::Block {
    llm::Block::ToolCall(llm::ToolCall {
        id: bytes(path.as_bytes()),
        name: bytes(b"read"),
        input: format!(r#"{{"path":"{path}"}}"#).into_bytes().into(),
        call: llm::Decoded::Owned {
            call: tools::Call::Read {
                path: tools::Path {
                    absolute: false,
                    parts: Box::new([tools::Part::Name {
                        name: tools::Name::new(bytes(path.as_bytes())).expect("fixture filename"),
                    }]),
                },
                offset: 1,
                window: tools::Budget::Default,
            },
        },
        replay: None,
    })
}

fn load(world: &Scripted, index: usize, path: &[u8]) -> Token {
    let (owner, Op::Load { at, .. }) = &world.operations[index] else { panic!("expected a file load") };
    assert_eq!(at.path.as_ref(), path);
    *owner
}

fn loaded(world: &mut Scripted, owner: Token, content: &[u8]) {
    world.step(session::Event::Done {
        owner,
        done: Done::Loaded { content: bytes(content), version: tools::Version::new([1; 4]) },
    });
}

// These stories have one answer and one kept result. Supply the opening that
// the parent would build from those parts, with freshly classified calls.
fn restore_parts(opening: &mut record::Opening, closed: &Scripted, content: Box<[llm::Block]>) {
    let [answer, result] = closed.parts.as_slice() else { panic!("one answer and one kept result") };
    assert_eq!((answer.kind, result.kind), (record::PartKind::Answer, record::PartKind::Result));
    assert_eq!((answer.body.sequence, result.body.sequence), (1, 1));
    let record::Kind::First(opener) = &answer.body.kind else { panic!("first answer keeps the opening") };
    opening.transcript = Some(record::Transcript {
        version: record::VERSION,
        endpoint: answer.body.endpoint,
        dialect: answer.body.dialect,
        turns: Box::new([answer.body.clone()]),
    });
    opening.cut = Some(record::Cut {
        previous: 0,
        answers: 1,
        usage: answer.body.usage,
        system: opener.system.clone(),
        waking_in_result: false,
        messages: u32::try_from(answer.body.messages.len()).expect("fixture message count"),
        content,
        results: result.body.messages[0].content.clone(),
    });
}

#[test]
fn a_cut_batch_runs_only_the_call_without_a_kept_result() {
    let mut first = Scripted::new(1);
    first.open(opening());
    let answer = [read("first"), read("second")];
    first.complete(Box::new(answer.clone()), llm::Stop::ToolUse, recorded::USAGE);
    assert_eq!(first.operations.len(), 2, "both reads started in the batch");
    let kept = load(&first, 0, b"first");
    let pending = load(&first, 1, b"second");
    loaded(&mut first, kept, b"kept bytes\n");
    first.step(session::Event::Cut { session: first.session.expect("open main session") });
    assert_eq!(first.cancelled_operations, [pending]);
    assert!(first.end.is_none(), "the cut waits for its outstanding read");
    first.step(session::Event::Done { owner: pending, done: Done::Cancelled });
    assert_eq!(first.end, Some(session::End::Closed));
    assert!(first.turns.is_empty(), "the cut leaves parts, not a whole turn");
    assert_eq!(first.prompts.len(), 1);
    first.close();

    let mut next = opening();
    restore_parts(&mut next, &first, Box::new(answer));
    let mut resumed = Scripted::new(2);
    resumed.open(next);
    assert!(resumed.prompts.is_empty(), "the kept answer needs no completion");
    assert_eq!(resumed.operations.len(), 1, "only the unanswered call runs");
    let owner = load(&resumed, 0, b"second");
    loaded(&mut resumed, owner, b"new bytes\n");
    assert_eq!(resumed.prompts.len(), 1);
    let messages = &resumed.prompts[0].messages;
    let [_, _, results] = messages.as_ref() else { panic!("opening, kept answer, and both results") };
    let [kept, new, wake] = results.content.as_ref() else { panic!("two results, then the waking text") };
    assert_read(kept, b"first", b"kept bytes\n");
    assert_read(new, b"second", b"new bytes\n");
    assert_eq!(*wake, llm::Block::Text { text: bytes(b"wake"), replay: None });
    resumed.close();
}

fn assert_read(block: &llm::Block, expected_id: &[u8], expected_content: &[u8]) {
    let llm::Block::ToolResult { id, result: llm::Returned::Owned { outcome: tools::Outcome::Read { content, .. } } } =
        block
    else {
        panic!("expected a read result, got {block:?}");
    };
    assert_eq!((id.as_ref(), content.as_ref()), (expected_id, expected_content));
}

#[test]
fn a_cut_child_session_continues_from_its_kept_answer_and_result() {
    let affinity = llm::Affinity { key: [0x42; 16], thread: 3 };
    let mut start = opening();
    start.spec.affinity = affinity;
    start.spec.system = bytes(b"child instructions");
    start.spec.prompt = bytes(b"review the file");
    let mut first = Scripted::new(3);
    first.open(start);
    let answer = read("data");
    first.complete(Box::new([answer.clone()]), llm::Stop::ToolUse, recorded::USAGE);
    let owner = load(&first, 0, b"data");
    loaded(&mut first, owner, b"child read this\n");
    assert_eq!(first.prompts.len(), 2);
    first.step(session::Event::Cut { session: first.session.expect("open child session") });
    assert!(first.end.is_none(), "the cut waits for the pending completion to cancel");
    first.close();
    assert_eq!(first.end, Some(session::End::Closed));
    assert!(first.parts.iter().all(|part| part.thread == affinity.thread));

    let mut next = opening();
    next.spec.affinity = llm::Affinity { key: [0x17; 16], thread: 5 };
    next.spec.system = bytes(b"recomposed instructions");
    restore_parts(&mut next, &first, Box::new([answer]));
    let mut resumed = Scripted::new(4);
    resumed.open(next);
    assert!(resumed.operations.is_empty(), "the child's read has its result already");
    resumed.resume();
    assert_eq!(resumed.prompts.len(), 1);
    let prompt = &resumed.prompts[0];
    assert_eq!(prompt.affinity, llm::Affinity { thread: 5, ..affinity });
    assert_eq!(prompt.system.as_ref(), b"child instructions");
    assert_eq!(prompt.messages, first.prompts[1].messages, "the next request carries the child's conversation");
    resumed.close();
}
