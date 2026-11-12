//! A root assignment as one Smith activation. The caller supplies Smith's
//! already decoded history and mounts; no encoding or file access occurs here.

use alloc::boxed::Box;
use jig_domain_brief as brief;
use jig_domain_link::{Form, GrantTable, Scheme};
use skein_lib::{List, ReplyTo, Token, Writer, bytes};
use smith_domain as smith;
use smith_domain_run as run;
use temper_engine_domain as engine;

/// Render one already bounded root brief section with its kind as a title.
#[must_use]
pub fn section(section: &engine::BriefSection) -> run::Section {
    let title: &[u8] = match &section.kind {
        engine::BriefKind::Core(brief::Core::Task) => b"Task",
        engine::BriefKind::Core(brief::Core::Now) => {
            let text = section_text(&section.body);
            return run::Section { title: jig_protocol_charter::now_title(&text), text };
        }
        engine::BriefKind::Core(brief::Core::Lineage) => b"Lineage",
        engine::BriefKind::Core(brief::Core::Results) => b"Dependencies",
        engine::BriefKind::Core(brief::Core::Plan) => b"Plan",
        engine::BriefKind::Core(brief::Core::Attempts) => b"Earlier attempts",
        engine::BriefKind::Core(brief::Core::Calls) => b"Calls",
        engine::BriefKind::Core(brief::Core::Waiting) => b"Waiting",
        engine::BriefKind::Core(brief::Core::NotesIndex) => b"Notes",
        engine::BriefKind::Core(brief::Core::Preferences) => crate::PREFERENCES_TITLE,
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
/// Jig owns the charter policy (jig/docs/design/domain/hosts/charters.md).
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
    temper_protocol_brief::fit_brief(&mut charter, workspace.as_ref(), limits);
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
        // Application grants are the worker's own, for git (domain/worker/README.md, 4.1).
        match grant.form.table() {
            GrantTable::Accounts | GrantTable::Name => grants
                .push(smith::Grant {
                    renewable: match grant.form {
                        Form::Credential(Scheme::Bearer) => true,
                        Form::Credential(Scheme::Key) | Form::Application | Form::Program(_) => false,
                    },
                    name: smith::GrantName { account: grant.account, generation: grant.generation },
                    valid: grant.valid,
                })
                .expect("bounded grants"),
            GrantTable::Application => {}
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
/// only when policy allows and turns are kept (domain/agent/charter.md, 5).
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
    for repository in &assignment.workspace.repositories {
        if repository.delivers {
            charter.delivers = true;
        }
    }
    charter.conventions = Some(jig_protocol_charter::Conventions {
        guide: bytes::copy_of(b"AGENTS.md"),
        checks: bytes::copy_of(b".temper/pre-pr"),
    });
    let mut charter = jig_protocol_charter::charter(charter).expect("bounded charter");
    charter.check_timeout = assignment.check_timeout;
    if assignment.charter == 1 && charter.mode == run::Mode::Finite {
        for tool in &mut charter.grants.host_tools {
            tool.reserve = tool.name.as_ref() == b"message";
        }
    }
    charter
}
