//! Version 1's report and printed line texts (playground.md, 6.1).
use crate::{bridge, fetch};
use skein_lib::{Writer, base64};
use std::fmt::Write;

pub(super) fn bytes(bytes: &[u8]) -> String {
    let len = u32::try_from(bytes.len()).expect("a log byte field fits u32");
    let len = base64::encoded_len(base64::Alphabet::Standard, len).expect("a log's base64 field fits u32");
    let mut out = Writer::new(usize::try_from(len).expect("encoded length fits usize"));
    base64::encode(base64::Alphabet::Standard, bytes, &mut out).expect("room for the measured encoding");
    String::from_utf8(out.finish().into_vec()).expect("base64 is ASCII")
}

pub(super) fn report(report: &fetch::Report) -> String {
    match report {
        fetch::Report::Answered { exchange, status, body } => {
            format!("answered id={exchange} status={status} body={}\n", bytes(body))
        }
        fetch::Report::Failed { exchange } => format!("failed id={exchange}\n"),
        fetch::Report::StreamOpened => "opened\n".into(),
        fetch::Report::StreamRefused { status } => format!("refused status={status}\n"),
        fetch::Report::StreamPiece { bytes: piece } => format!("piece bytes={}\n", bytes(piece)),
        fetch::Report::StreamEnded => "ended\n".into(),
    }
}

pub(super) fn printed(printed: &bridge::Printed) -> String {
    let stream = match printed.stream {
        bridge::Stream::Output => "output",
        bridge::Stream::Error => "error",
    };
    format!("{stream} process={} line={}\n", bytes(&printed.process), bytes(&printed.line))
}

pub(super) fn digest(said: super::Said) -> String {
    let mut out = String::with_capacity(64);
    for byte in said.0 {
        write!(out, "{byte:02x}").expect("writing to a String cannot fail");
    }
    out
}

pub(super) fn first(build: Option<&[u8]>, seed: u64, start: skein_lib::Wall) -> Vec<u8> {
    let mut out = b"skein-playground-log 1 build=".to_vec();
    out.extend_from_slice(build.unwrap_or(b"-"));
    out.extend_from_slice(format!(" seed={seed} start={}\n", start.as_nanos()).as_bytes());
    out
}

pub(super) fn entry(entry: &super::Entry) -> String {
    use crate::{bridge::Input, transport::StreamDuty};
    let mut out = format!(
        "input step={} instant={} said={} ",
        entry.stamp.step,
        entry.stamp.instant.as_nanos(),
        digest(entry.said)
    );
    match &entry.input {
        Input::Exchange(exchange) => {
            write!(
                out,
                "exchange id={} method={} path={} bound={} body={}",
                exchange.id,
                bytes(exchange.method.as_bytes()),
                bytes(&exchange.path),
                exchange.bound,
                bytes(&exchange.body)
            )
            .expect("writing to a String cannot fail");
            for (name, value) in &exchange.headers {
                header(&mut out, name.as_bytes(), value.as_bytes());
            }
        }
        Input::Abandon { exchange } => {
            write!(out, "abandon id={exchange}").expect("writing to a String cannot fail");
        }
        Input::Stream(stream) => match stream {
            StreamDuty::Open { path, headers, bound } => {
                write!(out, "open path={} bound={bound}", bytes(path)).expect("writing to a String cannot fail");
                for (name, value) in headers {
                    header(&mut out, name, value);
                }
            }
            StreamDuty::Close => out.push_str("close"),
        },
        Input::ReadOn => out.push_str("read-on"),
    }
    out.push('\n');
    out
}

fn header(out: &mut String, name: &[u8], value: &[u8]) {
    write!(out, " header={}:{}", bytes(name), bytes(value)).expect("writing to a String cannot fail");
}

pub(super) fn end(end: super::End) -> String {
    match end {
        super::End::At { step, instant, said } => {
            format!("end step={step} instant={} said={}\n", instant.as_nanos(), digest(said))
        }
        super::End::Cut => "cut\n".into(),
    }
}
