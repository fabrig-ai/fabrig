//! A change waiting on a forge review (temper/docs/design/client/forge/README.md, 4.1).
use super::ask;
use crate::{
    forge_sign_in::Forges,
    frame::Bindings,
    look,
    words::{TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::concat};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address,
    forge::{
        approvals::{Row, Yours},
        forges::Kind,
    },
    pages::Act,
};

pub(crate) fn address(row: &Row, sign_in: &Forges) -> Option<Box<[u8]>> {
    let address = if row.kind == Kind::Forgejo
        && let Some(forge) = sign_in.number(row.forge)
    {
        let repository = crate::forge::repository::web_address(b"", &row.owner, &row.name)?;
        let number = skein_lib::Decimal::of(row.pull);
        let path = concat(&[&repository, b"/pulls/", number.as_bytes(), b"/files"]);
        forge.open_path(&path)?
    } else {
        row.address.clone()
    };
    if skein_ui::markdown::safe_url(&address) { Some(address) } else { None }
}
fn forge(row: &Row, w: &Words) -> Box<[u8]> {
    w.say(
        match row.kind {
            Kind::Forgejo => Word::ApprovalForgejo,
            Kind::GitHub => Word::ApprovalGitHub,
        },
        &[],
    )
}
pub(crate) fn question(row: &Row, w: &Words) -> Box<[u8]> {
    w.say(Word::ApprovalQuestion, &[Hole::Name(row.title.clone()), Hole::Name(forge(row, w))])
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
pub(crate) fn sentences(row: &Row, w: &Words) -> Box<[Box<[u8]>]> {
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
            &[
                Hole::Name(rule),
                Hole::Number(u64::from(row.required)),
                Hole::Name(role(row, true, w)),
                Hole::Number(u64::from(row.counted)),
            ],
        )
    };
    lines.push(rule).expect("four context sentences fit");
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
pub(crate) fn when(row: &Row, w: &Words, times: &mut Times) -> Box<[u8]> {
    concat(&[
        &w.say(Word::ApprovalKind, &[]),
        &w.say(Word::ToolSeparator, &[]),
        &times.say(w, TimeForm::Span, row.since),
    ])
}
pub(crate) fn review_words(row: &Row, w: &Words) -> Box<[u8]> {
    w.say(if row.sets.is_empty() { Word::ApprovalReview } else { Word::ApprovalSignOff }, &[Hole::Name(forge(row, w))])
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
pub(crate) fn band(b: &mut Builder, row: &Row, w: &Words, bindings: &mut Bindings, sign_in: &Forges) {
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
    b.open(Role::Slot);
    b.style(look::GOAL_CHOICES);
    if !review(b, row, w, sign_in, look::GOAL_PRIMARY) {
        crate::link(
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
