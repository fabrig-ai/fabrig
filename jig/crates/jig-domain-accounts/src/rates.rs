//! A window's in-memory pace (jig/docs/design/domain/accounts-lists.md, 1).
use alloc::boxed::Box;
use skein_lib::{Duration, List, Wall};

const READING_INTERVAL: Duration = Duration::from_secs(5 * 60);
const READING_HORIZON: Duration = Duration::from_secs(3 * 60 * 60 + 5 * 60);
const ROLLING_SPAN: Duration = Duration::from_secs(3 * 60 * 60);
const READINGS: u32 = 2 * 38;
const HOUR: Duration = Duration::from_secs(60 * 60);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct WindowReading {
    pub at: Wall,
    pub share: u16,
    pub origin: u16,
    pub resets: Option<Wall>,
}

#[derive(Hash, Debug)]
pub(crate) struct Window {
    pub(crate) label: Box<[u8]>,
    pub(crate) readings: List<WindowReading>,
    pub(crate) rate: Option<u32>,
}

pub(crate) fn report(
    domain: &mut crate::Domain,
    account: u32,
    label: &[u8],
    reading: WindowReading,
    value: crate::FactWindow,
) {
    if !domain.rates.contains_key(&account) {
        assert!(
            domain.rates.insert(account, List::with_capacity(crate::FACTS)).is_ok(),
            "one rates record per account"
        );
    }
    let windows = domain.rates.get_mut(&account).expect("rates record admitted");
    for window in windows.iter_mut() {
        if window.label.as_ref() == label {
            if update(window, reading, value) {
                domain.rates_changed = Some(account);
            }
            return;
        }
    }
    let mut window = Window { label: label.into(), readings: List::with_capacity(READINGS), rate: None };
    if update(&mut window, reading, value) {
        domain.rates_changed = Some(account);
    }
    windows.push(window).expect("only held facts have readings");
}

fn update(window: &mut Window, reading: WindowReading, value: crate::FactWindow) -> bool {
    if let Some(newest) = window.readings.last()
        && reading.at < newest.at
    {
        return false;
    }
    let mut reset = false;
    for prior in window.readings.iter().rev() {
        if prior.origin == reading.origin {
            reset = reading.share < prior.share
                || (reading.resets.is_some() && prior.resets.is_some() && reading.resets != prior.resets);
            if !reset && age(reading, *prior) < READING_INTERVAL.as_nanos() {
                return false;
            }
            break;
        }
    }
    keep_readings(&mut window.readings, reading, reset);
    window.readings.push(reading).expect("five-minute readings fit the horizon for both origins");
    let rate = rate(&window.readings, reading, value);
    let changed = rate != window.rate;
    window.rate = rate;
    changed
}

fn age(newest: WindowReading, prior: WindowReading) -> u64 {
    newest.at.as_nanos().saturating_sub(prior.at.as_nanos())
}

fn keep_readings(readings: &mut List<WindowReading>, newest: WindowReading, reset: bool) {
    let mut kept = 0_u32;
    for index in 0..readings.len() {
        let prior = *readings.get(index).expect("held reading index");
        if (reset && prior.origin == newest.origin) || age(newest, prior) > READING_HORIZON.as_nanos() {
            continue;
        }
        if kept != index {
            *readings.get_mut(kept).expect("retained readings fit") = prior;
        }
        kept = kept.saturating_add(1);
    }
    readings.truncate(kept);
}

fn rate(readings: &List<WindowReading>, newest: WindowReading, value: crate::FactWindow) -> Option<u32> {
    let mut best = None;
    let mut distance = u64::MAX;
    for prior in readings {
        let span = age(newest, *prior);
        if prior.origin != newest.origin || span < HOUR.as_nanos() {
            continue;
        }
        let away = span.abs_diff(ROLLING_SPAN.as_nanos());
        // Readings are in wall order, so a tie replaces the older reading.
        if away <= distance {
            best = Some(*prior);
            distance = away;
        }
    }
    let (share, span) = if let Some(prior) = best {
        (newest.share.saturating_sub(prior.share), u128::from(age(newest, prior)))
    } else {
        // Subtract the reset after adding the length: the opening can precede the epoch.
        let span = u128::from(newest.at.as_nanos())
            .saturating_add(u128::from(value.length?.as_nanos()))
            .checked_sub(u128::from(value.resets?.as_nanos()))?;
        (newest.share, span)
    };
    let hourly = u128::from(share).saturating_mul(u128::from(HOUR.as_nanos())).checked_div(span)?;
    Some(u32::try_from(hourly).unwrap_or(u32::MAX))
}

/// A fact gone takes its readings; moving and removing accounts takes the whole record.
pub(crate) fn retain(domain: &mut crate::Domain, account: u32, facts: &[crate::AccountFact]) {
    let Some(windows) = domain.rates.get_mut(&account) else { return };
    for index in (0..windows.len()).rev() {
        let window = windows.get(index).expect("held window index");
        let mut held = false;
        for fact in facts {
            if fact.kind == crate::KIND_USAGE && fact.label == window.label {
                held = true;
            }
        }
        if !held {
            drop(windows.swap_remove(index));
        }
    }
    if windows.is_empty() {
        drop(domain.rates.remove(&account));
    }
}
