//! Finding answers in the tool's words (domain/agent/tools.md; client/ctl/tools.md).
use crate::{Problem, answers::Text};
use alloc::boxed::Box;
use skein_lib::{List, Writer, bytes};
use smith_domain_run::HostAnswer;
use temper_engine_domain::FindingToolAnswer;
use temper_engine_domain_forge::{self as f, settings::AreaOwner};

pub(crate) fn render(answer: &FindingToolAnswer, cap: usize) -> HostAnswer {
    if let FindingToolAnswer::List(doc) = answer { return list(doc, cap); }
    let mut text = Text::new();
    let result = match answer {
        FindingToolAnswer::Write(reply) => write(&mut text, reply),
        FindingToolAnswer::Show { project, number, finding } => shown(&mut text, *project, *number, finding.as_ref()),
        FindingToolAnswer::List(_) => unreachable!("list uses whole rows"),
    };
    let result = match result { Ok(error) => text.finish(error), Err(why) => Err(why) };
    match result {
        Ok(answer) if answer.text().len() <= cap => answer,
        Ok(answer) if answer.error() => clipped(answer.text(), cap, true),
        Ok(_) | Err(_) => clipped(b"Finding answer exceeds the host answer limit; use finding list.", cap, true),
    }
}
fn shown(text: &mut Text, project: u32, number: u32, finding: Option<&f::FindingAnswer>) -> Result<bool, Problem> {
    match finding {
        Some(finding) => { show(text, project, finding)?; Ok(false) }
        None => { text.add(b"no finding ")?; identity(text, project, number)?; Ok(true) }
    }
}
fn clipped(text: &[u8], cap: usize, error: bool) -> HostAnswer {
    let mut end = text.len().min(cap);
    while end > 0 && (match text.get(end) { Some(byte) => byte & 0xc0 == 0x80, None => false }) { end = end.saturating_sub(1); }
    HostAnswer::new(bytes::copy_of(text.get(..end).expect("bounded text prefix")), error).expect("bounded finding answer")
}
fn identity(text: &mut Text, project: u32, number: u32) -> Result<(), Problem> {
    text.number(u64::from(project))?;
    text.add(b"/")?;
    text.number(u64::from(number))
}
fn holder(text: &mut Text, holder: f::Holder) -> Result<(), Problem> {
    let (kind, number): (&[u8], u64) = match holder {
        f::Holder::Task(n) => (b"task ", n),
        f::Holder::Party(n) => (b"party ", n),
        f::Holder::Role(n) => (b"role ", u64::from(n)),
    };
    text.add(kind)?;
    text.number(number)
}
fn owner(text: &mut Text, owner: Option<AreaOwner>) -> Result<(), Problem> {
    match owner {
        Some(AreaOwner::Task(n)) => holder(text, f::Holder::Task(n)),
        Some(AreaOwner::Party(n)) => holder(text, f::Holder::Party(n)),
        Some(AreaOwner::Role(n)) => holder(text, f::Holder::Role(n)),
        Some(AreaOwner::Charter(n)) => { text.add(b"charter ")?; text.number(u64::from(n)) },
        None => text.add(b"-"),
    }
}
fn write(text: &mut Text, reply: &f::FindingReply) -> Result<bool, Problem> {
    let done = match &reply.result {
        Ok(done) => done,
        Err(why) => {
            text.literal(&why.words(reply.project, reply.number, reply.area.as_deref().unwrap_or(b"")))?;
            return Ok(true);
        }
    };
    text.add(match done {
        f::FindingDone::Filed(_) => b"Filed finding ",
        f::FindingDone::Handed(_) => b"Handed finding ",
        f::FindingDone::Routed(_) => b"Routed finding ",
        f::FindingDone::Backlog => b"Finding ",
        f::FindingDone::Dropped => b"Dropped finding ",
    })?;
    identity(text, reply.project, reply.number)?;
    match done {
        f::FindingDone::Handed(held) => {
            text.add(b" to area ")?;
            text.literal(reply.area.as_deref().unwrap_or(b"-"))?;
            text.add(b"; it is ")?;
            holder(text, *held)?;
            text.add(b"'s.")?;
        }
        f::FindingDone::Filed(held) => {
            text.add(b"; it is ")?;
            holder(text, *held)?;
            text.add(b"'s.")?;
        }
        f::FindingDone::Routed(goal) => { text.add(b" to task ")?; text.number(*goal)?; text.add(b".")?; }
        f::FindingDone::Backlog => text.add(b" is in the backlog.")?,
        f::FindingDone::Dropped => text.add(b".")?,
    }
    Ok(false)
}
fn row(text: &mut Text, project: u32, row: &f::FindingRow) -> Result<(), Problem> {
    identity(text, project, row.number)?;
    text.add(b" ")?;
    text.add(match row.state { f::FindingState::New => b"new", f::FindingState::Routed => b"routed", f::FindingState::Backlog => b"backlog", f::FindingState::Dropped => b"dropped" })?;
    text.add(b" ")?;
    text.add(match row.kind { f::FindingKind::Qa => b"qa", f::FindingKind::Flake => b"flake", f::FindingKind::Red => b"red", f::FindingKind::Review => b"review", f::FindingKind::Feedback => b"feedback" })?;
    text.add(b" ")?;
    text.add(match row.severity { f::Severity::Low => b"low", f::Severity::Medium => b"medium", f::Severity::High => b"high" })?;
    text.add(b" ")?;
    text.literal(row.area.as_deref().unwrap_or(b"-"))?;
    text.add(b" ")?;
    holder(text, row.holder)?;
    text.add(b": ")?;
    text.literal(&row.title)
}
fn show(text: &mut Text, project: u32, finding: &f::FindingAnswer) -> Result<(), Problem> {
    row(text, project, &finding.row)?;
    text.add(b"\nfiler: ")?;
    holder(text, match finding.filer { f::Filer::Task(n) => f::Holder::Task(n), f::Filer::Party(n) => f::Holder::Party(n) })?;
    text.add(b"\nat: ")?;
    text.number(finding.row.at.as_nanos())?;
    if let Some(why) = &finding.row.why { text.add(b"\nwhy: ")?; text.literal(why)?; }
    if let Some(goal) = finding.goal { text.add(b"\ngoal: task ")?; text.number(goal)?; }
    if let Some(reason) = &finding.reason { text.add(b"\nreason: ")?; text.literal(reason)?; }
    text.add(b"\n\n")?;
    text.literal(&finding.body)
}
fn list_lines(doc: &f::Findings) -> Result<List<Box<[u8]>>, Problem> {
    let mut lines = List::uncapped();
    for area in &doc.areas {
        let mut text = Text::new();
        text.literal(&area.name)?;
        text.add(b" (")?;
        owner(&mut text, Some(area.owner))?;
        text.add(b", ")?;
        match area.holder { Some(held) => holder(&mut text, held)?, None => text.add(b"-")? }
        text.add(b"): ")?;
        text.literal(&area.words)?;
        lines.push(text.finish(false)?.into_parts().0).expect("bounded project entries");
    }
    let mut text = Text::new();
    text.add(b"triager (")?;
    owner(&mut text, doc.triager)?;
    text.add(b", ")?;
    holder(&mut text, doc.holder)?;
    text.add(b")")?;
    lines.push(text.finish(false)?.into_parts().0).expect("one triager");
    for finding in &doc.rows {
        let mut text = Text::new();
        row(&mut text, doc.project, finding)?;
        lines.push(text.finish(false)?.into_parts().0).expect("bounded project entries");
    }
    Ok(lines)
}
fn more(count: u32) -> Box<[u8]> {
    let mut text = Text::new();
    text.add(b"and ").expect("fixed cut marker");
    text.number(u64::from(count)).expect("bounded count");
    text.add(b" more").expect("fixed cut marker");
    text.finish(false).expect("cut marker").into_parts().0
}
fn list(doc: &f::Findings, cap: usize) -> HostAnswer {
    let Ok(lines) = list_lines(doc) else { return clipped(b"Finding list cannot be rendered.", cap, true) };
    let mut count = 0_u32;
    let mut length = 0_usize;
    for line in &lines {
        let next_count = count.saturating_add(1);
        let next = length.saturating_add(line.len()).saturating_add(usize::from(count != 0));
        let remaining = lines.len().saturating_sub(next_count);
        let reserved = if remaining == 0 { 0 } else { more(remaining).len().saturating_add(1) };
        if next.saturating_add(reserved) > cap { break; }
        length = next;
        count = next_count;
    }
    let mut output = Writer::new(cap);
    for (at, line) in lines.iter().take(skein_lib::usize_of(count)).enumerate() {
        if at != 0 { output.put(b"\n").expect("measured separator"); }
        output.put(line).expect("measured finding line");
    }
    let remaining = lines.len().saturating_sub(count);
    if remaining != 0 {
        if count != 0 { output.put(b"\n").expect("reserved separator"); }
        let marker = more(remaining);
        let prefix = marker.get(..marker.len().min(output.room())).expect("bounded marker prefix");
        output.put(prefix).expect("reserved marker");
    }
    HostAnswer::new(output.finish(), false).expect("bounded finding list")
}
