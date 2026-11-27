//! Menu slots at the configuration and answer boundaries (skein/docs/design/testing/fakes/drawn-agent.md, 2).

use alloc::boxed::Box;
use core::mem::size_of;
use skein_lib::bytes::{concat, copy_of};
use skein_lib::{Decimal, Env, Queue, ReplyTo, Time, Token, Wall};

use super::Candidates;
use crate::api::{Answer, Bounds, Error, Failure, InvalidInput, Menu, Message, Part, Query, Role, Scripts, Slot};
use crate::tests::{CONFIG, query, says, script, user};
use crate::{Config, Domain, Event, Request, fire, limits, step};

mod capture;
mod linked;

fn slot(name: &[u8], before: &[u8], after: &[u8], unissued: &[u8]) -> Slot {
    Slot {
        name: copy_of(name),
        before: copy_of(before),
        after: copy_of(after),
        unissued: copy_of(unissued),
        weight: 0,
        with: Box::new([]),
        foreign: 0,
        numeric: None,
    }
}

fn path() -> Slot {
    slot(b"path", b"<", b">", b"missing")
}

fn job() -> Slot {
    Slot { numeric: Some(Bounds { low: 1, high: u64::from(u32::MAX) }), ..slot(b"job", b"job ", b" ", b"0") }
}

fn menu(body: &[u8], slots: Box<[Slot]>) -> Menu {
    Menu { arguments: Box::new([copy_of(body)]), invalid: Box::new([]), slots }
}

fn no_scripts() -> Scripts {
    Scripts { cued: Box::new([]), asked: Box::new([]), otherwise: Box::new([]) }
}

fn text(text: &[u8]) -> Part {
    Part::Text { text: copy_of(text) }
}

fn history(text_bytes: &[u8]) -> Query {
    query(Box::new([user(Box::new([text(text_bytes)]))]))
}

fn tool_call(arguments: &[u8]) -> Part {
    Part::ToolCall { id: copy_of(b"previous"), name: copy_of(b"ls"), arguments: copy_of(arguments) }
}

fn output(output: &[u8]) -> Part {
    Part::ToolOutput { id: copy_of(b"previous"), output: copy_of(output), is_error: false }
}

fn assistant(parts: Box<[Part]>) -> Message {
    Message { role: Role::Assistant, parts }
}

fn call(config: Config, menu: Menu, query: Query, seed: u64) -> Result<Answer, Failure> {
    let mut domain = Domain::played(&config, seed, no_scripts(), menu).expect("valid menu");
    reply(&mut domain, config, query)
}

fn reply(domain: &mut Domain, config: Config, query: Query) -> Result<Answer, Failure> {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config };
    let token = Token::new(1);
    let mut out = Queue::uncapped();
    step(domain, &env, Event::Call { reply_to: ReplyTo::new(token), call: token, query }, &mut out);
    fire(domain, &env, &mut out);
    let Some(Request::Reply { to, result }) = out.pop() else { panic!("one terminal reply") };
    assert_eq!(to, ReplyTo::new(token));
    assert!(out.is_empty());
    domain.reclaim();
    result
}

fn called(part: &Part) -> (&[u8], &[u8], &[u8]) {
    let Part::ToolCall { id, name, arguments } = part else { panic!("a random call") };
    (id, name, arguments)
}

fn arguments(answer: &Answer) -> &[u8] {
    assert_eq!(answer.parts.len(), 1);
    called(answer.parts.first().expect("one call")).2
}

#[expect(clippy::disallowed_methods, reason = "test helper compares the candidate sequence directly")]
fn candidates<'q>(scanned: &Candidates, index: u32, query: &'q Query) -> impl Iterator<Item = &'q [u8]> {
    scanned.slots.get(index).expect("declared slot").iter().map(|place| place.read(query))
}

#[test]
fn numeric_placeholder_in_a_call_is_a_json_number() {
    let menu = menu(br#"{"job":"{{#job}}"}"#, Box::new([job()]));
    let query = query(Box::new([
        user(Box::new([text(b"run")])),
        assistant(Box::new([tool_call(b"{}")])),
        user(Box::new([output(b"job 7 running; use job to read the rest")])),
    ]));
    let answer = call(Config { tool_rounds: 2, ..CONFIG }, menu, query, 1).expect("job call");
    assert_eq!(arguments(&answer), br#"{"job":7}"#);
}

#[test]
fn string_placeholder_escapes_quotes_backslashes_and_controls() {
    let menu = menu(br#"{"path":"{{path}}"}"#, Box::new([path()]));
    let answer = call(CONFIG, menu, history(b"path <a\"b\\c\td>"), 1).expect("path call");
    assert_eq!(arguments(&answer), br#"{"path":"a\"b\\c\u0009d"}"#);
}

#[test]
fn repeated_uses_share_one_draw() {
    let menu = menu(br#"["{{#job}}","{{job}}","{{#job}}"]"#, Box::new([job()]));
    // This seed picks 9, then would pick 7 if a use redrew the candidate.
    let answer = call(CONFIG, menu, history(b"job 7 idle; job 9 running"), 1).expect("job call");
    assert_eq!(arguments(&answer), br#"[9,"9",9]"#);
}

#[test]
fn scan_reads_messages_parts_and_occurrences_backwards_then_system() {
    let mut query = query(Box::new([
        user(Box::new([text(b"<old-part>")])),
        assistant(Box::new([tool_call(b"{}")])),
        user(Box::new([text(b"<new-part>"), output(b"<first-output> <last-output>")])),
    ]));
    query.system = copy_of(b"<system>");
    let scanned = Candidates::scan(&query, &[path()]);
    let expected: [&[u8]; 5] = [b"last-output", b"first-output", b"new-part", b"old-part", b"system"];
    assert!(candidates(&scanned, 0, &query).eq(expected), "the complete scan order");
}

#[test]
fn scan_does_not_read_tool_calls_or_opaque_parts() {
    let query = query(Box::new([
        user(Box::new([text(b"<request>")])),
        assistant(Box::new([tool_call(b"<arguments>")])),
        user(Box::new([output(b"no value"), Part::Opaque { bytes: copy_of(b"<opaque>") }])),
    ]));
    let scanned = Candidates::scan(&query, &[path()]);
    assert!(
        candidates(&scanned, 0, &query).eq([b"request".as_slice()]),
        "neither opaque data nor call arguments are read"
    );
}

#[test]
fn scan_never_spans_two_parts() {
    let query = query(Box::new([user(Box::new([text(b"<valid><cross"), text(b"part>")]))]));
    let scanned = Candidates::scan(&query, &[path()]);
    assert!(candidates(&scanned, 0, &query).eq([b"valid".as_slice()]), "delimiters must be in the same text");
}

#[test]
fn scan_keeps_only_nonempty_single_line_values_through_256_bytes() {
    let longest = [b'x'; 256];
    let too_long = [b'x'; 257];
    let text = concat(&[b"<a><", &longest, b"><", &too_long, b"><line\nfeed><>"]);
    let query = history(&text);
    let scanned = Candidates::scan(&query, &[path()]);
    assert!(candidates(&scanned, 0, &query).eq([longest.as_slice(), b"a"]), "only the two in-bound values");
}

#[test]
fn scan_keeps_only_the_newest_place_of_each_value() {
    let query = history(b"<seen><older><seen><newest>");
    let scanned = Candidates::scan(&query, &[path()]);
    assert!(candidates(&scanned, 0, &query).eq([b"newest".as_slice(), b"seen", b"older"]), "only the newest ‘seen’");
}

#[test]
fn scan_stops_at_the_newest_64_of_70_distinct_values() {
    let mut text_bytes = copy_of(b"");
    for n in 1..=70 {
        text_bytes = concat(&[&text_bytes, b"<", Decimal::of(n).as_bytes(), b">"]);
    }
    let query = history(&text_bytes);
    let scanned = Candidates::scan(&query, &[path()]);
    let mut kept = candidates(&scanned, 0, &query);
    for n in (7..=70).rev() {
        assert_eq!(kept.next(), Some(Decimal::of(n).as_bytes()), "candidate {n}");
    }
    assert_eq!(kept.next(), None, "the six oldest values are excluded");
}

#[test]
fn numeric_candidates_require_canonical_digits_within_bounds() {
    let query = history(b"job 1 started; job 4294967295 running; job 0 old; job 4294967296 old; job 18446744073709551616 bad; job 007 bad; use job to read");
    let scanned = Candidates::scan(&query, &[job()]);
    assert!(candidates(&scanned, 0, &query).eq([b"4294967295".as_slice(), b"1"]), "only the two inclusive bounds");
}

#[test]
fn the_first_declared_slot_takes_an_overlapping_place_in_each_part() {
    let slots = [slot(b"A", b"job ", b" running", b"none"), slot(b"B", b"job ", b" ", b"none")];
    let overlap_only = history(b"job 7 running");
    let scan = Candidates::scan(&overlap_only, &slots);
    assert!(candidates(&scan, 0, &overlap_only).eq([b"7".as_slice()]));
    assert_eq!(candidates(&scan, 1, &overlap_only).next(), None, "B cannot take A's place");

    let different_parts = query(Box::new([user(Box::new([text(b"job 7 running"), text(b"job 7 idle")]))]));
    let scan = Candidates::scan(&different_parts, &slots);
    assert!(
        candidates(&scan, 1, &different_parts).eq([b"7".as_slice()]),
        "B can take the same bytes in a different part"
    );
}

#[test]
fn overlap_includes_the_delimiters() {
    let slots = [path(), slot(b"tail", b">", b"!", b"none")];
    let query = history(b"<path>tail!");
    let scan = Candidates::scan(&query, &slots);
    assert_eq!(candidates(&scan, 1, &query).next(), None, "the slots share the closing ‘>’ byte");
}

#[test]
fn an_empty_scan_draws_the_never_issued_value() {
    let menu = menu(br#"{"path":"{{path}}"}"#, Box::new([path()]));
    let answer = call(CONFIG, menu, history(b"no path"), 1).expect("never-issued path");
    assert_eq!(arguments(&answer), br#"{"path":"missing"}"#);
}

#[test]
fn weight_1000_draws_the_never_issued_value_even_with_candidates() {
    let menu = menu(br#"{"path":"{{path}}"}"#, Box::new([Slot { weight: 1000, ..path() }]));
    let answer = call(CONFIG, menu, history(b"<found>"), 1).expect("never-issued path");
    assert_eq!(arguments(&answer), br#"{"path":"missing"}"#);
}

#[test]
fn weight_zero_draws_a_candidate_when_one_exists() {
    let menu = menu(br#"{"path":"{{path}}"}"#, Box::new([path()]));
    let answer = call(CONFIG, menu, history(b"<found>"), 1).expect("issued path");
    assert_eq!(arguments(&answer), br#"{"path":"found"}"#);
}

fn sized_batch() -> (Config, Menu, Query, Answer, u32) {
    let config = Config { calls_per_answer: 2, ..CONFIG };
    let menu = Menu {
        arguments: Box::new([copy_of(br#"["A","{{long}}","{{short}}"]"#), copy_of(br#"["B","{{long}}","{{short}}"]"#)]),
        invalid: Box::new([]),
        slots: Box::new([slot(b"long", b"<", b">", b"-"), slot(b"short", b"[", b"]", b"none")]),
    };
    let query = history(b"<really\tlong> [x]");
    let full = call(config, menu.clone(), query.clone(), 2).expect("filled batch under wide cap");
    assert_eq!(full.parts.len(), 2, "seed draws two calls");
    let cap = u32::try_from(limits::answer(&full).expect("measured batch")).expect("small fixture");
    (config, menu, query, full, cap)
}

#[test]
fn a_filled_batch_at_the_answer_byte_cap_keeps_its_draws() {
    let (config, menu, query, full, cap) = sized_batch();
    let at_cap = call(Config { answer_bytes: cap, ..config }, menu, query, 2).expect("exactly at cap");
    assert_eq!(at_cap, full);
    assert_eq!(called(at_cap.parts.first().expect("first call")).2, br#"["B","really\u0009long","x"]"#);
}

#[test]
fn one_byte_over_the_cap_uses_all_never_issued_values_without_redrawing_calls() {
    let (config, menu, query, full, cap) = sized_batch();
    let answer_bytes = cap.checked_sub(1).expect("nonempty batch");
    let fallback = call(Config { answer_bytes, ..config }, menu, query, 2).expect("never-issued batch fits");
    assert_eq!(fallback.parts.len(), full.parts.len());
    let expected: [&[u8]; 2] = [br#"["B","-","none"]"#, br#"["A","-","none"]"#];
    for ((unissued, drawn), body) in fallback.parts.iter().zip(&full.parts).zip(expected) {
        let (id, name, arguments) = called(unissued);
        let (drawn_id, drawn_name, _) = called(drawn);
        assert_eq!((id, name), (drawn_id, drawn_name), "same calls");
        assert_eq!(arguments, body, "same body, with every slot at its never-issued value");
    }
}

#[test]
fn oversized_never_issued_batch_fails_even_if_candidates_would_fit() {
    let menu =
        menu(br#"{"path":"{{path}}"}"#, Box::new([Slot { unissued: copy_of(b"much longer fallback"), ..path() }]));
    let unissued = call(CONFIG, menu.clone(), history(b"no path"), 1).expect("fits under a wide cap");
    let bytes = u32::try_from(limits::answer(&unissued).expect("measured answer")).expect("small fixture");
    let answer_bytes = bytes.checked_sub(1).expect("nonempty answer");
    let config = Config { answer_bytes, ..CONFIG };
    let failure = call(config, menu, history(b"<x>"), 1).expect_err("the never-issued form is not admitted");
    assert_eq!(failure.error, Error::ContextTooLong);
}

fn refused(body: &[u8], slots: Box<[Slot]>) {
    assert_eq!(Domain::played(&CONFIG, 1, no_scripts(), menu(body, slots)).err(), Some(Error::InvalidRequest));
}

#[test]
fn a_ninth_slot_is_refused() {
    let slots =
        [b"a", b"b", b"c", b"d", b"e", b"f", b"g", b"h", b"i"].map(|name| Slot { name: copy_of(name), ..path() });
    let eight = slots.get(..8).expect("first eight slots");
    let _domain = Domain::played(&CONFIG, 1, no_scripts(), menu(b"{}", eight.into())).expect("eight slots fit");
    refused(b"{}", Box::new(slots));
}

#[test]
fn duplicate_slot_names_are_refused() {
    refused(b"{}", Box::new([path(), path()]));
}

#[test]
fn malformed_slot_names_are_refused() {
    for name in [b"".as_slice(), &[b'x'; 33], b"dash-name", b"\xff"] {
        let slot = Slot { name: copy_of(name), ..path() };
        refused(b"{}", Box::new([slot]));
    }
}

#[test]
fn slot_markers_and_never_issued_values_reject_lengths_outside_1_to_256() {
    for invalid in [b"".as_slice(), &[b'x'; 257]] {
        refused(b"{}", Box::new([Slot { before: copy_of(invalid), ..path() }]));
        refused(b"{}", Box::new([Slot { unissued: copy_of(invalid), ..path() }]));
    }
    refused(b"{}", Box::new([Slot { after: copy_of(&[b'>'; 257]), ..path() }]));
}

#[test]
fn a_never_issued_value_with_a_line_feed_is_refused() {
    refused(b"{}", Box::new([Slot { unissued: copy_of(b"line\nfeed"), ..path() }]));
}

#[test]
fn a_numeric_never_issued_value_must_be_a_number() {
    refused(b"{}", Box::new([Slot { unissued: copy_of(b"none"), ..job() }]));
}

#[test]
fn a_weight_above_1000_is_refused() {
    refused(b"{}", Box::new([Slot { weight: 1001, ..path() }]));
}

#[test]
fn reversed_numeric_bounds_are_refused() {
    refused(b"{}", Box::new([Slot { numeric: Some(Bounds { low: 2, high: 1 }), ..job() }]));
}

#[test]
fn a_number_placeholder_naming_a_text_slot_is_refused() {
    refused(br#"{"path":"{{#path}}"}"#, Box::new([path()]));
}

#[test]
fn slots_share_the_cap_with_scripts_and_the_rest_of_the_menu() {
    let mut menu = menu(br#"{"path":"{{path}}"}"#, Box::new([path()]));
    menu.invalid = Box::new([InvalidInput { name: None, arguments: copy_of(b"{") }]);
    let scripts = Scripts { cued: Box::new([script(b"cue", Box::new([says(b"scripted")]))]), ..no_scripts() };
    let slot_bytes = u64::try_from(size_of::<Slot>()).expect("wrapper").checked_add(13).expect("path, <, >, missing");
    let cap = limits::scripts(&scripts.cued)
        .expect("scripts")
        .checked_add(limits::menu_inputs(&menu).expect("bodies"))
        .expect("scripts and bodies")
        .checked_add(slot_bytes)
        .expect("with slot");
    let script_bytes = u32::try_from(cap).expect("small fixture");
    let exact = Config { script_bytes, ..CONFIG };
    let _domain = Domain::played(&exact, 1, scripts.clone(), menu.clone()).expect("exact cap fits");
    let short = Config { script_bytes: script_bytes.checked_sub(1).expect("nonempty scripts and menu"), ..CONFIG };
    assert_eq!(Domain::played(&short, 1, scripts, menu).err(), Some(Error::InvalidRequest));
}

#[test]
fn invalid_entries_are_not_interpreted_as_templates() {
    let mut menu = menu(b"{}", Box::new([path()]));
    let invalid = br#"{"unchanged":"{{path}}","invalid":"{{#path}}"}"#;
    menu.invalid = Box::new([InvalidInput { name: None, arguments: copy_of(invalid) }]);
    let answer =
        call(Config { malformed: 1000, ..CONFIG }, menu, history(b"<found>"), 1).expect("literal invalid input");
    assert_eq!(arguments(&answer), invalid);
}

#[test]
fn placeholders_naming_no_slot_remain_literal() {
    let menu = menu(br#"["{{unused}}","{{#other}}","{{path}}"]"#, Box::new([path()]));
    let answer = call(CONFIG, menu, history(b"<found>"), 1).expect("only the known path is filled");
    assert_eq!(arguments(&answer), br#"["{{unused}}","{{#other}}","found"]"#);
}
