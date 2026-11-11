//! Decode an address read (temper/docs/design/domain/agent/ask-temper.md, 1).

pub(crate) mod addresses;
mod person;

use crate::{Problem, json::Value};
use alloc::boxed::Box;
use skein_lib::{Decimal, List, Writer, bytes};
use smith_domain_run::HostAnswer;
use temper_engine_domain::Tool;
use temper_engine_domain::{self as engine, SeeAnswer};

pub(crate) fn decode(value: &Value) -> Result<Result<Tool, HostAnswer>, Problem> {
    let Value::Object(fields) = value else { return Err(Problem::Type(b"see")) };
    for (name, _) in fields {
        if name.as_ref() != b"address" {
            return Err(Problem::Range(b"see"));
        }
    }
    let address = value.required(b"address")?.text(b"address")?;
    let line = address.trim_ascii();
    if line.is_empty() || line.len() > 512 {
        return Ok(Err(HostAnswer::new(b"see: address takes 1 to 512 bytes, trimmed.".as_slice().into(), true)
            .ok_or(Problem::TooLarge)?));
    }
    if let Some(encoded) = line.strip_prefix(b"more")
        && let Some(first) = encoded.first()
        && first.is_ascii_whitespace()
        && let Some((line, from)) = untoken(encoded.trim_ascii())
    {
        return Ok(Ok(Tool::See { line, from }));
    }
    Ok(Ok(Tool::See { line: line.into(), from: 0 }))
}

/// The most an answer's body holds (domain/agent/ask-temper.md, 1).
pub const SEE_PART: u32 = 6_144;
pub use addresses::ADDRESSES;

/// Render a `see` read's answer in its outer form, within the call and host allowances.
#[must_use]
pub fn see_answer(
    line: &[u8],
    from: u32,
    answer: &SeeAnswer,
    catalogue: &[jig_domain_core::settings::CatalogueModel],
    max_bytes: u32,
    forge: crate::ForgeReadBounds,
) -> HostAnswer {
    let text = match answer {
        SeeAnswer::Ways(document) => Ok(ways_text(document)),
        SeeAnswer::Person(value) => Ok(person::text(value, catalogue)),
        SeeAnswer::NotRead(why) => {
            let message = match why {
                engine::SeeNotRead::Unknown => b"not read: nothing you may read in this project is there.".as_slice(),
                engine::SeeNotRead::Busy => b"not read: temper is busy; read it again.",
                engine::SeeNotRead::NoPerson => b"not read: this chat has no person to read as.",
            };
            return refused(line, message, max_bytes);
        }
        SeeAnswer::Unknown => {
            let mut forms = Writer::new(ADDRESSES.len());
            addresses::forms(&mut forms).expect("the address table fits its declared line");
            return error(&join(&[b"see: no address ", line, b"; addresses: ", &forms.finish()]), max_bytes);
        }
        SeeAnswer::Settings(settings) => {
            if let jig_domain_core::SettingsAnswer::Unknown = &settings.answer {
                return refused(
                    line,
                    b"not read: no such setting. This says nothing about the settings: see settings lists them.",
                    max_bytes,
                );
            }
            crate::settings::settings_whole(
                &settings.answer,
                settings.requested,
                catalogue,
                settings.application.as_ref(),
                &settings.repositories,
                &settings.people,
                &settings.forge_people,
                &settings.offer,
            )
        }
        SeeAnswer::Call { tool, answer } => {
            let (text, error) = crate::answer_for_tool(
                answer,
                tool,
                u32::try_from(HostAnswer::CAPACITY).expect("host capacity"),
                forge,
            )
            .into_parts();
            if error {
                return refused(line, &text, max_bytes);
            }
            Ok(text)
        }
        SeeAnswer::Page { name } => page_text(name),
        SeeAnswer::Search { words } => search_text(words),
        SeeAnswer::Landing(landing) => landing_text(landing),
        SeeAnswer::Gates(repository) => gates_text(repository),
        SeeAnswer::Proposal(proposal) => proposal_text(proposal, catalogue),
    };
    match text {
        Ok(text) => page(line, from, &text, following(line, answer), max_bytes),
        Err(message) => refused(line, message, max_bytes),
    }
}

fn following(line: &[u8], answer: &SeeAnswer) -> &'static [u8] {
    let form = match answer {
        SeeAnswer::Settings(settings) => match &settings.answer {
            jig_domain_core::SettingsAnswer::Inventory(_) => "settings",
            jig_domain_core::SettingsAnswer::Setting(document) => match document.setting {
                jig_domain_core::settings::Setting::Charter(_) => "settings charter N",
                jig_domain_core::settings::Setting::Template(_) => "settings template N",
                jig_domain_core::settings::Setting::Policy(_)
                | jig_domain_core::settings::Setting::ProjectPolicy { .. } => "settings policy N",
                jig_domain_core::settings::Setting::Model(_) => "settings model N",
                jig_domain_core::settings::Setting::Rules => "settings rules",
                jig_domain_core::settings::Setting::Application { kind, .. } => match kind {
                    1 | 2 => "settings landing-rule N [project]",
                    3 => "settings forge-charter N",
                    6 => "settings project-questions",
                    7 => "settings project-ways",
                    8 => "settings project-classes",
                    9 => "settings charter-words N",
                    10 => "settings project-words",
                    11 => "settings template-goal N",
                    12 => "settings follow N",
                    _ => unreachable!("each application setting read has an address"),
                },
            },
            jig_domain_core::SettingsAnswer::Revisions(_) => "revisions",
            jig_domain_core::SettingsAnswer::Revision(_) => "revision N",
            jig_domain_core::SettingsAnswer::Unknown => unreachable!("unknown settings answer before its form"),
        },
        SeeAnswer::Page { .. } => {
            if line.trim_ascii() == b"help" {
                "help"
            } else {
                "help NAME"
            }
        }
        SeeAnswer::Search { .. } => "help search WORDS",
        SeeAnswer::Call { .. } => "file OWNER/NAME PATH [BRANCH]",
        SeeAnswer::Landing(_) => "settings landing",
        SeeAnswer::Gates(_) => "settings gates OWNER/NAME",
        SeeAnswer::Proposal(_) => "proposal tN/K or pN/K",
        SeeAnswer::Ways(_) => "ways",
        SeeAnswer::Person(value) => match &value.read {
            engine::SeeRead::Tasks { .. } => "tasks",
            engine::SeeRead::Me { .. } => "me",
            engine::SeeRead::Project { .. } => "project",
            engine::SeeRead::Goals { ended: false, .. } => "goals",
            engine::SeeRead::Goals { ended: true, .. } => "goals ended",
            engine::SeeRead::Task { .. } => "task N",
            engine::SeeRead::Tree { .. } => "task N tree",
            engine::SeeRead::Inbox { .. } => "inbox",
            engine::SeeRead::Activity { .. } => "task N activity",
            engine::SeeRead::Turn { .. } => "task N turn A.T",
            engine::SeeRead::Change { .. } => "change N",
            engine::SeeRead::Changes { .. } => "changes",
        },
        SeeAnswer::NotRead(_) | SeeAnswer::Unknown => {
            unreachable!("errors have no following addresses")
        }
    };
    addresses::following(form)
}

/// Each way's name, `now` where current, headline, line and path; then the reader's permission.
fn ways_text(document: &engine::ways::WaysDocument) -> Box<[u8]> {
    let mut parts = List::uncapped();
    for (at, way) in document.ways.iter().enumerate() {
        if at != 0 {
            parts.push(b"\n".as_slice().into()).expect("way separator");
        }
        let current = document.current == Some(u32::try_from(at).expect("stock way index"));
        let now = if current { " · now".as_bytes() } else { b"".as_slice() };
        parts
            .push(join(&[
                &way.name,
                now,
                b"\n",
                &way.headline,
                if way.headline.is_empty() { b"" } else { b"\n" },
                &way.line,
                b"\n",
                &path_text(&way.path),
                b"\n",
            ]))
            .expect("stock way text");
    }
    if document.current.is_none() {
        if !document.ways.is_empty() {
            parts.push(b"\n".as_slice().into()).expect("own separator");
        }
        parts.push(join(&["Yours · now\n".as_bytes(), &path_text(&document.own), b"\n"])).expect("own path text");
    }
    let permission = if document.may_change {
        b"\nYou may change this project's settings.\n".as_slice()
    } else {
        b"\nYou may not change this project's settings; an owner may.\n".as_slice()
    };
    parts.push(permission.into()).expect("settings permission");
    join_parts(&parts)
}

fn path_text(path: &engine::ways::Path) -> Box<[u8]> {
    let mut parts = List::uncapped();
    for (at, stop) in path.stops.iter().enumerate() {
        if at != 0 {
            parts.push(" → ".as_bytes().into()).expect("stop separator");
        }
        let actor = match stop.actor {
            engine::ways::Actor::Person => b"a person".as_slice(),
            engine::ways::Actor::Agent => b"an agent".as_slice(),
            engine::ways::Actor::Code => b"code".as_slice(),
            engine::ways::Actor::Bar => b"a branch".as_slice(),
        };
        parts
            .push(join(&[
                &stop.words,
                b" (",
                actor,
                if stop.needed { b", when needed" } else { b"" },
                if stop.over.is_empty() { b"" } else { b"; " },
                &stop.over,
                b")",
            ]))
            .expect("stop text");
    }
    join_parts(&parts)
}

const NOT_SETTING: &[u8] = b"not a setting yet: whoever runs temper here sets it in temper's configuration (its landing member); no proposal changes it.\n";

#[expect(clippy::unnecessary_wraps, reason = "each address text shares the renderer's refusal result")]
fn landing_text(landing: &engine::SeeLanding) -> Result<Box<[u8]>, &'static [u8]> {
    let mut parts = List::uncapped();
    let by = match landing.by {
        temper_engine_domain_forge_change::Way::Queue => b"queue".as_slice(),
        temper_engine_domain_forge_change::Way::Turn => b"turn".as_slice(),
    };
    parts
        .push(join(&[b"landing: by ", by, b"\nmerge job: ", landing.merge_job.as_deref().unwrap_or(b"none"), b"\n"]))
        .expect("admitted landing text");
    if let Some(after) = landing.after {
        let suspects = match after.verdict.suspects {
            temper_engine_domain_forge_tiers::Suspects::Bisect => b"bisect".as_slice(),
            temper_engine_domain_forge_tiers::Suspects::TellAll => b"tell all".as_slice(),
        };
        let retry = if after.verdict.retries == 1 { b" tier retry".as_slice() } else { b" tier retries".as_slice() };
        parts
            .push(join(&[
                b"after landing: ",
                Decimal::of(u64::from(after.verdict.retries)).as_bytes(),
                retry,
                b", a verification's bound of ",
                &duration_text(after.bound),
                b", suspects ",
                suspects,
                b"\n",
            ]))
            .expect("admitted landing text");
    }
    parts.push(b"repositories:\n".as_slice().into()).expect("admitted landing text");
    if landing.repositories.is_empty() {
        parts.push(b"  none\n".as_slice().into()).expect("admitted landing text");
    }
    for repository in &landing.repositories {
        parts
            .push(join(&[b"  ", &repository.name, b": ", &check_counts(&repository.checks), b"\n"]))
            .expect("admitted repository text");
    }
    parts.push(NOT_SETTING.into()).expect("admitted landing text");
    Ok(join_parts(&parts))
}

fn duration_text(bound: skein_lib::Duration) -> Box<[u8]> {
    let minutes = bound.as_millis().div_euclid(60_000);
    let hours = minutes.div_euclid(60);
    let minutes = minutes.rem_euclid(60);
    let hours_text = if hours == 0 { b"".as_slice().into() } else { join(&[Decimal::of(hours).as_bytes(), b"h"]) };
    let minutes_text =
        if minutes == 0 && hours != 0 { b"".as_slice().into() } else { join(&[Decimal::of(minutes).as_bytes(), b"m"]) };
    join(&[&hours_text, &minutes_text])
}

fn check_counts(checks: &temper_engine_domain_forge::RepositoryGates) -> Box<[u8]> {
    if checks.gates.is_empty() && checks.tiers.is_empty() {
        return b"no gates or tiers".as_slice().into();
    }
    let mut parts = List::uncapped();
    if !checks.gates.is_empty() {
        let suffix = if checks.gates.len() == 1 { b" gate".as_slice() } else { b" gates".as_slice() };
        parts
            .push(join(&[Decimal::of(skein_lib::len_u64(checks.gates.len())).as_bytes(), suffix]))
            .expect("gate count");
    }
    if !checks.tiers.is_empty() {
        if !checks.gates.is_empty() {
            parts.push(b", ".as_slice().into()).expect("count separator");
        }
        let suffix = if checks.tiers.len() == 1 { b" tier".as_slice() } else { b" tiers".as_slice() };
        parts
            .push(join(&[Decimal::of(skein_lib::len_u64(checks.tiers.len())).as_bytes(), suffix]))
            .expect("tier count");
    }
    join_parts(&parts)
}

#[expect(clippy::unnecessary_wraps, reason = "each address text shares the renderer's refusal result")]
fn gates_text(repository: &engine::SeeRepository) -> Result<Box<[u8]>, &'static [u8]> {
    let mut parts = List::uncapped();
    parts.push(join(&[b"gates of ", &repository.name, b":\n"])).expect("admitted gates text");
    if repository.checks.gates.is_empty() {
        parts.push(b"  none\n".as_slice().into()).expect("admitted gates text");
    }
    for gate in &repository.checks.gates {
        let blocking = if gate.blocking { b"blocking".as_slice() } else { b"not blocking".as_slice() };
        parts
            .push(join(&[
                b"  ",
                &gate.name,
                b": workflow ",
                &gate.workflow,
                b", ",
                blocking,
                &ignored_text(&gate.ignored),
                b"\n",
            ]))
            .expect("admitted gate text");
    }
    parts.push(join(&[b"tiers of ", &repository.name, b":\n"])).expect("admitted tiers text");
    if repository.checks.tiers.is_empty() {
        parts.push(b"  none\n".as_slice().into()).expect("admitted tiers text");
    }
    for tier in &repository.checks.tiers {
        let follows = match &tier.follows {
            temper_engine_domain_forge::Follows::Landing => b"the landing branch".as_slice(),
            temper_engine_domain_forge::Follows::Branch(branch) => branch,
            temper_engine_domain_forge::Follows::Goals => b"goals' branches".as_slice(),
        };
        let checks =
            if tier.description.is_empty() { Box::default() } else { join(&[b", checks ", &tier.description]) };
        parts
            .push(join(&[
                b"  ",
                &tier.name,
                b": workflow ",
                &tier.workflow,
                b", artifact ",
                &tier.artifact,
                b", follows ",
                follows,
                &checks,
                &ignored_text(&tier.ignored),
                b"\n",
            ]))
            .expect("admitted tier text");
    }
    parts.push(NOT_SETTING.into()).expect("admitted gates text");
    Ok(join_parts(&parts))
}

fn ignored_text(paths: &[Box<[u8]>]) -> Box<[u8]> {
    let mut parts = List::uncapped();
    for (at, path) in paths.iter().enumerate() {
        let separator = if at == 0 { b", ignoring ".as_slice() } else { b", ".as_slice() };
        parts.push(join(&[separator, path])).expect("admitted ignored paths");
    }
    join_parts(&parts)
}

fn join_parts(parts: &List<Box<[u8]>>) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("admitted answer text");
    }
    let mut text = Writer::new(length);
    for part in parts {
        text.put(part).expect("answer text length");
    }
    text.finish()
}

fn proposal_text(
    proposal: &engine::SeeProposal,
    catalogue: &[jig_domain_core::settings::CatalogueModel],
) -> Result<Box<[u8]>, &'static [u8]> {
    use jig_protocol_engine_clients::translate::objects;
    let limits = settings_form::Limits::standard();
    let models = jig_protocol_engine_clients::translate::llm::catalogue(catalogue);
    let unread = b"not read: proposal of settings could not be rendered.".as_slice();
    match proposal {
        engine::SeeProposal::Waiting(value) => {
            let value = objects::proposal(&models, value, &limits.settings).or(Err(unread))?;
            settings_form::proposal_text(&value, &limits).or(Err(unread))
        }
        engine::SeeProposal::Decided(value) => {
            let value = objects::ended(value, &models, &limits.settings).or(Err(unread))?;
            settings_form::ended_proposal_text(&value, &limits).or(Err(unread))
        }
        engine::SeeProposal::Unknown => Err(b"not read: no such proposal of settings. This says nothing about the settings: see settings lists those waiting."),
    }
}

fn page_text(name: &[u8]) -> Result<Box<[u8]>, &'static [u8]> {
    match temper_docs::find(name) {
        Some(page) => Ok(join(&[page.name.as_bytes(), b": ", page.title.as_bytes(), b"\n", page.text.as_bytes()])),
        None => {
            let mut words: Box<[u8]> = name.into();
            for byte in &mut words {
                if *byte == b'-' {
                    *byte = b' ';
                }
            }
            let found = search_text(&words)?;
            Ok(join(&[b"no page ", name, b"; the sections holding its words:\n", &found]))
        }
    }
}
fn search_text(words: &[u8]) -> Result<Box<[u8]>, &'static [u8]> {
    let hits = match temper_docs::search(&temper_docs::PAGES, words, 8) {
        Ok(hits) => hits,
        Err(refused) => return Err(refused.says().as_bytes()),
    };
    let mut parts = List::with_capacity(8);
    for hit in &hits.hits {
        parts
            .push(join(&[hit.page.as_bytes(), b": ", hit.heading.as_bytes(), b": ", hit.line.as_bytes(), b"\n"]))
            .expect("eight search hits");
    }
    let mut length = 0_usize;
    for part in &parts {
        length = length.checked_add(part.len()).expect("bounded search");
    }
    let last = if hits.hits.is_empty() {
        b"nothing matches\n".as_slice()
    } else if hits.more {
        b"more sections match; narrow the words\n".as_slice()
    } else {
        b"".as_slice()
    };
    let mut text = Writer::new(length.checked_add(last.len()).expect("bounded search"));
    for part in &parts {
        text.put(part).expect("search length");
    }
    text.put(last).expect("search length");
    Ok(text.finish())
}

fn join(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("admitted answer text");
    }
    let mut text = Writer::new(length);
    for part in parts {
        text.put(part).expect("answer text length");
    }
    text.finish()
}
fn refused(line: &[u8], message: &[u8], max_bytes: u32) -> HostAnswer {
    error(&join(&[b"see ", line, b": ", message]), max_bytes)
}
fn error(message: &[u8], max_bytes: u32) -> HostAnswer {
    let maximum = skein_lib::usize_of(max_bytes).min(HostAnswer::CAPACITY);
    HostAnswer::new(bytes::utf8_prefix(message, maximum).into(), true).expect("host feedback bound")
}

pub(crate) fn page(line: &[u8], from: u32, text: &[u8], next: &[u8], max_bytes: u32) -> HostAnswer {
    let maximum = skein_lib::usize_of(max_bytes).min(HostAnswer::CAPACITY);
    let mut start = skein_lib::usize_of(from).min(text.len());
    for _ in 0..3_u8 {
        if bytes::utf8_prefix(text, start).len() == start {
            break;
        }
        start = start.checked_add(1).expect("UTF-8 continuation offset");
    }
    if start >= text.len() {
        return refused(
            line,
            &join(&[
                b"nothing from byte ",
                Decimal::of(u64::from(from)).as_bytes(),
                b"; see ",
                line,
                b" reads it from its start.",
            ]),
            max_bytes,
        );
    }
    let footer = join(&[b"see: ", next, b"\n"]);
    let mut room = skein_lib::usize_of(SEE_PART).min(maximum);
    // The tail prices the room anew; decimal offsets have at most ten digits.
    for _ in 0..12_u8 {
        let mut end = cut(text, start, room);
        let mut separator =
            if text.get(..end).expect("text cut").ends_with(b"\n") { b"".as_slice() } else { b"\n".as_slice() };
        if end.checked_sub(start).expect("forward cut").checked_add(separator.len()).expect("part length") > room {
            end = cut(text, start, room.saturating_sub(1));
            separator =
                if text.get(..end).expect("text cut").ends_with(b"\n") { b"".as_slice() } else { b"\n".as_slice() };
        }
        if end == start {
            break;
        }
        let more = if end < text.len() {
            join(&[b"more: ", &token(line, u32::try_from(end).expect("text offset")), b"\n"])
        } else {
            Box::new([])
        };
        let body = text.get(start..end).expect("UTF-8 part");
        let length = body
            .len()
            .checked_add(separator.len())
            .expect("bounded part length")
            .checked_add(more.len())
            .expect("bounded part length")
            .checked_add(footer.len())
            .expect("bounded part length");
        if length <= maximum {
            return HostAnswer::new(join(&[body, separator, &more, &footer]), false).expect("host part bound");
        }
        room = maximum
            .saturating_sub(more.len().checked_add(footer.len()).expect("part tail bound"))
            .min(skein_lib::usize_of(SEE_PART));
    }
    refused(line, b"host answer limit cannot hold a part.", max_bytes)
}
fn cut(text: &[u8], start: usize, room: usize) -> usize {
    let limit = start.checked_add(room).expect("bounded part offset").min(text.len());
    let end = bytes::utf8_prefix(text, limit).len();
    if end == text.len() {
        return end;
    }
    let mut line_end = start;
    for (offset, byte) in text.get(start..end).expect("UTF-8 part").iter().enumerate() {
        if *byte == b'\n' {
            line_end = start.checked_add(offset).expect("text offset").checked_add(1).expect("newline offset");
        }
    }
    if line_end == start { end } else { line_end }
}

pub(crate) fn token(line: &[u8], from: u32) -> Box<[u8]> {
    let mut text = List::with_capacity(
        u32::try_from(
            line.len().checked_mul(3).expect("address token bound").checked_add(11).expect("address token bound"),
        )
        .expect("address token bound"),
    );
    let mut space = false;
    for byte in line.trim_ascii() {
        if byte.is_ascii_whitespace() {
            space = true;
            continue;
        }
        if space {
            text.push(b'+').expect("token bound");
            space = false;
        }
        let escaped = match *byte {
            b'+' => b"%2B".as_slice(),
            b'@' => b"%40".as_slice(),
            b'%' => b"%25".as_slice(),
            _ => core::slice::from_ref(byte),
        };
        for byte in escaped {
            text.push(*byte).expect("token bound");
        }
    }
    text.push(b'@').expect("token bound");
    for byte in Decimal::of(u64::from(from)).as_bytes() {
        text.push(*byte).expect("token bound");
    }
    text.into_boxed()
}

fn untoken(value: &[u8]) -> Option<(Box<[u8]>, u32)> {
    let at = bytes::find(value, b"@")?;
    let encoded = value.get(..at)?;
    let digits = value.get(at.checked_add(1)?..)?;
    if encoded.is_empty() || digits.is_empty() {
        return None;
    }
    let mut from = 0_u32;
    for byte in digits {
        if !byte.is_ascii_digit() {
            return None;
        }
        from = from.checked_mul(10)?.checked_add(u32::from(byte.checked_sub(b'0')?))?;
    }
    let mut words = List::with_capacity(u32::try_from(encoded.len()).ok()?);
    let mut skip = 0_usize;
    let mut empty = true;
    for (at, byte) in encoded.iter().enumerate() {
        if skip != 0 {
            skip = skip.checked_sub(1)?;
            continue;
        }
        if *byte == b'+' {
            if empty {
                return None;
            }
            words.push(b' ').ok()?;
            empty = true;
        } else {
            let byte = if *byte == b'%' {
                let after = at.checked_add(3)?;
                let escaped = encoded.get(at..after)?;
                skip = 2;
                match escaped {
                    b"%2B" => b'+',
                    b"%40" => b'@',
                    b"%25" => b'%',
                    _ => return None,
                }
            } else {
                if byte.is_ascii_whitespace() {
                    return None;
                }
                *byte
            };
            words.push(byte).ok()?;
            empty = false;
        }
    }
    if empty { None } else { Some((words.into_boxed(), from)) }
}
