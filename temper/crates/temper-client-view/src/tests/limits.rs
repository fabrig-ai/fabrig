//! A size-bounded window fits the derived page bounds (client/README.md, 5.4).
use super::fixtures as f;
use jig_client::{contract as c, thread::Thread};
use skein_lib::List;

#[test]
fn a_window_full_of_markdown_builds() {
    // Short list items and quotes stress nodes; links retain labels and targets.
    for pattern in [b"- x\n".as_slice(), b"> x\n", b"[x](https://x) "] {
        full_window(pattern, 4096);
    }
}

#[test]
fn a_window_full_of_small_lines_builds() {
    full_window(b"x", 4);
}

fn full_window(pattern: &[u8], turn_text: u32) {
    let mut domain = f::domain_limits();
    domain.client.domain.turn_text = turn_text;
    let rows = domain.client.domain.rows;
    let blocks = rows * rows;
    // Each turn fills every message and block, with a shared text budget.
    let weight = u64::from(c::LINE_ROOM) * u64::from(1 + rows + blocks) + u64::from(turn_text);
    let count = u32::try_from(1_048_576 / weight).unwrap();
    domain.client.domain.document_bytes = u64::from(count) * weight;
    let mut thread_limits = temper_client_domain::pages::chat::thread_limits(&domain);
    thread_limits.lines = u32::try_from(thread_limits.bytes / u64::from(c::LINE_ROOM)).unwrap();
    let mut thread = Thread::new(1, &thread_limits);
    let mut activity = List::with_capacity(count);
    for position in 1..=count {
        let mut messages = List::with_capacity(rows);
        for message in 0..rows {
            let mut kept = List::with_capacity(rows);
            for block in 0..rows {
                let index = message * rows + block;
                let length = turn_text / blocks + u32::from(index < turn_text % blocks);
                let input: Vec<u8> = pattern.iter().copied().cycle().take(usize::try_from(length).unwrap()).collect();
                kept.push(c::Block::Text(c::Text::new(input.into_boxed_slice(), turn_text).unwrap())).unwrap();
            }
            messages.push(c::Message { speaker: c::Speaker::Assistant, blocks: kept }).unwrap();
        }
        let mut line = f::first_run::turn(u64::from(position), 1, position, &[], turn_text);
        let c::TurnRead::Decoded(turn) = &mut line.decoded.as_mut().unwrap().turn else { panic!("decoded turn") };
        turn.messages = messages;
        assert_eq!(line.weight(), weight);
        activity.push(line).unwrap();
    }
    assert_eq!(activity.as_slice().iter().map(c::DecodedActivityLine::weight).sum::<u64>(), thread_limits.bytes);
    thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
    thread.page(c::ActivityPage { rows: activity, before: None, after: None, cut: false });
    assert_eq!(thread.lines().len(), count, "every admitted line is drawn");
    let words = f::words();
    let bounds = crate::Limits::of(&domain, &words).unwrap();
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
    assert!(tree.nodes().iter().map(skein_ui::Node::text_bytes).sum::<u32>() <= bounds.tree.text_bytes);
}
