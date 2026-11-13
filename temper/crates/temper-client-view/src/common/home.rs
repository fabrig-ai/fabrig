//! Shared home items (temper/docs/design/client/view-crates.md, 3.3).
use crate::words::{TimeForm, Times, Word, Words};
use alloc::boxed::Box;
use skein_ui::phrases::Hole;
use temper_client_domain::pages::{
    home::{Facts, Needs, Status},
    needs,
};

pub(crate) struct Intro {
    pub(crate) kicker: Box<[u8]>,
    pub(crate) headline: Box<[u8]>,
    pub(crate) lede: Option<Box<[u8]>>,
    pub(crate) byline: Option<Box<[u8]>>,
}

pub(crate) fn kicker(w: &Words, previous: Option<u64>, times: &mut Times) -> Box<[u8]> {
    // The kicker is this Home's day and time, not a project's creation time.
    let day = times.say(w, TimeForm::Weekday, times.now);
    let clock = times.say(w, TimeForm::Clock, times.now);
    let mut kicker = w.say(Word::HomeDayTime, &[Hole::Name(day), Hole::Name(clock)]);
    let minute = skein_lib::Duration::from_secs(60);
    let tick = times.now.as_nanos().checked_div(minute.as_nanos()).expect("minute is nonzero");
    let next = match tick.checked_add(1) {
        Some(tick) => match tick.checked_mul(minute.as_nanos()) {
            Some(nanos) => Some(skein_lib::Wall::from_nanos(nanos)),
            None => None,
        },
        None => None,
    };
    times.next = [times.next, next].into_iter().flatten().min();
    if let Some(previous) = previous
        && times.now.as_nanos().saturating_sub(previous) > minute.as_nanos()
    {
        let span = times.say(w, TimeForm::Looked, skein_lib::Wall::from_nanos(previous));
        kicker = w.say(Word::HomeLooked, &[Hole::Name(kicker), Hole::Name(span)]);
    }
    kicker
}

pub(crate) fn needs_sentence(w: &Words, needs: Needs, unknown: &[u8]) -> Box<[u8]> {
    match needs {
        Needs::Unknown => unknown.into(),
        Needs::Count { count, cut: true } => w.say(Word::HomeNeedsCut, &[Hole::Number(u64::from(count))]),
        Needs::Count { count: 0, cut: false } => w.say(Word::HomeNeedsNone, &[]),
        Needs::Count { count: 1, cut: false } => w.say(Word::HomeNeedsOne, &[]),
        Needs::Count { count, cut: false } => {
            let number = match number_word(count) {
                Some(word) => w.say(word, &[]),
                None => skein_lib::Decimal::of(u64::from(count)).as_bytes().into(),
            };
            w.say(Word::HomeNeedsMany, &[Hole::Name(number)])
        }
    }
}

pub(crate) fn intro_words(w: &Words, name: &[u8], facts: &Facts, times: &mut Times) -> Intro {
    let previous = match &facts.status {
        Status::Shown(status) => *status.previous(),
        Status::Unknown => None,
    };
    let mut kicker = kicker(w, previous, times);
    let needs = if facts.to_begin() {
        w.say(Word::HomeProjectReady, &[Hole::Name(name.into())])
    } else {
        needs_sentence(w, facts.needs, name)
    };
    let minute = skein_lib::Duration::from_secs(60);
    let landed = if facts.to_begin() { 0 } else { facts.landed_since() };
    let brief = if facts.to_begin() { None } else { facts.brief() };
    let sentence = match brief {
        Some(brief) => brief.sentence().into(),
        None => match landed {
            0 => match if facts.to_begin() { None } else { facts.last_landed() } {
                Some(last) => {
                    if times.now.as_nanos().saturating_sub(last.as_nanos()) < minute.as_nanos() {
                        w.say(Word::HomeLandedNow, &[])
                    } else {
                        let span = times.say(w, TimeForm::Looked, last);
                        w.say(Word::HomeLandedLast, &[Hole::Name(span)])
                    }
                }
                None => Box::default(),
            },
            1 => w.say(Word::HomeLandedOne, &[]),
            2..=u64::MAX => w.say(Word::HomeLandedMany, &[Hole::Number(landed)]),
        },
    };
    let (lede, byline) = match brief {
        Some(brief) => {
            let lede = if brief.lede().is_empty() { None } else { Some(brief.lede().into()) };
            let at = times.say(w, TimeForm::Clock, skein_lib::Wall::from_nanos(brief.at()));
            (lede, Some(w.say(Word::HomeByline, &[Hole::Name(at)])))
        }
        None => (None, None),
    };
    if let Some(spell) = &facts.spell {
        if let Some(end) = spell.end {
            let expires = skein_lib::Wall::from_nanos(end.as_nanos().saturating_add(86_400_000_000_000));
            times.next = [times.next, Some(expires)].into_iter().flatten().min();
        }
        let since = times.say(w, TimeForm::Clock, spell.since);
        let end = match spell.end {
            Some(end) => times.say(w, TimeForm::Clock, end),
            None => w.say(Word::HomeAwayNow, &[]),
        };
        kicker = w.say(Word::HomeAwaySpan, &[Hole::Name(since), Hole::Name(end)]);
        let decided = match &facts.decided {
            Some(shown) if shown.showing == 1 => w.say(Word::HomeDecidedOne, &[]),
            Some(shown) => {
                let number = match number_word(shown.showing) {
                    Some(word) => w.say(word, &[]),
                    None => skein_lib::Decimal::of(u64::from(shown.showing)).as_bytes().into(),
                };
                w.say(Word::HomeDecidedMany, &[Hole::Name(number)])
            }
            None => needs,
        };
        let landed = match spell.landed {
            None | Some(0) => Box::default(),
            Some(1) => w.say(Word::HomeAwayLandedOne, &[]),
            Some(count) => w.say(Word::HomeAwayLandedMany, &[Hole::Number(count)]),
        };
        let headline =
            if landed.is_empty() { decided } else { crate::accounts::words::join(&[&landed, b" ", &decided]) };
        return Intro { kicker, headline, lede, byline };
    }
    let headline = if sentence.is_empty() { needs } else { crate::accounts::words::join(&[&sentence, b" ", &needs]) };
    Intro { kicker, headline, lede, byline }
}

pub(crate) fn number_word(count: u32) -> Option<Word> {
    match count {
        2 => Some(Word::HomeTwo),
        3 => Some(Word::HomeThree),
        4 => Some(Word::HomeFour),
        5 => Some(Word::HomeFive),
        6 => Some(Word::HomeSix),
        7 => Some(Word::HomeSeven),
        8 => Some(Word::HomeEight),
        9 => Some(Word::HomeNine),
        0 | 1 | 10.. => None,
    }
}

/// Home's need words (temper/docs/design/client/pages/README.md, Home, Needs you).
pub(crate) fn need_words(need: &needs::Need<'_>, w: &Words) -> (Box<[u8]>, Box<[u8]>, crate::shared::NeedKind) {
    match need {
        needs::Need::Card(card) => {
            let asked = super::ask::read(card, w);
            let title = if card.page.is_none() { super::ask::need_title(card, w) } else { asked.title };
            let kind = match asked.kind {
                super::ask::Kind::Question => crate::shared::NeedKind::Question,
                super::ask::Kind::Proposal
                | super::ask::Kind::Held
                | super::ask::Kind::Funds { .. }
                | super::ask::Kind::PersonTask => crate::shared::NeedKind::Other,
            };
            (title, asked.short, kind)
        }
        needs::Need::Approval(row) => {
            (row.title.clone(), super::approval::question(row, w), crate::shared::NeedKind::Other)
        }
    }
}

pub(crate) fn need_sentence(title: &[u8], short: &[u8], w: &Words) -> Box<[u8]> {
    w.say(Word::HomeAskLine, &[Hole::Name(title.into()), Hole::Name(short.into())])
}
