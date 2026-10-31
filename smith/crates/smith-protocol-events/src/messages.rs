//! Host message content waiting for the run's admission or refusal fact.
//! The service copies decoded labels and text immediately before their domain
//! hand-off. Arrival order, including repeated names, joins each fact once;
//! this book knows neither admission policy nor rendered provider content.
//! Contracts: smith/docs/design/protocol/events.md, sections 3.7 and 5.3; smith/docs/design/domain/run.md, section 6.

use alloc::boxed::Box;

use skein_lib::{Queue, Token};

use crate::sinks::Capture;

/// One service-decoded message, retained until its run receipt.
#[derive(Debug)]
pub(crate) struct Message {
    name: Token,
    pub(crate) label: Box<[u8]>,
    pub(crate) text: Box<[u8]>,
}

/// Decoded content waiting for one run receipt, under Everything only.
#[derive(Debug)]
pub(crate) struct Book {
    capture_enabled: bool,
    pending: Queue<Message>,
}

impl Book {
    #[cfg(test)]
    pub(crate) fn disabled() -> Book {
        Book::new(Capture::None)
    }

    pub(crate) fn new(capture: Capture) -> Book {
        let capture_enabled = match capture {
            Capture::None | Capture::Calls => false,
            Capture::Everything => true,
        };
        Book { capture_enabled, pending: Queue::uncapped() }
    }

    pub(crate) fn received(&mut self, name: Token, label: &[u8], text: &[u8]) {
        if !self.capture_enabled {
            return;
        }
        self.pending.push(Message { name, label: Box::from(label), text: Box::from(text) });
    }

    pub(crate) fn take(&mut self, name: Token) -> Option<Message> {
        if !self.capture_enabled {
            return None;
        }
        let message = self.pending.pop().expect("the service observed the message before its receipt");
        assert_eq!(message.name, name, "run receipts retain message arrival order even for repeated names");
        Some(message)
    }

    pub(crate) fn clear(&mut self) {
        for _ in 0..self.pending.len() {
            self.pending.pop().expect("each captured message present at entry is cleared once");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_names_keep_each_original_label_and_text_until_its_receipt() {
        let mut book = Book::new(Capture::Everything);
        let name = Token::new(4);
        book.received(name, b"task", b"first input");
        book.received(name, b"relay", b"second, refused input");
        let first = book.take(name).unwrap();
        assert_eq!(first.label.as_ref(), b"task");
        assert_eq!(first.text.as_ref(), b"first input");
        let second = book.take(name).unwrap();
        assert_eq!(second.label.as_ref(), b"relay");
        assert_eq!(second.text.as_ref(), b"second, refused input");
        assert!(book.pending.is_empty());
        book.received(name, b"never admitted", b"start refused");
        book.clear();
        assert!(book.pending.is_empty());
        book.received(name, b"task", b"next run");
        assert_eq!(book.take(name).unwrap().text.as_ref(), b"next run");
    }

    #[test]
    fn absent_content_selection_owns_no_message_copy() {
        for capture in [Capture::None, Capture::Calls] {
            let mut book = Book::new(capture);
            book.received(Token::new(4), b"task", b"text");
            assert!(book.take(Token::new(4)).is_none());
            assert!(book.pending.is_empty());
        }
    }
}
