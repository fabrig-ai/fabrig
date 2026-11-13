//! Answers in the red tool's words (domain/forge/reds.md, 3).
use crate::{Problem, answers::Text};
use alloc::boxed::Box;
use skein_lib::{List, Wall, Writer, bytes};
use smith_domain_run::HostAnswer;
use temper_engine_domain::RedToolAnswer;
use temper_engine_domain_forge as f;

pub(crate) fn render(answer: &RedToolAnswer, cap: usize) -> HostAnswer {
    if let RedToolAnswer::List { goal, bound, reds } = answer {
        return list(*goal, *bound, reds, cap);
    }
    let mut text = Text::new();
    let result = match answer {
        RedToolAnswer::Write(reply) => write(&mut text, reply),
        RedToolAnswer::NoGoal { task } => match text.literal(&f::RedRefusal::NoGoal.words(*task, b"")) {
            Ok(()) => Ok(true), Err(why) => Err(why),
        },
        RedToolAnswer::List { .. } => unreachable!("list uses whole rows"),
    };
    let answer = match result {
        Ok(error) => text.finish(error),
        Err(why) => Err(why),
    };
    match answer {
        Ok(answer) if answer.text().len() <= cap => answer,
        Ok(answer) if answer.error() => clipped(answer.text(), cap, true),
        Ok(_) | Err(_) => clipped(b"Red answer exceeds the host answer limit; use red list.", cap, true),
    }
}
fn write(text: &mut Text, reply: &f::RedReply) -> Result<bool, Problem> {
    match &reply.result {
        Err(why) => { text.literal(&why.words(reply.goal, &reply.key))?; return Ok(true); }
        Ok(f::RedOutcome::Recorded { head, recorded }) => {
            text.add(b"Recorded red ")?;
            text.literal(&reply.key)?;
            text.add(b" at ")?;
            hex(text, head)?;
            text.add(b" (generation ")?;
            text.number(recorded.generation)?;
            text.add(b"); a check whose base contains ")?;
            hex(text, head)?;
            text.add(b" passes it.")?;
            if let Some(old) = &recorded.replaced {
                text.add(b" It replaced the one recorded at ")?;
                hex(text, &old.head)?;
                text.add(b" (generation ")?;
                text.number(old.generation)?;
                text.add(b").")?;
            }
        }
        Ok(f::RedOutcome::Ended(red)) => {
            text.add(b"Ended red ")?;
            text.literal(&red.key)?;
            text.add(b", recorded at ")?;
            hex(text, &red.head)?;
            text.add(b" by ")?;
            who(text, red.by)?;
            text.add(b" on ")?;
            date(text, red.at)?;
            text.add(b": ")?;
            text.literal(&red.why)?;
            text.add(b".")?;
        }
    }
    Ok(false)
}
fn hex(text: &mut Text, head: &[u8]) -> Result<(), Problem> {
    const DIGITS: &[u8] = b"0123456789abcdef";
    for &byte in head {
        text.add(&[*DIGITS.get(usize::from(byte >> 4_u8)).expect("high nibble"), *DIGITS.get(usize::from(byte & 15)).expect("low nibble")])?;
    }
    Ok(())
}
fn who(text: &mut Text, by: f::Recorder) -> Result<(), Problem> {
    let (kind, number): (&[u8], u64) = match by { f::Recorder::Task(n) => (b"task ", n), f::Recorder::Party(n) => (b"party ", n) };
    text.add(kind)?;
    text.number(number)
}
fn date(text: &mut Text, nanos: u64) -> Result<(), Problem> {
    let wall = Wall::from_nanos(nanos);
    let civil = wall.civil();
    let minutes = wall.as_secs().div_euclid(60);
    text.number(u64::from(civil.year))?;
    for (separator, value) in [(b"-".as_slice(), u64::from(civil.month)), (b"-", u64::from(civil.day)), (b" ", minutes.div_euclid(60).rem_euclid(24)), (b":", minutes.rem_euclid(60))] {
        text.add(separator)?;
        if value < 10 { text.add(b"0")?; }
        text.number(value)?;
    }
    Ok(())
}
fn row(red: &f::Red) -> Result<Box<[u8]>, Problem> {
    let mut text = Text::new();
    text.literal(&red.key)?;
    text.add(b" at ")?;
    hex(&mut text, &red.head)?;
    text.add(b", generation ")?;
    text.number(red.generation)?;
    text.add(b", by ")?;
    who(&mut text, red.by)?;
    text.add(b", ")?;
    date(&mut text, red.at)?;
    text.add(b": ")?;
    text.literal(&red.why)?;
    Ok(text.finish(false)?.into_parts().0)
}
fn more(count: usize) -> Box<[u8]> {
    let mut text = Text::new();
    text.add(b"and ").expect("fixed marker");
    text.number(u64::try_from(count).expect("bounded red count")).expect("count");
    text.add(b" more").expect("fixed marker");
    text.finish(false).expect("marker").into_parts().0
}
fn list(goal: u64, bound: u32, reds: &[f::Red], cap: usize) -> HostAnswer {
    let mut text = Text::new();
    text.add(b"Goal ").expect("header");
    text.number(goal).expect("goal");
    if reds.is_empty() {
        text.add(b" holds no reds.").expect("header");
        return clipped(text.finish(false).expect("header").text(), cap, false);
    }
    text.add(b" holds ").expect("header");
    text.number(u64::try_from(reds.len()).expect("bounded reds")).expect("count");
    text.add(b" reds (at most ").expect("header");
    text.number(u64::from(bound)).expect("bound");
    text.add(b"):").expect("header");
    let header = text.finish(false).expect("header").into_parts().0;
    let mut lines = List::uncapped();
    lines.push(header).expect("header");
    for red in reds { lines.push(row(red).expect("bounded red row")).expect("bounded reds"); }
    let mut count = 0_usize;
    let mut length = 0_usize;
    for line in &lines {
        let next_count = count.saturating_add(1);
        let next = length.saturating_add(line.len()).saturating_add(usize::from(count != 0));
        let remaining = reds.len().saturating_sub(next_count.saturating_sub(1));
        let reserved = if remaining == 0 { 0 } else { more(remaining).len().saturating_add(1) };
        if next.saturating_add(reserved) > cap { break; }
        length = next;
        count = next_count;
    }
    let remaining = reds.len().saturating_sub(count.saturating_sub(1));
    let size = if remaining == 0 { length } else { length.saturating_add(usize::from(count != 0)).saturating_add(more(remaining).len()).min(cap) };
    let mut output = Writer::new(size);
    for (at, line) in lines.iter().take(count).enumerate() {
        if at != 0 { output.put(b"\n").expect("separator"); }
        output.put(line).expect("measured whole line");
    }
    if remaining != 0 {
        if count != 0 { output.put(b"\n").expect("reserved separator"); }
        let marker = more(remaining);
        output.put(marker.get(..marker.len().min(output.room())).expect("marker prefix")).expect("reserved marker");
    }
    HostAnswer::new(output.finish(), false).expect("bounded red list")
}
fn clipped(text: &[u8], cap: usize, error: bool) -> HostAnswer {
    let mut end = text.len().min(cap);
    while end > 0 && (match text.get(end) { Some(byte) => byte & 0xc0 == 0x80, None => false }) { end = end.saturating_sub(1); }
    HostAnswer::new(bytes::copy_of(text.get(..end).expect("bounded prefix")), error).expect("bounded red answer")
}
