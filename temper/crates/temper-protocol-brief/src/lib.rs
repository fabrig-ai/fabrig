//! Fit a charter's brief to the room its run leaves
//! (temper/docs/design/domain/agent/tools.md).

#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use skein_lib::{Decimal, List, Writer};
use smith_domain_run as run;

/// Fit the brief after the workspace is prepared, using the receiving run's
/// bounds and schema costs (temper/docs/design/domain/agent/tools.md).
pub fn fit_brief(charter: &mut run::Charter, workspace: Option<&run::Workspace>, limits: &run::Limits) {
    let Some(room) = run::brief_room(charter, workspace, limits) else { return };
    let owned = charter
        .owned_bytes()
        .expect("bounded charter ownership")
        .min(run::charter_room(workspace, limits).unwrap_or(0));
    fit(charter, u64::from(room), owned);
}

/// Fit gathered sections to a total decoded charter ownership allowance before
/// assignment admission. Fixed instructions, tools and contracts stay whole;
/// the caller reserves any additional encoded framing (domain/agent/tools.md).
pub fn fit_brief_bytes(charter: &mut run::Charter, bytes: u64) {
    let owned = charter.owned_bytes().expect("bounded charter ownership").min(bytes);
    fit(charter, u64::MAX, owned);
}

fn fit(charter: &mut run::Charter, room: u64, owned: u64) {
    let mut omitted = List::with_capacity(u32::try_from(charter.brief.sections.len()).expect("bounded sections"));
    for index in (0..charter.brief.sections.len()).rev() {
        if run::brief_bytes(&charter.brief).expect("bounded brief bytes") <= room
            && charter.owned_bytes().expect("bounded charter ownership") <= owned
        {
            break;
        }
        let section = charter.brief.sections.get_mut(index).expect("brief section index");
        let text = core::mem::replace(&mut section.text, Box::new([]));
        // A marker ends in a newline, so it needs one fewer paragraph byte
        // than the empty section measured here by smith's renderer.
        let fixed = run::brief_bytes(&charter.brief).expect("bounded brief bytes").saturating_sub(1);
        // A cut line must also fit the ownership allowance under which the
        // host admitted the charter.
        let owned_fixed = charter.owned_bytes().expect("bounded charter ownership");
        let available = usize::try_from(room.saturating_sub(fixed).min(owned.saturating_sub(owned_fixed)))
            .expect("room bounded by u32");
        let section = charter.brief.sections.get_mut(index).expect("brief section index");
        match shortened(&text, &section.title, available) {
            Some(text) => section.text = text,
            None => {
                omitted.push(core::mem::replace(&mut section.title, Box::new([]))).expect("one title per section");
                let mut kept = List::with_capacity(u32::try_from(index).expect("bounded sections"));
                for section in core::mem::replace(&mut charter.brief.sections, Box::new([])).into_iter().take(index) {
                    kept.push(section).expect("the sections before the omitted one");
                }
                charter.brief.sections = kept.into_boxed();
            }
        }
    }
    note_omitted(charter, omitted.as_slice(), room, owned);
}

fn shortened(text: &[u8], title: &[u8], available: usize) -> Option<Box<[u8]>> {
    let mut kept = text.len().saturating_sub(1).min(available);
    // Bounded: each pass removes at least one original byte, or returns.
    loop {
        // The translated text is UTF-8; a continuation byte cannot start
        // the suffix removed from it.
        while kept > 0 && continuation(text, kept) {
            kept = kept.saturating_sub(1);
        }
        let prefix = text.get(..kept).expect("retained prefix");
        let separator: &[u8] = if prefix.is_empty() || prefix.ends_with(b"\n") { b"" } else { b"\n" };
        let marker = cut_line(text.len().saturating_sub(kept), title);
        let length = kept.saturating_add(separator.len()).saturating_add(marker.len());
        if length <= available {
            let mut body = Writer::new(length);
            body.put(prefix).expect("retained text fits");
            body.put(separator).expect("separator fits");
            body.put(&marker).expect("cut line fits");
            return Some(body.finish());
        }
        if kept == 0 {
            return None;
        }
        kept = kept.saturating_sub(length.saturating_sub(available));
    }
}

fn note_omitted(charter: &mut run::Charter, titles: &[Box<[u8]>], room: u64, owned: u64) {
    if titles.is_empty() {
        return;
    }
    let count = Decimal::of(skein_lib::len_u64(titles.len()));
    let mut length = b"[ sections omitted: ; whole: the task's activity]\n"
        .len()
        .saturating_add(count.as_bytes().len())
        .saturating_add(titles.len().saturating_sub(1).saturating_mul(2));
    for title in titles {
        length = length.saturating_add(title.len());
    }
    let mut line = Writer::new(length);
    line.put(b"[").expect("omission line");
    line.put(count.as_bytes()).expect("omitted count");
    line.put(b" sections omitted: ").expect("omission line");
    for (index, title) in titles.iter().rev().enumerate() {
        if index > 0 {
            line.put(b", ").expect("title separator");
        }
        line.put(title).expect("omitted title");
    }
    line.put(b"; whole: the task's activity]\n").expect("omission line");
    let line = line.finish();
    let previous = match charter.brief.sections.last_mut() {
        Some(section) => {
            let previous = core::mem::replace(&mut section.text, Box::new([]));
            let separator: &[u8] = if previous.is_empty() || previous.ends_with(b"\n") { b"" } else { b"\n" };
            let mut body = Writer::new(previous.len().saturating_add(separator.len()).saturating_add(line.len()));
            body.put(&previous).expect("retained text");
            body.put(separator).expect("omission separator");
            body.put(&line).expect("omission line");
            section.text = body.finish();
            Some(previous)
        }
        None => {
            charter.brief.sections = Box::new([run::Section { title: Box::new([]), text: line }]);
            None
        }
    };
    if run::brief_bytes(&charter.brief).expect("bounded brief bytes") > room
        || charter.owned_bytes().expect("bounded charter ownership") > owned
    {
        match previous {
            Some(text) => charter.brief.sections.last_mut().expect("retained last section").text = text,
            None => charter.brief.sections = Box::new([]),
        }
    }
}

fn continuation(text: &[u8], at: usize) -> bool {
    match text.get(at) {
        Some(byte) => byte & 0xc0 == 0x80,
        None => false,
    }
}

fn cut_line(lost: usize, title: &[u8]) -> Box<[u8]> {
    let read: &[u8] = match title {
        b"CI" => b"read_job/read_checks",
        b"Pull request" => b"read_pull/read_pull_files",
        b"Reviews" => b"read_reviews/read_remarks",
        b"Home" => b"read_docs",
        b"Notes" => b"recall",
        b"Plan" => b"the task's plan",
        b"Task" | b"Lineage" | b"Dependencies" | b"Earlier attempts" | b"Calls" | b"Waiting" | b"Transcript tail" => {
            b"the task's activity"
        }
        _ => b"read_docs or the task's activity",
    };
    let lost = Decimal::of(u64::try_from(lost).expect("text length fits u64"));
    let mut line =
        Writer::new(b"[cut  bytes; whole: ]\n".len().saturating_add(lost.as_bytes().len()).saturating_add(read.len()));
    line.put(b"[cut ").expect("cut line");
    line.put(lost.as_bytes()).expect("lost byte count");
    line.put(b" bytes; whole: ").expect("cut line");
    line.put(read).expect("recovery read");
    line.put(b"]\n").expect("cut line");
    line.finish()
}
