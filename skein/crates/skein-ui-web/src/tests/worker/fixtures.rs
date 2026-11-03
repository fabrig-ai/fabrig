use crate::{
    bridge,
    platform::{Platform, worker::Page},
    store::Record,
    transport::StreamDuty,
};
use skein_lib::{Duration, Time, Wall};

pub fn start() -> bridge::Start {
    bridge::Start { story: b"clock".as_slice().into(), seed: 7, start: Wall::from_nanos(1000) }
}

pub fn page() -> Page {
    Page::new(start(), Time::ZERO)
}

pub fn clock(now: u64, idle: bool, next_due: Option<u64>) -> bridge::Clock {
    bridge::Clock {
        step: 1,
        now: Time::from_nanos(now),
        wall: Wall::from_nanos(1000_u64.checked_add(now).expect("fixture wall fits")),
        idle,
        next_due: next_due.map(Time::from_nanos),
        memory: 0,
        slice: Duration::ZERO,
    }
}

pub fn hear(page: &mut Page, now: u64, browser: u64, idle: bool, next_due: Option<u64>) {
    page.heard(&bridge::ToPage::Clock(clock(now, idle, next_due)), Time::from_nanos(browser));
}

pub fn assert_clock(page: &mut Page, browser: u64, now: u64) {
    assert_eq!(
        page.clocks(Time::from_nanos(browser)),
        (Time::from_nanos(now), Wall::from_nanos(1000_u64.checked_add(now).expect("fixture wall fits"))),
        "browser {browser}"
    );
}

pub struct Stored(pub Page);

impl Platform for Stored {
    fn read_all(&self) -> Vec<Record> {
        self.0.read_all()
    }
    fn write(&mut self, key: &[u8], value: &[u8]) -> bool {
        self.0.write(key, value)
    }
    fn remove(&mut self, key: &[u8]) {
        self.0.remove(key);
    }
    fn send(&mut self, _: &crate::fetch::Exchange) {
        unreachable!("the store uses only record calls");
    }
    fn navigate(&mut self, _: &[u8]) {
        unreachable!("the store uses only record calls");
    }
    fn stream(&mut self, _: StreamDuty) {
        unreachable!("the store uses only record calls");
    }
    fn abandon(&mut self, _: u32) {
        unreachable!("the store uses only record calls");
    }
    fn read_on(&mut self) {
        unreachable!("the store uses only record calls");
    }
}
