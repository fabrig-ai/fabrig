//! The strictest subject wins (temper/docs/design/domain/forge/attention.md, 6).
use crate::pattern::set_bit;
use crate::{Ask, Class, Classes, Reach, Reason, Subject};

pub(crate) fn subject_class(classes: &Classes, subject: Subject, under: u8) -> (Class, u8) {
    match subject {
        Subject::Kind(kind) => {
            if kind.fixed() {
                return (Class::Wait, 0);
            }
        }
        Subject::Set(_number) => {}
    }
    if let Some(index) = under.checked_sub(1)
        && let Some(item) = classes.items.get(usize::from(index))
        && item.subject == subject
        && !item.words.is_empty()
    {
        return (item.class, under);
    }
    (classes.class_of(subject), 0)
}
#[must_use]
pub fn judge(classes: &Classes, ask: &Ask<'_>) -> Reach {
    let (class, under) = subject_class(classes, Subject::Kind(ask.kind), ask.under);
    let mut reach = Reach { kind: ask.kind, class, reason: Reason::Kind, under, sets: 0, matched: 0, away: ask.away };
    for set in &classes.sets {
        let matched = set.matched(ask.paths);
        if matched == 0 {
            continue;
        }
        reach.sets |= set_bit(set.number);
        let (class, under) = subject_class(classes, Subject::Set(set.number), ask.under);
        if under != 0 {
            reach.under = under;
        }
        let lower = match reach.reason {
            Reason::Set(number) => set.number < number,
            Reason::Kind | Reason::Always => false,
        };
        if class.strictness() > reach.class.strictness() || (class == reach.class && lower) {
            reach.class = class;
            reach.reason = Reason::Set(set.number);
            reach.matched = matched;
        }
    }
    if ask.always && reach.class.strictness() < Class::AskAway.strictness() {
        reach.class = Class::AskAway;
        reach.reason = Reason::Always;
        reach.matched = 0;
    }
    reach
}
impl Reach {
    #[must_use]
    pub const fn decided(&self) -> bool {
        match self.class {
            Class::Decide => true,
            Class::Ask => self.away,
            Class::AskAway | Class::Wait => false,
        }
    }
}
