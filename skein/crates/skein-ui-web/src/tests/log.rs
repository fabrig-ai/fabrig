//! The session log's folds and versioned text (playground.md, 6.1).
use crate::{
    bridge::{Printed, Stream},
    fetch::Report,
    log::Saying,
};
use skein_lib::sha256;

fn digest_text(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    for byte in bytes {
        write!(out, "{byte:02x}").expect("writing to a String cannot fail");
    }
    out
}

fn printed(stream: Stream) -> Printed {
    Printed { process: b"p".as_slice().into(), stream, line: b"x".as_slice().into() }
}

#[test]
fn the_digest_is_the_designs() {
    // Written directly from playground.md, 6.1, independently of the log writer.
    let reports = sha256::digest(b"answered id=7 status=200 body=b2s=\n");
    let lines = sha256::digest(b"error process=cA== line=eA==\n");
    let mut bytes = Vec::from(reports);
    bytes.extend_from_slice(&lines);
    bytes.push(0);
    let golden = "8a894be3e2c50fe07fc8fdd08e996daff999185b1cb9fe3c75859778de61574e";
    assert_eq!(bytes.len(), 65);
    assert_eq!(digest_text(&sha256::digest(&bytes)), golden);
    let mut saying = Saying::new();
    saying.report(&Report::Answered { exchange: 7, status: 200, body: b"ok".as_slice().into(), headers: Vec::new() });
    saying.printed(&printed(Stream::Error));
    assert_eq!(digest_text(&saying.said().0), golden);
    assert_eq!(
        digest_text(&Saying::new().said().0),
        "82c0f0f259e8cffecead54325fadb48f15a4e761dae5ffaf31209993eacbb24d"
    );
}

#[test]
fn the_digest_folds_each_stream_apart() {
    let first = Report::StreamOpened;
    let second = Report::StreamEnded;
    let output = printed(Stream::Output);
    let error = printed(Stream::Error);
    let mut one = Saying::new();
    one.report(&first);
    one.printed(&output);
    one.report(&second);
    one.printed(&error);
    let mut two = Saying::new();
    two.printed(&output);
    two.printed(&error);
    two.report(&first);
    two.report(&second);
    assert_eq!(one.said(), two.said());
    let mut swapped = Saying::new();
    swapped.report(&second);
    swapped.report(&first);
    swapped.printed(&output);
    swapped.printed(&error);
    assert_ne!(one.said(), swapped.said());
    let mut lines_swapped = Saying::new();
    lines_swapped.report(&first);
    lines_swapped.report(&second);
    lines_swapped.printed(&error);
    lines_swapped.printed(&output);
    assert_ne!(one.said(), lines_swapped.said());
    two.stopped();
    assert_ne!(one.said(), two.said());
    let stopped = two.said();
    two.stopped();
    assert_eq!(two.said(), stopped);
}

const EMPTY: &str = "82c0f0f259e8cffecead54325fadb48f15a4e761dae5ffaf31209993eacbb24d";
const GOLDEN: &str = concat!(
    "skein-playground-log 2 build=abcd seed=7 start=9\n",
    "input step=2 instant=3 said=82c0f0f259e8cffecead54325fadb48f15a4e761dae5ffaf31209993eacbb24d read-on\n",
    "end step=4 instant=5 said=8a894be3e2c50fe07fc8fdd08e996daff999185b1cb9fe3c75859778de61574e\n"
);

use crate::{
    bridge::{Clock, Input, Stamp, Start, ToPage},
    log::{End, Entry, Log},
};
use skein_lib::{Duration, Time, Wall};

fn start() -> Start {
    Start { story: b"story".as_slice().into(), seed: 7, start: Wall::from_nanos(9) }
}
fn clock(step: u64, instant: u64) -> ToPage {
    ToPage::Clock(Clock {
        step,
        now: Time::from_nanos(instant),
        wall: Wall::from_nanos(9_u64.saturating_add(instant)),
        idle: true,
        next_due: None,
        memory: 0,
        slice: Duration::ZERO,
    })
}
fn stamp(step: u64, instant: u64) -> ToPage {
    ToPage::Stamp(Stamp { step, instant: Time::from_nanos(instant) })
}
fn golden_log() -> Log {
    let mut log = Log::new(Some(b"abcd"), &start(), u32::MAX);
    log.sent(&Input::ReadOn);
    log.heard(&stamp(2, 3));
    log.heard(&ToPage::Report(Report::Answered {
        exchange: 7,
        status: 200,
        body: b"ok".as_slice().into(),
        headers: Vec::new(),
    }));
    log.heard(&ToPage::Printed(printed(Stream::Error)));
    log.heard(&clock(4, 5));
    log
}

#[test]
fn the_text_is_the_designs() {
    assert_eq!(golden_log().text().as_ref(), GOLDEN.as_bytes());
}

#[test]
fn a_native_log_names_no_build() {
    let log = Log::new(None, &start(), u32::MAX);
    let expected = format!("skein-playground-log 2 build=- seed=7 start=9\nend step=0 instant=0 said={EMPTY}\n");
    assert_eq!(log.text().as_ref(), expected.as_bytes());
    assert_eq!(read(&log.text()).expect("native log").build, None);
}

#[test]
fn what_comes_after_the_last_clock_is_not_in_the_end() {
    let mut log = golden_log();
    let before = log.text();
    log.heard(&ToPage::Report(Report::StreamEnded));
    log.heard(&ToPage::Printed(printed(Stream::Output)));
    log.heard(&ToPage::Stopped);
    log.heard(&ToPage::Failed { words: b"failure".as_slice().into() });
    assert_eq!(log.text(), before);
    log.heard(&clock(6, 7));
    assert_ne!(log.text(), before);
}

#[test]
fn a_log_past_its_bound_is_cut() {
    // The last line must still fit when both clock fields grow to u64::MAX.
    let first = "skein-playground-log 2 build=- seed=7 start=9\n";
    let input = format!("input step=2 instant=3 said={EMPTY} read-on\n");
    let last = format!("end step={} instant={} said={EMPTY}\n", u64::MAX, u64::MAX);
    let bound = u32::try_from(first.len().saturating_add(input.len()).saturating_add(last.len())).expect("small log");
    let mut exact = Log::new(None, &start(), bound);
    exact.sent(&Input::ReadOn);
    exact.heard(&stamp(2, 3));
    exact.heard(&clock(u64::MAX, u64::MAX));
    assert_eq!(exact.text().len(), usize::try_from(bound).expect("small bound"));
    let mut short = Log::new(None, &start(), bound.checked_sub(1).expect("nonzero bound"));
    short.sent(&Input::ReadOn);
    short.sent(&Input::Abandon { exchange: 42 });
    short.heard(&stamp(2, 3));
    short.heard(&stamp(4, 5));
    assert_eq!(short.text().as_ref(), format!("{first}cut\n").as_bytes());
    // Cut is permanent, including inputs sent after it and clocks heard afterward.
    exact.sent(&Input::ReadOn);
    exact.heard(&stamp(u64::MAX, u64::MAX));
    let cut = exact.text();
    assert_eq!(cut.as_ref(), format!("{first}{input}cut\n").as_bytes());
    exact.sent(&Input::ReadOn);
    exact.heard(&stamp(u64::MAX, u64::MAX));
    exact.heard(&clock(u64::MAX, u64::MAX));
    exact.heard(&ToPage::Report(Report::StreamEnded));
    assert_eq!(exact.text(), cut);
    let read = read(&cut).expect("cut log");
    assert_eq!(read.end, End::Cut);
    assert_eq!(read.entries.len(), 1);
    assert_eq!(read.entries.first().expect("kept input").input, Input::ReadOn);
}

#[test]
#[should_panic(expected = "a stamp has an input waiting")]
fn a_stamp_with_no_input_is_a_broken_contract() {
    Log::new(None, &start(), u32::MAX).heard(&stamp(0, 0));
}

use crate::{
    bridge::Exchange,
    log::{Read, Refused, read},
    transport::StreamDuty,
};

fn inputs() -> Vec<Input> {
    vec![
        Input::Exchange(Exchange {
            id: u32::MAX,
            method: "POST".into(),
            path: b"/api".as_slice().into(),
            bound: u32::MAX,
            body: Box::from([]),
            headers: vec![("content-type".into(), "application/json".into()), (String::new(), "é".into())],
            cache: crate::fetch::Cache::NoStore,
            keep: vec!["cache-control".into(), "age".into()],
        }),
        Input::Exchange(Exchange {
            id: 0,
            method: String::new(),
            path: Box::from([]),
            bound: 0,
            body: b"\xfb\xff\0".as_slice().into(),
            headers: Vec::new(),
            cache: crate::fetch::Cache::Browser,
            keep: Vec::new(),
        }),
        Input::Abandon { exchange: u32::MAX },
        Input::Stream(StreamDuty::Open {
            path: b"/events".as_slice().into(),
            bound: u32::MAX,
            headers: vec![
                (b"accept".as_slice().into(), b"text/event-stream".as_slice().into()),
                (Box::from([]), b"\xfb\xff".as_slice().into()),
            ],
        }),
        Input::Stream(StreamDuty::Open { path: Box::from([]), bound: 0, headers: Vec::new() }),
        Input::Stream(StreamDuty::Close),
        Input::ReadOn,
    ]
}

fn reports() -> Vec<Report> {
    vec![
        Report::Answered {
            exchange: u32::MAX,
            status: u16::MAX,
            body: b"\xfb\xff".as_slice().into(),
            headers: vec![("age".into(), "123".into())],
        },
        Report::Answered { exchange: 0, status: 0, body: Box::from([]), headers: Vec::new() },
        Report::Failed { exchange: 7 },
        Report::StreamOpened,
        Report::StreamRefused { status: 403 },
        Report::StreamPiece { bytes: b"a\0\n".as_slice().into() },
        Report::StreamPiece { bytes: Box::from([]) },
        Report::StreamEnded,
    ]
}

#[test]
fn a_log_reads_back_whole() {
    let start = Start { story: Box::from([]), seed: u64::MAX, start: Wall::from_nanos(u64::MAX) };
    let build = b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let mut log = Log::new(Some(build), &start, u32::MAX);
    let mut expected = Read {
        build: Some(build.as_slice().into()),
        seed: start.seed,
        start: start.start,
        entries: Vec::new(),
        end: End::At { step: 0, instant: Time::ZERO, said: Saying::new().said() },
    };
    let inputs = inputs();
    for input in &inputs {
        log.sent(input);
    }
    let mut fold = Saying::new();
    log.heard(&ToPage::Report(Report::StreamOpened));
    fold.report(&Report::StreamOpened);
    // Until the first clock, the end remains the start, even with something folded.
    assert_eq!(read(&log.text()), Ok(expected.clone()));
    for (index, input) in inputs.into_iter().enumerate() {
        let step = u64::try_from(index).expect("small input list").saturating_add(1);
        let moment = Stamp { step, instant: Time::from_nanos(step.saturating_mul(10)) };
        log.heard(&ToPage::Stamp(moment));
        expected.entries.push(Entry { stamp: moment, input, said: fold.said() });
        fold = Saying::new();
        for report in reports() {
            fold.report(&report);
            log.heard(&ToPage::Report(report));
        }
        for stream in [Stream::Output, Stream::Error] {
            let line = Printed { process: b"p\0\xff".as_slice().into(), stream, line: b"line\n\0".as_slice().into() };
            fold.printed(&line);
            log.heard(&ToPage::Printed(line));
        }
        fold.stopped();
        log.heard(&ToPage::Stopped);
        log.heard(&ToPage::Failed { words: b"ignored".as_slice().into() });
        log.heard(&clock(step, step.saturating_mul(10)));
        expected.end = End::At { step, instant: moment.instant, said: fold.said() };
        assert_eq!(read(&log.text()), Ok(expected.clone()));
    }
    log.heard(&clock(u64::MAX, u64::MAX));
    expected.end = End::At { step: u64::MAX, instant: Time::from_nanos(u64::MAX), said: fold.said() };
    assert_eq!(read(&log.text()), Ok(expected));
}

#[test]
fn every_report_has_the_designs_text() {
    let mut saying = Saying::new();
    for report in reports() {
        saying.report(&report);
    }
    let reports = concat!(
        "answered id=4294967295 status=65535 body=+/8= header=YWdl:MTIz\n",
        "answered id=0 status=0 body=\n",
        "failed id=7\nopened\nrefused status=403\npiece bytes=YQAK\npiece bytes=\nended\n"
    );
    let mut digest = sha256::Sha256::new();
    digest.update(&sha256::digest(reports.as_bytes()));
    digest.update(&sha256::digest(b""));
    digest.update(&[0]);
    assert_eq!(saying.said().0, digest.finish());
}

#[test]
fn another_format_or_version_is_refused() {
    assert_eq!(read(GOLDEN.replacen("skein-playground-log", "another-log", 1).as_bytes()), Err(Refused::Format));
    for version in ["1", "01", "next", "18446744073709551616"] {
        let text = GOLDEN.replacen("log 2 ", &format!("log {version} "), 1);
        assert_eq!(read(text.as_bytes()), Err(Refused::Version { found: version.as_bytes().into() }));
    }
}

#[test]
fn a_malformed_line_is_refused_naming_it() {
    let first = "skein-playground-log 2 build=- seed=7 start=9\n";
    let last = format!("end step=4 instant=5 said={EMPTY}\n");
    let prefix = format!("input step=2 instant=3 said={EMPTY} ");
    let input_cases = [
        "exchange id=1 method=R0VU path=Lw bound=0 body= cache=browser keep=0", // missing padding
        "exchange id=1 path=Lw== method=R0VU bound=0 body= cache=browser keep=0", // order
        "exchange id=1 method=R0VU path=Lw== body= cache=browser keep=0",       // missing bound
        "exchange id=1 method=R0VU path=Lw== bound=0 body= cache=browser keep=0 header=YQ==", // no colon
        "exchange id=1 method=R0VU path=Lw== bound=0 body= cache=browser keep=0 header=YQ==:Yg==:",
        "exchange id=1 method=R0VU path=Lw== bound=0 body= cache=browser keep=0 header=YQ==:Yh==", // bits
        "exchange id=1 method=R0VU path=Lw== bound=0 body= cache=browser keep=0 header=/w==:",     // UTF-8 name
        "exchange id=1 method=R0VU path=Lw== bound=0 body= cache=browser keep=0 header=:/w==",     // UTF-8 value
        "exchange id=1 method=/w== path=Lw== bound=0 body= cache=browser keep=0",                  // UTF-8 method
        "exchange id=4294967296 method= path= bound=0 body= cache=browser keep=0",                 // id overflow
        "exchange id=1 method= path= bound=4294967296 body= cache=browser keep=0",                 // bound overflow
        "exchange id=1 method= path= bound=00 body= cache=browser keep=0",                         // leading zero bound
        "exchange id=1 method= path= bound=0 body=_w== cache=browser keep=0",                      // URL alphabet
        "exchange id=1 method= path= bound=0 body=YQ cache=browser keep=0",                        // no padding
        "exchange id=1 method= path= bound=0 body=YQ=== cache=browser keep=0",                     // excess padding
        "exchange id=1 method= path= bound=0 body=YQ==Yg== cache=browser keep=0",                  // interior padding
        "exchange id=1 method= path= bound=0 body=Yg=A cache=browser keep=0",                      // padding placement
        "exchange id=1 method= path= bound=0 body=A cache=browser keep=0",                         // length
        "exchange id=1 method= path= bound=0 body=? cache=browser keep=0",                         // alphabet
        "exchange id=1 method= path= bound=0 body= body= cache=browser keep=0",                    // duplicate field
        "exchange id=1 method= path= bound=0 body= cache=unknown keep=0",
        "exchange id=1 method= path= bound=0 body= cache=browser keep=01 name=",
        "exchange id=1 method= path= bound=0 body= cache=browser keep=1",
        "exchange id=1 method= path= bound=0 body= cache=browser keep=1 name=/w==",
        "exchange id=1 method= path= bound=0 body= cache=browser keep=0 name=YWdl",
        "abandon id=01",
        "abandon id=4294967296",
        "abandon id=",
        "abandon id=+1",
        "abandon id=-1",
        "abandon id=1.0",
        "abandon",
        "abandon id=1 extra=0",
        "open path= bound=4294967296",
        "open bound=0 path=",
        "open path= bound=0 header=:?",
        "close extra=0",
        "read-on ",
        "read-on\t",
        "unknown",
        "",
        " read-on",
        "read-on\r",
    ];
    for input in input_cases {
        let text = format!("{first}{prefix}{input}\n{last}");
        assert_eq!(read(text.as_bytes()), Err(Refused::Line { line: 2 }), "{input:?}");
    }
    let cases = [
        (GOLDEN.replacen("seed=7", "seed=07", 1), 1),
        (GOLDEN.replacen("seed=7", "seed=18446744073709551616", 1), 1),
        (GOLDEN.replacen("start=9", "start=18446744073709551616", 1), 1),
        (GOLDEN.replacen("seed=7", "seed=", 1), 1),
        (GOLDEN.replacen("build=abcd", "build=", 1), 1),
        (GOLDEN.replacen("build=abcd seed=7", "seed=7 build=abcd", 1), 1),
        (GOLDEN.replacen("log 2 ", "log  ", 1), 1),
        (GOLDEN.replacen("log 2 ", "log 2  ", 1), 1),
        (GOLDEN.replacen("start=9", "start=9 extra=0", 1), 1),
        (GOLDEN.replacen("step=2", "step=02", 1), 2),
        (GOLDEN.replacen("instant=3", "instant=03", 1), 2),
        (GOLDEN.replacen("step=2", "step=18446744073709551616", 1), 2),
        (GOLDEN.replacen("instant=3", "instant=18446744073709551616", 1), 2),
        (GOLDEN.replacen(EMPTY, &EMPTY.to_uppercase(), 1), 2),
        (GOLDEN.replacen(EMPTY, "bad", 1), 2),
        (GOLDEN.replacen(EMPTY, &"g".repeat(64), 1), 2),
        (GOLDEN.replacen("step=2 instant=3", "instant=3 step=2", 1), 2),
        (GOLDEN.replace("end step=4", "end step=04"), 3),
        (GOLDEN.replace("end step=4 instant=5", "end instant=5 step=4"), 3),
        (GOLDEN.replace("instant=5", "instant=18446744073709551616"), 3),
        (format!("{GOLDEN}cut\n"), 4),
        (format!("{GOLDEN}\n"), 4),
        (first.to_owned(), 2),
        (format!("{first}{prefix}read-on\n"), 3),
        (format!("{first}cut extra=0\n"), 2),
        (format!("{first}cut\n{last}"), 3),
        (String::new(), 1),
    ];
    for (text, line) in cases {
        assert_eq!(read(text.as_bytes()), Err(Refused::Line { line }), "{text:?}");
    }
    assert_eq!(read(GOLDEN.trim_end().as_bytes()), Err(Refused::Line { line: 3 }));
    assert_eq!(read(first.trim_end().as_bytes()), Err(Refused::Line { line: 1 }));
    let mut non_ascii = GOLDEN.as_bytes().to_vec();
    *non_ascii.first_mut().expect("nonempty text") = 0xff;
    assert_eq!(read(&non_ascii), Err(Refused::Line { line: 1 }));
}
