//! A need addresses a person directly, without a template.
use crate::tests::fixtures as f;
use jig_client::contract as c;
const ASK: &[&str] = &[
    "need",
    "ask",
    "--project",
    "3",
    "--choice",
    "yes",
    "--choice",
    "no",
    "--why",
    "the deploy waits",
    "Ship it?",
    "--key",
    "ask",
];
#[test]
fn need_ask_makes_a_person_verdict_and_keeps_jigs_refusal_words() {
    for address in [None, Some("party"), Some("role")] {
        let mut root = f::signed_in(f::PARTY);
        let argv = match address {
            None => f::extended(ASK, &[]),
            Some("party") => f::extended(ASK, &["--party", "9"]),
            Some(_) => f::extended(ASK, &["--role", "2"]),
        };
        let call = root.submit(argv.as_slice());
        let c::Ask::Create { project, tasks } = f::act_ask(&root, call) else { panic!("Create") };
        assert_eq!(*project, 3);
        assert_eq!(tasks.len(), 1);
        let task = tasks.get(0).unwrap();
        assert_eq!(
            task.executor,
            c::Executor::Person(match address {
                None => c::PersonAddress::Role(0),
                Some("party") => c::PersonAddress::Party(9),
                Some(_) => c::PersonAddress::Role(2),
            })
        );
        assert_eq!(task.spec.words.bytes(), b"Ship it?\n\nWhy: the deploy waits");
        assert_eq!(task.spec.parameters.len(), 2);
        assert_eq!(task.spec.parameters.get(0), Some(&c::Parameter::Bytes { name: 1, value: f::act_text(b"yes") }));
        let c::NewContract::Verdict { choices } = &task.contract else { panic!("verdict") };
        assert_eq!(choices.len(), 2);
        assert_eq!(choices.get(0).unwrap().code, 1);
        assert_eq!(choices.get(1).unwrap().code, 2);
        assert_eq!(task.authority.spend, 0);
        assert_eq!(task.authority.tools, 0);
        assert!(task.llm.is_none());
        assert_eq!(task.flags.ended_by, c::EndedBy::Result);
        assert_eq!(root.sent_name(call), Some(f::named("ask")));
        root.done(call, c::Outcome::Document(c::DocumentOutcome::Created { first: 40, last: 40, given: None }));
        assert_eq!(root.answer(call).unwrap().stdout.as_ref(), b"done: task 40, need\n");
    }
    for extra in [&["--choice", "yes"][..], &["--choice", "yes", "--choice", "no", "--party", "9", "--role", "0"][..]] {
        let mut root = f::signed_in(f::PARTY);
        let argv =
            f::extended(&["need", "ask", "--project", "3", "--why", "waits", "Ship?", "--key", "invalid"], extra);
        let call = root.submit(argv.as_slice());
        assert_eq!(root.answer(call).unwrap().status, 1);
        assert!(root.nothing_sent());
    }
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(ASK);
    root.answered(call, c::Answer::Refused { refusal: c::Refusal::Unknown, at: None });
    let printed = root.answer(call).unwrap();
    assert_eq!(printed.status, 2);
    assert_eq!(printed.stderr.as_ref(), b"need ask: No visible object with that name\n");
}
