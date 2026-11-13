//! A change waiting on a forge review (temper/docs/design/client/forge/README.md, 4.1).
use super::ask;
use crate::common::approval::forge;
pub(crate) use crate::common::approval::{address, question, review_words};
use crate::{
    bindings::Bindings,
    forge_sign_in::Forges,
    look,
    words::{TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::concat};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address,
    forge::approvals::{ApprovalPerson, Row, Yours},
    pages::Act,
};

pub(crate) fn viewer(root: &temper_client_domain::Domain) -> u64 {
    match root.pages().me(root.client()) {
        Some(me) => me.party.number,
        None => 0,
    }
}

fn role(row: &Row, plural: bool, w: &Words) -> Box<[u8]> {
    let roles = [
        (Word::ApprovalOwner, Word::ApprovalOwners),
        (Word::ApprovalMaintainer, Word::ApprovalMaintainers),
        (Word::ApprovalMember, Word::ApprovalMembers),
        (Word::ApprovalObserver, Word::ApprovalObservers),
    ];
    match roles.get(usize::try_from(row.role).expect("role fits usize")) {
        Some((one, many)) => w.say(if plural { *many } else { *one }, &[]),
        None => {
            w.say(if plural { Word::ApprovalRoles } else { Word::ApprovalRole }, &[Hole::Number(u64::from(row.role))])
        }
    }
}
pub(crate) fn sentences(row: &Row, me: u64, w: &Words) -> Box<[Box<[u8]>]> {
    let mut lines = List::uncapped();
    let rule = w.say(
        if row.scope == 0 { Word::ApprovalDeploymentRule } else { Word::ApprovalProjectRule },
        &[Hole::Number(u64::from(row.rule))],
    );
    let rule = if row.required == 1 {
        w.say(Word::ApprovalOne, &[Hole::Name(rule), Hole::Name(role(row, false, w))])
    } else {
        w.say(
            Word::ApprovalMany,
            &[Hole::Name(rule), Hole::Number(u64::from(row.required)), Hole::Name(role(row, true, w))],
        )
    };
    lines.push(rule).expect("context fits");
    let mut names = None;
    let mut remaining = row.sets.len();
    for set in &row.sets {
        remaining = remaining.saturating_sub(1);
        let name = if set.name.is_empty() {
            w.say(Word::ApprovalPathSet, &[Hole::Number(u64::from(set.number))])
        } else {
            set.name.clone()
        };
        names = Some(match names {
            None => name,
            Some(before) => w.say(
                if remaining == 0 { Word::ApprovalAnd } else { Word::ApprovalJoin },
                &[Hole::Name(before), Hole::Name(name)],
            ),
        });
    }
    if let Some(names) = names {
        lines.push(w.say(Word::ApprovalSets, &[Hole::Name(names)])).expect("context fits");
    }
    if let Some(who) = who(row, me, w) {
        lines.push(who).expect("context fits");
    }
    if !row.whole {
        lines.push(w.say(Word::ApprovalCut, &[])).expect("context fits");
    }
    let why = match row.yours {
        Yours::Counts => None,
        Yours::NoAccount => Some(w.say(Word::ApprovalNoAccount, &[Hole::Name(forge(row, w))])),
        Yours::TooFew => Some(w.say(Word::ApprovalTooFew, &[])),
        Yours::OtherRole => Some(w.say(Word::ApprovalOtherRole, &[Hole::Name(role(row, false, w))])),
    };
    if let Some(why) = why {
        lines.push(why).expect("context fits");
    }
    lines.into_boxed()
}
/// Who has signed and who can still sign (client/forge/README.md, 4.1).
pub(crate) fn who(row: &Row, me: u64, w: &Words) -> Option<Box<[u8]>> {
    if row.counted == 0 && row.yours == Yours::Counts {
        return None;
    }
    let signed = if row.counted == 0 {
        None
    } else {
        let more = row.counted.saturating_sub(u32::try_from(row.signed.len()).expect("eight signers"));
        let names = people(&row.signed, more, me, false, w)?;
        Some(w.say(
            Word::ApprovalSigned,
            &[Hole::Number(u64::from(row.counted)), Hole::Number(u64::from(row.required)), Hole::Name(names)],
        ))
    };
    let can = if row.can_sign.is_empty() {
        None
    } else {
        let names = people(&row.can_sign, row.can_sign_more, me, true, w)?;
        let ordinals = [
            Word::ApprovalFirst,
            Word::ApprovalSecond,
            Word::ApprovalThird,
            Word::ApprovalFourth,
            Word::ApprovalFifth,
            Word::ApprovalSixth,
            Word::ApprovalSeventh,
            Word::ApprovalEighth,
            Word::ApprovalNinth,
        ];
        let next = if row.required == 1 {
            Word::ApprovalIt
        } else {
            match ordinals.get(usize::try_from(row.counted).expect("u32 fits usize")) {
                Some(word) => *word,
                None => Word::ApprovalNext,
            }
        };
        Some(w.say(Word::ApprovalCanGive, &[Hole::Name(names), Hole::Name(w.say(next, &[]))]))
    };
    match (signed, can) {
        (Some(signed), Some(can)) => Some(w.say(Word::ApprovalWhoBoth, &[Hole::Name(signed), Hole::Name(can)])),
        (Some(part), None) => Some(w.say(Word::ApprovalWhoEnd, &[Hole::Name(part)])),
        (None, Some(mut part)) => {
            if let Some(first) = part.first_mut() {
                first.make_ascii_uppercase();
            }
            Some(w.say(Word::ApprovalWhoEnd, &[Hole::Name(part)]))
        }
        (None, None) => None,
    }
}

fn people(values: &[ApprovalPerson], more: u32, me: u64, can: bool, w: &Words) -> Option<Box<[u8]>> {
    let mut names = List::with_capacity(9);
    for person in values {
        let name = if person.person == me {
            w.say(Word::ApprovalYou, &[])
        } else if person.name.is_empty() {
            w.say(Word::ApprovalSomeone, &[])
        } else {
            person.name.clone()
        };
        names.push(name).expect("eight people and others");
    }
    if more != 0 {
        names
            .push(if more == 1 {
                w.say(Word::ApprovalOneOther, &[])
            } else {
                w.say(Word::ApprovalOthers, &[Hole::Number(u64::from(more))])
            })
            .expect("eight people and others");
    }
    let mut joined = None;
    let mut left = names.len();
    for name in names.into_boxed() {
        left = left.saturating_sub(1);
        joined = Some(match joined {
            None => name,
            Some(before) => w.say(
                if left != 0 {
                    Word::ApprovalJoin
                } else if can {
                    Word::ApprovalOr
                } else {
                    Word::ApprovalAnd
                },
                &[Hole::Name(before), Hole::Name(name)],
            ),
        });
    }
    joined
}

fn size_count(count: u32, cut: bool, w: &Words) -> Box<[u8]> {
    w.say(if cut && count == u32::MAX { Word::CountLimited } else { Word::Count }, &[Hole::Number(u64::from(count))])
}
pub(crate) fn size(b: &mut Builder, row: &Row, w: &Words) {
    let Some(size) = row.size else { return };
    b.open(Role::Slot);
    b.style(look::CHIPS);
    b.open(Role::Slot);
    b.style(look::CHIP);
    b.text(
        Run::Plain,
        &w.say(
            Word::ApprovalSizeLines,
            &[Hole::Name(size_count(size.additions, size.cut, w)), Hole::Name(size_count(size.deletions, size.cut, w))],
        ),
    );
    let files = if size.files == 1 {
        w.say(Word::ApprovalSizeFile, &[])
    } else {
        w.say(Word::ApprovalSizeFiles, &[Hole::Name(size_count(size.files, size.cut, w))])
    };
    ask::text(b, Role::Text, &files, look::SIZE_FILES);
    b.close();
    b.close();
}
pub(crate) fn when(row: &Row, w: &Words, times: &mut Times) -> Box<[u8]> {
    concat(&[
        &w.say(Word::ApprovalKind, &[]),
        &w.say(Word::ToolSeparator, &[]),
        &times.say(w, TimeForm::Span, row.since),
    ])
}
fn review(b: &mut Builder, row: &Row, w: &Words, sign_in: &Forges, style: skein_ui::Style) -> bool {
    if row.yours != Yours::Counts {
        return false;
    }
    let Some(address) = address(row, sign_in) else { return false };
    let label = review_words(row, w);
    b.open(Role::Link);
    if style == look::GOAL_PRIMARY {
        b.style(look::GOAL_BTN);
    }
    b.style(style);
    b.name(&label);
    b.external(&address);
    b.text(Run::Plain, &label);
    b.close();
    true
}
pub(crate) fn band(b: &mut Builder, row: &Row, me: u64, w: &Words, bindings: &mut Bindings, sign_in: &Forges) {
    b.open(Role::Region);
    b.style(look::GOAL_BAND);
    b.name(&w.say(Word::NeedsYou, &[]));
    b.open(Role::Paragraph);
    ask::text(
        b,
        Role::Text,
        &concat(&[&w.say(Word::NeedsYou, &[]), &w.say(Word::ToolSeparator, &[]), &w.say(Word::ApprovalKind, &[])]),
        look::GOAL_BAND_TAG,
    );
    b.text(Run::Plain, &question(row, w));
    b.close();
    for line in &sentences(row, me, w) {
        ask::text(b, Role::Paragraph, line, look::ASK_CONTEXT);
    }
    b.open(Role::Slot);
    b.style(look::GOAL_CHOICES);
    if !review(b, row, w, sign_in, look::GOAL_PRIMARY) {
        crate::bindings::link(
            b,
            &w.say(Word::HomeOpen, &[]),
            bindings.insert(Act::Go { address: Address::Task { task: row.task } }),
        );
        b.style(look::GOAL_BTN);
        b.style(look::GOAL_PRIMARY);
        b.text(Run::Plain, &w.say(Word::HomeOpen, &[]));
        b.close();
    }
    b.close();
    b.close();
}
