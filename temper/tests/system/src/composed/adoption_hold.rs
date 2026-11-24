//! A new chat's run branch after home adoption (temper/docs/design/domain/forge/branches.md;
//! temper/docs/design/testing/tiers.md, 1.1).
use super::{Adoption, Key, World, call, engine, fake, parties, run, tasks};
use jig_domain_core::CoreRecord;
use skein_fake_llm_domain::api::{Finish, Line, Part, Script, Turn};

const HOME: &[u8] = b"org/repo";
const README: &[u8] = b"# app\n";
const REPLY: &[u8] = b"Delivered the adopted home";

fn config(seed: u64) -> engine::Config {
    let mut config = super::agreement::config(seed);
    config.core.defaults.charters[0].policy.modify = true;
    config
}

fn script() -> Script {
    Script {
        cue: b"@new".as_slice().into(),
        turns: Box::new([
            call(b"read", br#"{"path":"README.md"}"#),
            call(b"edit", br##"{"path":"README.md","edits":[{"old":"# app\n","new":"# app\nQA edit\n"}]}"##),
            call(b"deliver", br#"{"fields":{"title":"QA home edit","body":"Explicit mid-run delivery"}}"#),
            Turn {
                lines: Box::new([
                    Line::Text { text: REPLY.into() },
                    Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
                ]),
                finish: Finish::ToolCalls,
                tokens: 20,
            },
        ]),
    }
}

#[test]
#[ignore = "a chat's deliver before its end is not built: domain/agent/tools.md, and its status"]
fn a_chat_created_after_home_adoption_gets_a_writable_run_branch() {
    let mut world = World::configured_with(962, script(), true, config);
    world.party = super::signed_in_party(b"@new Read the home");
    world.forge.advance(HOME, b"main", b"README.md", README);
    world.run();

    let chat = world.assignment().task;
    assert!(
        matches!(world.party.replies.as_slice(), [(_, parties::Reply::SignedIn { .. }), (_, parties::Reply::Started { task })] if *task == chat)
    );
    assert_eq!(world.adoption, Adoption::Adopted);
    assert!(world.forge.branch(HOME, b"main").is_some(), "adopted main is readable");
    assert!(matches!(world.agent().answer(), run::Answer::Parked { .. }));

    let mut read_home = false;
    for prompt in world.agent().prompts() {
        let parts = || prompt.messages.iter().flat_map(|message| &message.parts);
        for part in parts() {
            if let Part::ToolOutput { id, output, is_error } = part {
                assert!(
                    !is_error && !output.starts_with(b"Error:"),
                    "tool refused: {}",
                    String::from_utf8_lossy(output)
                );
                read_home |= output.windows(b"# app".len()).any(|bytes| bytes == b"# app")
                    && parts().any(|part| {
                        matches!(part, Part::ToolCall { id: called, name, .. }
                        if called == id && name.as_ref() == b"read")
                    });
            }
        }
    }
    assert!(read_home, "the model received README content from the adopted home");
    assert!(
        world.store.rows.values().any(|row| matches!(
            row, engine::Record::Core(jig_domain_core::Record::Core(CoreRecord::Turn(turn)))
                if turn.task == chat && turn.transcript.windows(REPLY.len()).any(|bytes| bytes == REPLY)
        )),
        "the chat committed its reply"
    );
    let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(chat)));
    let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
        world.store.rows.get(&key)
    else {
        panic!("the chat is still live at the reply");
    };
    let branch = format!("temper/{}/r{chat}", row.root);
    let [home] = world.assignment().workspace.repositories.as_ref() else { panic!("one home checkout") };
    assert_eq!(home.push.as_deref(), Some(branch.as_bytes()), "the chat holds its own writer");
    let head = world.forge.branch(HOME, branch.as_bytes()).expect("the chat's own run branch");
    assert_eq!(
        world.forge.api(
            HOME,
            1,
            fake::Op::Read(fake::Read::File { commit: head, path: b"README.md".as_slice().into() })
        ),
        Ok(fake::Answer::File(b"# app\nQA edit\n".as_slice().into())),
        "a new task pushes its edited home to its run branch"
    );
}
