//! Tool line forms mirror ctl without a client dependency (domain/forge/follows.md, 7).
use crate::{Problem, answers::Text};
use alloc::boxed::Box;
use skein_lib::{List, Writer, bytes};
use smith_domain_run::HostAnswer;
use temper_engine_domain::FollowToolAnswer;
use temper_engine_domain_forge::{self as f, settings as s};

pub(crate) fn render(answer: &FollowToolAnswer, cap: usize) -> HostAnswer {
    match answer {
        FollowToolAnswer::Write { follow, refusal } => {
            let mut text = Text::new();
            let result = if let Some(words) = refusal { text.literal(words) } else { kept(&mut text, *follow) };
            finish(text, result, cap, refusal.is_some())
        }
        FollowToolAnswer::List(view) => {
            let mut lines = List::uncapped();
            if view.follows.is_empty() {
                return clipped(b"no follows", cap, false);
            }
            for row in &view.follows {
                let mut text = Text::new();
                if line(&mut text, row).is_err() || push(&mut lines, text).is_err() {
                    return clipped(b"Follow list cannot be rendered.", cap, true);
                }
            }
            fitted(&lines, cap)
        }
        FollowToolAnswer::Show { view, .. } => match view {
            Some(row) => match show(row) {
                Ok(lines) => fitted(&lines, cap),
                Err(_) => clipped(b"Follow show cannot be rendered.", cap, true),
            },
            None => clipped(b"no such follow", cap, true),
        },
    }
}
fn kept(text: &mut Text, follow: u32) -> Result<(), Problem> {
    text.add(b"Kept: follow ")?;
    text.number(u64::from(follow))?;
    text.add(b"'s next step applies it.")
}
fn finish(text: Text, result: Result<(), Problem>, cap: usize, error: bool) -> HostAnswer {
    match result {
        Ok(()) => match text.finish(error) {
            Ok(answer) => clipped(answer.text(), cap, error),
            Err(_) => clipped(b"Follow answer cannot be rendered.", cap, true),
        },
        Err(_) => clipped(b"Follow answer cannot be rendered.", cap, true),
    }
}
fn clipped(text: &[u8], cap: usize, error: bool) -> HostAnswer {
    let mut end = text.len().min(cap);
    while end > 0
        && match text.get(end) {
            Some(byte) => byte & 0xc0 == 0x80,
            None => false,
        }
    {
        end = end.saturating_sub(1);
    }
    HostAnswer::new(bytes::copy_of(text.get(..end).expect("bounded prefix")), error).expect("bounded follow answer")
}
fn after12(text: &mut Text, after: &[u8; 32]) -> Result<(), Problem> {
    text.add(f::commit_text(after).get(..12).expect("full commit"))
}
fn parity(text: &mut Text, parity: Option<&f::FollowParity>) -> Result<(), Problem> {
    let Some(parity) = parity else { return text.add(b"no check yet") };
    match parity.verdict {
        f::FollowVerdict::Passed => text.add(b"check passes")?,
        f::FollowVerdict::Failing(count) => {
            text.number(u64::from(count))?;
            text.add(b" failing")?;
        }
        f::FollowVerdict::Many => text.add(b"many failing")?,
    }
    text.add(b" at ")?;
    after12(text, &parity.source)
}
fn line(text: &mut Text, row: &f::FollowView) -> Result<(), Problem> {
    text.add(b"follow ")?;
    text.number(u64::from(row.number))?;
    text.add(b": ")?;
    text.literal(&row.source)?;
    text.add(b" ")?;
    text.literal(&row.source_branch)?;
    text.literal(" → ".as_bytes())?;
    text.literal(&row.target)?;
    text.add(b" ")?;
    text.literal(&row.target_branch)?;
    text.literal(" · ".as_bytes())?;
    if row.lag == 0 {
        text.add(b"caught up")?;
    } else {
        text.number(u64::from(row.lag))?;
        text.add(b" behind")?;
    }
    text.literal(" · ".as_bytes())?;
    parity(text, row.parity.as_ref())
}
fn state(text: &mut Text, entry: &f::FollowLagEntry) -> Result<(), Problem> {
    match &entry.state {
        f::FollowState::NotMade => text.add(b"not made yet"),
        f::FollowState::WaitsWord => text.add(b"waits for its owner's word"),
        f::FollowState::HeldByOwner(words) => {
            text.add(b"held by its owner: ")?;
            text.literal(words)
        }
        f::FollowState::WaitsFor(change) => {
            text.add(b"waits for change ")?;
            text.number(*change)
        }
        f::FollowState::Producing => match entry.change {
            Some(change) => {
                text.add(b"producing as change ")?;
                text.number(change)
            }
            None => text.add(b"not made yet"),
        },
        f::FollowState::Held(words) => {
            text.add(b"held: ")?;
            text.literal(words)
        }
        f::FollowState::Ended(words) => {
            text.add(b"ended without landing: ")?;
            text.literal(words)
        }
    }
}
fn repository(text: &mut Text, repository: skein_forge::api::Repository) -> Result<(), Problem> {
    text.number(u64::from(repository.forge))?;
    text.add(b"/")?;
    text.number(u64::from(repository.repository))
}
fn push(lines: &mut List<Box<[u8]>>, text: Text) -> Result<(), Problem> {
    lines.push(text.finish(false)?.into_parts().0).expect("bounded follow lines");
    Ok(())
}
fn show(row: &f::FollowView) -> Result<List<Box<[u8]>>, Problem> {
    let mut lines = List::uncapped();
    let mut text = Text::new();
    line(&mut text, row)?;
    text.add(b"\nsource: ")?;
    text.literal(&row.source)?;
    text.add(b" (")?;
    repository(&mut text, row.setting.source.repository)?;
    text.add(b")\nbranch: ")?;
    text.literal(&row.source_branch)?;
    text.add(b"\npaths:")?;
    if row.setting.source.paths.is_empty() {
        text.add(b" every path")?;
    }
    for path in &row.setting.source.paths {
        text.add(b" ")?;
        text.literal(path)?;
    }
    text.add(b"\ntarget: ")?;
    text.literal(&row.target)?;
    text.add(b" (")?;
    repository(&mut text, row.setting.target.repository)?;
    text.add(b")\nbranch: ")?;
    text.literal(&row.target_branch)?;
    text.add(b"\nowner: ")?;
    match row.setting.owner {
        s::FollowOwner::Person(person) => {
            text.add(b"person ")?;
            text.number(person)?;
        }
        s::FollowOwner::Task(task) => {
            text.add(b"task ")?;
            text.number(task)?;
        }
    }
    text.add(b"\nstart: ")?;
    text.add(match row.setting.start {
        s::FollowStart::Owner => b"owner",
        s::FollowStart::Code => b"code",
    })?;
    text.add(b"\nproducer: ")?;
    text.literal(row.setting.producer.as_deref().unwrap_or(b"default"))?;
    text.add(b"\ncheck: ")?;
    text.literal(&row.setting.check.workflow)?;
    text.literal(" · artifact ".as_bytes())?;
    text.literal(&row.setting.check.artifact)?;
    text.literal(" · ".as_bytes())?;
    text.add(match row.setting.check.format {
        temper_engine_domain_forge_tiers::Format::Junit => b"junit",
        temper_engine_domain_forge_tiers::Format::Lines => b"lines",
    })?;
    text.add(b"\ntask: ")?;
    match row.task {
        Some(task) => text.number(task)?,
        None => text.add(b"not made yet")?,
    }
    if let Some(stopped) = &row.stopped {
        text.add(b"\nstopped: ")?;
        text.literal(stopped)?;
    }
    text.add(b"\nlag: ")?;
    text.number(u64::from(row.lag))?;
    push(&mut lines, text)?;
    for entry in &row.entries {
        let mut text = Text::new();
        text.add(b"  ")?;
        after12(&mut text, &entry.after)?;
        if let Some(change) = entry.change {
            text.literal(" · change ".as_bytes())?;
            text.number(change)?;
        }
        text.literal(" · ".as_bytes())?;
        state(&mut text, entry)?;
        push(&mut lines, text)?;
    }
    lines
        .push(if row.skips.is_empty() { b"skips: none".as_slice().into() } else { b"skips:".as_slice().into() })
        .expect("one heading");
    for skip in &row.skips {
        let mut text = Text::new();
        text.add(b"  ")?;
        after12(&mut text, &skip.after)?;
        text.literal(" · ".as_bytes())?;
        text.literal(&skip.reason)?;
        push(&mut lines, text)?;
    }
    let mut text = Text::new();
    text.add(b"parity: ")?;
    parity(&mut text, row.parity.as_ref())?;
    if let Some(parity) = &row.parity {
        text.literal(" · target ".as_bytes())?;
        after12(&mut text, &parity.head)?;
        text.literal(" · ".as_bytes())?;
        text.literal(&parity.run)?;
    }
    text.add(b"\n")?;
    push(&mut lines, text)?;
    Ok(lines)
}
fn more(count: u32) -> Box<[u8]> {
    bytes::concat(&[b"and ", skein_lib::Decimal::of(u64::from(count)).as_bytes(), b" more"])
}
fn fitted(lines: &List<Box<[u8]>>, cap: usize) -> HostAnswer {
    let mut count = 0_u32;
    let mut length = 0_usize;
    for line in lines {
        let next_count = count.saturating_add(1);
        let next = length.saturating_add(line.len()).saturating_add(usize::from(count != 0));
        let remaining = lines.len().saturating_sub(next_count);
        let reserved = if remaining == 0 { 0 } else { more(remaining).len().saturating_add(1) };
        if next.saturating_add(reserved) > cap {
            break;
        }
        length = next;
        count = next_count;
    }
    let remaining = lines.len().saturating_sub(count);
    let size = if remaining == 0 {
        length
    } else {
        length.saturating_add(usize::from(count != 0)).saturating_add(more(remaining).len()).min(cap)
    };
    let mut output = Writer::new(size);
    for (at, line) in lines.iter().take(skein_lib::usize_of(count)).enumerate() {
        if at != 0 {
            output.put(b"\n").expect("measured separator");
        }
        output.put(line).expect("measured line");
    }
    if remaining != 0 {
        if count != 0 {
            output.put(b"\n").expect("reserved separator");
        }
        let marker = more(remaining);
        output.put(marker.get(..marker.len().min(output.room())).expect("bounded marker")).expect("reserved marker");
    }
    HostAnswer::new(output.finish(), false).expect("bounded follow read")
}
