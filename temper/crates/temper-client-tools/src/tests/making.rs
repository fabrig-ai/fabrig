use super::fixtures as f;
use crate::jig::{self, call::Parsed};
use jig_client::{self as client, contract as c};
use temper_client_protocol_wire::{self as wire, v1 as w};

/// A returned number distinct from both existing setup examples and the removed forge.
const MADE_FORGE: u16 = 27;
const REMOVE_FORGE: u16 = 19;
const API: &str = "forge.example:3000";
const GIT: &str = "https://forge.example";
const WEB: &str = "https://web.example";
/// The client wire's `ForgejoChoice` text ceilings (notes/cl-wire.md, 1).
const API_BYTES: usize = 259;
const URL_BYTES: usize = 1024;
/// One byte beyond the address ceiling must be refused before sending.
const OVER_BOUND: usize = 1;
#[expect(clippy::manual_map, reason = "the tools match without closures")]
fn forgejo(web: Option<&[u8]>) -> w::ForgeChoice {
    let web = match web {
        Some(bytes) => Some(bytes.into()),
        None => None,
    };
    w::ForgeChoice::Forgejo(
        w::ForgejoChoice::new(
            &w::CEILINGS,
            w::ForgejoChoiceParts { api: API.as_bytes().into(), git: GIT.as_bytes().into(), web },
        )
        .unwrap(),
    )
}
fn made() -> w::Outcome {
    w::Outcome::Made(w::MadeForge::new(&w::CEILINGS, w::MadeForgeParts { forge: MADE_FORGE }).unwrap())
}
fn sent(root: &f::Harness, call: u32) -> w::Request {
    let client::Ask::Family { scope, record } = root.adoption_bytes(call) else { panic!("forge family") };
    assert_eq!(*scope, c::Scope::Deployment);
    assert_eq!(record.family, wire::FORGE);
    assert_eq!(record.version, wire::CLIENT_VERSION);
    wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
}
fn check(argv: &[&str], outcome: w::Outcome, text: &str, json: &str, status: u8) {
    for structured in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit_prepared(f::call(argv), structured, None);
        root.adoption_answer(call, outcome.clone());
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, status);
        let (bytes, empty) =
            if status == 0 { (&printed.stdout, &printed.stderr) } else { (&printed.stderr, &printed.stdout) };
        assert!(empty.is_empty());
        assert_eq!(bytes.as_ref(), if structured { json.as_bytes() } else { text.as_bytes() });
    }
}
#[test]
fn forge_make_sends_each_kind_and_prints_made_with_its_number() {
    for (argv, choice) in [
        (&["forge", "make", "here"][..], w::ForgeChoice::Owned),
        (&["forge", "make", "github", "--key", "make-github"], w::ForgeChoice::Github),
        (&["forge", "make", "forgejo", API, GIT], forgejo(None)),
        (&["forge", "make", "forgejo", API, GIT, "--web", WEB, "--key", "make-forgejo"], forgejo(Some(WEB.as_bytes()))),
    ] {
        let parsed = f::call(argv);
        assert_eq!(f::printed(&parsed), Parsed::Call { call: parsed.clone(), json: false });
        assert_eq!(f::from_json(&jig::json::write_call(crate::table::TOOLS, &parsed).unwrap()), parsed);
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(argv);
        assert!(root.saved.contains_key(&call), "drawn and explicit keys are kept");
        if let Some(name) = &parsed.key {
            assert_eq!(root.sent_name(call), Some(crate::key::engine(name)));
        }
        let w::Request::MakeForge(request) = sent(&root, call) else { panic!("make forge") };
        assert_eq!(request.choice(), &choice);
        root.adoption_answer(call, made());
        assert_eq!(root.answer(call).unwrap().stdout.as_ref(), b"made: forge 27\n");
        check(argv, made(), "made: forge 27\n", r#"{"made":{"forge":27}}"#, 0);
    }
}
#[test]
fn forge_make_refusals_print_and_exit_two() {
    for (outcome, text, json) in [
        (w::Outcome::Full, "refused: full\n", r#"{"full":{}}"#),
        (w::Outcome::NotOffered, "refused: not offered\n", r#"{"not_offered":{}}"#),
    ] {
        check(&["forge", "make", "here"], outcome, text, json, 2);
    }
}
#[test]
fn forge_make_forgejo_refuses_an_empty_address_before_sending() {
    for argv in [
        &["forge", "make", "forgejo", "", GIT][..],
        &["forge", "make", "forgejo", API, ""],
        &["forge", "make", "forgejo", API, GIT, "--web", ""],
    ] {
        for json in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let call = root.submit_prepared(f::call(argv), json, None);
            assert!(root.nothing_sent());
            let printed = root.answer(call).unwrap();
            assert_eq!(printed.status, 1);
            assert!(skein_lib::bytes::find(&printed.stderr, b"invalid forge call").is_some());
        }
    }
}
#[test]
fn forge_remove_prints_removing_and_its_refusals() {
    let argv = &["forge", "remove", "19", "--key", "remove-forge"];
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(argv);
    let w::Request::RemoveForge(request) = sent(&root, call) else { panic!("remove forge") };
    assert_eq!(request.forge(), REMOVE_FORGE);
    assert_eq!(root.sent_name(call), Some(crate::key::engine(&f::name(b"remove-forge"))));
    for (outcome, text, json, status) in [
        (w::Outcome::Removing, "removing: forge 19\n", r#"{"removing":{"forge":19}}"#, 0),
        (
            w::Outcome::ConnectRefused(
                w::ConnectRefused::new(&w::CEILINGS, w::ConnectRefusedParts { why: w::ForgeRefusal::NoSuchForge })
                    .unwrap(),
            ),
            "refused: no such forge\n",
            r#"{"no_such_forge":{}}"#,
            2,
        ),
        (w::Outcome::Playground, "refused: the playground's forge\n", r#"{"playground":{}}"#, 2),
    ] {
        check(argv, outcome, text, json, status);
    }
}
#[test]
fn playground_reset_prints_reset_and_not_offered() {
    let argv = &["playground", "reset", "--key", "reset-playground"];
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(argv);
    assert_eq!(sent(&root, call), w::Request::EndPlayground);
    assert_eq!(root.sent_name(call), Some(crate::key::engine(&f::name(b"reset-playground"))));
    check(argv, w::Outcome::Reset, "reset\n", r#"{"reset":{}}"#, 0);
    check(argv, w::Outcome::NotOffered, "refused: not offered\n", r#"{"not_offered":{}}"#, 2);
}
#[test]
fn a_kept_make_prints_again_with_kept() {
    for (argv, outcome, text, json) in [
        (
            &["forge", "make", "here"][..],
            made(),
            "kept: forge-make made: forge 27\n",
            "{\"operation\":\"forge-make\",\"outcome\":{\"made\":{\"forge\":27}}}\n",
        ),
        (
            &["forge", "remove", "19"],
            w::Outcome::Removing,
            "kept: forge-remove removing: forge 19\n",
            "{\"operation\":\"forge-remove\",\"outcome\":{\"removing\":{\"forge\":19}}}\n",
        ),
        (
            &["playground", "reset"],
            w::Outcome::Reset,
            "kept: playground-reset reset\n",
            "{\"operation\":\"playground-reset\",\"outcome\":{\"reset\":{}}}\n",
        ),
    ] {
        for structured in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let call = root.submit(argv);
            let mut records = f::restored_adoption(f::PARTY);
            let record = records.get_mut(0).unwrap();
            record.key = root.sent_name(call).unwrap();
            record.request.request = root.adoption_bytes(call).clone();
            let mut root = f::started_with(records, f::SEED, structured);
            root.deliver(c::Delivery::Me(Box::new(f::me(f::PARTY))));
            root.adoption_answer(0, outcome.clone());
            let expected = if structured { json } else { text };
            assert_eq!(root.kept_lines.as_slice(), &[Box::<[u8]>::from(expected.as_bytes())]);
        }
    }
}
#[test]
fn unrelated_and_unknown_outcomes_print_unknown() {
    for argv in [&["forge", "make", "here"][..], &["forge", "remove", "19"], &["playground", "reset"]] {
        for outcome in [w::Outcome::Unknown, w::Outcome::Running] {
            check(argv, outcome, "unknown\n", r#"{"unknown":{}}"#, 1);
        }
    }
}
#[test]
fn forge_addresses_and_numbers_obey_their_bounds() {
    let api = "a".repeat(API_BYTES);
    let git = "g".repeat(URL_BYTES);
    let web = "w".repeat(URL_BYTES);
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["forge", "make", "forgejo", &api, &git, "--web", &web]);
    let w::Request::MakeForge(_) = sent(&root, call) else { panic!("make forge") };
    for argv in [&["forge", "make", "forgejo", "", ""][..], &["forge", "remove", "1"], &["forge", "remove", "65535"]] {
        f::parse(argv).expect("within bounds");
    }
    let long_api = "a".repeat(API_BYTES + OVER_BOUND);
    let long_git = "g".repeat(URL_BYTES + OVER_BOUND);
    for argv in [
        &["forge", "make", "forgejo", &long_api, GIT][..],
        &["forge", "make", "forgejo", API, &long_git],
        &["forge", "make", "forgejo", API, GIT, "--web", &long_git],
        &["forge", "remove", "0"],
        &["forge", "remove", "65536"],
        &["forge", "make", "elsewhere"],
        &["forge", "make", "here", API, GIT],
        &["forge", "make", "forgejo", API],
    ] {
        assert!(f::parse(argv).is_err(), "{argv:?}");
    }
}

#[test]
fn entrance_refusals_print_as_other_forge_tools_do() {
    for argv in [&["forge", "make", "here"][..], &["forge", "remove", "19"], &["playground", "reset"]] {
        for structured in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let call = root.submit_prepared(f::call(argv), structured, None);
            root.answered(call, c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None });
            let printed = root.answer(call).unwrap();
            assert_eq!(printed.status, 2);
            assert!(printed.stdout.is_empty());
            let expected = if structured {
                br#"{"entrance_refused":{"refusal":"not_offered"}}"#.as_slice()
            } else {
                b"This act is not offered\n"
            };
            assert_eq!(printed.stderr.as_ref(), expected);
        }
    }
}
