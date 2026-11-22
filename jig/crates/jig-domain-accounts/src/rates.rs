//! A window's in-memory pace (jig/docs/design/domain/accounts.md, 3.7).
use alloc::boxed::Box;
use skein_lib::{Duration, List, Wall};

const READING_INTERVAL: Duration = Duration::from_secs(5 * 60);
const READING_HORIZON: Duration = Duration::from_secs(60 * 60);
const ROLLING_SPAN: Duration = Duration::from_secs(30 * 60);
const READINGS: u32 = 13;
const HOUR: Duration = Duration::from_secs(60 * 60);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct WindowReading {
    pub at: Wall,
    pub share: u16,
}

#[derive(Hash, Debug)]
pub(crate) struct Window {
    pub(crate) label: Box<[u8]>,
    pub(crate) readings: List<WindowReading>,
    pub(crate) rate: Option<u32>,
}

pub(crate) fn report(domain: &mut crate::Domain, account: u32, label: &[u8], wall: Wall) {
    let mut value = None;
    for fact in domain.account_facts(account) {
        if fact.kind == crate::KIND_USAGE && fact.label.as_ref() == label {
            match fact.value {
                crate::FactValue::Window(window) => value = Some(window),
                crate::FactValue::Text(_)
                | crate::FactValue::Count(_)
                | crate::FactValue::Instant(_)
                | crate::FactValue::Span(_)
                | crate::FactValue::Share(_) => {}
            }
        }
    }
    let Some(value) = value else { return };
    if value.used.is_none() {
        return;
    }
    if !domain.rates.contains_key(&account) {
        assert!(
            domain.rates.insert(account, List::with_capacity(crate::FACTS)).is_ok(),
            "one rates record per account"
        );
    }
    let windows = domain.rates.get_mut(&account).expect("rates record admitted");
    for window in windows.iter_mut() {
        if window.label.as_ref() == label {
            if update(window, value, wall) {
                domain.rates_changed = Some(account);
            }
            return;
        }
    }
    let mut window = Window { label: label.into(), readings: List::with_capacity(READINGS), rate: None };
    if update(&mut window, value, wall) {
        domain.rates_changed = Some(account);
    }
    windows.push(window).expect("only held facts have readings");
}

fn update(window: &mut Window, value: crate::FactWindow, wall: Wall) -> bool {
    let Some(share) = value.used else { return false };
    if let Some(newest) = window.readings.last() {
        if share < newest.share {
            window.readings.clear();
        } else if wall.as_nanos().saturating_sub(newest.at.as_nanos()) < READING_INTERVAL.as_nanos() {
            return false;
        }
    }
    let mut readings = List::with_capacity(READINGS);
    for reading in &window.readings {
        if wall.as_nanos().saturating_sub(reading.at.as_nanos()) <= READING_HORIZON.as_nanos() {
            readings.push(*reading).expect("kept readings fit");
        }
    }
    readings.push(WindowReading { at: wall, share }).expect("five-minute readings in one hour fit");
    window.readings = readings;
    let rate = rate(window, value);
    let changed = rate != window.rate;
    window.rate = rate;
    changed
}

fn rate(window: &Window, value: crate::FactWindow) -> Option<u32> {
    let oldest = window.readings.get(0)?;
    let newest = window.readings.last()?;
    let span = newest.at.as_nanos().saturating_sub(oldest.at.as_nanos());
    let (share, span) = if span >= ROLLING_SPAN.as_nanos() {
        (newest.share.saturating_sub(oldest.share), u128::from(span))
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
