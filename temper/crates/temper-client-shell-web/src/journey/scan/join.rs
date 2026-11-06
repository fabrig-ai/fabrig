//! Offset joins of `RunText` and `WritingCall` (temper/docs/design/client/journeys.md, 3.7).
use super::{Originals, Streams, What, bytes};
use jig_client_protocol_wire as w;
use skein_lib::Reader;

#[derive(PartialEq, Eq)]
struct Key {
    watch: u64,
    attempt: u64,
    turn: w::ProvisionalTurnName,
    call: Option<Box<[u8]>>,
}
struct Delivery {
    line: u32,
    from: u32,
    through: u32,
    text: Box<[u8]>,
}
struct Group {
    key: Key,
    deliveries: Vec<Delivery>,
}
pub(super) fn scan(streams: &Streams, originals: &Originals) -> Option<(u32, What)> {
    let mut groups = Vec::new();
    for event in streams.events() {
        let mut reader = Reader::new(&event.record);
        let Ok(decoded) = w::StreamEvent::decode(&w::CEILINGS, &mut reader) else {
            continue;
        };
        if !reader.is_empty() {
            continue;
        }
        match decoded {
            w::StreamEvent::Text(text) => {
                let run = text.value();
                add(
                    &mut groups,
                    Key { watch: text.watch(), attempt: run.attempt(), turn: run.turn().clone(), call: None },
                    Delivery { line: event.line, from: run.from(), through: run.through(), text: run.text().into() },
                );
                if let Some(call) = run.writing() {
                    add(
                        &mut groups,
                        Key {
                            watch: text.watch(),
                            attempt: run.attempt(),
                            turn: run.turn().clone(),
                            call: Some(call.id().into()),
                        },
                        Delivery {
                            line: event.line,
                            from: call.from(),
                            through: call.through(),
                            text: call.input().into(),
                        },
                    );
                }
            }
            w::StreamEvent::Hello(_) | w::StreamEvent::Delivery(_) | w::StreamEvent::Ended(_) => {}
        }
    }
    for group in &mut groups {
        group.deliveries.sort_by_key(|delivery| delivery.from);
        let mut segment = Vec::new();
        let mut through = 0;
        let mut line = 0;
        for delivery in &group.deliveries {
            if segment.is_empty() || delivery.from > through {
                if let Some(what) = bytes(&segment, originals) {
                    return Some((line, what));
                }
                segment.clear();
                segment.extend_from_slice(&delivery.text);
                through = delivery.through;
                line = delivery.line;
            } else if delivery.through > through {
                let overlap = usize::try_from(through - delivery.from).expect("delivery offset fits usize");
                segment.extend_from_slice(&delivery.text[overlap..]);
                through = delivery.through;
            }
        }
        if let Some(what) = bytes(&segment, originals) {
            return Some((line, what));
        }
    }
    None
}
fn add(groups: &mut Vec<Group>, key: Key, delivery: Delivery) {
    if delivery.through.checked_sub(delivery.from).and_then(|length| usize::try_from(length).ok())
        != Some(delivery.text.len())
    {
        return;
    }
    if let Some(group) = groups.iter_mut().find(|group| group.key == key) {
        group.deliveries.push(delivery);
    } else {
        groups.push(Group { key, deliveries: vec![delivery] });
    }
}
