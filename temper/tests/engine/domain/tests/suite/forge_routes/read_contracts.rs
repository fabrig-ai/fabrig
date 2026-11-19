//! Worker contracts survive adoption; read selection does not depend on the tool name.
use super::*;

const STEPS: u32 = 1000;

#[test]
fn worker_forge_reads_keep_their_contract_and_renderer_across_restart() {
    let (mut world, chat) = change_tool::chat_world_with_words(true, b"Read the repository");
    world.protocol.host_calls = true;
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"contents", 1).unwrap();
    let digits = engine::commit_text(&translate::commit(head));
    let digits = std::str::from_utf8(&digits).unwrap();
    let old_file = format!(r#"{{"head":"{digits}","path":"file"}}"#);
    let file = format!(r#"{{"what":"file","commit":"{digits}","path":"file"}}"#);
    let file_answer = format!("file at {digits}:\ncontents");
    let checks = format!(r#"{{"what":"checks","commit":"{digits}"}}"#);
    let current = temper_engine_smith::wire::VERSION;
    let cases = [
        (1, "read_file", old_file.as_str(), "contents"),
        (2, "read_file", old_file.as_str(), "contents"),
        (3, "read_file", old_file.as_str(), "contents"),
        (4, "read_file", old_file.as_str(), "contents"),
        (current, "read_forge", file.as_str(), "contents"),
        (current, "read_forge", r#"{"what":"file","branch":"main","path":"file"}"#, file_answer.as_str()),
        (current, "read_forge", r#"{"what":"file","path":"file"}"#, file_answer.as_str()),
        (current, "read_forge", r#"{"what":"branch"}"#, digits),
        (9, "read_forge", r#"{"what":"file","branch":"main","path":"file"}"#, "contents"),
        (current, "read_issues", r#"{"what":"items"}"#, "Items:"),
        (current, "read_ci", checks.as_str(), ""),
        (
            current,
            "read_forge",
            r#"{"what":"branches","prefix":""}"#,
            "read_forge what=branches needs a nonempty prefix",
        ),
        (current, "read_branches", r#"{"prefix":""}"#, "read_forge what=branches needs a nonempty prefix"),
    ];
    let mut first = Vec::new();
    for pass in 0..2 {
        if pass == 1 {
            for _ in 0..STEPS {
                if world.root.quiescent()
                    && world.pending.is_empty()
                    && world.store.pending.is_empty()
                    && world.events.is_empty()
                {
                    break;
                }
                world.tick();
            }
            world.restart_same();
            world.until(Until::Ready);
        }
        for (position, (contract, tool, input, expected)) in cases.iter().enumerate() {
            let completion = u32::try_from(position + 93).unwrap();
            let mut name = skein_lib::Writer::new(16);
            name.put(&chat.attempt.to_be_bytes()).unwrap();
            name.put(&completion.to_be_bytes()).unwrap();
            name.put(&1_u32.to_be_bytes()).unwrap();
            let name = name.finish();
            let before = world.protocol.host_replies.len();
            world.send(protocol::up(
                Token::new(7),
                host::LinkUp::Call {
                    contract: *contract,
                    run: Token::new(chat.task),
                    attempt: Token::new(chat.attempt),
                    call: name.clone(),
                    tool: tool.as_bytes().into(),
                    input: input.as_bytes().into(),
                    writes: false,
                    deadline: Duration::from_secs(10),
                },
                engine::Parts::none(),
            ));
            for _ in 0..STEPS {
                world.tick();
                if world.protocol.host_replies.len() > before {
                    break;
                }
            }
            let (returned_name, error, text) =
                world.protocol.host_replies.get(before).expect("adopted worker receives its read");
            assert_eq!(returned_name, &name);
            assert_eq!(*error, input.contains("\"prefix\":\"\""), "{tool}: {}", String::from_utf8_lossy(text));
            assert!(String::from_utf8_lossy(text).contains(expected), "{tool}: {}", String::from_utf8_lossy(text));
            if *tool == "read_ci" {
                assert!(matches!(&world.protocol.answers.last().unwrap().1, engine::CallAnswer::ForgeRead(read)
                    if matches!(read.as_ref(), Ok(skein_forge::api::Answer::Checks(_)))));
            }
            if pass == 0 {
                first.push(text.clone());
            } else {
                assert_eq!(text, &first[position], "same read after adoption");
            }
        }
    }
}
