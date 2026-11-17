//! Agents setting rules (temper/docs/design/protocol/configuration/figures.md, 5).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary Rust fixtures compare smith declarations and bounded refusal words"
)]
use super::{BOUNDS, Item, Value, judge};
use crate::settings::{self as s, agents::PROFILE_FIGURES};
use skein_lib::{List, Queue};
use temper_client_protocol_wire::v1 as w;

fn figures(profile: &[(&[u8], u64)], endpoints: &[&[u8]], limits: w::AgentLimitsParts) -> w::AgentFigures {
    let mut rows = List::with_capacity(128);
    for &(name, value) in profile {
        rows.push(w::AgentFigure::new(&w::CEILINGS, w::AgentFigureParts { name: name.into(), value }).unwrap())
            .unwrap();
    }
    let limits = w::AgentLimits::new(&w::CEILINGS, limits).unwrap();
    let mut ends = List::with_capacity(64);
    for &endpoint in endpoints {
        ends.push(
            w::AgentEndpointLimits::new(
                &w::CEILINGS,
                w::AgentEndpointLimitsParts { endpoint: endpoint.into(), limits: limits.clone() },
            )
            .unwrap(),
        )
        .unwrap();
    }
    w::AgentFigures::new(&w::CEILINGS, w::AgentFiguresParts { profile: rows, endpoints: ends, fallback: limits })
        .unwrap()
}
fn limits() -> w::AgentLimitsParts {
    w::AgentLimitsParts {
        window: 1,
        output: 1,
        reasoning_item: 1,
        head_ms: 1,
        idle_ms: 1,
        oversized: w::Oversized::Fail,
    }
}
fn item(value: w::AgentFigures) -> Item {
    Item { kind: 21, scope: 0, number: 1, value: Some(Value::AgentFigures(value)) }
}
#[test]
fn agent_figures_judge_names_each_invalid_item() {
    assert_eq!(s::Kind::of(21), Some(s::Kind::Agents));
    assert_eq!(s::Kind::Agents.number(), 21);
    for &(name, maximum) in PROFILE_FIGURES {
        assert_eq!(judge(&item(figures(&[(name, maximum)], &[b"openai"], limits())), &BOUNDS), Ok(()));
        if maximum < u64::MAX {
            let refusal = judge(&item(figures(&[(name, maximum + 1)], &[], limits())), &BOUNDS).unwrap_err();
            assert!(refusal.words().windows(name.len()).any(|word| word == name));
        }
    }
    for (profile, endpoints, expected) in [
        (vec![(b"policy.max_turnz".as_slice(), 900)], vec![], "policy.max_turnz"),
        (vec![(b"policy.compaction_threshold".as_slice(), 1)], vec![], "policy.compaction_threshold"),
        (vec![(b"policy.max_turns".as_slice(), 1); 2], vec![], "policy.max_turns"),
        (vec![], vec![b"openai".as_slice(); 2], "openai"),
    ] {
        let refusal = judge(&item(figures(&profile, &endpoints, limits())), &BOUNDS).unwrap_err();
        let words = core::str::from_utf8(&refusal.words()).unwrap().to_string();
        assert!(words.contains(expected), "{words}");
    }
    for field in ["window", "output", "reasoning_item", "head_ms", "idle_ms"] {
        let mut bad = limits();
        match field {
            "window" => bad.window = 0,
            "output" => bad.output = 0,
            "reasoning_item" => bad.reasoning_item = 0,
            "head_ms" => bad.head_ms = 0,
            "idle_ms" => bad.idle_ms = 0,
            _ => unreachable!(),
        }
        for endpoints in [&[][..], &[b"openai".as_slice()][..]] {
            let refusal = judge(&item(figures(&[], endpoints, bad.clone())), &BOUNDS).unwrap_err();
            let words = String::from_utf8(refusal.words().into_vec()).unwrap();
            assert!(words.contains(field), "{words}");
            assert!(words.contains(if endpoints.is_empty() { "fallback" } else { "openai" }), "{words}");
        }
    }
    let base = item(figures(&[], &[], limits()));
    assert_eq!(judge(&Item { scope: 1, ..base.clone() }, &BOUNDS), Err(s::Refusal::Scope));
    assert_eq!(judge(&Item { number: 2, ..base }, &BOUNDS), Err(s::Refusal::Number));
}
#[test]
fn profile_figure_names_follow_smiths_integer_overrides() {
    let document = include_str!("../../../../../smith/crates/smith-agent-document/src/config/document.rs");
    let mut expected = Vec::new();
    for (group, record) in [("declared", "DeclaredOverrides"), ("policy", "PolicyOverrides")] {
        let body = document.split_once(&format!("struct {record} {{")).unwrap().1.split_once('}').unwrap().0;
        for line in body.lines() {
            if let Some(line) = line.trim().strip_prefix("pub(super) ") {
                let (name, ty) = line.split_once(": ").unwrap();
                if ty == "Option<u32>," || ty == "Option<u64>," {
                    expected.push((
                        format!("{group}.{name}"),
                        if ty == "Option<u32>," { u64::from(u32::MAX) } else { u64::MAX },
                    ));
                }
            }
        }
    }
    let mut actual: Vec<_> =
        PROFILE_FIGURES.iter().map(|&(name, maximum)| (String::from_utf8(name.to_vec()).unwrap(), maximum)).collect();
    actual.sort();
    expected.sort();
    assert_eq!(actual, expected);
}
#[test]
fn agent_figures_read_defaults_and_changes_and_resets() {
    let default = figures(&[(b"policy.max_turns", 800)], &[b"openai"], limits());
    let changed = figures(&[(b"policy.max_turns", 900)], &[], limits());
    let mut domain = super::super::domain();
    domain.give_settings_defaults(Box::new([s::Stock {
        kind: 21,
        scope: 0,
        number: 1,
        value: s::SizedValue { size: default.measure(), value: Value::AgentFigures(default.clone()) },
    }]));
    assert_eq!(domain.agent_figures(), &default);
    let mut out = Queue::uncapped();
    for (revision, edit, expected) in [
        (
            2,
            s::Edit::Set(s::SizedValue { size: changed.measure(), value: Value::AgentFigures(changed.clone()) }),
            &changed,
        ),
        (3, s::Edit::Remove, &default),
        (4, s::Edit::Reset, &default),
    ] {
        let asked = s::Asked { kind: 21, scope: 0, number: 1, edit };
        assert!(matches!(domain.judge_settings(&BOUNDS, core::slice::from_ref(&asked))[0], s::Judged::Accept { .. }));
        domain.keep_settings(&super::super::env(), revision, None, &[asked], &mut out);
        assert_eq!(domain.agent_figures(), expected);
    }
}
