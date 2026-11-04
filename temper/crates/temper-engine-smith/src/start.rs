//! A root assignment as one Smith activation. The caller supplies Smith's
//! already decoded history and mounts; no encoding or file access occurs here.

use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_brief as brief;
use skein_lib::{Decimal, List, ReplyTo, Token, Writer, bytes};
use smith_domain as smith;
use smith_domain_run as run;
use temper_engine_domain as engine;

/// Render one already bounded root brief section with its kind as a title.
#[must_use]
pub fn section(section: &engine::BriefSection) -> run::Section {
    let title: &[u8] = match &section.kind {
        engine::BriefKind::Core(brief::Core::Task) => b"Task",
        engine::BriefKind::Core(brief::Core::Lineage) => b"Lineage",
        engine::BriefKind::Core(brief::Core::Results) => b"Dependencies",
        engine::BriefKind::Core(brief::Core::Plan) => b"Plan",
        engine::BriefKind::Core(brief::Core::Attempts) => b"Earlier attempts",
        engine::BriefKind::Core(brief::Core::Calls) => b"Calls",
        engine::BriefKind::Core(brief::Core::Waiting) => b"Waiting",
        engine::BriefKind::Core(brief::Core::NotesIndex) => b"Notes",
        engine::BriefKind::Core(brief::Core::TranscriptTail) => b"Transcript tail",
        engine::BriefKind::Forge(engine::ForgeBriefKind::Document(words)) => {
            let mut title = Writer::new(b"Document ".len().checked_add(words.len()).expect("bounded document title"));
            title.put(b"Document ").expect("title prefix");
            title.put(words).expect("title words");
            return run::Section { title: title.finish(), text: section_text(&section.body) };
        }
        engine::BriefKind::Forge(engine::ForgeBriefKind::Home) => b"Home",
        engine::BriefKind::Forge(engine::ForgeBriefKind::Ci) => b"CI",
        engine::BriefKind::Forge(engine::ForgeBriefKind::Reviews) => b"Reviews",
        engine::BriefKind::Forge(engine::ForgeBriefKind::Pull) => b"Pull request",
    };
    run::Section { title: bytes::copy_of(title), text: section_text(&section.body) }
}

fn section_text(body: &engine::BriefBody) -> Box<[u8]> {
    match body {
        engine::BriefBody::Text(text) => text.clone(),
        engine::BriefBody::Missing(reason) => match reason {
            brief::GatherMissing::Failed => bytes::copy_of(b"[read failed]"),
            brief::GatherMissing::Late => bytes::copy_of(b"[read timed out]"),
            brief::GatherMissing::Budget => bytes::copy_of(b"[read exceeded limit]"),
        },
    }
}

/// Produce Smith's complete typed start after the root's claim is durable.
/// The caller owns the live reply right, prepared workspace and concrete
/// history; the engine deliberately retains its stored transcript as bytes.
/// Jig owns the charter policy (jig/docs/design/domain/hosts.md, 8).
#[must_use]
pub fn start(
    assignment: &engine::Assignment,
    workspace: Option<run::Workspace>,
    transcript: Option<smith::Transcript>,
    reply_to: ReplyTo,
    host_run: Token,
    window: smith::Window,
    limits: &run::Limits,
) -> smith::Event {
    let mut charter = charter_with_history(assignment, transcript.is_some());
    fit_brief(&mut charter, workspace.as_ref(), limits);
    let mut messages = List::with_capacity(assignment.messages.len().try_into().expect("bounded start messages"));
    for word in &assignment.messages {
        messages
            .push(run::Message {
                name: Token::new(word.number),
                label: engine::inbox_sender(word),
                text: engine::inbox_words(assignment.task, word),
                finish_requested: word.finish_requested(),
            })
            .expect("bounded start messages");
    }
    let mut grants = List::with_capacity(assignment.grants.len().try_into().expect("bounded grants"));
    for grant in &assignment.grants {
        // Application grants are the worker's own, for git (domain/worker.md, 4.1).
        match grant.form.table() {
            accounts::GrantTable::Accounts | accounts::GrantTable::Name => grants
                .push(smith::Grant {
                    renewable: match grant.form {
                        accounts::Form::Credential(accounts::Scheme::Bearer) => true,
                        accounts::Form::Credential(accounts::Scheme::Key)
                        | accounts::Form::Application
                        | accounts::Form::Program(_) => false,
                    },
                    name: smith::GrantName { account: grant.account, generation: grant.generation },
                    valid: grant.valid,
                })
                .expect("bounded grants"),
            accounts::GrantTable::Application => {}
        }
    }
    smith::Event::Start(smith::Start {
        reply_to,
        host_run,
        activation: assignment.attempt,
        identity: None,
        window,
        messages: messages.into_boxed(),
        charter,
        workspace,
        transcript,
        // The host will supply recovered answers when it keeps their durable
        // records; this typed system route has none to pass yet.
        answered: Box::new([]),
        grants: grants.into_boxed(),
    })
}

/// The channel charter takes its mode from the task's ending and resumes
/// only when policy allows and turns are kept (domain/agent.md, 4.6).
#[must_use]
pub fn charter(assignment: &engine::Assignment) -> run::Charter {
    charter_with_history(assignment, !assignment.transcript.is_empty())
}

fn charter_with_history(assignment: &engine::Assignment, resumed: bool) -> run::Charter {
    let mut sections = List::with_capacity(assignment.sections.len().try_into().expect("bounded brief sections"));
    for item in &assignment.sections {
        let item = section(item);
        sections
            .push(jig_protocol_charter::Section { title: item.title, text: item.text })
            .expect("bounded brief sections");
    }
    let policy = &assignment.run.policy;
    let declarations = crate::calls::tools_for(policy.call_timeout, policy.tools, assignment.run.authority.tools);
    let mut tools = List::with_capacity(u32::try_from(declarations.len()).expect("bounded host tools"));
    for tool in declarations {
        tools
            .push(jig_protocol_charter::Tool {
                name: tool.name,
                description: tool.description,
                schema: tool.schema,
                timeout: tool.timeout,
                effect: match tool.effect {
                    run::HostEffect::Read => jig_protocol_charter::ToolEffect::Read,
                    run::HostEffect::Write => jig_protocol_charter::ToolEffect::Write,
                },
            })
            .expect("bounded host tools");
    }
    let mut charter =
        jig_protocol_charter::assignment(*assignment.run.clone(), sections.into_boxed(), tools.into_boxed(), resumed)
            .expect("agent contract and retained source bindings");
    charter.conventions = Some(jig_protocol_charter::Conventions {
        guide: bytes::copy_of(b"AGENTS.md"),
        checks: bytes::copy_of(b".temper/pre-pr"),
    });
    let mut charter = jig_protocol_charter::charter(charter).expect("bounded charter");
    // A workspace edit needs a delivery path (temper/docs/design/domain/agent.md, 4.3).
    charter.grants.tools.modify &= charter.grants.deliver.is_some();
    if assignment.charter == 1 && charter.mode == run::Mode::Finite {
        for tool in &mut charter.grants.host_tools {
            tool.reserve = tool.name.as_ref() == b"message";
        }
    }
    charter
}

/// Fit the brief after the workspace is prepared, using the receiving run's
/// bounds and schema costs (temper/docs/design/domain/agent.md, 4.3).
pub fn fit_brief(charter: &mut run::Charter, workspace: Option<&run::Workspace>, limits: &run::Limits) {
    let Some(room) = run::brief_room(charter, workspace, limits) else { return };
    let room = u64::from(room);
    let owned = charter.owned_bytes().expect("bounded charter ownership");
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
    let count = Decimal::of(u64::try_from(titles.len()).expect("bounded sections"));
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
