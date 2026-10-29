use smith_local_process_world::{
    Placement,
    referee::{CheckoutRead, review},
};
use smith_real_world::{Scratch, World};

#[test]
fn a_chat_ends_with_a_commit_in_place() {
    let scratch = Scratch::new();
    let mut world = World::changed(12, &scratch, Placement::Spawned);
    let ending = world.scenario.ending.clone();
    world.settle();
    review(&world.seen(), world.checkout(), ending).assert_passed(12);
    let head = world.checkout().head().expect("committed checkout");
    let files = world.checkout().files(&head);
    assert_eq!(files[b"original.txt".as_slice()], b"edited\n");
    assert_eq!(files[b"result.txt".as_slice()], b"new\n");
    assert_eq!(files[b"command.txt".as_slice()], b"ran\n");
    assert_eq!(files[b"checks-ran.txt".as_slice()], b"checked\n");
    let seen = world.seen();
    assert_eq!(seen.queries.len(), 6, "every scripted tool reached the real agent");
    for (query, name) in seen.queries.iter().skip(1).zip([b"read".as_slice(), b"search", b"edit", b"write", b"shell"]) {
        let id = query
            .messages
            .iter()
            .flat_map(|message| message.parts.iter())
            .find_map(|part| match part {
                skein_fake_llm_domain::api::Part::ToolCall { id, name: called, .. } if called.as_ref() == name => {
                    Some(id)
                }
                _ => None,
            })
            .expect("next provider request includes the tool call");
        let output = query
            .messages
            .iter()
            .flat_map(|message| message.parts.iter())
            .find_map(|part| match part {
                skein_fake_llm_domain::api::Part::ToolOutput { id: answered, output, .. } if answered == id => {
                    Some(output)
                }
                _ => None,
            })
            .expect("actual tool terminal reaches the provider");
        assert!(
            !output.starts_with(b"Error:"),
            "tool {}: {}",
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(output)
        );
        if name == b"search" {
            assert!(
                output.windows(12).any(|bytes| bytes == b"original.txt"),
                "actual rg output names the matched file"
            );
            assert!(output.windows(8).any(|bytes| bytes == b"original"), "actual rg output contains the matched text");
        }
        if name == b"read" {
            assert!(output.windows(8).any(|bytes| bytes == b"original"), "actual file bytes reached the provider");
        }
        if name == b"shell" {
            assert!(output.starts_with(b"exit code 0"), "the real command exited successfully");
        }
    }
}
