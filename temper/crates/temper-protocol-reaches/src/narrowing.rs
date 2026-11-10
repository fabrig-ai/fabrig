//! Stop asking's proposed tables (temper/docs/design/domain/forge/attention.md, 6).
use crate::pattern::pattern_bit;
use crate::{
    Class, Classes, ITEMS, Item, NAME_BYTES, Offer, PathSet, Reach, Reason, Refusal, SETS, Subject, WORDS_BYTES,
};
use alloc::boxed::Box;
use skein_lib::List;

fn item_place(classes: &Classes, reach: Reach, subject: Subject) -> Option<usize> {
    if let Some(index) = reach.under.checked_sub(1)
        && let Some(item) = classes.items.get(usize::from(index))
        && item.subject == subject
        && !item.words.is_empty()
    {
        return Some(usize::from(index));
    }
    for (index, item) in classes.items.iter().enumerate() {
        if item.subject == subject && item.words.is_empty() {
            return Some(index);
        }
    }
    None
}
#[expect(clippy::manual_find, reason = "step code walks free numbers without closures")]
fn free_number(classes: &Classes) -> Option<u8> {
    for number in 1..=u8::try_from(SETS).expect("six sets") {
        if classes.set(number).is_none() {
            return Some(number);
        }
    }
    None
}
#[must_use]
pub fn offers(classes: &Classes, reach: &Reach) -> Box<[Offer]> {
    let mut result = List::with_capacity(3);
    if reach.kind.fixed() || reach.decided() {
        return result.into_boxed();
    }
    match reach.reason {
        Reason::Always => {}
        Reason::Kind => {
            let subject = Subject::Kind(reach.kind);
            if item_place(classes, *reach, subject).is_some() || classes.items.len() < ITEMS {
                result.push(Offer::Decide(subject)).expect("one offer");
            }
        }
        Reason::Set(number) => {
            if classes.items.len() < ITEMS {
                result.push(Offer::Words(number)).expect("words offer");
                if let Some(set) = classes.set(number) {
                    let mut selected = 0_usize;
                    for index in 0..set.patterns.len() {
                        if reach.matched & pattern_bit(index) != 0 {
                            selected = selected.saturating_add(1);
                        }
                    }
                    if selected > 0
                        && selected < set.patterns.len()
                        && classes.sets.len() < SETS
                        && free_number(classes).is_some()
                    {
                        result.push(Offer::Split(number)).expect("split offer");
                    }
                }
            }
        }
    }
    result.into_boxed()
}
pub fn narrowed(classes: &Classes, reach: &Reach, offer: Offer, words: &[u8]) -> Result<Classes, Refusal> {
    classes.check()?;
    let mut items = List::with_capacity(u32::try_from(ITEMS.saturating_add(1)).expect("bounded items"));
    let mut sets = List::with_capacity(u32::try_from(SETS.saturating_add(1)).expect("bounded sets"));
    for item in &classes.items {
        items.push(item.clone()).expect("checked table");
    }
    for set in &classes.sets {
        sets.push(set.clone()).expect("checked table");
    }
    match offer {
        Offer::Decide(subject) => {
            if let Some(index) = item_place(classes, *reach, subject) {
                items.get_mut(u32::try_from(index).expect("checked item")).expect("existing item").class =
                    Class::Decide;
            } else {
                items.push(Item { subject, words: Box::default(), class: Class::Decide }).expect("one additional item");
            }
        }
        Offer::Words(number) => {
            let item = u8::try_from(classes.items.len().saturating_add(1)).expect("checked item count");
            if words.is_empty() || words.len() > WORDS_BYTES {
                return Err(Refusal::Words { item });
            }
            items
                .push(Item { subject: Subject::Set(number), words: words.into(), class: Class::Decide })
                .expect("one additional item");
        }
        Offer::Split(number) => {
            let place = u8::try_from(classes.sets.len().saturating_add(1)).expect("checked set count");
            let Some(new_number) = free_number(classes) else {
                return Err(Refusal::Sets);
            };
            let mut moved = List::with_capacity(6);
            let mut name = Box::default();
            let mut found = false;
            for set in sets.iter_mut() {
                if set.number != number {
                    continue;
                }
                found = true;
                name = split_name(&set.name);
                let mut kept = List::with_capacity(6);
                for (index, pattern) in set.patterns.iter().enumerate() {
                    if reach.matched & pattern_bit(index) != 0 {
                        moved.push(pattern.clone()).expect("checked patterns");
                    } else {
                        kept.push(pattern.clone()).expect("checked patterns");
                    }
                }
                set.patterns = kept.into_boxed();
            }
            if !found {
                return Err(Refusal::Number { set: place });
            }
            sets.push(PathSet { number: new_number, name, patterns: moved.into_boxed() }).expect("one additional set");
            items
                .push(Item { subject: Subject::Set(new_number), words: Box::default(), class: Class::Decide })
                .expect("one additional item");
        }
    }
    let result = Classes { items: items.into_boxed(), sets: sets.into_boxed() };
    result.check()?;
    Ok(result)
}
fn split_name(name: &[u8]) -> Box<[u8]> {
    let mut bytes = List::with_capacity(u32::try_from(NAME_BYTES.saturating_add(9)).expect("bounded name"));
    for byte in name {
        bytes.push(*byte).expect("checked name");
    }
    for byte in b", in part" {
        bytes.push(*byte).expect("suffix room");
    }
    let mut end = NAME_BYTES.min(usize::try_from(bytes.len()).expect("bounded name"));
    // UTF-8 continuation bytes cannot begin the omitted suffix.
    for _byte in 0..NAME_BYTES {
        match bytes.as_slice().get(end) {
            Some(byte) => {
                if byte & 0xc0 == 0x80 {
                    end = end.saturating_sub(1);
                } else {
                    break;
                }
            }
            None => break,
        }
    }
    bytes.as_slice().get(..end).expect("character boundary within name").into()
}
