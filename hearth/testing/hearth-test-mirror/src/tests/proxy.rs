use crate::{
    Method, Served,
    proxy::answer,
    target,
    testing::{self as fixtures, FILE, URL},
};
use std::{
    cell::RefCell,
    io::{self, Read, Write},
    path::PathBuf,
};

const HEAD_ONLY: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\n";
const NOT_FOUND: &[u8] = b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
const XZ: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nxz\n";
const TARGETS: &[(&str, Method, &str)] = &[("GET", Method::Get, FILE), ("HEAD", Method::Head, FILE)];
#[test]
fn a_snapshot_request_names_its_path_below_the_origin() {
    for &(verb, method, path) in TARGETS {
        let head = format!("{verb} {URL} HTTP/1.1\r\n\r\n");
        assert_eq!(target(head.as_bytes()), Some((method, PathBuf::from(path))), "{verb}");
    }
}
const REFUSED: &[&str] = &[
    "GET http://elsewhere.test/archive/file HTTP/1.1\r\n\r\n",
    "GET /archive/file HTTP/1.1\r\n\r\n",
    "POST http://snapshot.debian.org/archive/file HTTP/1.1\r\n\r\n",
    "GET http://snapshot.debian.org/archive/file?x HTTP/1.1\r\n\r\n",
    "GET http://snapshot.debian.org/archive/%61 HTTP/1.1\r\n\r\n",
    "GET http://snapshot.debian.org/archive/file#x HTTP/1.1\r\n\r\n",
    "GET http://snapshot.debian.org/checked/file HTTP/1.1\r\n\r\n",
    "GET http://snapshot.debian.org:80/archive/file HTTP/1.1\r\n\r\n",
];
#[test]
fn another_host_origin_form_or_another_method_is_not_found() {
    for head in REFUSED {
        assert_eq!(target(head.as_bytes()), None, "{head}");
    }
}
#[test]
fn a_by_hash_request_is_answered_the_file_it_names() {
    let url = format!(
        "http://snapshot.debian.org/{}/main/binary-amd64/by-hash/SHA256/{}",
        fixtures::SNAPSHOT,
        fixtures::XZ_SHA256
    );
    assert_eq!(request(&fixtures::get_head(&url)), XZ);
}
#[test]
fn a_head_request_is_answered_the_head_alone() {
    let head = format!("HEAD {URL} HTTP/1.1\r\nRange: bytes=0-1\r\nIf-None-Match: anything\r\n\r\n");
    assert_eq!(request(&head), HEAD_ONLY);
}
const LOGGED: &[(&str, u16, u64)] = &[(URL, 200, 7), ("/missing", 404, 0)];
#[test]
fn each_answer_is_logged_before_its_connection_closes() {
    for &(url, status, length) in LOGGED {
        let (_, events) = exchange(&fixtures::get_head(url), End::Eof);
        assert_eq!(
            events,
            [Event::Logged(Served { target: url.as_bytes().into(), status, length }), Event::Closed],
            "{url}"
        );
    }
}
#[test]
fn a_stalled_head_is_answered_404() {
    let (answer, _) = exchange(&format!("GET {URL} HTTP/1.1\r\n"), End::Timeout);
    assert_eq!(answer, NOT_FOUND);
}
const REFUSED_FILES: &[&str] = &[
    "http://snapshot.debian.org/checked/main/binary-amd64/Packages",
    "http://snapshot.debian.org/archive/../checked/main/binary-amd64/Packages",
    "http://snapshot.debian.org/archive/missing",
];
#[test]
fn an_unavailable_or_unserved_file_is_answered_404() {
    for url in REFUSED_FILES {
        assert_eq!(request(&fixtures::get_head(url)), NOT_FOUND, "{url}");
    }
}
#[test]
fn a_head_past_the_limit_is_answered_404() {
    let mut head = format!("GET {URL} HTTP/1.1\r\nX: ");
    head.extend(std::iter::repeat_n('a', crate::HEAD - head.len()));
    assert_eq!(request(&head), NOT_FOUND);
}

enum End {
    Eof,
    Timeout,
}

#[derive(Debug, PartialEq, Eq)]
enum Event {
    Logged(Served),
    Closed,
}

struct Stream<'a> {
    input: &'a [u8],
    end: End,
    output: &'a mut Vec<u8>,
    events: &'a RefCell<Vec<Event>>,
}

impl Read for Stream<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.input.is_empty() && matches!(self.end, End::Timeout) {
            return Err(io::ErrorKind::TimedOut.into());
        }
        self.input.read(buf)
    }
}

impl Write for Stream<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}

impl Drop for Stream<'_> {
    fn drop(&mut self) {
        self.events.borrow_mut().push(Event::Closed);
    }
}

fn exchange(head: &str, end: End) -> (Vec<u8>, Vec<Event>) {
    let events = RefCell::new(Vec::new());
    let mut output = Vec::new();
    let stream = Stream { input: head.as_bytes(), end, output: &mut output, events: &events };
    answer(stream, Ok(()), fixtures::file, |served| events.borrow_mut().push(Event::Logged(served)));
    (output, events.into_inner())
}

fn request(head: &str) -> Vec<u8> {
    exchange(head, End::Eof).0
}
