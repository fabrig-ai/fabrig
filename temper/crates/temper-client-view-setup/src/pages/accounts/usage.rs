//! Usage from held account windows alone (temper/docs/design/client/pages/accounts-words.md, 1, Usage).
use crate::{
    accounts::words,
    look,
    words::{Shell, Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::{List, Wall};
use skein_ui::{Builder, Fraction, Role, Run, State, phrases::Hole};

const MINUTE: u64 = 60_000_000_000;
const HOUR: u64 = 3_600_000_000_000;
const SEPARATOR: &[u8] = b" \xc2\xb7 ";

#[derive(Debug, PartialEq, Eq)]
enum Look {
    Plain,
    Warned,
    Limited,
}
#[derive(Debug)]
struct Window {
    name: Box<[u8]>,
    share: Box<[u8]>,
    fraction: Fraction,
    look: Look,
    pace: Box<[u8]>,
    fills: Box<[u8]>,
    line: Box<[u8]>,
}

fn windows(facts: &List<c::AccountFact>, now: Wall, w: &Words) -> List<Window> {
    let mut order = List::with_capacity(facts.len());
    for (index, fact) in facts.iter().enumerate() {
        if fact.kind != c::FACT_USAGE {
            continue;
        }
        let Some(window) = fact_window(&fact.value) else { continue };
        let length = match window.length {
            Some(length) => length.as_nanos(),
            None => 0,
        };
        order.push((window.length.is_none(), length, fact.label.clone(), index)).expect("at most one entry per fact");
    }
    let mut order = order.into_boxed();
    order.sort_unstable();
    let mut result = List::with_capacity(facts.len());
    for (_, _, _, index) in &order {
        let fact = facts.as_slice().get(*index).expect("ordered fact");
        if let Some(window) = fact_window(&fact.value)
            && let Some(line) = window_line(&fact.label, window, now, w)
        {
            result.push(line).expect("at most one line per fact");
        }
    }
    result
}

fn fact_window(value: &c::AccountFactValue) -> Option<c::AccountWindow> {
    match value {
        c::AccountFactValue::Window(window) => Some(*window),
        c::AccountFactValue::Text(_)
        | c::AccountFactValue::Count(_)
        | c::AccountFactValue::Instant(_)
        | c::AccountFactValue::Span(_)
        | c::AccountFactValue::Share(_) => None,
    }
}

fn window_line(label: &[u8], window: c::AccountWindow, now: Wall, w: &Words) -> Option<Window> {
    let used = window.used?;
    let remaining = match window.resets {
        Some(reset) if reset <= now => return None,
        Some(reset) => Some(reset.as_nanos().checked_sub(now.as_nanos()).expect("future reset")),
        None => None,
    };
    let limited = window.state == c::WINDOW_LIMITED;
    let full = match (window.rate, remaining) {
        (Some(rate), Some(remaining)) if rate != 0 => {
            let until = u128::from(10_000_u16.saturating_sub(used))
                .checked_mul(u128::from(HOUR))
                .expect("share times hour fits u128");
            let reset = u128::from(rate).checked_mul(u128::from(remaining)).expect("rate times span fits u128");
            if until < reset {
                Some(u64::try_from(until.checked_div(u128::from(rate)).expect("nonzero rate")).expect("before reset"))
            } else {
                None
            }
        }
        (Some(_) | None, Some(_) | None) => None,
    };
    let model = match label.strip_prefix(b"seven_day_") {
        Some(model) => model,
        None => b"",
    };
    if (label == b"five_hour" || !model.is_empty()) && !limited && full.is_none() {
        return None;
    }
    let name = match label {
        b"five_hour" => skein_lib::bytes::copy_of(b"5 h"),
        b"seven_day" => skein_lib::bytes::copy_of(b"Week"),
        other => match model {
            [first, rest @ ..] => words::join(&[&[first.to_ascii_uppercase()], rest]),
            [] => skein_lib::bytes::copy_of(other),
        },
    };
    let share = words::join(&[&number(w, u64::from(used).checked_div(100).expect("nonzero percent")), b"%"]);
    let mut pace = match remaining {
        Some(remaining) => words::join(&[b"resets in ", &span(w, remaining)]),
        None => Box::new([]),
    };
    let mut fills: Box<[u8]> = Box::new([]);
    if !limited && let Some(hourly) = window.rate {
        pace = words::join(&[&pace, if pace.is_empty() { b"" } else { SEPARATOR }, &rate(w, hourly)]);
        if let Some(full) = full {
            pace = words::join(&[&pace, SEPARATOR]);
            fills = words::join(&[b"full in ", &span(w, full)]);
        } else if let Some(remaining) = remaining {
            let projected = u128::from(hourly)
                .checked_mul(u128::from(remaining))
                .expect("rate times span fits u128")
                .checked_add(u128::from(used).checked_mul(u128::from(HOUR)).expect("share times hour fits u128"))
                .expect("bounded projection");
            let percent = rounded(projected, u128::from(HOUR).checked_mul(100).expect("hour times percent fits u128"));
            pace = words::join(&[&pace, SEPARATOR, b"about ", &number(w, percent), b"% at reset"]);
        }
    }
    let look = if limited {
        Look::Limited
    } else if used >= 8_000 {
        Look::Warned
    } else {
        Look::Plain
    };
    let line =
        words::join(&[&name, b": ", &share, b" used", if pace.is_empty() { b"" } else { SEPARATOR }, &pace, &fills]);
    Some(Window { name, share, fraction: Fraction { done: u64::from(used), total: 10_000 }, look, pace, fills, line })
}
fn number(w: &Words, value: u64) -> Box<[u8]> {
    words::say(w, words::Word::Number, &[Hole::Number(value)])
}
fn rounded(numerator: u128, denominator: u128) -> u64 {
    u64::try_from(
        numerator
            .checked_add(denominator.checked_div(2).expect("nonzero two"))
            .expect("bounded rate or projection")
            .checked_div(denominator)
            .expect("nonzero elapsed"),
    )
    .expect("bounded rate or projection")
}
fn rate(w: &Words, hourly: u32) -> Box<[u8]> {
    if hourly < 1000 {
        let tenths = rounded(u128::from(hourly), 10);
        words::join(&[
            &number(w, tenths.checked_div(10).expect("nonzero ten")),
            b".",
            &number(w, tenths.checked_rem(10).expect("nonzero ten")),
            b"%/h",
        ])
    } else {
        words::join(&[&number(w, rounded(u128::from(hourly), 100)), b"%/h"])
    }
}
fn span(w: &Words, nanos: u64) -> Box<[u8]> {
    let minutes = nanos.checked_div(MINUTE).expect("nonzero minute");
    if minutes < 1440 {
        return w.time_span(minutes);
    }
    let days = minutes.checked_div(1440).expect("nonzero day");
    let hours = minutes.checked_rem(1440).expect("nonzero day").checked_div(60).expect("nonzero hour");
    let day = w.say(Word::TimeDays, &[Hole::Number(days)]);
    if hours == 0 { day } else { words::join(&[&day, b" ", &w.say(Word::TimeHours, &[Hole::Number(hours)])]) }
}

pub(super) fn draw(b: &mut Builder, w: &Words, facts: &List<c::AccountFact>, times: &mut Times) {
    let windows = windows(facts, times.now, w);
    if windows.is_empty() {
        return;
    }
    let next = match times.now.as_nanos().checked_div(MINUTE).expect("nonzero minute").checked_add(1) {
        Some(minute) => minute.checked_mul(MINUTE),
        None => None,
    };
    if let Some(next) = next {
        times.next = [times.next, Some(Wall::from_nanos(next))].into_iter().flatten().min();
    }
    match w.shell {
        Shell::Browser => {
            b.open(Role::Group);
            b.name(b"Usage");
            b.style(look::USAGE);
            b.state(State::ShownFolded);
            for window in &windows {
                b.open(Role::Group);
                b.style(look::USAGE_WINDOW);
                match window.look {
                    Look::Plain => {}
                    Look::Warned => b.style(look::USAGE_WARNED),
                    Look::Limited => b.style(look::USAGE_LIMITED),
                }
                styled_text(b, &window.name, look::USAGE_NAME);
                b.open(Role::Meter);
                b.style(look::USAGE_BAR);
                b.name(&window.name);
                b.fraction(window.fraction);
                b.run(Run::Plain, &words::join(&[&window.share, b" used"]));
                b.close();
                styled_text(b, &window.share, look::USAGE_USED);
                b.open(Role::Group);
                b.style(look::USAGE_PACE);
                b.text(Run::Plain, &window.pace);
                if !window.fills.is_empty() {
                    styled_text(b, &window.fills, look::USAGE_FILLS);
                }
                b.close();
                b.close();
            }
            b.close();
        }
        Shell::Terminal => {
            for window in &windows {
                b.open(Role::Paragraph);
                b.state(State::ShownFolded);
                b.name(&window.line);
                b.text(Run::Plain, &window.line);
                b.close();
            }
        }
    }
}
fn styled_text(b: &mut Builder, text: &[u8], style: skein_ui::Style) {
    b.open(Role::Text);
    b.style(style);
    b.run(Run::Plain, text);
    b.close();
}

#[cfg(test)]
mod tests;
