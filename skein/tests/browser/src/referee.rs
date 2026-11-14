//! Independent checks on the browser's observable contract.

use std::collections::{BTreeMap, BTreeSet};

use skein_browser::boundary::{Event, Expect};
use skein_lib::Token;

/// Checks terminals, closure, press refusal, and `Await` truth at the time
/// the result is emitted. The fake supplies its visible match count.
#[derive(Debug, Default)]
pub struct Referee {
    terminal: BTreeSet<u64>,
    warned: BTreeSet<u64>,
    expectations: BTreeMap<u64, Expect>,
    presses: BTreeMap<u64, usize>,
    mouse_events: usize,
    closed: bool,
    downloads: BTreeMap<Vec<u8>, (Token, Token)>,
    closed_persons: BTreeSet<Token>,
}

impl Referee {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn await_op(&mut self, op: Token, expect: Expect) -> Result<(), String> {
        if self.expectations.insert(op.raw(), expect).is_some() {
            return Err("operation registered once".into());
        }
        Ok(())
    }

    pub fn press(&mut self, op: Token) -> Result<(), String> {
        if self.presses.insert(op.raw(), self.mouse_events).is_some() {
            return Err("press registered once".into());
        }
        Ok(())
    }

    pub fn download_begun(&mut self, person: Token, page: Token, name: &[u8]) -> Result<(), String> {
        if self.closed_persons.contains(&person) {
            return Err("no download begins after its person's Closed".into());
        }
        if self.downloads.insert(name.to_vec(), (person, page)).is_some() {
            return Err("assertion failed: self.downloads.insert(name.to_vec(), (person, page)).is_none()".into());
        }
        Ok(())
    }

    pub fn check_downloads_settled(&self) -> Result<(), String> {
        if !self.downloads.is_empty() {
            return Err("each download begun ends once".into());
        }
        Ok(())
    }

    pub fn command(&mut self, bytes: &[u8]) {
        if bytes.windows(b"Input.dispatchMouseEvent".len()).any(|part| part == b"Input.dispatchMouseEvent") {
            self.mouse_events += 1;
        }
    }

    pub fn event(&mut self, event: &Event, visible_matches: u32) -> Result<(), String> {
        if self.closed {
            return Err("nothing is emitted after Closed".into());
        }
        if let Event::Downloaded { person, name, .. } = event {
            if self.closed_persons.contains(person) {
                return Err("no download after its person's Closed".into());
            }
            let (owner, _) = self.downloads.remove(name.as_ref()).ok_or("each download ends once")?;
            if *person != owner {
                return Err(format!("assertion `left == right` failed\n  left: {person:?}\n right: {owner:?}"));
            }
        }
        if let Event::Trouble { page, trouble: skein_browser::boundary::Trouble::Download, text, .. } = event {
            let (_, owner_page) = self.downloads.remove(text.as_ref()).ok_or("each download ends once")?;
            if *page != owner_page {
                return Err(format!("assertion `left == right` failed\n  left: {page:?}\n right: {owner_page:?}"));
            }
        }
        if let Event::Closed { owner } = event {
            self.closed_persons.insert(*owner);
            self.downloads.retain(|_, (person, _)| person != owner);
        }
        if let Event::Warned { op } = event {
            if self.terminal.contains(&op.raw()) {
                return Err("Warned precedes its go's terminal".into());
            }
            if !self.warned.insert(op.raw()) {
                return Err("a go warns at most once".into());
            }
        }
        let op = match event {
            Event::Found { op, .. }
            | Event::Met { op, .. }
            | Event::Missed { op, .. }
            | Event::Done { op }
            | Event::Refused { op, .. }
            | Event::Attribute { op, .. }
            | Event::Tabs { op, .. }
            | Event::Address { op, .. }
            | Event::Snapshot { op, .. }
            | Event::Screenshot { op, .. } => Some(*op),
            Event::Downloaded { .. }
            | Event::Warned { .. }
            | Event::Ready { .. }
            | Event::Opened { .. }
            | Event::Trouble { .. }
            | Event::Closed { .. } => None,
        };
        if let Some(op) = op
            && !self.terminal.insert(op.raw())
        {
            return Err(format!("one terminal per operation: {op:?}"));
        }
        if let Event::Met { op, .. } = event {
            let expected = self.expectations.get(&op.raw()).ok_or("Met belongs to a registered Await")?;
            let true_now = match expected {
                Expect::Present => visible_matches > 0,
                Expect::Count(count) => visible_matches == *count,
                Expect::Absent => visible_matches == 0,
            };
            if !true_now {
                return Err("Met requires the fake page to satisfy its expectation".into());
            }
        }
        if let Event::Refused { op, .. } = event
            && let Some(at) = self.presses.get(&op.raw())
            && self.mouse_events != *at
        {
            return Err(format!(
                "refused press dispatched a mouse event\n  left: {:?}\n right: {at:?}",
                self.mouse_events
            ));
        }
        if matches!(event, Event::Closed { owner } if *owner == Token::new(1)) {
            self.closed = true;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Referee;
    use skein_browser::boundary::{Event, Expect, Refusal};
    use skein_lib::{List, Token};

    #[test]
    fn rejects_duplicate_registrations() {
        let mut referee = Referee::new();
        let op = Token::new(4);
        referee.await_op(op, Expect::Present).expect("first await");
        assert_eq!(referee.await_op(op, Expect::Absent), Err("operation registered once".into()));
        referee.press(op).expect("first press");
        assert_eq!(referee.press(op), Err("press registered once".into()));
    }

    #[test]
    fn downloads_report_the_first_broken_contract() {
        let mut referee = Referee::new();
        let person = Token::new(2);
        let page = Token::new(3);
        assert_eq!(referee.check_downloads_settled(), Ok(()));
        referee.download_begun(person, page, b"save").expect("download begins");
        assert_eq!(referee.check_downloads_settled(), Err("each download begun ends once".into()));
        referee.event(&Event::Closed { owner: person }, 0).expect("person closes");
        assert_eq!(referee.check_downloads_settled(), Ok(()));
        assert_eq!(
            referee.download_begun(person, page, b"save"),
            Err("no download begins after its person's Closed".into())
        );
        // This also has no matching beginning; the closed-person contract comes first.
        assert_eq!(
            referee.event(
                &Event::Downloaded { person, name: b"save".as_slice().into(), path: b"/save".as_slice().into() },
                0
            ),
            Err("no download after its person's Closed".into())
        );
    }

    #[test]
    fn rejects_two_terminals() {
        let mut referee = Referee::new();
        referee.event(&Event::Done { op: Token::new(4) }, 0).expect("first terminal");
        let finding = referee.event(&Event::Done { op: Token::new(4) }, 0).expect_err("second terminal");
        assert!(finding.starts_with("one terminal per operation"), "{finding}");
    }

    #[test]
    fn rejects_event_after_browser_closed() {
        let mut referee = Referee::new();
        referee.event(&Event::Closed { owner: Token::new(1) }, 0).expect("browser closes");
        assert_eq!(
            referee.event(&Event::Ready { version: b"late".to_vec().into_boxed_slice() }, 0),
            Err("nothing is emitted after Closed".into())
        );
    }

    #[test]
    fn rejects_mouse_before_refused_press() {
        let mut referee = Referee::new();
        referee.press(Token::new(4)).expect("register press");
        referee.command(br#"{"method":"Input.dispatchMouseEvent"}"#);
        let finding = referee
            .event(&Event::Refused { op: Token::new(4), why: Refusal::Covered, detail: Box::default() }, 0)
            .expect_err("refused press dispatched input");
        assert!(finding.starts_with("refused press dispatched a mouse event"), "{finding}");
    }

    #[test]
    fn rejects_false_met() {
        let mut referee = Referee::new();
        referee.await_op(Token::new(4), Expect::Present).expect("register await");
        assert_eq!(
            referee.event(&Event::Met { op: Token::new(4), seen: List::with_capacity(0) }, 0),
            Err("Met requires the fake page to satisfy its expectation".into())
        );
    }
}
