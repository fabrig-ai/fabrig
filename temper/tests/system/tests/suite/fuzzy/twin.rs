//! Shared observations for the recorded ladder walks (temper/docs/design/testing/tiers.md, 1.1).
use jig_domain_tasks::{self as tasks, TaskResult};
use serde_json::Value;
use skein_fake_forge_domain::{
    api::{Checks, Protection, Setup},
    object_bytes,
};
use skein_fake_llm_domain::api::{Part, Query, Role as LlmRole};
use smith_domain_session::llm;
use std::fmt::Write;
use temper_engine_domain_forge::{Adoption, Role};
use temper_engine_service_vocabulary::settings;
use temper_system_world::composed::{WalkRepository, World};

pub(super) const HOST: &str = "twin.test";

pub(super) fn repository(repository: &temper_fake::forge::Repository, checks: Checks) -> WalkRepository {
    let protection = Protection {
        branch: repository.landing.clone(),
        contexts: checks.contexts.clone(),
        approvals: 0,
        dismiss_stale: false,
        queue: false,
        linear: false,
        code_owners: false,
        bypassing_users: Box::default(),
    };
    let host = HOST.as_bytes();
    WalkRepository {
        seed: Setup {
            id: repository.id.expect("recorded repository id"),
            name: [&*repository.owner, b"/", &*repository.name].concat().into_boxed_slice(),
            default: repository.landing.clone(),
            tree: repository.tree.clone().expect("recorded widget seed"),
            labels: Box::default(),
            checks,
            workflows: Box::default(),
            protection: Some(protection),
            hooked: false,
        },
        adoption: Adoption {
            project: 1,
            home: true,
            forge: 1,
            host: host.into(),
            owner: repository.owner.clone(),
            name: repository.name.clone(),
            prefix: b"temper/".as_slice().into(),
            role: Role::Owned,
            landing: repository.landing.clone(),
            ci: true,
            checks: Box::default(),
        },
        binding: settings::Forge {
            log_origins: None,
            mode: temper_engine_domain_forge::Mode::Connected,
            number: 1,
            kind: settings::Kind::Forgejo,
            api: jig_protocol_config::Address::Host { name: host.into(), port: 80 },
            transport: settings::Transport::Plaintext,
            git: [b"http://".as_slice(), host].concat().into_boxed_slice(),
            git_program: None,
            web: None,
            app_read_only: false,
            user: 19,
            identity: 4,
        },
        person: 11,
    }
}

pub(super) fn query(world: &World, task: u64, said: usize) -> &Query {
    let mut queries =
        world.observed.queries.iter().filter(|(owner, _, query)| *owner == task && replies(query) == said);
    let (_, _, query) = queries.next().unwrap_or_else(|| panic!("task {task} with {said} prior replies"));
    assert!(queries.next().is_none(), "one request for task {task}'s turn {said}");
    query
}

pub(super) fn replies(query: &Query) -> usize {
    query.messages.iter().filter(|message| message.role == LlmRole::Assistant).count()
}

pub(super) fn calls(query: &Query) -> Vec<(&str, Value)> {
    query
        .messages
        .iter()
        .filter(|message| message.role == LlmRole::Assistant)
        .flat_map(|message| &message.parts)
        .filter_map(|part| {
            let Part::ToolCall { name, arguments, .. } = part else { return None };
            Some((utf8(name), serde_json::from_slice(arguments).expect("observed tool JSON")))
        })
        .collect()
}

pub(super) fn told(query: &Query) -> String {
    query
        .messages
        .iter()
        .filter(|message| message.role == LlmRole::User)
        .flat_map(|message| &message.parts)
        .filter_map(|part| match part {
            Part::Text { text } | Part::ToolOutput { output: text, .. } => Some(utf8(text)),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn heard_tool(query: &Query, words: &str) -> bool {
    query
        .messages
        .iter()
        .flat_map(|message| &message.parts)
        .any(|part| matches!(part, Part::ToolOutput { output, is_error: false, .. } if utf8(output).contains(words)))
}

pub(super) fn chat_text(world: &World, chat: u64) -> String {
    world
        .committed_turns(chat)
        .iter()
        .flat_map(|turn| &turn.messages)
        .filter(|message| message.role == llm::Role::Assistant)
        .flat_map(|message| &message.content)
        .filter_map(|block| block.text().map(utf8))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn done(world: &World, task: u64) -> &TaskResult {
    let row = world.ended_task(task).unwrap_or_else(|| panic!("task {task} ended"));
    let tasks::Phase::Ended(tasks::Ending::Done(result)) = &row.phase else {
        panic!("task {task} succeeded: {:?}", row.phase)
    };
    result
}

pub(super) fn utf8(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("the peer's text is UTF-8")
}

pub(super) fn hex(commit: u64) -> String {
    object_bytes(commit).iter().fold(String::new(), |mut hex, byte| {
        write!(hex, "{byte:02x}").expect("commit hex");
        hex
    })
}
