//! A chat's folded call run stays open across new words (client/README.md, 3.5, its calls).
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Harness, Outcome, Script, Step};
use skein_ui::Role;
use temper_client_browser_world::{ABSENT, Tier, WITHIN};

#[test]
fn a_run_of_calls_opens_to_its_rows() {
    const CUE: &str = "Add CSV export to widget";
    let tier = Tier::demo_asked(
        "chat-call-line",
        CUE,
        serde_json::json!([
            {"lines":[{"call":"read_docs","arguments":{"topic":"hold"}}],"finish":"tool_calls","tokens":8},
            {"lines":[{"call":"read_docs","arguments":{"topic":"chat"}}],"finish":"tool_calls","tokens":8},
            {"lines":[{"text":"I read the hold and chat pages."},{"call":"wait","arguments":{}}],"finish":"tool_calls","tokens":16}
        ]),
    );
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let gone = |find| Step::Person(PersonStep::Gone { find, within: ABSENT });
    let row = |topic: &[u8]| Find::named(Role::Paragraph, &[b"read_docs ".as_slice(), topic].concat());
    let line = || Find::exactly_named(Role::Disclosure, b"Read 2 docs");
    let kept = || Step::Keep {
        selector: b"details.temper-work-run[open]".as_slice().into(),
        attribute: Some(b"data-node".as_slice().into()),
        name: b"call-line".as_slice().into(),
    };
    // The read_docs answer starts with its topic's title; its page begins on the second line.
    let body = || Find::text(b"# Hold");
    let mut steps = temper_client_browser_world::start_playground_chat(CUE.as_bytes());
    steps.extend([
        see(Find::exactly_named(Role::Paragraph, b"I read the hold and chat pages.")),
        see(line()),
        gone(row(b"hold")),
        gone(row(b"chat")),
        gone(body()),
        Step::Person(PersonStep::Press { find: line() }),
        see(row(b"hold")),
        see(row(b"chat")),
        kept(),
        gone(body()),
        Step::Person(PersonStep::Type {
            find: Find::exactly_named(Role::TextArea, b"Message"),
            words: b"What about those pages?".as_slice().into(),
        }),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Send") }),
        see(Find::exactly_named(Role::Paragraph, b"What about those pages?")),
        see(Find::named(Role::Paragraph, b"I am the scripted model,")),
        see(row(b"hold")),
        see(row(b"chat")),
        kept(),
        gone(body()),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    let mut harness = Harness::new(tier.setup());
    assert_eq!(harness.run(&mut [&mut script]), Outcome::Done);
    let lines: Vec<_> = harness.kept().iter().filter(|kept| kept.name.as_ref() == b"call-line").collect();
    let [before, after] = lines.as_slice() else { panic!("the open call line is kept before and after the reply") };
    assert_eq!(before.value, after.value, "new words preserve the open call line's node");
}
