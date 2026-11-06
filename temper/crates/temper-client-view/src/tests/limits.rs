//! A size-bounded window fits the derived page bounds (client/README.md, 5.4).
use super::fixtures as f;
use jig_client::{contract as c, thread::Thread};
use skein_lib::{List, Writer};

#[test]
fn a_window_full_of_markdown_builds() {
    // Empty three-cell table rows stress nodes; plain text is copied into
    // names and runs, and links retain labels and targets.
    for (prefix, pattern) in
        [(b"|||\n|-|-|\n".as_slice(), b"||||\n".as_slice()), (b"", b"x"), (b"", b"[x](https://x) ")]
    {
        full_window(prefix, pattern, 4096);
    }
}

#[test]
fn a_window_full_of_small_lines_builds() {
    full_window(b"", b"x", 4);
}

#[test]
fn a_window_full_of_calls_builds() {
    let tool = b"read_docs";
    let prefix = br#"{"topic":""#;
    let suffix = br#"","from":18446744073709551615}"#;
    let mut input = Writer::new(prefix.len().checked_add(240).unwrap().checked_add(suffix.len()).unwrap());
    input.put(prefix).unwrap();
    input.put(&[b'x'; 240]).unwrap();
    input.put(suffix).unwrap();
    let input = input.finish();
    let blocks = [
        f::first_run::call(1, tool, &input, Some(&[b'x'; 160])),
        f::first_run::call(2, b"settings", br#"{"setting":{"kind":"project","number":18446744073709551615}}"#, None),
        f::first_run::call(3, tool, &input, None),
        c::Block::Text(f::first_run::text(b"x")),
    ];
    let mut messages = List::with_capacity(2);
    for blocks in blocks.chunks(2) {
        messages.push(f::first_run::message(c::Speaker::Assistant, blocks)).unwrap();
    }
    build_window(messages, 4096);
}

fn full_window(prefix: &[u8], pattern: &[u8], turn_text: u32) {
    let rows = f::domain_limits().client.domain.rows;
    let blocks = rows.checked_mul(rows).unwrap();
    let mut messages = List::with_capacity(rows);
    for message in 0..rows {
        let mut kept = List::with_capacity(rows);
        for block in 0..rows {
            let index = message.checked_mul(rows).unwrap().checked_add(block).unwrap();
            let length = turn_text
                .checked_div(blocks)
                .unwrap()
                .checked_add(u32::from(index < turn_text.checked_rem(blocks).unwrap()))
                .unwrap();
            let mut input = Writer::new(usize::try_from(length).unwrap());
            for byte in
                prefix.iter().copied().chain(pattern.iter().copied().cycle()).take(usize::try_from(length).unwrap())
            {
                input.put(&[byte]).unwrap();
            }
            kept.push(c::Block::Text(c::Text::new(input.finish(), turn_text).unwrap())).unwrap();
        }
        messages.push(c::Message { speaker: c::Speaker::Assistant, blocks: kept }).unwrap();
    }
    build_window(messages, turn_text);
}

fn build_window(messages: List<c::Message>, turn_text: u32) {
    let mut domain = f::domain_limits();
    domain.client.domain.turn_text = turn_text;
    let rows = domain.client.domain.rows;
    let mut line = f::first_run::turn(1, 1, 1, &[], turn_text);
    let c::TurnRead::Decoded(turn) = &mut line.decoded.as_mut().unwrap().turn else { panic!("decoded turn") };
    turn.messages = messages;
    let weight = line.weight();
    let count = u32::try_from(1_048_576_u64.checked_div(weight).unwrap()).unwrap();
    domain.client.domain.document_bytes = u64::from(count).checked_mul(weight).unwrap();
    let mut thread_limits = temper_client_domain::pages::chat::thread_limits(&domain);
    thread_limits.lines = u32::try_from(thread_limits.bytes.checked_div(u64::from(c::LINE_ROOM)).unwrap()).unwrap();
    let mut thread = Thread::new(1, &thread_limits);
    let mut activity = List::with_capacity(count);
    let mut weight_sum = 0_u64;
    for position in 1..=count {
        let mut line = line.clone();
        line.source.position = u64::from(position);
        line.source.line = c::Activity::Turn { reply: u64::from(position), attempt: 1, turn: position, read: None };
        let decoded = line.decoded.as_mut().unwrap();
        decoded.name.number = position;
        decoded.cumulative = u64::from(position);
        assert_eq!(line.weight(), weight);
        weight_sum = weight_sum.checked_add(line.weight()).unwrap();
        activity.push(line).unwrap();
    }
    assert_eq!(weight_sum, thread_limits.bytes);
    thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
    thread.page(c::ActivityPage { rows: activity, before: None, after: None, cut: false });
    assert_eq!(thread.lines().len(), count, "every admitted line is drawn");
    let words = f::words();
    let bounds = crate::Limits::of(&domain, &words).unwrap();
    assert!(crate::valid(&bounds, &domain, &words));
    let mut builder = skein_ui::Builder::new(&bounds.tree);
    crate::pages::chat::conversation(
        &mut builder,
        &thread,
        &words,
        (rows, domain.client.domain.text),
        crate::pages::chat::Speakers { person: None, requester: false },
        crate::pages::chat::Drawn::Chat { request: None, delegations: None },
        &mut crate::frame::Bindings::new(bounds.tree.bindings),
    );
    let tree = builder.finish();
    assert!(tree.len() <= bounds.tree.nodes);
    let mut text_bytes = 0_u32;
    for node in tree.nodes() {
        text_bytes = text_bytes.checked_add(node.text_bytes()).unwrap();
    }
    assert!(text_bytes <= bounds.tree.text_bytes);
}
