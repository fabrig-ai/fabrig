//! Kept-version coverage and dispatch use the same contract seam.
#![expect(clippy::disallowed_macros, reason = "tests format file paths and expected version diagnostics")]

use super::{Adapter, CONTRACT, Contract, KEPT, Mapped};
use crate::{Decoded, Problem, answers::Text, wire};
use smith_domain_run as run;
use temper_engine_domain as engine;

const OLD: &str = concat!(
    r#"{"name":"legacy_docs","effect":"read","input":"{\"required\":[\"words\"]}"}"#,
    "\n",
    r#"{"name":"read_docs","effect":"read","input":"{\"required\":[\"search\"]}"}"#,
    "\n",
    r#"{"name":"release","effect":"write","input":"{\"required\":[\"task\"]}"}"#,
    "\n",
);
const CURRENT: &str = concat!(
    r#"{"name":"read_docs","effect":"read","input":"{\"required\":[\"search\"]}"}"#,
    "\n",
    r#"{"name":"release","effect":"write","input":"{\"required\":[\"target\",\"revision\"]}"}"#,
    "\n",
);
const TWO: Contract = Contract {
    floor: 1,
    current: 2,
    kept: &[OLD, CURRENT],
    adapters: &[
        Adapter { version: 1, tool: b"legacy_docs", map: docs },
        Adapter { version: 1, tool: b"release", map: release },
    ],
};
const NAME: run::CallName = run::CallName { activation: 7, completion: 2, position: 1 };

fn docs(input: &run::HostInput) -> Result<Mapped, Problem> {
    let words = crate::json::parse(input.bytes())?.required(b"words")?.text(b"words")?;
    assert_eq!(words.as_ref(), b"WireGuard");
    Ok(Mapped {
        tool: b"read_docs".as_slice().into(),
        input: run::HostInput::attested(br#"{"search":"WireGuard"}"#.as_slice().into()).unwrap(),
    })
}

fn release(input: &run::HostInput) -> Result<Mapped, Problem> {
    let value = crate::json::parse(input.bytes())?;
    let mut text = Text::new();
    text.add(br#"{"target":"#)?;
    text.number(value.required(b"task")?.number(b"task")?)?;
    text.add(br#", "revision":1}"#)?;
    let (bytes, _) = text.finish(false)?.into_parts();
    Ok(Mapped { tool: b"release".as_slice().into(), input: run::HostInput::attested(bytes).unwrap() })
}

fn decode(contract: &Contract, version: u16, tool: &[u8], input: &[u8]) -> Result<Decoded, Problem> {
    let input = run::HostInput::attested(input.into()).unwrap();
    contract.call(version, NAME, tool, &input, &[], &[], &[])
}

fn answered(decoded: Decoded) -> (Box<[u8]>, bool) {
    let Decoded::Answered(answer) = decoded else { panic!("answered at decode") };
    answer.into_parts()
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the test prints a contract-coverage failure as UTF-8 text")]
fn kept_contract_floor_and_adapters_match_the_files() {
    let directory = skein_scratch::paths::package().join("contract");
    let floor = std::fs::read_to_string(directory.join("floor")).unwrap();
    assert_eq!(floor, std::format!("{}\n", wire::FLOOR));
    let number: u16 = floor.trim().parse().unwrap();
    assert!((1..=wire::VERSION).contains(&number));
    assert_eq!(KEPT.len(), usize::from(wire::VERSION));
    for version in 1..=wire::VERSION {
        let text = std::fs::read_to_string(directory.join(std::format!("host-tools-v{version}.txt"))).unwrap();
        assert_eq!(CONTRACT.text(version), text);
    }
    if let Err(why) = CONTRACT.check() {
        panic!("{}", core::str::from_utf8(&why).unwrap());
    }
}

#[test]
fn changed_or_removed_declarations_require_their_versioned_adapters() {
    let mut contract = Contract { adapters: &[], ..TWO };
    let why = contract.check().unwrap_err();
    assert_eq!(
        why.as_ref(),
        b"host tool legacy_docs in version 1 needs an adapter, or the floor raised by a breaking move"
    );
    contract.adapters = &[Adapter { version: 2, tool: b"legacy_docs", map: docs }];
    assert_eq!(contract.check().unwrap_err(), why, "an adapter of another version does not cover it");
    contract.adapters = &[Adapter { version: 1, tool: b"legacy_docs", map: docs }];
    assert_eq!(
        contract.check().unwrap_err().as_ref(),
        b"host tool release in version 1 needs an adapter, or the floor raised by a breaking move"
    );
    assert_eq!(TWO.check(), Ok(()));
    contract.floor = 2;
    assert_eq!(contract.check(), Ok(()), "a breaking floor retires the old declarations");
}

#[test]
fn calls_choose_current_unchanged_or_adapted_decoding() {
    let input = run::HostInput::attested(br#"{"search":"WireGuard"}"#.as_slice().into()).unwrap();
    let today = answered(crate::call_in(NAME, b"read_docs", &input, &[], &[], &[]).unwrap());
    assert!(!today.1);
    assert!(!today.0.is_empty());
    assert_eq!(answered(decode(&TWO, 2, b"read_docs", input.bytes()).unwrap()), today);
    assert_eq!(answered(decode(&TWO, 1, b"read_docs", input.bytes()).unwrap()), today);
    assert_eq!(answered(decode(&TWO, 1, b"legacy_docs", br#"{"words":"WireGuard"}"#).unwrap()), today);
    let Decoded::Call(call) = decode(&TWO, 1, b"release", br#"{"task":5}"#).unwrap() else {
        panic!("mapped call decoded by the current tool")
    };
    assert_eq!(
        call,
        engine::Call {
            completion: 2,
            position: 1,
            tool: engine::Tool::Release { target: 5, revision: 1, again: None }
        }
    );
    assert_eq!(decode(&TWO, 2, b"legacy_docs", br#"{"words":"WireGuard"}"#).unwrap_err(), Problem::UnknownTool);
    assert_eq!(decode(&TWO, 1, b"new_tool", b"{}").unwrap_err(), Problem::UnknownTool);
}

#[test]
fn versions_outside_the_answered_range_are_error_answers() {
    for version in [0, 3] {
        let (body, error) = answered(decode(&TWO, version, b"release", b"{}").unwrap());
        assert!(error);
        assert_eq!(
            body.as_ref(),
            std::format!("this run's host tools are version {version}; this engine answers 1..2").as_bytes()
        );
    }
    let raised = Contract { floor: 2, ..TWO };
    let (body, error) = answered(decode(&raised, 1, b"legacy_docs", b"{}").unwrap());
    assert!(error);
    assert_eq!(body.as_ref(), b"this run's host tools are version 1; this engine answers 2..2");
}
