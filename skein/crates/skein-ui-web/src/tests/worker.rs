//! Step tests of the worker's platform (ui-web.md, 18).
mod fixtures;

use crate::{
    bridge::{self, Input, Speed, ToPage, ToWorld},
    platform::worker::{Heard, Page, shown},
    store::{Record, Refused, Store},
    transport::StreamDuty,
};
use fixtures::{Stored, assert_clock, clock, hear, page, start};
use skein_lib::{Duration, Time};

#[test]
fn nothing_leaves_before_the_worker_is_ready() {
    let mut page = page();
    let inputs = [
        Input::Exchange((&super::fixtures::exchange(1)).into()),
        Input::Abandon { exchange: 1 },
        Input::Stream(StreamDuty::Open { path: b"/events".as_slice().into(), headers: vec![], bound: 8 }),
        Input::ReadOn,
        Input::Stream(StreamDuty::Close),
    ];
    for input in &inputs {
        page.send(input.clone());
    }
    page.pace(Speed(500), Time::ZERO);
    page.visibility(true, Time::ZERO);
    let mut out = Vec::new();
    page.outgoing(&mut out);
    assert!(out.is_empty());
    page.ready();
    page.outgoing(&mut out);
    let mut expected = vec![ToWorld::Start(start()), ToWorld::Pace(Speed::ONE)];
    expected.extend(inputs.into_iter().map(ToWorld::Input));
    expected.extend([ToWorld::Pace(Speed(500)), ToWorld::Pace(Speed::PAUSED)]);
    assert_eq!(out, expected);
    page.ready();
    page.outgoing(&mut out);
    assert_eq!(out, expected, "held messages leave only once and append to the caller's buffer");
    out.clear();
    page.send(Input::ReadOn);
    page.outgoing(&mut out);
    assert_eq!(out, [ToWorld::Input(Input::ReadOn)]);
}

#[test]
fn the_clock_starts_at_the_visits_wall_until_the_first_world_clock() {
    let mut page = Page::new(start(), Time::from_nanos(200));
    assert_clock(&mut page, 500, 0);
    page.pace(Speed(10000), Time::from_nanos(600));
    assert_clock(&mut page, 800, 0);
    assert_eq!(page.timer(Time::ZERO, Time::from_nanos(800)), None);
    hear(&mut page, 10, 900, true, None);
    assert_clock(&mut page, 901, 20);
}

#[test]
fn the_clock_moves_while_the_world_waits_idle() {
    for (speed, before_due) in [(Speed::ONE, 120), (Speed(500), 110), (Speed(10000), 300)] {
        let mut page = page();
        page.pace(speed, Time::ZERO);
        hear(&mut page, 100, 50, true, Some(300));
        assert_clock(&mut page, 50, 100);
        assert_clock(&mut page, 70, before_due);
        assert_clock(&mut page, 1000, 300);
        assert_clock(&mut page, 2000, 300);
    }
    let mut page = page();
    hear(&mut page, 100, 50, true, None);
    assert_clock(&mut page, 2000, 2050);
}

#[test]
fn the_clock_holds_while_the_world_steps() {
    let mut page = page();
    hear(&mut page, 100, 50, false, Some(300));
    assert_clock(&mut page, 2000, 100);
    page.pace(Speed(10000), Time::from_nanos(2000));
    assert_clock(&mut page, 3000, 100);
    hear(&mut page, 200, 3000, false, None);
    assert_clock(&mut page, 4000, 200);
}

#[test]
fn the_clock_holds_while_paused() {
    let mut page = page();
    hear(&mut page, 100, 50, true, None);
    page.pace(Speed::PAUSED, Time::from_nanos(70));
    assert_clock(&mut page, 1000, 120);
    page.pace(Speed::ONE, Time::from_nanos(1000));
    assert_clock(&mut page, 1001, 121);
    page.pace(Speed::PAUSED, Time::from_nanos(1002));
    hear(&mut page, 200, 1100, true, None);
    assert_clock(&mut page, 2000, 200);
}

#[test]
fn the_clock_never_goes_back() {
    let mut page = page();
    hear(&mut page, 100, 50, true, None);
    assert_clock(&mut page, 100, 150);
    hear(&mut page, 120, 100, true, None);
    assert_clock(&mut page, 110, 150);
    page.pace(Speed(500), Time::from_nanos(110));
    assert_clock(&mut page, 130, 150);
    assert_clock(&mut page, 160, 155);
    assert_clock(&mut page, 100, 155);
    hear(&mut page, 140, 200, false, None);
    assert_clock(&mut page, 300, 155);
    hear(&mut page, 160, 300, false, None);
    assert_clock(&mut page, 400, 160);
}

#[test]
fn a_pace_change_rebases_at_its_browser_instant() {
    let mut page = page();
    hear(&mut page, 100, 50, true, Some(500));
    page.pace(Speed(500), Time::from_nanos(70));
    assert_clock(&mut page, 90, 130);
    page.pace(Speed(10000), Time::from_nanos(90));
    assert_clock(&mut page, 100, 230);
    assert_clock(&mut page, 200, 500);
}

#[test]
fn a_hidden_tab_pauses_and_the_shown_tab_resumes_its_speed() {
    let mut page = page();
    page.ready();
    let mut out = Vec::new();
    page.outgoing(&mut out);
    out.clear();
    page.pace(Speed(500), Time::ZERO);
    hear(&mut page, 100, 50, true, None);
    page.visibility(true, Time::from_nanos(70));
    assert_clock(&mut page, 1000, 110);
    page.visibility(true, Time::from_nanos(1100));
    page.pace(Speed(10000), Time::from_nanos(1200));
    assert_clock(&mut page, 2000, 110);
    assert_eq!(page.timer(Time::from_nanos(111), Time::from_nanos(2000)), None);
    page.visibility(false, Time::from_nanos(2000));
    page.visibility(false, Time::from_nanos(2001));
    assert_clock(&mut page, 2002, 130);
    page.outgoing(&mut out);
    assert_eq!(out, [ToWorld::Pace(Speed(500)), ToWorld::Pace(Speed::PAUSED), ToWorld::Pace(Speed(10000))]);
}

#[test]
fn the_timer_is_the_browser_instant_the_clock_reaches_the_deadline() {
    for (speed, at_deadline) in [(Speed::ONE, 150), (Speed(10000), 60)] {
        let mut page = page();
        page.pace(speed, Time::ZERO);
        hear(&mut page, 100, 50, true, Some(200));
        assert_eq!(page.timer(Time::from_nanos(200), Time::from_nanos(50)), Some(Time::from_nanos(at_deadline)));
        assert_eq!(page.timer(Time::from_nanos(201), Time::from_nanos(50)), None);
        assert_eq!(page.timer(Time::from_nanos(100), Time::from_nanos(50)), Some(Time::from_nanos(50)));
        assert_eq!(page.timer(Time::from_nanos(110), Time::from_nanos(70)), Some(Time::from_nanos(70)));
        assert_clock(&mut page, at_deadline, 200);
        page.pace(Speed::PAUSED, Time::from_nanos(at_deadline));
        assert_eq!(page.timer(Time::from_nanos(200), Time::from_nanos(1000)), None);
        page.pace(speed, Time::from_nanos(1000));
        hear(&mut page, 200, 1000, false, None);
        assert_eq!(page.timer(Time::from_nanos(200), Time::from_nanos(2000)), None);
        assert_eq!(page.timer(Time::from_nanos(201), Time::from_nanos(2000)), None);
    }
}

#[test]
fn timers_follow_pace_changes_and_a_clock_held_at_a_reading() {
    let mut page = page();
    hear(&mut page, 100, 50, true, Some(200));
    page.pace(Speed(500), Time::from_nanos(70));
    assert_eq!(page.timer(Time::from_nanos(200), Time::from_nanos(80)), Some(Time::from_nanos(230)));
    assert_clock(&mut page, 100, 135);
    hear(&mut page, 110, 100, true, Some(200));
    assert_eq!(page.timer(Time::from_nanos(135), Time::from_nanos(100)), Some(Time::from_nanos(100)));
    assert_eq!(page.timer(Time::from_nanos(140), Time::from_nanos(100)), Some(Time::from_nanos(160)));
}

#[test]
fn the_clock_rounds_down_and_the_timer_rounds_up_to_nanoseconds() {
    let mut page = page();
    page.pace(Speed(500), Time::ZERO);
    hear(&mut page, 100, 50, true, None);
    assert_clock(&mut page, 51, 100);
    assert_clock(&mut page, 52, 101);
    assert_eq!(page.timer(Time::from_nanos(102), Time::from_nanos(52)), Some(Time::from_nanos(54)));
    page.pace(Speed(10000), Time::from_nanos(52));
    assert_eq!(page.timer(Time::from_nanos(102), Time::from_nanos(52)), Some(Time::from_nanos(53)));
    assert_clock(&mut page, 53, 111);
    page.pace(Speed(3000), Time::from_nanos(53));
    assert_eq!(page.timer(Time::from_nanos(115), Time::from_nanos(53)), Some(Time::from_nanos(55)));
    assert_clock(&mut page, 54, 114);
    assert_clock(&mut page, 55, 117);
}

#[test]
fn clock_scaling_uses_the_whole_product_and_bounds_the_result() {
    let mut page = page();
    page.pace(Speed(u32::MAX), Time::ZERO);
    hear(&mut page, 0, 0, true, Some(500));
    assert_clock(&mut page, u64::MAX, 500);
    let mut page = fixtures::page();
    page.pace(Speed(500), Time::ZERO);
    hear(&mut page, 0, 0, true, None);
    let half = u64::MAX.checked_div(2).expect("two is nonzero");
    assert_clock(&mut page, u64::MAX, half);
}

#[test]
fn a_timer_outside_browser_time_is_not_armed() {
    let mut page = page();
    page.pace(Speed(1), Time::ZERO);
    hear(&mut page, 0, 0, true, None);
    assert_eq!(page.timer(Time::from_nanos(u64::MAX), Time::ZERO), None);
    page.pace(Speed::ONE, Time::ZERO);
    hear(&mut page, 0, u64::MAX, true, None);
    assert_eq!(page.timer(Time::from_nanos(1), Time::from_nanos(u64::MAX)), None);
}

#[test]
fn the_high_waters_are_the_largest_heard() {
    let mut page = page();
    assert_eq!(page.high_water(), (0, Duration::ZERO));
    for (memory, slice, expected) in [(64, 20, (64, 20)), (128, 10, (128, 20)), (32, 30, (128, 30))] {
        let mut clock = clock(0, true, None);
        clock.memory = memory;
        clock.slice = Duration::from_nanos(slice);
        page.heard(&ToPage::Clock(clock), Time::ZERO);
        assert_eq!(page.high_water(), (expected.0, Duration::from_nanos(expected.1)));
    }
}

#[test]
fn other_messages_leave_the_clock_and_high_waters_alone() {
    let mut page = page();
    hear(&mut page, 100, 50, true, None);
    let printed = bridge::Printed {
        process: b"server".as_slice().into(),
        stream: bridge::Stream::Error,
        line: b"line".as_slice().into(),
    };
    for message in [
        ToPage::Report(crate::fetch::Report::StreamEnded),
        ToPage::Printed(printed.clone()),
        ToPage::Stamp(bridge::Stamp { step: 1, instant: Time::from_nanos(100) }),
        ToPage::Stopped,
        ToPage::Failed { words: b"failure".as_slice().into() },
    ] {
        page.heard(&message, Time::from_nanos(100));
        assert_clock(&mut page, 100, 150);
        assert_eq!(page.high_water(), (0, Duration::ZERO));
    }
    for heard in
        [Heard::Printed(printed), Heard::Stopped, Heard::Failed { words: b"failure".as_slice().into() }, Heard::Clock]
    {
        assert_eq!(heard.clone(), heard);
        assert!(!format!("{heard:?}").is_empty());
    }
}

#[test]
fn the_store_keeps_records_in_memory() {
    let mut platform = Stored(page());
    let mut store = Store::new(b"p-", Some(20));
    assert_eq!(store.write(b"a", b"one", &mut platform), Ok(()));
    assert_eq!(store.write(b"b", b"two", &mut platform), Ok(()));
    assert_eq!(store.read_all(&platform), [record(b"a", b"one"), record(b"b", b"two")]);
    assert_eq!(store.write(b"a", b"new", &mut platform), Ok(()));
    assert_eq!(store.read_all(&platform), [record(b"a", b"new"), record(b"b", b"two")]);
    let before = platform.0.read_all();
    assert_eq!(store.write(b"c", b"three", &mut platform), Err(Refused));
    assert_eq!(platform.0.read_all(), before, "the Store refuses before writing to the page");
    store.remove(b"a", &mut platform);
    store.remove(b"absent", &mut platform);
    assert_eq!(store.read_all(&platform), [record(b"b", b"two")]);
    assert_eq!(store.write(b"a", b"end", &mut platform), Ok(()));
    assert_eq!(store.read_all(&platform), [record(b"b", b"two"), record(b"a", b"end")]);
    assert!(fixtures::page().read_all().is_empty(), "a new visit has no records");
}

fn record(key: &[u8], value: &[u8]) -> Record {
    Record { key: key.into(), value: value.into() }
}

#[test]
fn only_a_path_on_the_origin_is_shown() {
    for to in [b"/".as_slice(), b"/page?next=https://example.com#part", b"/a%20b"] {
        assert!(shown(to), "{to:?}");
    }
    for to in [
        b"".as_slice(),
        b"page",
        b"?query",
        b"#fragment",
        b"//example.com",
        b"/\\example.com",
        b"/a\\b",
        b"https://example.com",
        b"http://127.0.0.1:80/",
        b"http://[::1]:80/",
        b"javascript:alert(1)",
        b"/space here",
        b"/\r\n",
        b"/\x7f",
        b"/\xff",
    ] {
        assert!(!shown(to), "{to:?}");
    }
}
