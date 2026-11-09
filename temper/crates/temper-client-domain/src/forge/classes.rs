//! Settings' view of a project's classes (temper/docs/design/client/README.md, 5.13, While you're away).
use alloc::boxed::Box;
use skein_lib::List;
use temper_protocol_reaches::{Class, Classes, Item, Kind, Subject};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Asking {
    pub subject: Subject,
    /// Its place from 1, or 0 for a kind with no item of its own.
    pub item: u8,
    /// Whether Away may decide it: its class is Ask.
    pub away: bool,
}

#[must_use]
pub fn asking(classes: &Classes) -> Box<[Asking]> {
    let mut asking = List::uncapped();
    for kind in Kind::ALL {
        if kind.fixed() {
            continue;
        }
        let subject = Subject::Kind(kind);
        let mut own = None;
        for (index, item) in classes.items.iter().enumerate() {
            if item.subject == subject && item.words.is_empty() {
                own = Some((index, item));
                break;
            }
        }
        match own {
            Some((index, item)) => push(&mut asking, subject, index, item.class),
            None => asking.push(Asking { subject, item: 0, away: false }).expect("settings items grow"),
        }
    }
    for (index, item) in classes.items.iter().enumerate() {
        match item.subject {
            Subject::Kind(_) if item.words.is_empty() => {}
            Subject::Kind(_) | Subject::Set(_) => push(&mut asking, item.subject, index, item.class),
        }
    }
    asking.into_boxed()
}
fn push(asking: &mut List<Asking>, subject: Subject, index: usize, class: Class) {
    match class {
        Class::Ask | Class::AskAway => asking
            .push(Asking {
                subject,
                item: u8::try_from(index.saturating_add(1)).expect("checked table's item fits"),
                away: class == Class::Ask,
            })
            .expect("settings items grow"),
        Class::Decide | Class::Wait => {}
    }
}

#[must_use]
pub fn turned(classes: &Classes, asking: Asking) -> Classes {
    let mut items = List::uncapped();
    for item in &classes.items {
        items.push(item.clone()).expect("settings items grow");
    }
    if asking.item == 0 {
        items.push(Item { subject: asking.subject, words: Box::new([]), class: Class::Ask }).expect("new kind fits");
    } else if let Some(item) = items.get_mut(u32::from(asking.item).saturating_sub(1)) {
        item.class = if asking.away { Class::AskAway } else { Class::Ask };
    }
    Classes { items: items.into_boxed(), sets: classes.sets.clone() }
}
