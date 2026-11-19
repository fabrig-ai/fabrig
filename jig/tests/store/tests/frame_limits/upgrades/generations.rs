use jig_protocol_store::{Generations, Older, OpenRefusal};

#[test]
fn generation_refusal_words_match_operator_instructions() {
    // protocol/store/opening.md 1: Each Generations refusal's text, whole, is what OpenRefusal::words gives.
    for (why, expected) in [
        (
            Generations::Busy,
            "Another process has <deployment> open, and this start has changed nothing in it. Stop that process first, then start again.",
        ),
        (
            Generations::Unreadable,
            "The `active` file in <deployment> does not name a generation, and this start has changed nothing in it. Restore <deployment> from a backup, or write `active` again to name a generation under `generations/` that stands, and start again.",
        ),
        (
            Generations::Missing { generation: 17 },
            "The generation `active` names in <deployment> has lost its `store/` or `secrets/`, and this start has changed nothing in it. Restore <deployment> from a backup, or write `active` again to name a generation under `generations/` that stands, and start again.",
        ),
        (
            Generations::Unnamed,
            "<deployment> holds generations that no `active` names, and this start has changed nothing in it. Name one again from a backup of <deployment>, or write `active` to name the generation under `generations/` to open, and start again.",
        ),
    ] {
        assert_eq!(OpenRefusal::Generations { why }.words().join("<deployment>"), expected);
    }
}

#[test]
fn an_older_builds_refusal_names_the_whole_layout_to_move() {
    // protocol/store/README.md 2: Older::Directory carries the whole instruction to move store/, secrets/ and transcripts/ together.
    let words = OpenRefusal::OlderBuild { why: Older::Directory }.words().join("<deployment>");
    assert_eq!(
        words,
        "<deployment> holds a store from an older build, in the layout before generations (`store/`, `secrets/` or `transcripts/` at its top), and this start has changed nothing in it. No build translates a store from before generations, build `<bridge>` included, so every start refuses this one until it is moved. To start afresh, stop the service, move `store/`, `secrets/` and `transcripts/` out of <deployment> together, and start again: nothing is deleted, and the next start makes a new, empty store. The new store has no tasks, sign-ins, accounts or keys, and every client signs in again.".replace("<bridge>", jig_protocol_store::FLOOR_BRIDGE)
    );
}
