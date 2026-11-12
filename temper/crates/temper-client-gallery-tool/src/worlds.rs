//! Mockup documents, answered through the same windows as the engine.
//! First-run records follow temper/docs/design/client/forge/deployment.md; temper/docs/design/client/pages/README.md and temper/docs/design/client/forge/family.md, 7.
use temper_fixtures::client_documents as documents;
mod chat_work;
mod team;
pub(super) fn team_world(name: &str) -> Domain {
    team::world(name)
}
mod rail_work;
pub(super) fn rail_work_world(name: &str, address: Address) -> Domain {
    rail_work::world(name, address)
}
mod views;
mod ways;
pub(crate) mod whole_story;
pub(super) fn whole_story_world(name: &str, address: Address) -> Domain {
    whole_story::domain(name, address)
}
pub(super) fn ways_world(name: &str) -> Domain {
    ways::world(name)
}
pub(super) fn view_world(name: &str) -> Domain {
    views::world(name)
}
mod picker;
pub(super) fn picker_world(name: &str) -> Domain {
    picker::world(name)
}
mod github;
use crate::driver::{answer, deliver, limits, step};
use crate::samples::{Confirmation, Then, World};
use World::{FirstRun, Reference, Solo};
use jig_client::contract as c;
use skein_lib::{Duration, List, Queue, Token, Wall};
use temper_client_domain::{Address, Domain, Event, Request, pages::goals::State as GoalState};
const FORGE: u16 = temper_client_protocol_wire::FORGE;
const PROJECT: u32 = 1;
const SEED: u64 = 11;
const CHAT_LABEL: u32 = 1;
const CHAT_FLAGS: c::TaskFlags =
    c::TaskFlags { ended_by: c::EndedBy::Result, conversation: true, listed: false, retention: c::Retention::Kept };
pub(super) const PROPOSAL: c::ProposalKey = c::ProposalKey { proposer: c::Proposer::Task(10), number: 1 };
const JSON_TITLE: &str = "JSON output for tally report";
const JSON_WHY: &str = "It changes report’s output and its tests, and lands into main: more than a chat does alone.";
const QUESTION: &str =
    "The old parser accepts a trailing comma as an empty last field. RFC 4180 does not. Keep it, or drop it in 0.5?";
fn person(w: World) -> &'static str {
    match w {
        Reference | World::Bare | World::WholeStory => "Pat",
        Solo => "Robin",
        FirstRun { .. } => "Sam",
    }
}
fn project(w: World) -> &'static str {
    match w {
        Reference => "factory",
        Solo => "tally",
        World::Bare | World::WholeStory => "gallery",
        FirstRun { step: 22 } => "widget",
        FirstRun { .. } => "Hello",
    }
}
fn doc(s: &str) -> c::DocumentText {
    c::DocumentText::complete(text(s))
}
fn has_playground(w: World) -> bool {
    match w {
        FirstRun { step: 1..=5 | 10 } => true,
        FirstRun { .. } | Reference | Solo | World::Bare | World::WholeStory => false,
    }
}
fn model(w: World) -> c::LlmChoice {
    c::LlmChoice {
        later: None,
        account: Some(1),
        main: c::ModelEffort {
            model: text(if has_playground(w) { "Scripted model" } else { "Opus" }),
            effort: match w {
                Reference => Some(text("xhigh")),
                Solo | World::Bare | World::WholeStory => Some(text("high")),
                FirstRun { .. } => None,
            },
        },
        subagents: List::with_capacity(0),
    }
}
fn frame(w: World, connecting: bool, asking: bool) -> c::Me {
    let mut me = base_frame(person(w), project(w));
    let member = me.projects.get_mut(0).expect("one project");
    let template = member.templates.get_mut(0).expect("chat template");
    template.llm = match w {
        FirstRun { .. } => None,
        Reference | Solo | World::Bare | World::WholeStory => Some(model(w)),
    };
    if matches!(w, FirstRun { .. } | Solo | Reference) {
        let chat = member.templates.get(0).expect("chat template").clone();
        let mut templates = List::with_capacity(6);
        templates.push(chat.clone()).expect("chat template");
        templates.push(goal_template(&chat, 2, "goal")).expect("goal template");
        if matches!(w, Solo | Reference) {
            templates.push(goal_template(&chat, 4, "goal for a draft")).expect("draft goal template");
        }
        if w != Solo {
            templates.push(asking_template(&chat)).expect("Ask temper template");
        }
        for (number, name) in [(6, "Plan, then wait for me"), (7, "One agent, one branch")] {
            if w != Reference {
                continue;
            }
            templates.push(goal_template(&chat, number, name)).expect("reference goal templates");
        }
        member.templates = templates;
    }
    member.models = List::with_capacity(1);
    let choice = model(w);
    let mut efforts = List::with_capacity(1);
    if let Some(effort) = choice.main.effort {
        efforts.push(effort).expect("effort");
    }
    member
        .models
        .push(c::MeModel {
            new_until: None,
            number: 1,
            source: jig_client::contract::ModelSource::Stock,
            estimated: false,
            name: choice.main.model,
            provider: Some(17),
            description: None,
            prices: c::ModelPrices { input: 0, cached: 0, cache_write: None, output: 0, unit: 1 },
            window: 200_000,
            output: 16_384,
            effort: efforts.get(0).cloned(),
            shown: None,
            accounts: {
                let mut accounts = List::with_capacity(1);
                accounts.push(1).expect("account");
                accounts
            },
            picker: true,
            efforts,
        })
        .expect("model");
    if matches!(w, World::Bare | FirstRun { step: 0 }) {
        member.models.clear();
    }
    if matches!(w, FirstRun { step: 6..=9 }) {
        let mut models = List::with_capacity(2);
        let mut model = member.models.get(0).expect("demo model").clone();
        model.name = text("gpt-6-luna");
        model.provider = Some(18);
        models.push(model).expect("plan model");
        models.push(member.models.get(0).expect("demo model").clone()).expect("demo model");
        member.models = models;
        template_preview(member);
    }
    if matches!(w, FirstRun { step: 7 | 8 }) {
        let mut projects = List::with_capacity(2);
        let mut hello = member.clone();
        hello.project = 2;
        hello.name = doc("hello");
        projects.push(member.clone()).expect("retired playground");
        projects.push(hello).expect("new project");
        me.projects = projects;
    }
    me.providers = providers(w);
    if connecting {
        for index in 0..me.providers.len() {
            let provider = me.providers.get_mut(index).expect("provider index");
            if provider.name.bytes() == b"openai" {
                provider.name = doc("openai");
                provider.sign_in = true;
            } else if provider.name.bytes() == b"anthropic" {
                provider.name = doc("anthropic");
            }
        }
    }
    me.needs = needs(w);
    me.needs_total = if w == Solo { 2 } else { 0 };
    me.updates = c::Count { value: if w == Solo { 2 } else { u32::from(w == Reference) }, limited: false };
    me.deployment_role = Some(c::HeldRole::Owner);
    if asking {
        ask_frame(&mut me);
    }
    me
}
fn goal_template(chat: &c::Template, number: u32, name: &str) -> c::Template {
    let mut template = chat.clone();
    template.number = number;
    template.name = doc(name);
    template.tracked = Some(1);
    template.label = 2;
    template.flags.conversation = false;
    template.flags.listed = true;
    template
}
fn asking_template(chat: &c::Template) -> c::Template {
    let mut template = chat.clone();
    template.number = temper_client_domain::pages::chats::ASKING_TEMPLATE;
    template.name = doc("ask");
    template.executor = c::TemplateExecutor::Charter(11);
    template
}
fn template_preview(member: &mut c::MeProject) {
    member.defaults = List::with_capacity(1);
    member
        .defaults
        .push(c::MeDefault {
            template: jig_client::shapes::CHAT,
            choice: c::LlmChoice {
                later: None,
                account: None,
                main: c::ModelEffort { model: text("gpt-6-luna"), effort: None },
                subagents: List::with_capacity(0),
            },
        })
        .expect("template preview");
}
fn providers(w: World) -> List<c::ProviderOffer> {
    let mut rows = List::with_capacity(3);
    let names: &[(u32, &str)] = match w {
        Reference => &[(17, "anthropic"), (18, "openai")],
        Solo => &[(17, "anthropic")],
        FirstRun { step: 1..=5 | 10 } => &[(17, "Playground"), (18, "openai"), (19, "anthropic")],
        FirstRun { .. } => &[(18, "openai"), (19, "anthropic")],
        World::Bare | World::WholeStory => &[(18, "openai")],
    };
    for (number, name) in names {
        let mut programs = List::with_capacity(1);
        if *name == "anthropic" {
            programs.push(text("Claude Code")).expect("program");
        }
        rows.push(c::ProviderOffer {
            provider: *number,
            name: doc(name),
            key_bytes: 4096,
            sign_in: *name == "openai" && w == Reference,
            key: true,
            programs,
            programs_cut: false,
            none: false,
            signed_in_plan: *name == "openai",
            manage_usage: None,
        })
        .expect("provider");
    }
    rows
}
fn needs(w: World) -> c::Needs {
    c::Needs { proposals: u32::from(w == Solo), questions: u32::from(w == Solo), escalations: 0, person_tasks: 0 }
}
/// The gallery's T10 is Ask temper beside ordinary chats.
fn asking_chip(chip: &mut c::Chip) {
    if chip.task == 10 {
        chip.executor = c::ChipExecutor::Agent { charter: 11 };
    }
}
/// The Ask temper gallery states list template 5 and its creating bounds.
fn ask_frame(me: &mut c::Me) {
    let member = me.projects.get_mut(0).expect("asking project");
    let stock = member.templates.get(0).expect("stock chat").clone();
    let mut asking = stock.clone();
    asking.number = 5;
    asking.name = doc("Ask temper");
    asking.executor = c::TemplateExecutor::Charter(11);
    member.templates = List::with_capacity(2);
    member.templates.push(stock).expect("stock");
    member.templates.push(asking).expect("asking");
    let mut charters = List::with_capacity(2);
    charters.push(1).expect("stock charter");
    charters.push(11).expect("asking charter");
    member.creating = Some(c::Creating {
        within: c::CreateBounds {
            authority: Box::new(c::AuthorityDocument::complete(c::Authority {
                models: None,
                tools: 0,
                grants: List::with_capacity(0),
                delegation: c::Delegation { kinds: List::with_capacity(0), tasks: 1, depth: 2 },
                spend: 100,
                deadline: None,
                notes: 0,
                note_resources: List::with_capacity(0),
            })),
            pool_left: 100,
            charters,
            procedures: List::with_capacity(0),
            latest: None,
            budget: 100,
            maximum_budget: 100,
            maximum_deadline: None,
        },
        beyond: c::Beyond::Refused(c::Refusal::NotOffered),
    });
}

pub(super) fn world(w: World, address: Address, then: Then) -> (Domain, Option<Token>) {
    match w {
        World::WholeStory => (whole_story::domain("all-home", address), None),
        Reference | Solo | FirstRun { .. } | World::Bare => world_in_layout(w, address, then, false, None, None, false),
    }
}
/// Approval cards reached through the root's events (client/web/gallery.md, 2).
pub(super) fn approve_world(name: &str) -> Domain {
    use jig_client::{Approval, ApprovalCode, ApprovalState, Decided};
    use temper_client_domain::{Action, Place, approve::Act};
    let (mut root, _) =
        world(World::Bare, Address::Inbox { filter: temper_client_domain::address::Filter::ALL }, Then::Shown);
    let mut out = Queue::uncapped();
    let code = ApprovalCode::parse(b"WDJB-MJHT").expect("gallery code");
    step(
        &mut root,
        Event::Went {
            place: Place::Approve {
                code: if matches!(name, "approve-typing" | "approve-unread") { None } else { Some(code) },
            },
        },
        &mut out,
    );
    let asked = Wall::from_nanos(
        sample_now(World::Bare, name).as_nanos().checked_sub(120_000_000_000).expect("two minutes before gallery now"),
    );
    match name {
        "approve-typing" | "approve-unread" => {
            let words = if name == "approve-typing" { b"wdjb mjht".as_slice() } else { b"WDJB-MJH" };
            step(&mut root, Event::Act { action: Action::Approve(Act::Words { words: words.into() }) }, &mut out);
            if name == "approve-unread" {
                step(&mut root, Event::Act { action: Action::Approve(Act::Check) }, &mut out);
            }
        }
        "approve-signed-out" => step(&mut root, Event::SignedOut, &mut out),
        _ => {
            let state = match name {
                "approve-asking" | "approve-used" => ApprovalState::Waiting { asked },
                "approve-approved" => ApprovalState::Approved,
                "approve-signed-in" => ApprovalState::Taken,
                "approve-refused" => ApprovalState::Refused,
                "approve-expired" => ApprovalState::Expired,
                "approve-unknown" => ApprovalState::Unknown,
                "approve-busy" => ApprovalState::Busy,
                _ => unreachable!("named approval sample"),
            };
            step(
                &mut root,
                Event::Client { event: jig_client::Event::ApprovalAnswer { answer: Approval::Read { code, state } } },
                &mut out,
            );
            if name == "approve-used" {
                step(&mut root, Event::Act { action: Action::Approve(Act::Approve) }, &mut out);
                step(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::ApprovalAnswer {
                            answer: Approval::Decided { code, decided: Decided::Used },
                        },
                    },
                    &mut out,
                );
            }
        }
    }
    root
}
fn agent_work_task() -> c::Task {
    let mut task = chat_work::task("chat-work-running");
    task.label = 0;
    task.flags.conversation = false;
    task.chip.executor = c::ChipExecutor::Agent { charter: 11 };
    task.chip.details.details.executor = c::Executor::Agent { charter: 11 };
    task.chip.details.details.attempt = Some(1);
    task.chip.details.details.run_since = Some(Wall::from_nanos(chat_work::AT));
    task
}
pub(super) fn chat_work_world(name: &str) -> Domain {
    let address = if name == "agent-work" { Address::Task { task: 1 } } else { Address::Chat { task: 1 } };
    let mut root = world_in_layout(World::Bare, address, Then::Shown, false, Some(name), None, false).0;
    if name == "chat-work-reading" {
        step(
            &mut root,
            Event::Act { action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Older) },
            &mut Queue::uncapped(),
        );
    }
    root
}
pub(super) fn drafts_world() -> Domain {
    world_in_layout(
        Reference,
        Address::Chats { filter: temper_client_domain::ChatsFilter::Drafts },
        Then::Shown,
        false,
        Some("chats-drafts"),
        None,
        false,
    )
    .0
}
pub(super) fn linked_chat_world(w: World, address: Address) -> Domain {
    world_in_layout(
        w,
        address,
        Then::Mode(temper_client_domain::pages::chat::document::Mode::Chat),
        false,
        Some("chat-goal-linked"),
        None,
        false,
    )
    .0
}
/// First-run Ask leaves only the way of working unfinished (client/web/gallery.md, 2).
pub(super) fn to_begin_world(w: World, address: Address, then: Then) -> Domain {
    world_in_layout(w, address, then, false, None, Some("ask-first-run"), false).0
}
pub(super) fn asking_world(w: World, address: Address, then: Then) -> (Domain, Option<Token>) {
    world_in_layout(w, address, then, true, None, None, false)
}
pub(super) fn goals_world(w: World, address: Address, name: &str) -> Domain {
    world_in_layout(w, address, Then::Shown, false, Some(name), None, false).0
}
/// The browser reports its own facts; the watch supplies two real device rows (client/pages/told.md).
pub(super) fn told_world(name: &str) -> Domain {
    use temper_client_domain::pages::told::{Act, Browser, Duty, Permission, Refusal, Report};
    let mut root = world_in_layout(Reference, Address::Told, Then::Shown, false, Some(name), None, false).0;
    let mut out = Queue::uncapped();
    step(
        &mut root,
        Event::Push(Report::State(Browser {
            capable: name != "told-cannot",
            permission: Permission::Granted,
            endpoint: (name == "told-on").then(|| b"https://fcm.googleapis.com/subscription/gallery".as_slice().into()),
            user_agent: b"Mozilla/5.0 (iPhone) Version/17 Safari/605.1".as_slice().into(),
        })),
        &mut out,
    );
    if name == "told-refused" {
        step(
            &mut root,
            Event::Act {
                action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Told(Act::TurnOn)),
            },
            &mut out,
        );
        while let Some(request) = out.pop() {
            if let Request::Push(Duty::Subscribe { token, .. }) = request {
                step(&mut root, Event::Push(Report::Refused { token, why: Refusal::TooMany }), &mut out);
            }
        }
    }
    root
}
fn told_devices(root: &mut Domain, watch: Token, out: &mut Queue<Request>) {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut rows = List::with_capacity(w::CEILINGS.devices_rows);
    for (number, label, endpoint) in [
        (1, b"Safari on iPhone".as_slice(), b"https://fcm.googleapis.com/subscription/gallery".as_slice()),
        (
            2,
            b"Firefox on Linux".as_slice(),
            b"https://updates.push.services.mozilla.com/subscription/gallery".as_slice(),
        ),
    ] {
        rows.push(
            w::DeviceRow::new(
                &w::CEILINGS,
                w::DeviceRowParts {
                    number,
                    label: label.into(),
                    at: now(Reference).as_nanos(),
                    endpoint: endpoint.into(),
                },
            )
            .expect("device row"),
        )
        .expect("two devices");
    }
    let document = w::Devices::new(&w::CEILINGS, w::DevicesParts { rows }).expect("devices");
    step(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::devices(&document),
                    },
                },
            },
        },
        out,
    );
}

/// Remembered states use the person's party documents and public acts (client/pages/remembered.md).
pub(super) fn remembered_world(name: &str) -> Domain {
    use temper_client_domain::pages::{Act, remembered::Act as R};
    let mut root = world_in_layout(Reference, Address::Remembered, Then::Shown, false, Some(name), None, false).0;
    if matches!(name, "remembered-changing" | "remembered-refused") {
        let mut out = Queue::uncapped();
        for act in [
            R::Change { note: 2 },
            R::Words { words: Box::from(b"Use short Rust examples with their output.".as_slice()) },
        ] {
            step(&mut root, Event::Act { action: temper_client_domain::Action::Pages(Act::Remembered(act)) }, &mut out);
        }
        if name == "remembered-refused" {
            step(
                &mut root,
                Event::Act { action: temper_client_domain::Action::Pages(Act::Remembered(R::Save)) },
                &mut out,
            );
            step(&mut root, Event::Client { event: jig_client::Event::Stored }, &mut out);
            while let Some(request) = out.pop() {
                match crate::driver::down(request) {
                    Some(jig_client::Down::Send { attempt, .. }) => step(
                        &mut root,
                        Event::Client {
                            event: jig_client::Event::Up {
                                up: jig_client::Up::Answer {
                                    attempt,
                                    envelope: skein_lib::RequestEnvelope::Final,
                                    why: None,
                                    answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                                        refusal: c::Refusal::Moved {
                                            now: c::Revision { value: 2 },
                                            what: c::Moved::Note,
                                        },
                                        at: None,
                                    })),
                                },
                            },
                        },
                        &mut out,
                    ),
                    Some(jig_client::Down::Read {
                        read,
                        what: jig_client::Readable::Jig(c::Read::Notes { scope, .. }),
                    }) => {
                        answer(
                            &mut root,
                            read,
                            c::ReadValue::Notes(Box::new(remembered_notes(name, &scope))),
                            &mut out,
                        );
                    }
                    _ => {}
                }
            }
        }
    }
    root
}
fn remembered_notes(name: &str, scope: &c::NoteScope) -> c::NotePage {
    assert_eq!(*scope, c::NoteScope::Party { party: 73 }, "the signed-in person's notes");
    let mut rows = List::with_capacity(12);
    let lines = [
        "Keep answers concise and start with the result.",
        "Use Rust examples when explaining code.",
        "Ask before changing a public interface.",
        "Show the command that checks a change.",
        "Explain unfamiliar abbreviations the first time.",
        "Use metric units in examples.",
        "Keep a summary of decisions at the end of a long chat.",
        "Prefer plain text examples I can copy into a terminal.",
        "Say which assumptions a calculation depends on.",
        "Use UTC when comparing times from different places.",
        "Give one next step when a check fails.",
        "When comparing approaches, explain the cost of each and which evidence would make you choose a different one next time.",
    ];
    let count = match name {
        "remembered-none" => 0,
        "remembered-full" => 12,
        _ => 3,
    };
    for (index, line) in lines.iter().take(count).enumerate() {
        assert!(line.len() <= 120, "a party line fits its byte bound");
        let number = u64::try_from(index).expect("twelve notes").checked_add(1).expect("note number");
        rows.push(c::Note {
            number,
            revision: c::Revision { value: 1 },
            scope: scope.clone(),
            description: doc(line),
            body: doc(""),
            writer: if number == 3 {
                c::NoteAuthor::Party { party: 73 }
            } else {
                c::NoteAuthor::Task { task: 14, attempt: 1 }
            },
            tasks: List::from_boxed(Box::new([14])),
            changed: Some(now(Reference)),
            acts: List::with_capacity(0),
            cut: false,
        })
        .expect("twelve party lines");
    }
    c::NotePage { rows, before: None, after: None, cut: false }
}
pub(super) fn copies_world(address: Address, then: Then) -> Domain {
    world_in_layout(FirstRun { step: 7 }, address, then, false, None, None, true).0
}
#[expect(clippy::too_many_lines, reason = "one ordered dispatch answers the gallery page’s document reads")]
fn world_in_layout(
    w: World,
    address: Address,
    then: Then,
    asking_chat: bool,
    goals: Option<&str>,
    settings: Option<&str>,
    copies: bool,
) -> (Domain, Option<Token>) {
    let view = goals.filter(|name| name.starts_with("chat-view-"));
    let how = settings.is_some_and(ways::is_how);
    let to_begin = (how && !settings.is_some_and(|name| ways::is_draft(name) || name == "how-team"))
        || settings.is_some_and(|name| name == "ask-first-run" || name.starts_with("ask-ways-"));
    let picker = goals.filter(|name| picker::is_state(name));
    let agent_work = goals == Some("agent-work");
    let goals = if agent_work { Some("chat-work-running") } else { goals };
    let asking = asking_chat || matches!(address, Address::Ask { .. });
    let connecting = matches!(
        address,
        Address::ConnectModel { .. } | Address::Welcome { step: temper_client_domain::Step::Model { .. } }
    );
    let limits = limits();
    let (mut root, mut out) = started(address);
    if goals == Some("chat-terminal-question") {
        step(
            &mut root,
            Event::Act {
                action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Terminal(
                    temper_client_domain::pages::terminal::Act::Enter,
                )),
            },
            &mut out,
        );
    }
    let mut forge_watch = None;
    let mut accounts_watch = None;
    let mut settings_chat_watch = None;
    let mut settings_watch = None;
    let settings_chat = settings.filter(|name| name.starts_with("ask-chat-"));
    let mut playground_read = false;
    // Every response answers an open/read, including the playground family read.
    for _ in 0_u32..256 {
        let Some(request) = out.pop() else { break };
        if let Some(down) = crate::driver::down(request) {
            match down {
                jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(subject) } => {
                    let document = match subject {
                        c::Subject::Me => {
                            let mut me = frame(w, connecting, asking);
                            if let Some(name) = settings.filter(|name| ways::is_how(name)) {
                                ways::frame(&mut me, name);
                            }
                            if matches!(then, Then::Sheet(Confirmation::CloseWork | Confirmation::CloseFailed)) {
                                me.projects
                                    .get_mut(0)
                                    .expect("project")
                                    .templates
                                    .get_mut(0)
                                    .expect("chat template")
                                    .grace = Some(Duration::from_secs(5));
                            }
                            if matches!(then, Then::Mode(_) | Then::DraftEmpty) {
                                let templates = me.projects.get(0).expect("draft project").templates.clone();
                                ask_frame(&mut me);
                                me.projects.get_mut(0).expect("draft project").templates = templates;
                            }
                            if settings_chat.is_some() {
                                settings_chat_frame(&mut me);
                            }
                            if settings == Some("ask-ways-member") {
                                me.projects.get_mut(0).expect("ways project").role = c::HeldRole::Member;
                                me.deployment_role = None;
                            }
                            if let Then::Hosts { role: false, .. } = then {
                                me.deployment_role = None;
                            }
                            if let Then::Models { one_account, state } = then {
                                models_frame(&mut me, one_account, state);
                            }
                            if let Some(name) = picker {
                                picker::frame(&mut me, name);
                            }
                            if goals == Some("no-project") {
                                me.projects.clear();
                            }
                            Some(c::Delivery::Me(Box::new(me)))
                        }
                        c::Subject::Accounts => {
                            accounts_watch = Some(watch);
                            let mut doc = accounts(w, then);
                            if let Then::Models { one_account, state } = then {
                                if one_account {
                                    doc.accounts.truncate(1);
                                }
                                if matches!(
                                    state,
                                    super::samples::ModelState::Read
                                        | super::samples::ModelState::ReadFailed
                                        | super::samples::ModelState::New
                                ) {
                                    let at = offset(Reference, -120);
                                    doc.accounts.get_mut(1).expect("OpenAI account").list = Some(c::AccountList {
                                        read: Some(at),
                                        tried: Some(at),
                                        failure: if state == super::samples::ModelState::ReadFailed {
                                            Some(c::AccountListFailure::Deadline)
                                        } else {
                                            None
                                        },
                                        left_out: 0,
                                    });
                                }
                            }
                            if let Some(name) = picker {
                                picker::accounts(&mut doc, name);
                            }
                            Some(c::Delivery::Accounts(Box::new(doc)))
                        }
                        c::Subject::Tasks { query, window } => {
                            let mut tasks = if settings == Some("how-follows") && query.label == Some(3) {
                                ways::standing()
                            } else if query.tracked == Some(true) {
                                goal_window(w, window, goals)
                            } else {
                                task_window(w, window)
                            };
                            if asking_chat {
                                for row in tasks.rows.iter_mut() {
                                    asking_chip(&mut row.chip);
                                }
                            }
                            Some(c::Delivery::Tasks(Box::new(tasks)))
                        }
                        c::Subject::Inbox { window } => Some(c::Delivery::Inbox(Box::new(inbox_window(w, window)))),
                        c::Subject::Task { task: number } => {
                            let mut task = if view.is_some() {
                                views::task()
                            } else if picker.is_some() {
                                chat_work::task("chat-work-running")
                            } else if agent_work {
                                agent_work_task()
                            } else if let Some(name) = goals.filter(|name| name.starts_with("chat-work-")) {
                                chat_work::task(name)
                            } else {
                                page_goal(goals)
                                    .filter(|g| g.facts.number == number)
                                    .map_or_else(|| shown_task(w, number, then), |g| g.document(w))
                            };
                            if goals == Some("chat-terminal-question") && number == 1 {
                                task.result = None;
                                task.chip.details.details.ending = None;
                                task.chip.details.details.ended_at = None;
                                task.chip.phase = c::Phase::Active;
                                task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Idle);
                                task.run_wait = None;
                                task.waits_on = c::WaitsOn::Party {
                                    whom: c::Whom::Person(frame(w, false, false).party.number),
                                    entries: 1,
                                    task: 3,
                                    kind: c::WaitedFor::Question,
                                };
                            }
                            if goals == Some("chat-terminal-working") {
                                task.chip.phase = c::Phase::Running;
                                task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Running { attempt: 1 });
                                task.chip.details.details.attempt = Some(1);
                                task.waits_on = c::WaitsOn::Nothing;
                            }
                            if asking_chat {
                                asking_chip(&mut task.chip);
                            }
                            if let Some(name) = settings_chat {
                                settings_chat_watch = Some(watch);
                                task.activity = if name == "ask-chat-replaced" { 6 } else { 3 };
                                task.proposals = List::with_capacity(2);
                                task.proposals.push(PROPOSAL).expect("settings proposal");
                                if name == "ask-chat-replaced" {
                                    task.proposals.push(SETTINGS_NEXT).expect("later proposal");
                                }
                            }
                            if let Some(name) = picker {
                                picker::task(&mut task, name);
                            }
                            Some(c::Delivery::Task(Box::new(task)))
                        }
                        c::Subject::Tree { .. } if agent_work => Some(c::Delivery::Tree(Box::new(c::Tree {
                            root: agent_work_task().chip,
                            rows: List::with_capacity(0),
                            folded: List::with_capacity(0),
                            held_below: 0,
                            next: None,
                        }))),
                        c::Subject::Tree { task, window } => Some(
                            page_goal(goals)
                                .filter(|g| g.facts.number == task)
                                .map_or_else(|| plan_tree(w, task, window, then), |g| g.tree(w, window)),
                        ),
                        c::Subject::Deployment => {
                            let mut doc = deployment(w);
                            if let Then::Hosts { connected: true, .. } = then {
                                doc.hosts = builders();
                            }
                            Some(c::Delivery::Deployment(Box::new(doc)))
                        }
                        c::Subject::Settings => {
                            settings_watch = Some(watch);
                            Some(c::Delivery::Settings(Box::new(c::SettingsDocument {
                                newest: 0,
                                settings: List::with_capacity(0),
                                held: List::with_capacity(0),
                                pending: List::with_capacity(0),
                            })))
                        }
                        c::Subject::Chips { tasks } if matches!(goals, Some("chat-goal-linked" | "chats-drafts")) => {
                            let mut chips = List::with_capacity(tasks.len());
                            for number in &tasks {
                                let mut chip = document(w, 1).chip;
                                chip.task = *number;
                                chip.phase = c::Phase::Running;
                                chip.title = doc("Away mode");
                                let (phase, details) = owner(&mut chip);
                                *phase = c::ChipPhase::Active(c::RunStage::Running { attempt: 1 });
                                details.ending = None;
                                details.ended_at = None;
                                chips.push(chip).expect("linked goal chip");
                            }
                            Some(c::Delivery::Chips(chips))
                        }
                        c::Subject::Run { task, .. } => {
                            if goals == Some("chat-terminal-working") {
                                Some(c::Delivery::RunState(Box::new(c::RunState {
                                    task,
                                    attempt: Some(1),
                                    committed_turn: 0,
                                    writing: None,
                                    call: None,
                                })))
                            } else {
                                goals.filter(|name| name.starts_with("chat-work-")).map(chat_work::run)
                            }
                        }
                        c::Subject::Terminal { .. } | c::Subject::Chips { .. } => None,
                    };
                    if let Some(document) = document {
                        deliver(&mut root, watch, document, &mut out);
                    }
                }
                jig_client::Down::Read { read, what: jig_client::Readable::Jig(what) } => {
                    let value = if settings == Some("how-follows") && matches!(what, c::Read::Project { .. }) {
                        Some(c::ReadValue::Project(Box::new(ways::how_project())))
                    } else if let Some(name) = view {
                        match &what {
                            c::Read::Activity { .. } => Some(c::ReadValue::Activity(Box::new(views::activity(name)))),
                            _ => read_value(w, &what, then, goals),
                        }
                    } else if picker.is_some() {
                        match &what {
                            c::Read::Activity { before, .. } => Some(c::ReadValue::Activity(Box::new(
                                chat_work::activity("chat-work-running", before.as_ref()),
                            ))),
                            _ => read_value(w, &what, then, goals),
                        }
                    } else if let Some(name) = settings_chat {
                        match &what {
                            c::Read::Proposal { proposal } => Some(settings_proposal(*proposal, name)),
                            c::Read::Activity { .. } => Some(c::ReadValue::Activity(Box::new(settings_activity(name)))),
                            _ => read_value(w, &what, then, goals),
                        }
                    } else {
                        read_value(w, &what, then, goals)
                    };
                    if let Some(value) = value {
                        answer(&mut root, read, value, &mut out);
                    }
                }
                jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } } => {
                    use temper_client_protocol_wire::{self as wire, v1};
                    match wire::decode_reading(&record.bytes, &v1::CEILINGS).expect("gallery forge read").read() {
                        v1::Read::Deployment => deployment_read(
                            &mut root,
                            read,
                            goals != Some("no-project"),
                            goals.is_some_and(|name| name.starts_with("told-") && name != "told-not-set-up"),
                            &mut out,
                        ),
                        v1::Read::Playground => {
                            // A resetting sample leaves the next read pending.
                            if w != (FirstRun { step: 10 }) || !playground_read {
                                playground(&mut root, w, read, &mut out);
                                playground_read = true;
                            }
                        }
                        v1::Read::Setting(setting) => {
                            let drafts =
                                matches!(then, Then::Mode(_) | Then::DraftEmpty) || goals == Some("chats-drafts");
                            if setting.kind() == 7 && (goals.is_some() || drafts) {
                                project_ways(&mut root, read, drafts, goals.is_some(), false, &mut out);
                                continue;
                            }
                            let bytes = settings.and_then(|name| {
                                if setting.kind() == 8
                                    && !matches!(
                                        name,
                                        "how-approve" | "how-reaches" | "how-rules" | "how-rules-card" | "how-team"
                                    )
                                {
                                    return None;
                                }
                                if how && setting.kind() == 7 {
                                    Some(ways::setting(name))
                                } else {
                                    Some(settings_bytes(name, setting.kind()))
                                }
                            });
                            let value = v1::SettingAnswer::new(&v1::CEILINGS, v1::SettingAnswerParts { value: bytes })
                                .expect("stock setting");
                            step(
                                &mut root,
                                Event::Client {
                                    event: jig_client::Event::Up {
                                        up: jig_client::Up::ReadDone {
                                            read,
                                            result: jig_client::Readout::Family {
                                                family: FORGE,
                                                record: Some(wire::setting_answer(&value)),
                                            },
                                        },
                                    },
                                },
                                &mut out,
                            );
                        }
                        v1::Read::ProposalItems(_) if settings_chat.is_some() => {
                            settings_proposal_values(&mut root, read, &mut out);
                        }
                        v1::Read::ProgramEnvironment => {
                            if let Address::ConnectModel { .. } = root.pages().address() {
                                let value = v1::ProgramEnvironmentAnswer::new(
                                    &v1::CEILINGS,
                                    v1::ProgramEnvironmentAnswerParts { environment: Box::new([]) },
                                )
                                .expect("default program environment");
                                step(
                                    &mut root,
                                    Event::Client {
                                        event: jig_client::Event::Up {
                                            up: jig_client::Up::ReadDone {
                                                read,
                                                result: jig_client::Readout::Family {
                                                    family: FORGE,
                                                    record: Some(wire::program_environment_answer(&value)),
                                                },
                                            },
                                        },
                                    },
                                    &mut out,
                                );
                            }
                        }
                        v1::Read::View(view) => {
                            if settings.is_some_and(|name| ways::is_draft(name) || name == "how-team")
                                && view.address() == b"roles"
                            {
                                ways::roles(&mut root, read, settings == Some("how-team"), &mut out);
                            } else if goals.is_some_and(|name| name.starts_with("chat-view-")) {
                                views::answer(&mut root, read, view.address(), &mut out);
                            }
                        }
                        v1::Read::RevisionSetting(_) | v1::Read::ProposalItems(_) => {}
                    }
                }
                jig_client::Down::Open { watch, subject: jig_client::Watchable::Family { record, .. } }
                    if record.family == FORGE =>
                {
                    use temper_client_protocol_wire::{self as wire, v1 as w};
                    let watching = wire::decode_watching(
                        &record.bytes,
                        &temper_client_domain::forge::wire_limits(&limits.forge).expect("wire limits"),
                    )
                    .expect("gallery forge watch");
                    let subject = match watching.watch() {
                        w::Watch::Devices => {
                            told_devices(&mut root, watch, &mut out);
                            continue;
                        }
                        w::Watch::Forges => {
                            forge_watch = Some(watch);
                            if settings == Some("repositories-making") {
                                repositories_making_watched(&mut root, watch, &mut out);
                            } else if to_begin {
                                to_begin_forges_watched(&mut root, w, watch, &mut out);
                            } else if copies {
                                copies_watched(&mut root, watch, &mut out);
                            } else {
                                forges_watched(&mut root, w, watch, &mut out);
                            }
                            continue;
                        }
                        w::Watch::Dismissed if to_begin => {
                            let bytes = wire::dismissals(
                                &w::Dismissals::new(
                                    &w::CEILINGS,
                                    w::DismissalsParts { dismissed: List::with_capacity(0) },
                                )
                                .expect("unfinished To begin"),
                            );
                            step(
                                &mut root,
                                Event::Client {
                                    event: jig_client::Event::Up {
                                        up: jig_client::Up::FamilyDelivered {
                                            watch,
                                            record: jig_client::Family {
                                                family: wire::FORGE,
                                                version: wire::CLIENT_VERSION,
                                                bytes,
                                            },
                                        },
                                    },
                                },
                                &mut out,
                            );
                            continue;
                        }
                        w::Watch::Dismissed if matches!(goals, Some("chat-goal-linked" | "chats-drafts")) => {
                            let mut dismissed = List::with_capacity(1);
                            dismissed
                                .push(w::PersonFact::GoalFrom(
                                    w::GoalFrom::new(
                                        &w::CEILINGS,
                                        w::GoalFromParts {
                                            project: PROJECT,
                                            chat: 1,
                                            goal: 2,
                                            at: offset(w, -1).as_nanos(),
                                        },
                                    )
                                    .expect("goal start"),
                                ))
                                .expect("one start");
                            let bytes = wire::dismissals(
                                &w::Dismissals::new(&w::CEILINGS, w::DismissalsParts { dismissed })
                                    .expect("person facts"),
                            );
                            step(
                                &mut root,
                                Event::Client {
                                    event: jig_client::Event::Up {
                                        up: jig_client::Up::FamilyDelivered {
                                            watch,
                                            record: jig_client::Family {
                                                family: wire::FORGE,
                                                version: wire::CLIENT_VERSION,
                                                bytes,
                                            },
                                        },
                                    },
                                },
                                &mut out,
                            );
                            continue;
                        }
                        w::Watch::Ways(subject) if (asking || how) && !asking_chat => {
                            ways::deliver(&mut root, watch, subject.project(), settings, &mut out);
                            continue;
                        }
                        w::Watch::Follows(_) if settings == Some("how-follows") => {
                            ways::follows(&mut root, watch, &mut out);
                            continue;
                        }
                        w::Watch::Change(subject) => subject,
                        w::Watch::Away
                        | w::Watch::Landing(_)
                        | w::Watch::Landings(_)
                        | w::Watch::Follows(_)
                        | w::Watch::Decided(_)
                        | w::Watch::Status(_)
                        | w::Watch::Drafts(_)
                        | w::Watch::Ways(_)
                        | w::Watch::Trials
                        | w::Watch::Approvals
                        | w::Watch::Changes(_)
                        | w::Watch::Dismissed => {
                            continue;
                        }
                    };
                    if let Some(panel) = change_panel(w, subject.task()) {
                        for up in [
                            jig_client::Up::Watched { watch, event: c::WatchEvent::Opened },
                            jig_client::Up::FamilyDelivered {
                                watch,
                                record: jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes: wire::change_panel(&panel),
                                },
                            },
                        ] {
                            step(&mut root, Event::Client { event: jig_client::Event::Up { up } }, &mut out);
                        }
                    }
                }
                jig_client::Down::Send { attempt, .. } if w == (FirstRun { step: 21 }) => {
                    github::answer_page(&mut root, attempt, &mut out);
                }
                jig_client::Down::Save { .. }
                | jig_client::Down::Erase { .. }
                | jig_client::Down::Send { .. }
                | jig_client::Down::Open { .. }
                | jig_client::Down::Continue { .. }
                | jig_client::Down::Close { .. } => {}
            }
        }
    }
    assert!(out.is_empty(), "world fixture settles: {w:?} {settings:?} {out:?}");
    first_run_ready(&mut root, w, forge_watch, &mut out);
    if let Some(name) = settings_chat
        && !name.starts_with("ask-chat-proposal")
    {
        finish_settings_proposal(&mut root, settings_chat_watch.expect("settings chat watch"), name, &mut out);
    }
    if let Some(name) = settings.filter(|name| name.starts_with("ask-ways-") || ways::is_how(name)) {
        ways::press(&mut root, name, settings_watch.expect("ways settings watch"), &mut out);
    }
    (root, accounts_watch)
}
fn started(address: Address) -> (Domain, Queue<Request>) {
    let limits = limits();
    let mut root = temper_client_domain::unlinked(&limits, SEED);
    let mut out = Queue::uncapped();
    step(&mut root, Event::Start { address, kept: List::with_capacity(0), with: None }, &mut out);
    for event in [
        jig_client::Event::StartUnlinked { kept: List::with_capacity(0) },
        jig_client::Event::SignIn { with: jig_client::With::Transport },
        jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
    ] {
        step(&mut root, Event::Client { event }, &mut out);
    }
    (root, out)
}
fn first_run_ready(root: &mut Domain, w: World, forge_watch: Option<Token>, out: &mut Queue<Request>) {
    if w == (FirstRun { step: 8 }) {
        adopt_owned_sample(root, forge_watch.expect("owned forge watch"), out);
    }
    if matches!(w, FirstRun { step: 1..=4 }) {
        assert!(root.forge().playground_running());
    }
}
fn forges_watched(root: &mut Domain, world: World, watch: Token, out: &mut Queue<Request>) {
    use temper_client_protocol_wire as wire;
    let record = jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: forges(world) };
    step(
        root,
        Event::Client { event: jig_client::Event::Up { up: jig_client::Up::FamilyDelivered { watch, record } } },
        out,
    );
}
pub(super) fn repositories_making_world() -> Domain {
    world_in_layout(
        FirstRun { step: 7 },
        Address::Repositories { project: PROJECT },
        Then::Shown,
        false,
        None,
        Some("repositories-making"),
        false,
    )
    .0
}
fn to_begin_forges_watched(root: &mut Domain, world: World, watch: Token, out: &mut Queue<Request>) {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut parts = wire::decode_forges(&forges(world), &w::CEILINGS).expect("Ask forges").into_parts();
    let row = parts.forges.get_mut(0).expect("Ask forge");
    let mut forge = row.clone().into_parts();
    let repository = forge.repositories.get_mut(0).expect("Ask repository");
    let mut adoption = repository.clone().into_parts();
    adoption.project = PROJECT;
    *repository = w::Adoption::new(&w::CEILINGS, adoption).expect("Ask adoption");
    *row = w::ForgeRow::new(&w::CEILINGS, forge).expect("Ask forge");
    let record = jig_client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::forges(&w::Forges::new(&w::CEILINGS, parts).expect("Ask forges")),
    };
    step(
        root,
        Event::Client { event: jig_client::Event::Up { up: jig_client::Up::FamilyDelivered { watch, record } } },
        out,
    );
}
fn repositories_making_watched(root: &mut Domain, watch: Token, out: &mut Queue<Request>) {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut rows =
        wire::decode_forges(&forges(FirstRun { step: 7 }), &w::CEILINGS).expect("making forge").into_parts().forges;
    let row = rows.get_mut(0).expect("owned forge");
    let mut parts = home_copies(row.clone().into_parts(), "home-copying");
    let mut makings = List::with_capacity(3);
    for making in parts.makings.into_boxed() {
        makings.push(making).expect("making fixture rows");
    }
    parts.makings = makings;
    parts
        .makings
        .push(
            w::RepositoryMaking::new(
                &w::CEILINGS,
                w::RepositoryMakingParts {
                    org: b"octo".as_slice().into(),
                    name: b"vsr".as_slice().into(),
                    from: w::MakeFrom::Copy(
                        w::MakeCopy::new(
                            &w::CEILINGS,
                            w::MakeCopyParts { address: b"https://github.com/octo/vsr".as_slice().into() },
                        )
                        .expect("failed source"),
                    ),
                    into: Some(
                        w::MakeInto::new(&w::CEILINGS, w::MakeIntoParts { project: PROJECT, home: false })
                            .expect("failed project"),
                    ),
                    state: w::MakingState::Failed(
                        w::MakingFailed::new(
                            &w::CEILINGS,
                            w::MakingFailedParts {
                                step: w::MakeStep::Repository,
                                reason: w::MakeFailure::NotAllowed,
                                words: b"You can not import from disallowed hosts.".as_slice().into(),
                            },
                        )
                        .expect("forge words"),
                    ),
                },
            )
            .expect("failed making"),
        )
        .expect("failed row room");
    *row = w::ForgeRow::new(&w::CEILINGS, parts).expect("making fixture");
    let record = jig_client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::forges(
            &w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows, linked: false }).expect("making rows"),
        ),
    };
    step(
        root,
        Event::Client { event: jig_client::Event::Up { up: jig_client::Up::FamilyDelivered { watch, record } } },
        out,
    );
}
fn copies_watched(root: &mut Domain, watch: Token, out: &mut Queue<Request>) {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut rows =
        wire::decode_forges(&forges(FirstRun { step: 7 }), &w::CEILINGS).expect("Ready forges").into_parts().forges;
    let row = rows.get_mut(0).expect("Ready forge");
    let mut parts = row.clone().into_parts();
    parts.first = None;
    parts.repositories.clear();
    let mut addresses = List::with_capacity(2);
    for address in [b"https://github.com/acme/widget".as_slice(), b"https://source.invalid/team/tool".as_slice()] {
        addresses.push(address.into()).expect("two copies");
    }
    parts.starts_with = w::StartsWith::Copies(
        w::CopyAddresses::new(&w::CEILINGS, w::CopyAddressesParts { addresses }).expect("Ready copies"),
    );
    *row = w::ForgeRow::new(&w::CEILINGS, parts).expect("Ready forge");
    let record = jig_client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::forges(
            &w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows, linked: false }).expect("Ready rows"),
        ),
    };
    step(
        root,
        Event::Client { event: jig_client::Event::Up { up: jig_client::Up::FamilyDelivered { watch, record } } },
        out,
    );
}
#[expect(clippy::too_many_lines, reason = "complete illustrative forge rows keep their additive fields together")]
fn forges(world: World) -> Box<[u8]> {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    if let FirstRun { step: 20..=22 } = world {
        return github::forges(world);
    }
    let limits = temper_client_domain::forge::wire_limits(&limits().forge).expect("wire limits");
    let mut rows = List::with_capacity(3);
    if has_playground(world) && world != (FirstRun { step: 7 }) {
        rows.push(
            w::ForgeRow::new(
                &limits,
                w::ForgeRowParts {
                    starts_with: w::StartsWith::Repository,
                    makings: List::with_capacity(0),
                    host: b"git.example:3000".as_slice().into(),
                    number: FORGE,
                    kind: w::Kind::Forgejo,
                    mode: w::Mode::Fake,
                    connection: w::Connection::Connected(
                        w::Connected::new(&limits, w::ConnectedParts { id: 1, login: b"temper".as_slice().into() })
                            .expect("demo login"),
                    ),
                    offer: w::Offer::NotOffered,
                    first: None,
                    installations: List::with_capacity(0),
                    ended: None,
                    app: None,
                    web: None,
                    repositories: List::with_capacity(0),
                    owned: None,
                    admin: false,
                    read_only: false,
                    ended_stop: None,
                },
            )
            .expect("demo forge"),
        )
        .expect("one forge");
    }
    if world == Solo || world == Reference {
        let mut installations = List::with_capacity(1);
        installations
            .push(
                w::Installation::new(
                    &limits,
                    w::InstallationParts { id: 1, account: None, suspended: false, stray: false },
                )
                .expect("accepted installation"),
            )
            .expect("installation room");
        rows.push(
            w::ForgeRow::new(
                &limits,
                w::ForgeRowParts {
                    starts_with: w::StartsWith::Repository,
                    makings: List::with_capacity(0),
                    host: b"git.example:3000".as_slice().into(),
                    number: 2,
                    kind: w::Kind::Github,
                    mode: w::Mode::Managed,
                    connection: w::Connection::Connected(
                        w::Connected::new(&limits, w::ConnectedParts { id: 2, login: b"temper".as_slice().into() })
                            .expect("connected forge"),
                    ),
                    offer: w::Offer::NotOffered,
                    first: None,
                    installations,
                    ended: None,
                    app: None,
                    web: None,
                    repositories: if world == Solo {
                        let mut repositories = List::with_capacity(1);
                        repositories
                            .push(
                                w::Adoption::new(
                                    &limits,
                                    w::AdoptionParts {
                                        owner: b"robin".as_slice().into(),
                                        name: b"tally".as_slice().into(),
                                        project: PROJECT,
                                    },
                                )
                                .expect("solo's adopted repository"),
                            )
                            .expect("one repository");
                        repositories
                    } else {
                        List::with_capacity(0)
                    },
                    owned: None,
                    admin: false,
                    read_only: false,
                    ended_stop: None,
                },
            )
            .expect("personal forge"),
        )
        .expect("forge row");
    }
    if let FirstRun { step: step @ (6..=9 | 23..=25) } = world {
        let mut repositories = List::with_capacity(1);
        if step == 7 {
            repositories
                .push(
                    w::Adoption::new(
                        &limits,
                        w::AdoptionParts {
                            owner: b"work".as_slice().into(),
                            name: b"hello".as_slice().into(),
                            project: 2,
                        },
                    )
                    .expect("first adoption"),
                )
                .expect("one repository");
        }
        let connection = match step {
            9 => w::Connection::SetUpFailed(
                w::SetUpFailed::new(
                    &limits,
                    w::SetUpFailedParts {
                        why: "its port, 127.0.0.1:8368, is taken by another program. temper box’s output has the rest."
                            .as_bytes()
                            .into(),
                        at: Some(w::FailedStep::Prepare),
                    },
                )
                .expect("failed preparation"),
            ),
            6 => w::Connection::SettingUp(
                w::SettingUp::new(&limits, w::SettingUpParts { step: Some(w::SetUpStep::Prepare), fetched: None })
                    .expect("preparing"),
            ),
            24 | 25 => w::Connection::SettingUp(
                w::SettingUp::new(
                    &limits,
                    w::SettingUpParts {
                        step: Some(w::SetUpStep::Fetch),
                        fetched: Some(
                            w::SetUpFetched::new(
                                &limits,
                                w::SetUpFetchedParts {
                                    binary: w::SetUpBinary::Forgejo,
                                    done: if step == 24 { 42_000_000 } else { 41_000_000 },
                                    total: if step == 24 { Some(100_000_000) } else { None },
                                },
                            )
                            .expect("fetch progress"),
                        ),
                    },
                )
                .expect("fetching"),
            ),
            7 => w::Connection::Connected(
                w::Connected::new(&limits, w::ConnectedParts { id: 2, login: b"temper".as_slice().into() })
                    .expect("connected"),
            ),
            23 => w::Connection::NotConnected,
            _ => w::Connection::NotSetUp,
        };
        rows.push(
            w::ForgeRow::new(
                &limits,
                w::ForgeRowParts {
                    starts_with: w::StartsWith::Repository,
                    makings: List::with_capacity(0),
                    host: b"git.example:3000".as_slice().into(),
                    number: 3,
                    kind: w::Kind::Forgejo,
                    mode: if step == 23 { w::Mode::Managed } else { w::Mode::Owned },
                    connection,
                    offer: if step == 23 { w::Offer::NotOffered } else { w::Offer::Offered },
                    first: if matches!(step, 6 | 7 | 9 | 24 | 25) {
                        Some(
                            w::FirstRepository::new(
                                &limits,
                                w::FirstRepositoryParts {
                                    playground: false,
                                    org: b"work".as_slice().into(),
                                    repository: b"hello".as_slice().into(),
                                    state: if step == 7 {
                                        w::FirstState::Made(
                                            w::Made::new(&limits, w::MadeParts { id: 42 }).expect("made"),
                                        )
                                    } else {
                                        w::FirstState::Making
                                    },
                                    label: b"temper-host".as_slice().into(),
                                },
                            )
                            .expect("first repository"),
                        )
                    } else {
                        None
                    },
                    installations: List::with_capacity(0),
                    ended: None,
                    app: None,
                    web: if step == 23 { Some(b"http://forgejo.test".as_slice().into()) } else { None },
                    repositories,
                    owned: if step == 23 {
                        None
                    } else {
                        Some(
                            w::OwnedForge::new(
                                &limits,
                                w::OwnedForgeParts {
                                    forgejo: b"17.0.0".as_slice().into(),
                                    runner: Some(b"13.2.0".as_slice().into()),
                                    password: b"/fake/forge/person_password".as_slice().into(),
                                },
                            )
                            .expect("owned pins"),
                        )
                    },
                    admin: step == 23,
                    read_only: false,
                    ended_stop: None,
                },
            )
            .expect("owned Forgejo"),
        )
        .expect("forge row");
    }
    wire::forges(&w::Forges::new(&limits, w::ForgesParts { forges: rows, linked: false }).expect("gallery forges"))
}
fn change_panel(w: World, task: u64) -> Option<temper_client_protocol_wire::v1::ChangePanel> {
    use temper_client_protocol_wire::v1 as w;
    let l = temper_client_domain::forge::wire_limits(&limits().forge).expect("wire limits");
    let (branch, head_prefix, pull_number, pull_address, step) = match (w, task) {
        (Reference, 28) => {
            let mut ahead = List::with_capacity(1);
            ahead.push(26).expect("task ahead");
            (
                b"temper/28/build-path-one-cache-per-lane".as_slice(),
                [0xc4, 0x0e, 0x9b, 0x20],
                216,
                b"https://github.com/acme/kit/pull/216".as_slice(),
                w::Step::Queued(w::Queued::new(&l, w::QueuedParts { place: 2, ahead }).expect("queued")),
            )
        }
        (Reference, 24) => {
            let mut merge = [0; 32];
            merge[..4].copy_from_slice(&[0x4e, 0x1a, 0x9c, 0x70]);
            (
                b"temper/24/fake-server-replay-fixtures".as_slice(),
                [0x8b, 0x2d, 0x41, 0x00],
                212,
                b"https://github.com/acme/kit/pull/212".as_slice(),
                w::Step::Landed(w::Landed::new(&l, w::LandedParts { merge }).expect("landed")),
            )
        }
        _ => return None,
    };
    let mut head = [0; 32];
    head[..4].copy_from_slice(&head_prefix);
    Some(
        w::ChangePanel::new(
            &l,
            w::ChangePanelParts {
                way: w::Way::Queue,
                in_landing: None,
                task,
                forge: 1,
                owner: Box::from(b"acme".as_slice()),
                name: Box::from(b"kit".as_slice()),
                branch: Box::from(branch),
                landing: Box::from(b"main".as_slice()),
                head: Some(head),
                pull: Some(gallery_pull(&l, pull_number, pull_address)),
                step,
                hold: None,
                runs: None,
                ci_required: true,
                ci: w::Ci::Passed,
                approvals: List::with_capacity(0),
                unmapped_reviews: List::with_capacity(0),
                gates: List::with_capacity(0),
                repairs: w::Used::new(&l, w::UsedParts { used: 0, limit: 3 }).expect("repairs"),
                resolutions: w::Used::new(&l, w::UsedParts { used: 0, limit: 3 }).expect("resolutions"),
                updates: w::Used::new(&l, w::UsedParts { used: 0, limit: 3 }).expect("updates"),
                questions: w::Questions::new(
                    &w::CEILINGS,
                    w::QuestionsParts {
                        route: w::Route::None,
                        taken: w::Used::new(&w::CEILINGS, w::UsedParts { used: 0, limit: 4 }).expect("question limit"),
                        open: List::with_capacity(0),
                    },
                )
                .expect("no questions"),
            },
        )
        .expect("change panel"),
    )
}
fn gallery_pull(
    limits: &temper_client_protocol_wire::v1::Limits,
    number: u64,
    address: &[u8],
) -> temper_client_protocol_wire::v1::Pull {
    use temper_client_protocol_wire::v1 as w;
    w::Pull::new(limits, w::PullParts { number, address: Box::from(address) }).expect("pull")
}
fn deployment_read(root: &mut Domain, read: Token, welcome: bool, push: bool, out: &mut Queue<Request>) {
    use temper_client_protocol_wire::{self as wire, v1};
    let answer = v1::DeploymentAnswer::new(
        &v1::CEILINGS,
        v1::DeploymentAnswerParts { push: push.then_some([4; 65]), welcome, owned: true, program_login: None },
    )
    .expect("deployment answer");
    step(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family {
                        family: wire::FORGE,
                        record: Some(wire::deployment_answer(&answer)),
                    },
                },
            },
        },
        out,
    );
}
fn playground(root: &mut Domain, w: World, read: Token, out: &mut Queue<Request>) {
    use temper_client_protocol_wire::{self as wire, v1};
    let record = if has_playground(w) && !matches!(w, FirstRun { step: 0 | 7 }) {
        let mut cues = List::with_capacity(3);
        for cue in ["Add CSV export to widget", "Add a dark theme to widget", "Fix widget’s empty search state"] {
            cues.push(Box::from(cue.as_bytes())).expect("cue");
        }
        let value = v1::PlaygroundAnswer::new(
            &v1::CEILINGS,
            v1::PlaygroundAnswerParts {
                state: if w == (FirstRun { step: 10 }) {
                    v1::PlaygroundState::Resetting
                } else {
                    v1::PlaygroundState::Running
                },
                project: Some(PROJECT),
                forge: 1,
                provider: 17,
                owner: Box::from(b"playground".as_slice()),
                name: Box::from(b"widget".as_slice()),
                cues,
            },
        )
        .expect("checked demo");
        Some(wire::playground_answer(&value))
    } else {
        None
    };
    step(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family { family: wire::FORGE, record },
                },
            },
        },
        out,
    );
}
fn owner(chip: &mut c::Chip) -> (&mut c::ChipPhase, &mut c::ChipDetails) {
    let c::ChipPhaseFacts { phase, details } = chip.details.as_mut();
    (phase, details)
}
fn finish_document(mut t: c::Task) -> c::Task {
    let compact = t.chip.phase;
    let executor = t.chip.executor;
    match executor {
        c::ChipExecutor::Agent { .. } => {}
        c::ChipExecutor::Job { .. }
        | c::ChipExecutor::Procedure { .. }
        | c::ChipExecutor::Party { .. }
        | c::ChipExecutor::Role { .. } => {
            t.settings = None;
        }
    }
    let (phase, details) = owner(&mut t.chip);
    *phase = match compact {
        c::Phase::Ended | c::Phase::Done => c::ChipPhase::Ended,
        c::Phase::Running => c::ChipPhase::Active(c::RunStage::Running { attempt: 1 }),
        c::Phase::Waiting
        | c::Phase::Idle
        | c::Phase::Due
        | c::Phase::Preparing
        | c::Phase::Claimed
        | c::Phase::BackingOff
        | c::Phase::Active
        | c::Phase::Closing
        | c::Phase::Held
        | c::Phase::Failed
        | c::Phase::Cancelled => c::ChipPhase::Active(c::RunStage::Idle),
    };
    details.executor = match executor {
        c::ChipExecutor::Job { kind } => c::Executor::Job { kind },
        c::ChipExecutor::Agent { charter } => c::Executor::Agent { charter },
        c::ChipExecutor::Procedure { connector, kind } => c::Executor::Procedure { connector, code: kind },
        c::ChipExecutor::Party { .. } | c::ChipExecutor::Role { .. } => {
            unreachable!("worlds have agents and change procedures")
        }
    };
    details.wake = t.wake;
    details.budget = t.budget.clone();
    details.activity = t.activity;
    details.live_delegates = t.delegates.len();
    if compact == c::Phase::Ended || compact == c::Phase::Done {
        details.ended_at = Some(t.last_activity);
    }
    match &t.result {
        Some(c::TaskResult::Ending(ending)) => {
            details.ending = Some(*ending.clone());
            details.ended_at = Some(t.last_activity);
        }
        Some(c::TaskResult::Document(_)) | None => {}
    }
    t
}
fn task_line(t: c::Task) -> c::TaskLine {
    c::TaskLine {
        ended_at: if t.chip.phase == c::Phase::Ended || t.chip.phase == c::Phase::Done {
            Some(t.last_activity)
        } else {
            None
        },
        chip: t.chip,
        project: t.project,
        requester: t.requester,
        tracked: t.priority,
        label: t.label,
        flags: t.flags,
        changed: t.changed,
        last_activity: t.last_activity,
        budget: t.budget,
        waits_on: t.waits_on,
        phases: c::PhaseCounts::ZERO,
        delegates_done: 0,
        delegates_total: 0,
        held_below: 0,
        conversation: None,
    }
}
fn chat_rows(w: World, ended: bool) -> List<c::TaskLine> {
    let numbers: &[u64] = match (w, ended) {
        (Reference | FirstRun { step: 2 | 3 }, false) | (Solo, true) => &[1],
        (Reference, true) => &[3, 2],
        (Solo, false) => &[6, 10],
        (FirstRun { step: 4..=5 }, false) => &[4, 1],
        (FirstRun { .. } | World::Bare | World::WholeStory, _) => &[],
    };
    let mut rows = List::with_capacity(u32::try_from(numbers.len()).expect("small chats"));
    for number in numbers {
        rows.push(task_line(document(w, *number))).expect("row");
    }
    rows
}
fn inbox_document(w: World) -> c::Inbox {
    let keys: &[(c::ObjectKey, c::InboxGroup)] = match w {
        Reference => &[(c::ObjectKey::Task(1), c::InboxGroup::Replies)],
        Solo => &[
            (c::ObjectKey::Proposal(PROPOSAL), c::InboxGroup::Proposals),
            (c::ObjectKey::Question { task: 3, message: 1 }, c::InboxGroup::Questions),
            (c::ObjectKey::Task(10), c::InboxGroup::Replies),
            (c::ObjectKey::Task(6), c::InboxGroup::Replies),
        ],
        FirstRun { .. } | World::Bare | World::WholeStory => &[],
    };
    let mut entries = List::with_capacity(u32::try_from(keys.len()).expect("small inbox"));
    for (object, _group) in keys {
        entries
            .push(documents::entry(
                *object,
                PROJECT,
                1,
                match object {
                    c::ObjectKey::Proposal(_) => offset(w, -12),
                    c::ObjectKey::Question { .. } => offset(w, -18),
                    c::ObjectKey::Task(task) => document(w, *task).last_activity,
                    c::ObjectKey::Escalation(_)
                    | c::ObjectKey::PersonTask(_)
                    | c::ObjectKey::Result(_)
                    | c::ObjectKey::Note { .. } => unreachable!("world inbox has proposals, questions and replies"),
                },
                match object {
                    c::ObjectKey::Task(task) => Some(Box::new(reply_line(w, *task))),
                    c::ObjectKey::Proposal(_)
                    | c::ObjectKey::Escalation(_)
                    | c::ObjectKey::Question { .. }
                    | c::ObjectKey::PersonTask(_)
                    | c::ObjectKey::Result(_)
                    | c::ObjectKey::Note { .. } => None,
                },
            ))
            .expect("entry");
    }
    {
        let counts = needs(w);
        documents::inbox(
            entries,
            counts
                .questions
                .saturating_add(counts.proposals)
                .saturating_add(counts.escalations)
                .saturating_add(counts.person_tasks),
            c::Count { value: if w == Solo { 2 } else { u32::from(w == Reference) }, limited: false },
            0,
            false,
        )
    }
}
fn proposed() -> List<c::ProposedTask> {
    let mut base = base_task(0, JSON_TITLE);
    base.authority.authority.spend = 6;
    // Both distinct written fields fit the proposed task's specification.
    base.spec.specification.words = doc(
        "JSON output for tally report\n\nA goal of yours: `tally report --json`, one object per row, with tests, landed into main.",
    );
    let mut resources = List::with_capacity(1);
    resources
        .push(c::Part {
            connector: FORGE,
            kind: c::PartKind::Resource,
            key: c::PartKey::Path(path(&[b"forge", b"github.com", b"robin", b"tally", b"branch", b"main"])),
        })
        .expect("landing branch");
    let mut tasks = List::with_capacity(1);
    tasks
        .push(c::ProposedTask {
            number: 1,
            spec: base.spec,
            executor: c::Executor::Agent { charter: 2 },
            contract: base.result_contract,
            authority: base.authority,
            deadline: c::NewDeadline::Named,
            budget: 6,
            funder: c::Funder::Pool { project: PROJECT, person: 73, period: 1 },
            dependencies: List::with_capacity(0),
            resources,
            wake: base.wake,
            priority: Some(2),
            cut: false,
        })
        .expect("proposed goal");
    tasks
}
fn proposal() -> c::ReadValue {
    c::ReadValue::Proposal(Box::new({
        let mut funding = skein_lib::List::with_capacity(1);
        funding
            .push(c::FundingOffer {
                source: c::Funder::Pool { project: PROJECT, person: 73, period: 1 },
                needed: 6,
                left: Some(40),
                after: Some(34),
            })
            .expect("one funding source");
        c::Proposal {
            projects: List::with_capacity(0),
            key: PROPOSAL,
            project: PROJECT,
            reason: c::DocumentText::complete(text(JSON_WHY)),
            action: c::ProposalAction::Batch(proposed()),
            as_holder: true,
            waiting: c::ProposalWaiting { holder: c::HistoryHolder::Person(73), since: offset(Solo, -12) },
            funding,
            acts: offers(&[c::ActKind::Accept, c::ActKind::Reject]),
        }
    }))
}
#[expect(clippy::too_many_lines, reason = "one dispatch answers every gallery read with its world fixture")]
fn read_value(w: World, what: &c::Read, then: Then, goals: Option<&str>) -> Option<c::ReadValue> {
    if let Some(name) = goals.filter(|name| name.starts_with("chat-work-"))
        && let c::Read::Activity { before, .. } = what
    {
        return Some(c::ReadValue::Activity(Box::new(chat_work::activity(name, before.as_ref()))));
    }
    if let Some(goal) = page_goal(goals)
        && let Some(value) = goal.read(w, what)
    {
        return Some(value);
    }
    match what {
        c::Read::Tasks { query, .. } if query.tracked == Some(true) && query.requester.is_some() => {
            let mut rows = List::with_capacity(1);
            for number in [3, 12] {
                if (w == Solo && number == 3) || (w == Reference && number == 12) {
                    let task = document(w, number);
                    if *query == temper_client_domain::pages::goal::own_line_query(&task) {
                        let children = facts(w, number).children;
                        let mut line = task_line(task);
                        line.delegates_total = u32::try_from(children.len()).expect("few delegates");
                        line.delegates_done = u32::try_from(
                            children
                                .iter()
                                .filter(|n| matches!(facts(w, **n).phase, c::Phase::Done | c::Phase::Ended))
                                .count(),
                        )
                        .expect("few delegates");
                        rows.push(line).expect("one goal");
                    }
                }
            }
            Some(c::ReadValue::Tasks(Box::new(c::TaskPage {
                live: rows,
                live_after: None,
                ended: c::EndedGroup::None,
            })))
        }
        c::Read::Tasks { query, .. } if query.tracked == Some(true) => {
            Some(c::ReadValue::Tasks(Box::new(goal_page(w, goals))))
        }
        c::Read::Notes { scope, .. } => Some(c::ReadValue::Notes(Box::new(
            if let Some(name) = goals.filter(|name| name.starts_with("remembered-")) {
                remembered_notes(name, scope)
            } else if goals == Some("chats-drafts") && *scope == (c::NoteScope::Tasks { project: PROJECT }) {
                let mut rows = List::with_capacity(2);
                for (task, title) in [(1, "Away mode"), (3, "Report format")] {
                    rows.push(c::Note {
                        number: 1,
                        revision: c::Revision { value: 1 },
                        scope: c::NoteScope::Task { project: PROJECT, task },
                        description: doc(title),
                        body: c::DocumentText { bytes: text(""), cut: true },
                        writer: c::NoteAuthor::Task { task, attempt: 1 },
                        tasks: List::with_capacity(0),
                        changed: None,
                        acts: List::with_capacity(0),
                        cut: false,
                    })
                    .expect("chat document");
                }
                c::NotePage { rows, before: None, after: None, cut: false }
            } else if matches!(then, Then::Mode(_) | Then::Sheet(Confirmation::StartOver))
                && *scope == (c::NoteScope::Task { project: PROJECT, task: 10 })
            {
                asking_note(scope)
            } else if matches!(then, Then::Mode(_)) && *scope == (c::NoteScope::Task { project: PROJECT, task: 1 }) {
                chat_note(scope)
            } else {
                goal_notes(w, scope)
            },
        ))),
        c::Read::Inbox { group, project, goal, .. } => {
            Some(c::ReadValue::Inbox(Box::new(goal_inbox(w, *group, *project, *goal, goals))))
        }
        c::Read::Tasks { query: c::TaskQuery { parent: Some(parent), phase, .. }, .. }
            if *phase == Some(c::PhaseClass::Ended) =>
        {
            let children = facts(w, *parent).children;
            let mut rows = List::with_capacity(u32::try_from(children.len()).expect("small children"));
            for child in children {
                let task = document(w, *child);
                if matches!(task.chip.phase, c::Phase::Done | c::Phase::Ended) {
                    rows.push(task_line(task)).expect("ended child");
                }
            }
            Some(c::ReadValue::Tasks(Box::new(c::TaskPage {
                live: List::with_capacity(0),
                live_after: None,
                ended: c::EndedGroup::Rows { rows, after: None, cut: false },
            })))
        }
        c::Read::Tasks { .. } => Some(c::ReadValue::Tasks(Box::new(c::TaskPage {
            live: List::with_capacity(0),
            live_after: None,
            ended: c::EndedGroup::Rows { rows: ended_rows(w), after: None, cut: false },
        }))),
        c::Read::Task { task: number } => Some(c::ReadValue::Task(Box::new(shown_task(w, *number, then)))),
        c::Read::ActivityBeside { .. } => Some(c::ReadValue::Activity(Box::new(c::ActivityPage {
            rows: List::with_capacity(0),
            before: None,
            after: None,
            cut: false,
        }))),
        c::Read::Activity { task: number, before } => Some(c::ReadValue::Activity(Box::new(
            if matches!(then, Then::Mode(_) | Then::DraftEmpty) && *number == 10 {
                asking_activity()
            } else if then == Then::DraftEmpty && *number == 1 {
                c::ActivityPage { rows: List::with_capacity(0), before: None, after: None, cut: false }
            } else if matches!(then, Then::Mode(_)) && *number == 1 {
                chat_activity()
            } else {
                activity_window(w, *number, before.as_ref())
            },
        ))),
        c::Read::Proposal { .. } => {
            let mut value = proposal();
            if then == Then::HeldTask
                && let c::ReadValue::Proposal(proposal) = &mut value
            {
                let task = shown_task(Solo, 3, Then::Held);
                proposal.reason = doc("The CSV import task is held after its run failed. Release it to try again.");
                proposal.action = c::ProposalAction::Release { task: c::TaskLink::Live(task.chip), hold: task.hold };
                proposal.funding = List::with_capacity(0);
            }
            Some(value)
        }
        c::Read::Question { .. } => Some(c::ReadValue::Question(Box::new(c::Question {
            task: document(Solo, 3).chip,
            from: Some(c::Requester::Task(3)),
            message: 1,
            words: c::DocumentText::complete(text(if goals == Some("chat-terminal-question") {
                "Keep the trailing comma?\n\nChoice: Keep it (recommended)\nChoice: Drop it"
            } else {
                QUESTION
            })),
            since: offset(Solo, -18),
            acts: offers(&[c::ActKind::Answer]),
            ending: None,
        }))),
        c::Read::Settings
        | c::Read::Setting(_)
        | c::Read::Revision(_)
        | c::Read::TaskText { .. }
        | c::Read::Revisions(_)
        | c::Read::Me
        | c::Read::Turn { .. }
        | c::Read::Escalation { .. }
        | c::Read::PersonTask { .. }
        | c::Read::Result { .. }
        | c::Read::Ended { .. }
        | c::Read::History { .. }
        | c::Read::Spend { .. }
        | c::Read::Project { .. }
        | c::Read::Deployment => None,
    }
}
fn agent_line(words: &str, number: u32, read: Option<u64>) -> c::DecodedActivityLine {
    let mut blocks = List::with_capacity(1);
    let bytes = words.as_bytes();
    let limit = usize::try_from(limits().client.domain.turn_text).expect("turn text limit");
    let length = words.floor_char_boundary(bytes.len().min(limit));
    let text = c::Text::new(Box::from(&bytes[..length]), limits().client.domain.turn_text).expect("received turn text");
    blocks.push(c::Block::Text(text)).expect("agent text");
    turn_line(&blocks, number, read, length < bytes.len())
}
fn turn_line(source: &List<c::Block>, number: u32, read: Option<u64>, mut cut: bool) -> c::DecodedActivityLine {
    let mut blocks = List::with_capacity(limits().client.domain.rows.min(source.len()));
    for block in source {
        if blocks.len() == blocks.capacity() {
            cut = true;
            break;
        }
        blocks.push(block.clone()).expect("receiving block window");
    }
    let mut messages = List::with_capacity(1);
    messages.push(c::Message { speaker: c::Speaker::Assistant, blocks }).expect("message");
    documents::turn(
        1,
        Wall::EPOCH,
        c::TurnName { attempt: 1, number },
        c::TurnRead::Decoded(c::Turn {
            messages,
            used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
            spent: 0,
            read_through: read,
            llm: None,
            cut,
        }),
    )
}
fn tools_line(calls: &[(&str, u32)], made: &[u64], proposes: bool, number: u32) -> c::DecodedActivityLine {
    let mut blocks = List::with_capacity(32);
    let mut call_number: u64 = 0;
    for (name, count) in calls {
        for _ in 0..*count {
            call_number = call_number.checked_add(1).expect("small calls");
            let mut objects = List::with_capacity(4);
            if *name == "delegate" {
                for task in made {
                    objects.push(c::ObjectKey::Task(*task)).expect("delegate object");
                }
            }
            if *name == "propose" && proposes {
                objects.push(c::ObjectKey::Proposal(PROPOSAL)).expect("proposal object");
            }
            blocks
                .push(c::Block::Call(c::Call {
                    number: call_number,
                    tool: text(name),
                    input: text(""),
                    outcome: None,
                    failed: false,
                    objects,
                    parts: List::with_capacity(0),
                }))
                .expect("tool call");
        }
    }
    turn_line(&blocks, number, None, false)
}
// BEGIN WORLD DATA
/// Goals' owner facts and report headlines (temper/docs/design/client/web/gallery.md, 2).
struct GoalFacts {
    number: u64,
    title: &'static str,
    state: GoalState,
    done: u32,
    total: u32,
    headline: Option<&'static str>,
    at: i64,
}
#[expect(clippy::too_many_lines, reason = "one table keeps the gallery’s goal facts beside their report headlines")]
fn goal_facts(w: World) -> &'static [GoalFacts] {
    match w {
        Solo => &[
            GoalFacts {
                number: 101,
                title: "CSV import",
                state: GoalState::Needs { funds: true },
                done: 1,
                total: 3,
                headline: Some("The quoted-field parser landed; errors with line and column need more funds."),
                at: -12,
            },
            GoalFacts {
                number: 102,
                title: JSON_TITLE,
                state: GoalState::Needs { funds: false },
                done: 0,
                total: 2,
                headline: Some("Waiting on whether JSON output keeps the text report’s totals."),
                at: -12,
            },
            GoalFacts {
                number: 103,
                title: "Make tally sum stream its input",
                state: GoalState::Running,
                done: 1,
                total: 2,
                headline: Some("The streaming reader landed; the large-file check runs."),
                at: -12,
            },
            GoalFacts {
                number: 104,
                title: "CSV export",
                state: GoalState::Planning,
                done: 0,
                total: 0,
                headline: None,
                at: -12,
            },
            GoalFacts {
                number: 105,
                title: "Profile tally sum on a 2 GB file",
                state: GoalState::Paused,
                done: 0,
                total: 1,
                headline: None,
                at: -12,
            },
            GoalFacts {
                number: 106,
                title: "CSV import: the quoted-field parser",
                state: GoalState::Landed,
                done: 2,
                total: 2,
                headline: Some("Quoted fields and their checks landed."),
                at: -12,
            },
            GoalFacts {
                number: 107,
                title: "The process documents, read aloud",
                state: GoalState::Closed,
                done: 0,
                total: 1,
                headline: None,
                at: -12,
            },
            GoalFacts {
                number: 108,
                title: "CSV import: errors with line and column",
                state: GoalState::Failed,
                done: 0,
                total: 1,
                headline: None,
                at: -12,
            },
        ],
        Reference => &[
            GoalFacts {
                number: 101,
                title: "first-run-projects",
                state: GoalState::Needs { funds: true },
                done: 6,
                total: 9,
                headline: Some("Six of nine tasks landed; the store task is split in two after two size failures."),
                at: -2,
            },
            GoalFacts {
                number: 102,
                title: "welcome-stall",
                state: GoalState::Needs { funds: false },
                done: 1,
                total: 3,
                headline: Some("Reproduced in a box test; the fault is in jig’s store."),
                at: -36,
            },
            GoalFacts {
                number: 103,
                title: "forge-cards",
                state: GoalState::Needs { funds: false },
                done: 0,
                total: 4,
                headline: Some("Four tasks planned from the draft; two are running."),
                at: -50,
            },
            GoalFacts {
                number: 104,
                title: "A hundred tasks",
                state: GoalState::Running,
                done: 96,
                total: 112,
                headline: None,
                at: -12,
            },
            GoalFacts {
                number: 106,
                title: "pane-chat-line",
                state: GoalState::Landed,
                done: 2,
                total: 2,
                headline: Some("Landed whole: two tasks and one journey print."),
                at: -408,
            },
            GoalFacts {
                number: 107,
                title: "welcome-model-cards",
                state: GoalState::Landed,
                done: 3,
                total: 3,
                headline: Some("Landed Monday: the Model step is two cards."),
                at: -3680,
            },
        ],
        FirstRun { .. } | World::Bare | World::WholeStory => &[],
    }
}
fn goal_ended(f: &GoalFacts) -> bool {
    matches!(f.state, GoalState::Landed | GoalState::Closed | GoalState::Failed)
}
fn goal_in_sample(f: &GoalFacts, name: Option<&str>) -> bool {
    match name {
        Some("goals-none") => false,
        Some("goals-filter-empty") => goal_ended(f),
        _ => true,
    }
}
fn goal_line(w: World, f: &GoalFacts) -> c::TaskLine {
    let mut task = base_task(f.number, f.title);
    task.label = 2;
    task.priority = Some(u32::try_from(f.number).expect("small goal priority"));
    task.goal = Some(f.number);
    task.flags.conversation = false;
    task.flags.listed = true;
    task.chip.phase = if goal_ended(f) { c::Phase::Ended } else { c::Phase::Running };
    task.last_activity = offset(w, f.at);
    task.changed = task.last_activity;
    let mut line = task_line(finish_document(task));
    line.delegates_done = f.done;
    line.delegates_total = f.total;
    let (phase, details) = owner(&mut line.chip);
    match f.state {
        GoalState::Needs { funds: true } => {
            line.chip.phase = c::Phase::Held;
            line.chip.details.phase = c::ChipPhase::Held {
                reason: c::HoldCategory::Exceeded(c::Exceeded { bound: c::Bound::Answer, value: 1 }),
                revision: 1,
            };
        }
        GoalState::Paused => {
            *phase = c::ChipPhase::Held { reason: c::HoldCategory::Stopped, revision: 1 };
            details.hold = Some(c::ChipHold {
                placed: Some(c::PlacedHold {
                    number: 1,
                    owner: c::HoldOwner::Party { person: 73 },
                    mode: c::HoldMode::Stop,
                    scope: c::HoldScope::Subtree { task: f.number },
                    reason: Box::default(),
                    placed_at: offset(w, f.at),
                    revision: c::Revision { value: 1 },
                    lifted: false,
                    may_lift: true,
                }),
                words: None,
                circle: None,
                account: None,
                source: None,
                reason: c::HoldCategory::Stopped,
                revision: 1,
                before: c::PriorPhase::Active(c::RunStage::Idle),
                escalation: None,
                tries: c::FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0 },
                hold_wait_since: None,
            });
            line.chip.phase = c::Phase::Held;
        }
        GoalState::Landed => {
            details.ending = Some(c::ChipEnding::Done(c::ResultValue::Report(doc("Landed."))));
        }
        GoalState::Closed => {
            details.ending = Some(c::ChipEnding::Cancelled {
                reason: doc("Closed by you."),
                result: None,
                by: c::Requester::Person(73),
                at: offset(w, f.at),
            });
        }
        GoalState::Failed => details.ending = Some(c::ChipEnding::Failed(doc("Failed."))),
        GoalState::Planning => {
            *phase = c::ChipPhase::Active(c::RunStage::Idle);
            line.chip.phase = c::Phase::Idle;
        }
        GoalState::Needs { funds: false } | GoalState::Running => {}
    }
    line
}
fn goal_window(w: World, window: u32, name: Option<&str>) -> c::Tasks {
    let mut rows = List::with_capacity(window);
    let mut more = false;
    for f in goal_facts(w) {
        if !goal_ended(f) && goal_in_sample(f, name) {
            if rows.len() < window {
                rows.push(goal_line(w, f)).expect("goal window");
            } else {
                more = true;
            }
        }
    }
    c::Tasks { rows, more }
}
fn goal_page(w: World, name: Option<&str>) -> c::TaskPage {
    let mut rows = List::with_capacity(limits().client.domain.rows);
    let window = if name == Some("goals-more") { 1 } else { rows.capacity() };
    let mut more = false;
    for f in goal_facts(w) {
        if goal_ended(f) && goal_in_sample(f, name) {
            if rows.len() < window {
                rows.push(goal_line(w, f)).expect("ended goal window");
            } else {
                more = true;
            }
        }
    }
    c::TaskPage {
        live: List::with_capacity(0),
        live_after: None,
        ended: c::EndedGroup::Rows {
            rows,
            after: more.then(|| {
                c::Cursor::new(Box::from(b"goals-next".as_slice()), limits().client.domain.text).expect("goal cursor")
            }),
            cut: false,
        },
    }
}
fn asking_activity() -> c::ActivityPage {
    let mut blocks = List::with_capacity(3);
    blocks.push(c::Block::Text(text("Drafts first, then goals that land as one. I’ve written how Hello would run that, beside us; it grows as we talk."))).expect("reply");
    blocks
        .push(c::Block::Call(c::Call {
            number: 1,
            tool: text("note"),
            input: text(""),
            outcome: Some(text("Note 1 revision 2 written · +1 −1 · line 23")),
            failed: false,
            objects: List::with_capacity(0),
            parts: List::with_capacity(0),
        }))
        .expect("write");
    blocks.push(c::Block::Text(text("One question before I propose it: should a goal wait for you before it lands, or land once its checks pass?"))).expect("question");
    let mut line = turn_line(&blocks, 1, None, false);
    line.source.task = 10;
    if let Some(decoded) = &mut line.decoded {
        decoded.task = 10;
    }
    let mut rows = List::with_capacity(1);
    rows.push(line).expect("asking turn");
    c::ActivityPage { rows, before: None, after: None, cut: false }
}

fn asking_note(scope: &c::NoteScope) -> c::NotePage {
    let mut rows = List::with_capacity(1);
    rows.push(c::Note {
        number: 1,
        revision: c::Revision { value: 1 },
        scope: scope.clone(),
        description: doc("How Hello works"),
        body: c::DocumentText::complete(c::Text::new(Box::from("# How Hello works\n\nDraft, 2026-11-05, with the user.\n\n## How work starts\n\nYou write a design draft in a chat, an agent beside you. When it reads complete, you start a goal from it; nothing is planned before.\n\n## How it lands\n\n- Agents plan the goal’s tasks and change the code on the goal’s own branch.\n- Each change is checked: CI, and a review by an agent.\n- The goal lands on main as one, through temper’s merge queue.\n\n## Held by temper, or by agents\n\n| Rule | Held by |\n| --- | --- |\n| A review by an agent before a change lands | temper |\n| Up to date with main before it lands | temper |\n| A design draft before any goal | your agents |\n\n## Still open\n\nWhether a goal waits for you before it lands, or lands once its checks pass.\n".as_bytes()), limits().client.domain.long_text).expect("document body")),
        writer: c::NoteAuthor::Task { task: 10, attempt: 1 },
        tasks: List::with_capacity(0),
        changed: Some(now(Solo)),
        acts: List::with_capacity(0),
        cut: false,
    }).expect("asking document");
    c::NotePage { rows, before: None, after: None, cut: false }
}

fn goal_notes(w: World, scope: &c::NoteScope) -> c::NotePage {
    let mut rows = List::with_capacity(1);
    for f in goal_facts(w) {
        if *scope == (c::NoteScope::Task { project: PROJECT, task: f.number })
            && let Some(headline) = f.headline
        {
            rows.push(c::Note {
                number: 1,
                revision: c::Revision { value: 1 },
                scope: scope.clone(),
                description: doc(&format!("report: {headline}")),
                body: doc(""),
                writer: c::NoteAuthor::Task { task: f.number, attempt: 1 },
                tasks: List::with_capacity(0),
                changed: Some(offset(w, f.at)),
                acts: List::with_capacity(0),
                cut: false,
            })
            .expect("goal report");
        }
    }
    c::NotePage { rows, before: None, after: None, cut: false }
}
fn goal_inbox(
    w: World,
    group: c::InboxReadGroup,
    project: Option<u32>,
    goal: Option<u64>,
    name: Option<&str>,
) -> c::InboxPage {
    let room = u32::try_from(goal_facts(w).len()).expect("small goals");
    let mut rows = List::with_capacity(room);
    let mut selection = List::with_capacity(room);
    if w == Solo
        && (goal == Some(3) || (goal == Some(1) && name == Some("chat-terminal-question")))
        && group == c::InboxReadGroup::Needs
        && project.is_none_or(|project| project == PROJECT)
    {
        let object = c::InboxObject::Question { message: 1 };
        let at = Some(offset(w, -18));
        let cursor = c::InboxCursor::Inbox { task: 3, object, at };
        rows.push(c::InboxEntry {
            projects: List::with_capacity(0),
            reply: None,
            task: c::TaskLink::Live(document(w, 3).chip),
            project: Some(PROJECT),
            object,
            activity: Some(1),
            at,
            unread: Some(true),
            cursor,
        })
        .expect("solo goal question");
        selection
            .push(c::InboxSelection { cursor, goals: List::from_boxed(Box::new([3])) })
            .expect("solo goal selection");
    }
    for f in goal_facts(w) {
        if group == c::InboxReadGroup::Needs
            && project.is_none_or(|project| project == PROJECT)
            && goal.is_none_or(|goal| goal == f.number)
            && matches!(f.state, GoalState::Needs { .. })
            && goal_in_sample(f, name)
        {
            let object = if f.state == (GoalState::Needs { funds: true }) {
                c::InboxObject::Escalation { revision: 1 }
            } else {
                c::InboxObject::Question { message: 1 }
            };
            let cursor = c::InboxCursor::Inbox { task: f.number, object, at: Some(offset(w, f.at)) };
            rows.push(c::InboxEntry {
                projects: List::with_capacity(0),
                reply: None,
                task: c::TaskLink::Live(goal_line(w, f).chip),
                project: Some(PROJECT),
                object,
                activity: Some(1),
                at: Some(offset(w, f.at)),
                unread: Some(true),
                cursor,
            })
            .expect("goal need");
            selection
                .push(c::InboxSelection { cursor, goals: List::from_boxed(Box::new([f.number])) })
                .expect("goal selection");
        }
    }
    c::InboxPage { rows, selection: Some(selection), before: None, after: None, cut: false }
}

/// The goal page's mock records (temper/docs/design/client/web/gallery.md, 2).
struct GoalPage {
    facts: GoalFacts,
    brief: &'static str,
    body: &'static str,
    spent: u64,
    budget: u64,
    made: i64,
    whole: bool,
    delegates: &'static [(&'static str, c::Phase, i64)],
}
#[expect(clippy::too_many_lines, reason = "four mock goals keep their report words and delegate records together")]
fn page_goal(name: Option<&str>) -> Option<GoalPage> {
    let name = name?;
    let mut page = match name {
        "goal-running" | "goal-needs-funds" | "goal-paused" | "goal-no-report" => GoalPage {
            facts: GoalFacts {
                number: 108,
                title: "first-run-projects",
                state: GoalState::Running,
                done: 6,
                total: 9,
                headline: Some(
                    "Six of nine tasks landed. The store task is split in two after two size failures, and needs about 8 more to finish.",
                ),
                at: -2,
            },
            brief: "Build first-run-projects.\nDraft: docs/design/draft/first-run-projects.md\nChat: T1",
            body: "The project records, the project page’s empty state and the welcome’s Default project landed on the goal branch overnight, each checked. The Ready page waits on the store task.\n\nThe store task adds the project policy record to jig’s store. It failed twice on the wasm client’s size limit, so I split it: the record and its migration first, the client’s lazy loading second. Both run now, about 4 each.\n\nLeft: the two halves, the Ready page’s Open Default, and the welcome journey’s print.\nRisk: lazy loading touches every page that lists projects; its check runs the client’s journeys.\nDecided: Split the store task, 05:40 · Retried it once, 05:12 · Kept the Default project’s name in English, 01:20",
            spent: 31,
            budget: 40,
            made: -770,
            whole: true,
            delegates: &[
                ("Project records", c::Phase::Ended, -620),
                ("Project page, empty state", c::Phase::Ended, -465),
                ("Welcome’s Default project", c::Phase::Ended, -358),
                ("Playground project", c::Phase::Ended, -270),
                ("Banner’s set-up link", c::Phase::Ended, -230),
                ("Pane’s playground line", c::Phase::Ended, -179),
                ("Store: policy record", c::Phase::Running, -22),
                ("Client: lazy project loading", c::Phase::Running, -12),
                ("Ready page’s Open Default", c::Phase::Waiting, -12),
            ],
        },
        "goal-needs-question" | "goal-closed" => GoalPage {
            facts: GoalFacts {
                number: 118,
                title: "welcome-stall",
                state: GoalState::Needs { funds: false },
                done: 1,
                total: 3,
                headline: Some("Reproduced in a box test; the fault is in jig’s store. Waiting on where to fix it."),
                at: -36,
            },
            brief: "Fix the welcome stall.\nChat: T2",
            body: "The box test walks the welcome to Ready and presses Open Default. MakeProject’s write lands in the store, then the client’s watch closes before the answer arrives, so the page waits forever.\n\nTwo fixes work in a spike: jig’s store answers before it closes the watch, which changes a stored record; or temper serve keeps the watch open until the answer. Both pass the box test.\n\nLeft: the fix, wherever it goes, and the box test landing with it.\nRisk: none known beyond the choice itself.\nDecided: Wrote the box test before the fix, 19:05",
            spent: 4,
            budget: 15,
            made: -820,
            whole: false,
            delegates: &[
                ("Box test: Open Default", c::Phase::Ended, -560),
                ("The fix", c::Phase::Waiting, -40),
                ("Welcome-set-up journey", c::Phase::Waiting, -40),
            ],
        },
        "goal-planning" | "goal-failed" => GoalPage {
            facts: GoalFacts {
                number: 122,
                title: "forge-cards",
                state: GoalState::Planning,
                done: 0,
                total: 0,
                headline: None,
                at: -50,
            },
            brief: "Build forge-cards.\nDraft: docs/design/draft/forge-cards.md\nChat: T1",
            body: "",
            spent: 3,
            budget: 20,
            made: -235,
            whole: true,
            delegates: &[],
        },
        "goal-landed" | "goal-landed-open" => GoalPage {
            facts: GoalFacts {
                number: 123,
                title: "pane-chat-line",
                state: GoalState::Landed,
                done: 2,
                total: 2,
                headline: Some("Landed whole at 01:12: two tasks and one journey print."),
                at: -408,
            },
            brief: "Make the pane’s chat one compact line.\nChat: T3",
            body: "A chat in the side pane is one compact line: its title and when, then its status and T number, the whole line its link.\n\nLeft: nothing.\nRisk: none.",
            spent: 5,
            budget: 10,
            made: -570,
            whole: true,
            delegates: &[("Pane chat line", c::Phase::Ended, -430), ("Journey print", c::Phase::Ended, -408)],
        },
        _ => return None,
    };
    match name {
        "goal-needs-funds" => page.facts.state = GoalState::Needs { funds: true },
        "goal-paused" => page.facts.state = GoalState::Paused,
        "goal-no-report" => page.facts.headline = None,
        "goal-closed" => {
            page.facts.state = GoalState::Closed;
            page.facts.headline = None;
        }
        "goal-failed" => page.facts.state = GoalState::Failed,
        _ => {}
    }
    Some(page)
}
fn goal_doc(words: &str) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(words.as_bytes().into(), 4096).expect("mock report and question words"))
}
// The goal mock’s morning is Tuesday, two days before the Reference world’s.
fn goal_wall(w: World, minutes: i64) -> Wall {
    offset(w, minutes - 2880)
}
impl GoalPage {
    fn document(&self, w: World) -> c::Task {
        let mut task = base_task(self.facts.number, self.facts.title);
        task.chip = goal_line(w, &self.facts).chip;
        task.label = 2;
        task.priority = Some(1);
        task.acts = offers(&[
            c::ActKind::Message,
            c::ActKind::Amend,
            if self.facts.state == GoalState::Paused { c::ActKind::Release } else { c::ActKind::Stop },
            c::ActKind::Cancel,
        ]);
        task.goal = Some(self.facts.number);
        task.requester = match self.facts.number {
            118 => c::Requester::Task(2),
            123 => c::Requester::Task(3),
            _ => c::Requester::Person(73),
        };
        task.flags.conversation = false;
        task.flags.listed = true;
        task.spec.specification.words = doc(self.brief);
        task.made = goal_wall(w, self.made);
        task.changed = goal_wall(w, self.facts.at);
        task.last_activity = task.changed;
        task.budget.numbers.spent = self.spent;
        task.budget.numbers.budget = self.budget;
        task.budget.left = self.budget.saturating_sub(self.spent);
        task.chip.details.details.budget = task.budget.clone();
        task.hold = task.chip.details.details.hold.clone().map(Box::new);
        if self.facts.state == (GoalState::Needs { funds: true }) {
            let mut prototype = shown_task(Solo, 3, Then::Held);
            let mut hold = prototype.hold.take().expect("held task");
            hold.reason = c::HoldCategory::Budget;
            hold.words = Some(doc(
                "Both halves of the store task run, about 4 each. Without funds the goal stops after they land, with the Ready page undone.",
            ));
            task.chip.details.phase = c::ChipPhase::Held {
                reason: c::HoldCategory::Exceeded(c::Exceeded { bound: c::Bound::Answer, value: 1 }),
                revision: hold.revision,
            };
            task.chip.details.details.hold = Some(*hold.clone());
            task.hold = Some(hold);
        }
        if goal_ended(&self.facts) {
            let ending = match self.facts.state {
                GoalState::Landed => c::ChipEnding::Done(c::ResultValue::Report(doc(
                    "Landed whole at 01:12: two tasks and one journey print.",
                ))),
                GoalState::Closed => c::ChipEnding::Cancelled {
                    reason: doc("Closed by you."),
                    result: None,
                    by: c::Requester::Person(73),
                    at: task.changed,
                },
                GoalState::Failed => c::ChipEnding::Failed(doc("The owned set up could not create work/hello.")),
                _ => unreachable!("ended goal state"),
            };
            task.result = Some(c::TaskResult::Ending(Box::new(ending.clone())));
            task.chip.details.details.ending = Some(ending);
            task.chip.details.details.ended_at = Some(task.changed);
            task.acts.clear();
        }
        if self.whole {
            let number = self.facts.number.to_string();
            task.resources = List::with_capacity(1);
            task.resources
                .push(c::ResourceLine {
                    part: c::Part {
                        connector: FORGE,
                        kind: c::PartKind::Resource,
                        key: c::PartKey::Path(path(&[b"temper", number.as_bytes(), format!("g{number}").as_bytes()])),
                    },
                    kind: 1,
                    taken: true,
                    holder: Some(self.facts.number),
                    writer: None,
                    writer_lost: false,
                })
                .expect("goal branch");
        }
        task
    }
    fn delegate(&self, w: World, index: usize) -> c::TaskLine {
        let (title, phase, at) = self.delegates[index];
        let number = self.facts.number + u64::try_from(index).expect("small delegate index") + 1;
        let mut task = base_task(number, title);
        task.flags.conversation = false;
        task.requester = c::Requester::Task(self.facts.number);
        task.chip.phase = phase;
        task.last_activity = goal_wall(w, at);
        if phase == c::Phase::Ended {
            task.result = Some(c::TaskResult::Ending(Box::new(c::ChipEnding::Done(c::ResultValue::Change {
                kind: 1,
                part: c::Part { connector: FORGE, kind: c::PartKind::Result, key: c::PartKey::Number(number) },
                words: doc("Landed."),
            }))));
        }
        let mut line = task_line(finish_document(task));
        let dependencies: &[u64] = match (self.facts.number, index) {
            (108, 8) => &[115, 116],
            (118, 2) => &[120],
            _ => &[],
        };
        if !dependencies.is_empty() {
            line.chip.details.phase = c::ChipPhase::Waiting {
                dependencies: List::from_boxed(Box::from(dependencies)),
                more: false,
                holds: false,
            };
        }
        line
    }
    fn tree(&self, w: World, window: u32) -> c::Delivery {
        let mut rows = List::with_capacity(window);
        for index in 0..self.delegates.len() {
            let line = self.delegate(w, index);
            if line.chip.phase != c::Phase::Ended {
                rows.push(c::TreeRow {
                    chip: line.chip,
                    parent: self.facts.number,
                    depth: 1,
                    proposals: List::with_capacity(0),
                    spent: 0,
                    dependencies: List::with_capacity(0),
                    cut: false,
                })
                .expect("goal live delegates fit");
            }
        }
        c::Delivery::Tree(Box::new(c::Tree {
            root: self.document(w).chip,
            rows,
            folded: List::with_capacity(0),
            held_below: 0,
            next: None,
        }))
    }
    #[expect(clippy::too_many_lines, reason = "one dispatch answers the goal’s related read documents")]
    fn read(&self, w: World, what: &c::Read) -> Option<c::ReadValue> {
        match what {
            c::Read::Tasks { query, .. }
                if *query == temper_client_domain::pages::goal::own_line_query(&self.document(w)) =>
            {
                let mut line = task_line(self.document(w));
                line.delegates_done = self.facts.done;
                line.delegates_total = self.facts.total;
                let rows = List::from_boxed(Box::new([line]));
                let page = if goal_ended(&self.facts) {
                    c::TaskPage {
                        live: List::with_capacity(0),
                        live_after: None,
                        ended: c::EndedGroup::Rows { rows, after: None, cut: false },
                    }
                } else {
                    c::TaskPage { live: rows, live_after: None, ended: c::EndedGroup::None }
                };
                Some(c::ReadValue::Tasks(Box::new(page)))
            }
            c::Read::Tasks { query, .. } if query.parent == Some(self.facts.number) => {
                let mut rows = List::with_capacity(self.facts.total);
                for index in 0..self.delegates.len() {
                    let line = self.delegate(w, index);
                    if line.chip.phase == c::Phase::Ended {
                        rows.push(line).expect("ended delegates");
                    }
                }
                Some(c::ReadValue::Tasks(Box::new(c::TaskPage {
                    live: List::with_capacity(0),
                    live_after: None,
                    ended: c::EndedGroup::Rows { rows, after: None, cut: false },
                })))
            }
            c::Read::Notes { scope, .. }
                if *scope == (c::NoteScope::Task { project: PROJECT, task: self.facts.number }) =>
            {
                let mut rows = List::with_capacity(1);
                if let Some(headline) = self.facts.headline {
                    rows.push(c::Note {
                        number: 1,
                        revision: c::Revision { value: 1 },
                        scope: scope.clone(),
                        description: doc(&format!("report: {headline}")),
                        body: goal_doc(self.body),
                        writer: c::NoteAuthor::Task { task: self.facts.number, attempt: 1 },
                        tasks: List::with_capacity(0),
                        changed: Some(goal_wall(w, self.facts.at)),
                        acts: List::with_capacity(0),
                        cut: false,
                    })
                    .expect("goal report");
                }
                Some(c::ReadValue::Notes(Box::new(c::NotePage { rows, before: None, after: None, cut: false })))
            }
            c::Read::Inbox { group, goal: Some(goal), .. } if *goal == self.facts.number => {
                let mut rows = List::with_capacity(1);
                let mut selection = List::with_capacity(1);
                if *group == c::InboxReadGroup::Needs && matches!(self.facts.state, GoalState::Needs { .. }) {
                    let (task, object) = if self.facts.state == (GoalState::Needs { funds: true }) {
                        (self.facts.number, c::InboxObject::Escalation { revision: 7 })
                    } else {
                        (120, c::InboxObject::Question { message: 1 })
                    };
                    let cursor = c::InboxCursor::Inbox { task, object, at: Some(goal_wall(w, -40)) };
                    let chip = if task == self.facts.number { self.document(w).chip } else { self.delegate(w, 1).chip };
                    rows.push(c::InboxEntry {
                        projects: List::with_capacity(0),
                        reply: None,
                        task: c::TaskLink::Live(chip),
                        project: Some(PROJECT),
                        object,
                        activity: Some(1),
                        at: Some(goal_wall(w, -40)),
                        unread: Some(true),
                        cursor,
                    })
                    .expect("goal need");
                    selection
                        .push(c::InboxSelection { cursor, goals: List::from_boxed(Box::new([self.facts.number])) })
                        .expect("goal selection");
                }
                Some(c::ReadValue::Inbox(Box::new(c::InboxPage {
                    rows,
                    selection: Some(selection),
                    before: None,
                    after: None,
                    cut: false,
                })))
            }
            c::Read::Escalation { task: 108, .. } if self.facts.state == (GoalState::Needs { funds: true }) => {
                Some(c::ReadValue::Escalation(Box::new(c::Escalation {
                    task: Box::new(self.document(w)),
                    acts: offers(&[c::ActKind::Release, c::ActKind::LeaveHeld]),
                })))
            }
            c::Read::Question { task: 120, .. } if self.facts.state == (GoalState::Needs { funds: false }) => {
                Some(c::ReadValue::Question(Box::new(c::Question {
                    task: self.delegate(w, 1).chip,
                    from: Some(c::Requester::Task(self.facts.number)),
                    message: 1,
                    words: goal_doc(
                        "Fix the dropped watch in jig’s store, or keep it alive in temper serve?\n\nMakeProject answers after the client’s watch drops. jig’s fix changes a stored record; serve’s fix is local but leaves jig’s behaviour as it is.\n\nSaw: The box test reproduces it every time: the store’s write lands, then the watch closes before the answer comes. The test and its log are on the goal.\nChoice: In jig’s store (recommended)\nChoice: In temper serve",
                    ),
                    since: goal_wall(w, -40),
                    acts: offers(&[c::ActKind::Answer]),
                    ending: None,
                })))
            }
            _ => None,
        }
    }
}

struct TaskFacts {
    title: &'static str,
    brief: &'static str,
    charter: u32,
    parent: Option<u64>,
    phase: c::Phase,
    wait: u8,
    spent_tenths: u64,
    budget: u64,
    priority: Option<u32>,
    change: bool,
    result: Option<&'static str>,
    merge: bool,
    model: &'static str,
    effort: Option<&'static str>,
    account: u32,
    children: &'static [u64],
    proposal: bool,
    activity_at: u64,
}
fn chip(w: World, number: u64) -> c::Chip {
    let f = facts(w, number);
    let mut t = base_task(number, f.title);
    t.chip.phase = f.phase;
    t.chip.executor = if f.change {
        c::ChipExecutor::Procedure { connector: FORGE, kind: 1 }
    } else {
        c::ChipExecutor::Agent { charter: f.charter }
    };
    t.budget = budget(&f);
    finish_document(t).chip
}
fn budget(f: &TaskFacts) -> c::Budget {
    let spent = round_mockup_spend(f.spent_tenths);
    c::Budget {
        numbers: c::Numbers { budget: f.budget, spent, spent_below: 0, reserved: 0 },
        left: f.budget.saturating_sub(spent),
        funder: match f.parent {
            Some(parent) => c::Funder::Task(parent),
            None => c::Funder::Pool { project: PROJECT, person: 73, period: 1 },
        },
        deadline: None,
    }
}
fn document(w: World, number: u64) -> c::Task {
    let f = facts(w, number);
    let mut t = base_task(number, f.title);
    t.chip = chip(w, number);
    t.priority = f.priority;
    if f.priority.is_some() {
        t.goal = Some(number);
        t.flags.conversation = false;
        t.flags.listed = true;
        // A goal offers amend, stop and cancel (client README 3.5, the Task row).
        t.acts = offers(&[c::ActKind::Amend, c::ActKind::Stop, c::ActKind::Cancel]);
    }
    t.kept_account = if has_playground(w) || f.change { None } else { Some(f.account) };
    t.last_activity = Wall::from_nanos(f.activity_at);
    t.changed = t.last_activity;
    t.requester = match f.parent {
        Some(parent) => c::Requester::Task(parent),
        None => c::Requester::Person(73),
    };
    t.spec.specification.words = doc(f.brief);
    // Mockup amounts have decimals; the view prints integer deployment units.
    // Round to the nearest unit, and keep the missing below-ledger at zero.
    t.budget = budget(&f);
    t.llm = if f.change || has_playground(w) {
        None
    } else {
        Some(c::LlmChoice {
            later: None,
            account: Some(f.account),
            main: c::ModelEffort { model: text(f.model), effort: f.effort.map(text) },
            subagents: List::with_capacity(0),
        })
    };
    t.waits_on = match f.wait {
        1 => c::WaitsOn::Delegates,
        2 => c::WaitsOn::News,
        3 => c::WaitsOn::Party {
            whom: c::Whom::Person(73),
            entries: 1,
            task: number,
            kind: if number == 10 { c::WaitedFor::Proposal } else { c::WaitedFor::Question },
        },
        _ => c::WaitsOn::Nothing,
    };
    if f.change {
        t.result_contract = c::TaskContractDocument::Change { connector: FORGE, kind: 1, words: 4096 };
    }
    if let Some(result) = f.result {
        let value = if f.merge {
            c::ResultValue::Change {
                kind: 1,
                part: c::Part { connector: FORGE, kind: c::PartKind::Result, key: c::PartKey::Number(number) },
                words: doc(result),
            }
        } else {
            c::ResultValue::Report(doc(result))
        };
        t.result = Some(c::TaskResult::Ending(Box::new(c::ChipEnding::Done(value))));
    }
    if f.phase == c::Phase::Done || f.phase == c::Phase::Ended {
        t.acts.clear();
    }
    t.delegates = List::with_capacity(16);
    for child in f.children {
        let child_facts = facts(w, *child);
        if child_facts.phase != c::Phase::Done && child_facts.phase != c::Phase::Ended {
            t.delegates.push(c::LiveDelegate { chip: chip(w, *child), takeover_budget: 0 }).expect("live delegate");
        }
    }
    if f.proposal {
        t.proposals = List::with_capacity(1);
        t.proposals.push(PROPOSAL).expect("proposal");
    }
    t.activity = activity(w, number).rows.len().into();
    finish_document(t)
}
// Plausible gallery figures, not the engine's rule: a person's "as allowed"
// task gets min(pool left, its role's ceiling) less the same batch's named
// amounts; a delegate gets its parent's remainder after its spend, the settled
// spend below, reservations and named siblings.
#[expect(clippy::too_many_lines, reason = "literal task facts copied from the three mockup worlds")]
fn facts(w: World, number: u64) -> TaskFacts {
    match (w, number) {
        (Reference, 1) => TaskFacts {
            title: "Director",
            brief: "Morning. I'm landing Export's draft within the hour.",
            charter: 1,
            parent: None,
            phase: c::Phase::Idle,
            wait: 1,
            spent_tenths: 62,
            budget: 100,
            priority: None,
            change: false,
            result: None,
            merge: false,
            model: "Opus",
            effort: Some("xhigh"),
            account: 1,
            children: &[12, 7, 8],
            proposal: false,
            activity_at: 1_792_653_000_000_000_000,
        },
        (Reference, 7) => TaskFacts {
            title: "Owners' status digest",
            brief: "Every two hours, read each owner's ledger and write the director a digest.",
            charter: 7,
            parent: Some(1),
            phase: c::Phase::Idle,
            wait: 2,
            spent_tenths: 9,
            budget: 94,
            priority: None,
            change: false,
            result: None,
            merge: false,
            model: "GPT",
            effort: Some("medium"),
            account: 2,
            children: &[],
            proposal: false,
            activity_at: 1_792_524_000_000_000_000,
        },
        (Reference, 8) => TaskFacts {
            title: "Completion checker",
            brief: "When an owner reports done, check its draft's items against what landed.",
            charter: 7,
            parent: Some(1),
            phase: c::Phase::Idle,
            wait: 2,
            spent_tenths: 4,
            budget: 94,
            priority: None,
            change: false,
            result: None,
            merge: false,
            model: "GPT",
            effort: Some("medium"),
            account: 2,
            children: &[],
            proposal: false,
            activity_at: 1_792_524_000_000_000_000,
        },
        (Reference, 12) => TaskFacts {
            title: "Harness",
            brief: "Own the draft Harness: kit's fake server, the flaky tests and the build path. Take taste from one-shot Opus advisors; bring significant decisions to the director.",
            charter: 2,
            parent: Some(1),
            phase: c::Phase::Idle,
            wait: 1,
            spent_tenths: 186,
            budget: 60,
            priority: Some(1),
            change: false,
            result: None,
            merge: false,
            model: "GPT",
            effort: Some("high"),
            account: 2,
            children: &[13, 14, 24, 26, 28, 31, 33, 34, 35],
            proposal: false,
            activity_at: 1_792_598_700_000_000_000,
        },
        (Reference, 13) => TaskFacts {
            title: "Advise: the fake server's API",
            brief: "Read the draft's fake server section and the three candidate shapes; recommend one, with its reasons.",
            charter: 3,
            parent: Some(12),
            phase: c::Phase::Done,
            wait: 0,
            spent_tenths: 13,
            budget: 41,
            priority: None,
            change: false,
            result: Some("Shape 2: a scripted server with typed replies; streams as a reply kind, not a mode."),
            merge: false,
            model: "Opus",
            effort: Some("xhigh"),
            account: 1,
            children: &[],
            proposal: false,
            activity_at: 1_792_614_000_000_000_000,
        },
        (Reference, 14) => TaskFacts {
            title: "Survey: kit's flaky tests",
            brief: "Run kit's suite 40 times and list every test that fails without a change, with its signature.",
            charter: 4,
            parent: Some(12),
            phase: c::Phase::Done,
            wait: 0,
            spent_tenths: 8,
            budget: 41,
            priority: None,
            change: false,
            result: Some("One flake: reconnect_after_drop, 3 in 40, timed out waiting for reconnect."),
            merge: false,
            model: "GPT",
            effort: Some("medium"),
            account: 2,
            children: &[],
            proposal: false,
            activity_at: 1_792_648_320_000_000_000,
        },
        (Reference, 24) => TaskFacts {
            title: "Fake server: replay fixtures",
            brief: "",
            charter: 1,
            parent: Some(12),
            phase: c::Phase::Done,
            wait: 0,
            spent_tenths: 0,
            budget: 41,
            priority: None,
            change: true,
            result: Some("4e1a9c7"),
            merge: true,
            model: "Opus",
            effort: Some("high"),
            account: 1,
            children: &[25],
            proposal: false,
            activity_at: 1_792_651_680_000_000_000,
        },
        (Reference, 25) => TaskFacts {
            title: "Produce T24's branch",
            brief: "Add replay fixtures to the fake server: record a session and replay it byte for byte, with tests.",
            charter: 5,
            parent: Some(24),
            phase: c::Phase::Done,
            wait: 0,
            spent_tenths: 34,
            budget: 41,
            priority: None,
            change: false,
            result: Some("Pushed temper/24/fake-server-replay-fixtures at 8b2d410."),
            merge: false,
            model: "GPT",
            effort: Some("high"),
            account: 2,
            children: &[],
            proposal: false,
            activity_at: 1_792_647_000_000_000_000,
        },
        (Reference, 26) => TaskFacts {
            title: "Fake server: typed errors",
            brief: "",
            charter: 1,
            parent: Some(12),
            phase: c::Phase::Running,
            wait: 0,
            spent_tenths: 0,
            budget: 41,
            priority: None,
            change: true,
            result: None,
            merge: false,
            model: "Opus",
            effort: Some("high"),
            account: 1,
            children: &[27],
            proposal: false,
            activity_at: 1_792_638_900_000_000_000,
        },
        (Reference, 27) => TaskFacts {
            title: "Produce T26's branch",
            brief: "Give the fake server typed errors in place of strings, with tests for each.",
            charter: 5,
            parent: Some(26),
            phase: c::Phase::Done,
            wait: 0,
            spent_tenths: 29,
            budget: 41,
            priority: None,
            change: false,
            result: Some("Pushed temper/26/fake-server-typed-errors at 51ce0a7."),
            merge: false,
            model: "GPT",
            effort: Some("high"),
            account: 2,
            children: &[],
            proposal: false,
            activity_at: 1_792_648_500_000_000_000,
        },
        (Reference, 28) => TaskFacts {
            title: "Build path: one cache per lane",
            brief: "",
            charter: 1,
            parent: Some(12),
            phase: c::Phase::Running,
            wait: 0,
            spent_tenths: 0,
            budget: 41,
            priority: None,
            change: true,
            result: None,
            merge: false,
            model: "Opus",
            effort: Some("high"),
            account: 1,
            children: &[29],
            proposal: false,
            activity_at: 1_792_639_200_000_000_000,
        },
        (Reference, 29) => TaskFacts {
            title: "Produce T28's branch",
            brief: "One build cache per lane in scripts/build, so lanes stop evicting each other.",
            charter: 5,
            parent: Some(28),
            phase: c::Phase::Done,
            wait: 0,
            spent_tenths: 22,
            budget: 41,
            priority: None,
            change: false,
            result: Some("Pushed temper/28/build-path-one-cache-per-lane at c40e9b2."),
            merge: false,
            model: "GPT",
            effort: Some("medium"),
            account: 2,
            children: &[],
            proposal: false,
            activity_at: 1_792_650_300_000_000_000,
        },
        (Reference, 31) => TaskFacts {
            title: "Fix the reconnect race",
            brief: "",
            charter: 1,
            parent: Some(12),
            phase: c::Phase::Running,
            wait: 0,
            spent_tenths: 0,
            budget: 41,
            priority: None,
            change: true,
            result: None,
            merge: false,
            model: "Opus",
            effort: Some("high"),
            account: 1,
            children: &[32],
            proposal: false,
            activity_at: 1_792_648_320_000_000_000,
        },
        (Reference, 32) => TaskFacts {
            title: "Produce T31's branch",
            brief: "Fix reconnect_after_drop's race (finding F): the reconnect timer must not fire before the drop is seen. Prove it with 200 runs.",
            charter: 5,
            parent: Some(31),
            phase: c::Phase::Done,
            wait: 0,
            spent_tenths: 11,
            budget: 41,
            priority: None,
            change: false,
            result: Some("Pushed temper/31/fix-the-reconnect-race at e7a31d5."),
            merge: false,
            model: "GPT",
            effort: Some("medium"),
            account: 2,
            children: &[],
            proposal: false,
            activity_at: 1_792_654_800_000_000_000,
        },
        (Reference, 33) => TaskFacts {
            title: "Benchmark: the fake server under load",
            brief: "Measure replies per second and tail latency at 1, 8 and 64 clients. Needs s1 quiet.",
            charter: 5,
            parent: Some(12),
            phase: c::Phase::Running,
            wait: 0,
            spent_tenths: 16,
            budget: 41,
            priority: None,
            change: false,
            result: None,
            merge: false,
            model: "GPT",
            effort: Some("medium"),
            account: 2,
            children: &[],
            proposal: false,
            activity_at: 1_792_652_640_000_000_000,
        },
        (Reference, 34) => TaskFacts {
            title: "Fake server: streaming replies",
            brief: "Add streamed replies to the fake server as a reply kind (T13), with its tests. Hand back the branch.",
            charter: 5,
            parent: Some(12),
            phase: c::Phase::Running,
            wait: 0,
            spent_tenths: 24,
            budget: 41,
            priority: None,
            change: false,
            result: None,
            merge: false,
            model: "GPT",
            effort: Some("high"),
            account: 2,
            children: &[],
            proposal: false,
            activity_at: 1_792_655_940_000_000_000,
        },
        (Reference, 35) => TaskFacts {
            title: "Investigate: the slow build path",
            brief: "Find why kit's build path takes 9 minutes from clean; name the cause with evidence.",
            charter: 6,
            parent: Some(12),
            phase: c::Phase::Running,
            wait: 0,
            spent_tenths: 9,
            budget: 41,
            priority: None,
            change: false,
            result: None,
            merge: false,
            model: "GPT",
            effort: Some("medium"),
            account: 2,
            children: &[],
            proposal: false,
            activity_at: 1_792_654_440_000_000_000,
        },
        (Reference, 3) => TaskFacts {
            title: "Why did kit’s slow tier fail on Monday?",
            brief: "",
            charter: 1,
            parent: None,
            phase: c::Phase::Ended,
            wait: 0,
            spent_tenths: 10,
            budget: 100,
            priority: None,
            change: false,
            result: None,
            merge: false,
            model: "Opus",
            effort: Some("high"),
            account: 1,
            children: &[],
            proposal: false,
            activity_at: 1_792_422_000_000_000_000,
        },
        (Reference, 2) => TaskFacts {
            title: "The process documents, read aloud",
            brief: "",
            charter: 1,
            parent: None,
            phase: c::Phase::Ended,
            wait: 0,
            spent_tenths: 10,
            budget: 100,
            priority: None,
            change: false,
            result: None,
            merge: false,
            model: "Opus",
            effort: Some("high"),
            account: 1,
            children: &[],
            proposal: false,
            activity_at: 1_792_332_000_000_000_000,
        },
        (Solo, 3) => TaskFacts {
            title: "CSV import",
            brief: "Import CSV with quoted fields (RFC 4180), keeping the old parser’s behaviour where tests pin it.",
            charter: 2,
            parent: None,
            phase: c::Phase::Idle,
            wait: 3,
            spent_tenths: 48,
            budget: 20,
            priority: Some(1),
            change: false,
            result: None,
            merge: false,
            model: "Opus",
            effort: Some("high"),
            account: 1,
            children: &[4, 8],
            proposal: false,
            activity_at: 1_793_003_400_000_000_000,
        },
        (Solo, 4) => TaskFacts {
            title: "CSV import: the quoted-field parser",
            brief: "",
            charter: 1,
            parent: Some(3),
            phase: c::Phase::Running,
            wait: 0,
            spent_tenths: 0,
            budget: 15,
            priority: None,
            change: true,
            result: None,
            merge: false,
            model: "Opus",
            effort: Some("high"),
            account: 1,
            children: &[5],
            proposal: false,
            activity_at: 1_793_004_600_000_000_000,
        },
        (Solo, 5) => TaskFacts {
            title: "Produce T4's branch",
            brief: "Replace the CSV splitter with a parser for quoted fields, escaped quotes and embedded newlines (docs/csv.md), with tests for each.",
            charter: 2,
            parent: Some(4),
            phase: c::Phase::Done,
            wait: 0,
            spent_tenths: 21,
            budget: 15,
            priority: None,
            change: false,
            result: Some("Pushed temper/3/csv-import-the-quoted-field-parser at 3c9e1f0."),
            merge: false,
            model: "Sonnet",
            effort: Some("medium"),
            account: 1,
            children: &[],
            proposal: false,
            activity_at: 1_793_091_000_000_000_000,
        },
        (Solo, 6) => TaskFacts {
            title: "Why is tally sum slow on big files?",
            brief: "Why is tally sum slow on big files?\n2 GB takes 40 s.",
            charter: 1,
            parent: None,
            phase: c::Phase::Idle,
            wait: 2,
            spent_tenths: 8,
            budget: 40,
            priority: None,
            change: false,
            result: None,
            merge: false,
            model: "Opus",
            effort: Some("high"),
            account: 1,
            children: &[7],
            proposal: false,
            activity_at: 1_793_088_120_000_000_000,
        },
        (Solo, 10) => TaskFacts {
            title: "Add a --json flag to tally report",
            brief: "Add a --json flag to tally report\nOne object per row: my dashboard script reads it.",
            charter: 1,
            parent: None,
            phase: c::Phase::Idle,
            wait: 3,
            spent_tenths: 3,
            budget: 40,
            priority: None,
            change: false,
            result: None,
            merge: false,
            model: "Opus",
            effort: Some("high"),
            account: 1,
            children: &[],
            proposal: true,
            activity_at: 1_793_092_680_000_000_000,
        },
        (Solo, 7) => TaskFacts {
            title: "Profile tally sum on a 2 GB file",
            brief: "Profile `tally sum` on a generated 2 GB CSV and name where the time goes.",
            charter: 2,
            parent: Some(6),
            phase: c::Phase::Done,
            wait: 0,
            spent_tenths: 5,
            budget: 39,
            priority: None,
            change: false,
            result: Some(
                "92% of the time is the CSV splitter allocating a string per line. A reused buffer cuts the run from 40 s to 1.8 s.",
            ),
            merge: false,
            model: "Sonnet",
            effort: Some("medium"),
            account: 1,
            children: &[],
            proposal: false,
            activity_at: 1_793_088_000_000_000_000,
        },
        (Solo, 8) => TaskFacts {
            title: "CSV import: errors with line and column",
            brief: "",
            charter: 1,
            parent: Some(3),
            phase: c::Phase::Running,
            wait: 0,
            spent_tenths: 0,
            budget: 15,
            priority: None,
            change: true,
            result: None,
            merge: false,
            model: "Opus",
            effort: Some("high"),
            account: 1,
            children: &[9],
            proposal: false,
            activity_at: 1_793_092_020_000_000_000,
        },
        (Solo, 9) => TaskFacts {
            title: "Produce T8's branch",
            brief: "Give CSV import’s errors the line and column where they occur, counting embedded newlines, with tests.",
            charter: 2,
            parent: Some(8),
            phase: c::Phase::Running,
            wait: 0,
            spent_tenths: 6,
            budget: 15,
            priority: None,
            change: false,
            result: None,
            merge: false,
            model: "Sonnet",
            effort: Some("medium"),
            account: 1,
            children: &[],
            proposal: false,
            activity_at: 1_793_093_340_000_000_000,
        },
        (Solo, 1) => TaskFacts {
            title: "Make tally sum stream its input",
            brief: "",
            charter: 1,
            parent: None,
            phase: c::Phase::Ended,
            wait: 0,
            spent_tenths: 10,
            budget: 40,
            priority: None,
            change: false,
            result: None,
            merge: false,
            model: "Opus",
            effort: Some("high"),
            account: 1,
            children: &[],
            proposal: false,
            activity_at: 1_792_998_000_000_000_000,
        },
        (FirstRun { step: 2 }, 1) => TaskFacts {
            title: "Add CSV export to widget",
            brief: "Add CSV export to widget",
            charter: 1,
            parent: None,
            phase: c::Phase::Idle,
            wait: 1,
            spent_tenths: 0,
            budget: 20,
            priority: None,
            change: false,
            result: None,
            merge: false,
            model: "Scripted model",
            effort: None,
            account: 1,
            children: &[2],
            proposal: false,
            activity_at: 1_792_702_860_000_000_000,
        },
        (FirstRun { step: 2 }, 2) => TaskFacts {
            title: "Export widget's table as CSV",
            brief: "",
            charter: 1,
            parent: Some(1),
            phase: c::Phase::Running,
            wait: 0,
            spent_tenths: 0,
            budget: 20,
            priority: None,
            change: true,
            result: None,
            merge: false,
            model: "Scripted model",
            effort: None,
            account: 1,
            children: &[3],
            proposal: false,
            activity_at: 1_792_702_860_000_000_000,
        },
        (FirstRun { step: 2 }, 3) => TaskFacts {
            title: "Export widget's table as CSV",
            brief: "Export widget's table as CSV",
            charter: 2,
            parent: Some(2),
            phase: c::Phase::Running,
            wait: 0,
            spent_tenths: 0,
            budget: 20,
            priority: None,
            change: false,
            result: None,
            merge: false,
            model: "Scripted model",
            effort: None,
            account: 1,
            children: &[],
            proposal: false,
            activity_at: 1_792_702_860_000_000_000,
        },
        (FirstRun { step: 3..=7 }, 1) => TaskFacts {
            title: "Add CSV export to widget",
            brief: "Add CSV export to widget",
            charter: 1,
            parent: None,
            phase: c::Phase::Idle,
            wait: 2,
            spent_tenths: 0,
            budget: 20,
            priority: None,
            change: false,
            result: None,
            merge: false,
            model: "Scripted model",
            effort: None,
            account: 1,
            children: &[2],
            proposal: false,
            activity_at: 1_792_703_040_000_000_000,
        },
        (FirstRun { step: 3..=7 }, 2) => TaskFacts {
            title: "Export widget's table as CSV",
            brief: "",
            charter: 1,
            parent: Some(1),
            phase: c::Phase::Done,
            wait: 0,
            spent_tenths: 0,
            budget: 20,
            priority: None,
            change: true,
            result: Some("e3b8d4192c0f5a7e"),
            merge: true,
            model: "Scripted model",
            effort: None,
            account: 1,
            children: &[3],
            proposal: false,
            activity_at: 1_792_703_040_000_000_000,
        },
        (FirstRun { step: 3..=7 }, 3) => TaskFacts {
            title: "Export widget's table as CSV",
            brief: "Export widget's table as CSV",
            charter: 2,
            parent: Some(2),
            phase: c::Phase::Done,
            wait: 0,
            spent_tenths: 0,
            budget: 20,
            priority: None,
            change: false,
            result: Some(
                "**Add CSV export to widget**\n\n`widget --csv` prints the table as CSV: a header line, `name,count`, then a line for each row.",
            ),
            merge: false,
            model: "Scripted model",
            effort: None,
            account: 1,
            children: &[],
            proposal: false,
            activity_at: 1_792_702_980_000_000_000,
        },
        (FirstRun { step: 4..=7 }, 4) => TaskFacts {
            title: "Add a dark theme to widget",
            brief: "Add a dark theme to widget",
            charter: 1,
            parent: None,
            phase: c::Phase::Idle,
            wait: 2,
            spent_tenths: 0,
            budget: 20,
            priority: None,
            change: false,
            result: None,
            merge: false,
            model: "Scripted model",
            effort: None,
            account: 1,
            children: &[],
            proposal: false,
            activity_at: 1_792_703_100_000_000_000,
        },
        _ => panic!("task absent from this world step"),
    }
}
fn activity(w: World, task: u64) -> c::ActivityPage {
    let mut events = List::with_capacity(16);
    match (w, task) {
        (Reference, 1) => {
            events
                .push(agent_line("Noted. I'll keep s1's three spare slots free until then.", 2, Some(1)))
                .expect("world activity");
            events
                .push(tools_line(
                    &[("recall", 2), ("read_pull", 3), ("read_items", 1), ("workers", 1), ("accounts", 2)],
                    &[],
                    false,
                    3,
                ))
                .expect("world activity");
            events.push(agent_line("**Check · two-hour timer**\n\n| | |\n|---|---|\n| Harness | T12 on s1 · 3 delegates running (T33 benchmark, T34 streaming, T35 build path) · 3 changes in kit’s queue |\n| Landed | kit: T24 replay fixtures at 06:48, slow tier passed · app: nothing since yesterday |\n| Findings | F, kit’s reconnect flake · B’s · fix f1 (T31) ready, 3rd in kit’s queue |\n| Workers | ws idle · s1 4 of 7 slots, disk 142 GB free (floor 80) |\n| Plans | Claude week 58%, above its pace of 48% · ChatGPT week 22%, under its 28% |\n| Spend | since 05:10: Harness 3.1 · Director 0.4 |\n| Needs you | nothing |\n\nNext check at 09:10.", 4, Some(1))).expect("world activity");
        }
        (Solo, 6) => {
            events
                .push(agent_line("I'll profile it on a generated 2 GB file first.", 2, Some(1)))
                .expect("world activity");
            events.push(tools_line(&[("delegate", 1)], &[7], false, 3)).expect("world activity");
            events.push(agent_line("T7 found it: **92%** of the time is the CSV splitter allocating a string per line.\n\nA reused buffer cuts the run from 40 s to 1.8 s. Shall I make that a change?", 4, Some(1))).expect("world activity");
        }
        (Solo, 9) => {
            events.push(tools_line(&[("read", 4), ("shell", 1)], &[], false, 1)).expect("world activity");
            events.push(agent_line("Errors carry their line now, and an embedded newline moves it on. The column next: counted from the start of its line, quotes included.", 2, None)).expect("world activity");
            events.push(tools_line(&[("edit", 3), ("shell", 2)], &[], false, 3)).expect("world activity");
            // Its last mockup turn is live; static gallery states show committed turns only (client/web/gallery.md, 1).
        }
        (Solo, 10) => {
            events.push(agent_line("That changes `report`'s output and its tests, and lands into main, so I've proposed it as a goal of yours, with a budget of its own.", 2, Some(1))).expect("world activity");
            events.push(tools_line(&[("read", 2), ("propose", 1)], &[], true, 3)).expect("world activity");
        }
        (FirstRun { step: 2 }, 1) => {
            events.push(agent_line("I’ll delegate this change in playground/widget. It will push a branch, open a pull request, check CI and land into main on Playground forge.", 2, Some(1))).expect("world activity");
            events.push(tools_line(&[("delegate", 1)], &[2], false, 3)).expect("world activity");
            events.push(tools_line(&[("wait", 1)], &[], false, 4)).expect("world activity");
        }
        (FirstRun { step: 3..=7 }, 1) => {
            events.push(agent_line("I’ll delegate this change in playground/widget. It will push a branch, open a pull request, check CI and land into main on Playground forge.", 2, Some(1))).expect("world activity");
            events.push(tools_line(&[("delegate", 1)], &[2], false, 3)).expect("world activity");
            events.push(tools_line(&[("wait", 1)], &[], false, 4)).expect("world activity");
            events.push(agent_line("CSV export is available from widget’s table. The change landed on Playground forge. This was a scripted change.", 5, Some(1))).expect("world activity");
            events.push(tools_line(&[("wait", 1)], &[], false, 6)).expect("world activity");
        }
        (FirstRun { step: 4..=7 }, 4) => {
            events.push(agent_line("I would give widget a dark theme that follows the system's preference, with its colours as variables in one place so both themes stay in step. I am the scripted model and play this from a script, so nothing is changed: connect a model from Accounts to have the work done.", 2, Some(1))).expect("world activity");
        }
        _ => {}
    }
    let minutes: &[i64] = match (w, task) {
        (Reference, 1) => &[-64, -50, -50],
        (Solo, 6) => &[-118, -117, -88],
        (Solo, 10) => &[-12, -12],
        (Solo, 9) => &[-21, -15, -14],
        (FirstRun { step: 2 }, 1) => &[1, 1, 1],
        (FirstRun { step: 3..=7 }, 1) => &[1, 1, 1, 4, 4],
        (FirstRun { step: 4..=7 }, 4) => &[5],
        _ => &[],
    };
    let mut rows = List::with_capacity(events.len());
    for i in (0..events.len()).rev() {
        rows.push({
            let mut line = events.get(i).expect("event").clone();
            line.source.position = u64::from(i).checked_add(1).expect("position");
            line.source.at =
                offset(w, *minutes.get(usize::try_from(i).expect("event index")).expect("transcript time"));
            line.source.task = task;
            if let Some(decoded) = &mut line.decoded {
                decoded.task = task;
                decoded.cumulative = line.source.position;
                decoded.at = line.source.at;
            }
            if let c::Activity::Turn { reply, .. } = &mut line.source.line {
                *reply = line.source.position;
            }
            line
        })
        .expect("activity row");
    }
    c::ActivityPage { rows, after: None, before: None, cut: false }
}
#[expect(clippy::too_many_lines, reason = "literal account and source facts from the mockup worlds")]
pub(super) fn accounts(w: World, then: Then) -> c::Accounts {
    let mut rows = List::with_capacity(2);
    match w {
        Reference => {
            let mut sources = List::with_capacity(2);
            sources
                .push(c::AccountSource {
                    name: Some(Box::from("claude-pat".as_bytes())),
                    place: 1,
                    number: 1,
                    kind: c::AccountKind::Program,
                    state: c::AccountState::Granted,
                    usable: true,
                    expiring: false,
                    attention: false,
                    spent_until: None,
                })
                .expect("source");
            sources
                .push(c::AccountSource {
                    name: Some(Box::from("claude-pat-2".as_bytes())),
                    place: 2,
                    number: 2,
                    kind: c::AccountKind::Program,
                    state: c::AccountState::Granted,
                    usable: true,
                    expiring: false,
                    attention: false,
                    spent_until: None,
                })
                .expect("source");
            rows.push(c::AccountLine {
                list: None,
                facts: List::with_capacity(0),
                number: 1,
                place: 1,
                uses: None,
                plan: false,
                provider: Some(17),
                provider_name: Some(doc("anthropic")),
                manage_usage: None,
                display: doc("Claude"),
                holder: c::AccountHolder::Party(73),
                sources,
                cut: false,
            })
            .expect("account");
            let mut sources = List::with_capacity(2);
            sources
                .push(c::AccountSource {
                    name: Some(Box::from("pat@example.com".as_bytes())),
                    place: 1,
                    number: 3,
                    kind: c::AccountKind::SignIn,
                    state: c::AccountState::Granted,
                    usable: true,
                    expiring: false,
                    attention: false,
                    spent_until: None,
                })
                .expect("source");
            rows.push(c::AccountLine {
                list: None,
                facts: List::with_capacity(0),
                number: 2,
                place: 2,
                uses: None,
                plan: true,
                provider: Some(18),
                provider_name: Some(doc("openai")),
                manage_usage: None,
                display: doc(if matches!(then, Then::SignInReady | Then::Declined | Then::Renewed) {
                    "pat@example.com"
                } else {
                    "ChatGPT"
                }),
                holder: c::AccountHolder::Party(73),
                sources,
                cut: false,
            })
            .expect("account");
        }
        Solo => {
            let mut sources = List::with_capacity(2);
            sources
                .push(c::AccountSource {
                    name: Some(Box::from("claude-robin".as_bytes())),
                    place: 1,
                    number: 1,
                    kind: c::AccountKind::Program,
                    state: c::AccountState::Granted,
                    usable: true,
                    expiring: false,
                    attention: false,
                    spent_until: None,
                })
                .expect("source");
            sources
                .push(c::AccountSource {
                    name: Some(Box::from("robin-key".as_bytes())),
                    place: 2,
                    number: 2,
                    kind: c::AccountKind::Key,
                    state: c::AccountState::Granted,
                    usable: true,
                    expiring: false,
                    attention: false,
                    spent_until: None,
                })
                .expect("source");
            rows.push(c::AccountLine {
                list: None,
                facts: List::with_capacity(0),
                number: 1,
                place: 1,
                uses: None,
                plan: false,
                provider: Some(17),
                provider_name: Some(doc("anthropic")),
                manage_usage: None,
                display: doc("Claude"),
                holder: c::AccountHolder::Party(73),
                sources,
                cut: false,
            })
            .expect("account");
        }
        FirstRun { .. } | World::Bare | World::WholeStory => {
            let mut sources = List::with_capacity(2);
            sources
                .push(c::AccountSource {
                    name: Some(Box::from(b"playground".as_slice())),
                    place: 1,
                    number: 1,
                    kind: if matches!(w, FirstRun { step: 5..=9 }) {
                        c::AccountKind::SignIn
                    } else {
                        c::AccountKind::Key
                    },
                    state: c::AccountState::Granted,
                    usable: true,
                    expiring: false,
                    attention: false,
                    spent_until: None,
                })
                .expect("source");
            rows.push(c::AccountLine {
                list: None,
                facts: if matches!(w, FirstRun { step: 5..=9 }) {
                    List::from_boxed(Box::new([c::AccountFact {
                        kind: c::FACT_EMAIL,
                        label: Box::new([]),
                        value: c::AccountFactValue::Text(Box::from(b"sam@example.com".as_slice())),
                        seen: now(w),
                        source: Some(1),
                        origin: 1,
                    }]))
                } else {
                    List::with_capacity(0)
                },
                number: 1,
                place: 1,
                uses: None,
                plan: matches!(w, FirstRun { step: 5..=9 }),
                provider: Some(if matches!(w, FirstRun { step: 5..=9 }) { 18 } else { 17 }),
                provider_name: Some(doc(if matches!(w, FirstRun { step: 5..=9 }) { "openai" } else { "Playground" })),
                manage_usage: if matches!(w, FirstRun { step: 5..=9 }) {
                    Some(Box::from(b"https://chatgpt.com/settings/usage".as_slice()))
                } else {
                    None
                },
                display: doc(if matches!(w, FirstRun { step: 5..=9 }) { "" } else { "Scripted model" }),
                holder: c::AccountHolder::Party(73),
                sources,
                cut: false,
            })
            .expect("account");
        }
    }
    if w == World::Bare || w == (FirstRun { step: 0 }) {
        rows.clear();
    }
    let mut hosts = List::with_capacity(2);
    if w == Reference || w == Solo {
        let names: &[(&str, &str)] = if w == Reference {
            &[("ws", "claude-pat"), ("s1", "claude-pat-2")]
        } else {
            &[("laptop", "claude-robin")]
        };
        for (i, (name, login)) in names.iter().enumerate() {
            let mut programs = List::with_capacity(1);
            programs
                .push(c::ProgramLogin {
                    name: Box::from(login.as_bytes()),
                    state: Some(c::ProgramState::Usable),
                    sign_out: None,
                    cleanup: false,
                    cut: false,
                })
                .expect("login");
            hosts
                .push(c::ProgramHost {
                    name: Box::from(name.as_bytes()),
                    channel: u64::try_from(i).expect("host").checked_add(1).expect("host"),
                    connected: true,
                    awaiting_logins: false,
                    programs,
                    cut: false,
                })
                .expect("host");
        }
    }
    let mut readiness = List::with_capacity(3);
    for provider in &providers(w) {
        readiness
            .push(c::PayerReadiness {
                provider: provider.provider,
                payer: c::AccountUse::Party(73),
                readiness: if w == World::Bare
                    || w == (FirstRun { step: 0 })
                    || matches!(w, FirstRun { step: 1..=4 }) && provider.provider == 18
                {
                    c::Readiness::Unavailable
                } else {
                    c::Readiness::Usable
                },
                settled: Some(true),
            })
            .expect("provider readiness");
    }
    c::Accounts {
        program_hosts: hosts,
        accounts: rows,
        signing: List::with_capacity(0),
        lendable: false,
        lending: None,
        lent_end: None,
        last_end: match then {
            Then::SignInReady | Then::Declined => Some(c::AccountLastEnd {
                source: 43,
                end: c::AccountEnd::Made { account: 2, declined: then == Then::Declined },
            }),
            Then::Renewed => {
                Some(c::AccountLastEnd { source: 43, end: c::AccountEnd::Renewed { account: 2, declined: false } })
            }
            Then::SignInRefused => Some(c::AccountLastEnd {
                source: 43,
                end: c::AccountEnd::Refused { refusal: c::AccountRefusal::Held { provider: 18 } },
            }),
            Then::Models { .. }
            | Then::Hosts { .. }
            | Then::Shown
            | Then::DraftEmpty
            | Then::Mode(_)
            | Then::Working
            | Then::DelegatedGoal
            | Then::HeldChange
            | Then::Paused
            | Then::HeldTask
            | Then::SignedOut
            | Then::GoogleSignIn
            | Then::OutOfDate
            | Then::Closing
            | Then::Sheet(_)
            | Then::Rejecting
            | Then::Held
            | Then::TaskPaused
            | Then::Releasing
            | Then::SigningIn
            | Then::Paste(_)
            | Then::AccountRefused => None,
        },
        usable: w != World::Bare && w != (FirstRun { step: 0 }),
        settled: true,
        cut: false,
        readiness,
        readiness_cut: false,
    }
}

/// The mockup's tenths rounded to the contract's whole deployment unit.
fn round_mockup_spend(tenths: u64) -> u64 {
    (tenths + 5) / 10
}
fn text(s: &str) -> c::Text {
    c::Text::new(s.as_bytes().into(), limits().client.domain.text).expect("mockup text fits shipped text limit")
}
fn path(segments: &[&[u8]]) -> List<Box<[u8]>> {
    let mut result = List::with_capacity(u32::try_from(segments.len()).expect("short path"));
    for segment in segments {
        result.push(Box::from(*segment)).expect("path capacity");
    }
    result
}
fn offers(kinds: &[c::ActKind]) -> List<c::Offered> {
    let mut result = List::with_capacity(u32::try_from(kinds.len()).expect("few acts"));
    for act in kinds {
        let effect = match act {
            c::ActKind::Cancel => Some(c::Effect::Cancel(c::Impact {
                tasks: c::ImpactCounts {
                    waiting: 0,
                    running: 0,
                    held: 0,
                    closing: 0,
                    ended: 0,
                    done: 0,
                    failed: 0,
                    cancelled: 0,
                },
                live_runs: 0,
                proposals: 0,
            })),
            c::ActKind::Talk
            | c::ActKind::Leave
            | c::ActKind::Message
            | c::ActKind::Answer
            | c::ActKind::Take
            | c::ActKind::HandBack
            | c::ActKind::Accept
            | c::ActKind::Reject
            | c::ActKind::PassUp
            | c::ActKind::Release
            | c::ActKind::LeaveHeld
            | c::ActKind::Amend
            | c::ActKind::Stop
            | c::ActKind::Move
            | c::ActKind::Reorder
            | c::ActKind::Seen
            | c::ActKind::EditNote
            | c::ActKind::DeleteNote
            | c::ActKind::Adopt
            | c::ActKind::ChangePolicy
            | c::ActKind::SetRoles
            | c::ActKind::MakeService
            | c::ActKind::SetPool
            | c::ActKind::RemoveProject
            | c::ActKind::Rename
            | c::ActKind::Withdraw => None,
        };
        result.push(c::Offered { act: *act, admission: c::Admission::Allowed, effect }).expect("act capacity");
    }
    result
}
fn base_frame(person: &str, project: &str) -> c::Me {
    let mut templates = List::with_capacity(1);
    templates
        .push(c::Template {
            resources: List::with_capacity(0),
            number: 1,
            name: doc("chat"),
            executor: c::TemplateExecutor::Charter(1),
            contract: c::TaskContractDocument::Report { words: limits().client.domain.text },
            shallower: 1,
            budget: c::TemplateBudget::AsAllowed { cap: None },
            deadline: c::TemplateDeadline::None,
            tracked: None,
            flags: CHAT_FLAGS,
            label: CHAT_LABEL,
            grace: Some(Duration::from_secs(30)),
            llm: None,
        })
        .expect("one template");
    let mut projects = List::with_capacity(1);
    projects
        .push(c::MeProject {
            left_out: 0,
            defaults: List::with_capacity(0),
            public: None,
            project: PROJECT,
            name: doc(project),
            role: c::HeldRole::Owner,
            pool: None,
            templates,
            creating: None,
            models: List::with_capacity(0),
        })
        .expect("one project");
    c::Me {
        party: c::MeParty { number: 73, kind: c::PartyKind::Person, login: doc(person), name: doc(person) },
        projects,
        needs: c::Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        needs_total: 0,
        updates: c::Count { value: 0, limited: false },
        results_seen: 0,
        cut: false,
        retention: Duration::from_secs(1000),
        deployment_role: Some(c::HeldRole::Owner),
        providers: List::with_capacity(0),
        providers_cut: false,
        key_bytes: 4096,
        redirect_bytes: 4096,
        words_bytes: 8192,
    }
}
#[expect(clippy::too_many_lines, reason = "the gallery's base task spells out every specification field")]
fn base_task(number: u64, title: &str) -> c::Task {
    let wake = c::WakePolicy {
        words: c::WakeRule::Immediate,
        notices: c::WakeRule::Never,
        news: c::WakeRule::Never,
        results: c::ResultsWake::Each,
        questions: true,
        answers: true,
        timers: false,
    };
    let budget = c::Budget {
        numbers: c::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 },
        left: 0,
        funder: c::Funder::Pool { project: PROJECT, person: 73, period: 1 },
        deadline: None,
    };
    c::Task {
        talk: None,
        session: None,
        settings: Some(c::TaskSettings { task: 1, run: None, newest: 1 }),
        chip: c::Chip {
            task: number,
            title: doc(title),
            executor: c::ChipExecutor::Agent { charter: 1 },
            phase: c::Phase::Idle,
            details: Box::new(c::ChipPhaseFacts {
                phase: c::ChipPhase::Active(c::RunStage::Idle),
                details: Box::new(c::ChipDetails {
                    dependencies: List::with_capacity(0),
                    project: PROJECT,
                    executor: c::Executor::Agent { charter: 1 },
                    phase_since: Wall::EPOCH,
                    run_since: None,
                    attempt: None,
                    retry: None,
                    wake,
                    person_wait: None,
                    hold: None,
                    budget: budget.clone(),
                    activity: 0,
                    resources: List::with_capacity(0),
                    resources_cut: false,
                    resources_observed: false,
                    procedure: None,
                    live_delegates: 0,
                    closing_connectors: None,
                    ending: None,
                    ended_at: None,
                    observation: None,
                }),
            }),
        },
        label: CHAT_LABEL,
        flags: CHAT_FLAGS,
        last_activity: Wall::EPOCH,
        made: Wall::EPOCH,
        waits_on: c::WaitsOn::Nothing,
        goal: None,
        priority: None,
        lineage_chips: List::with_capacity(0),
        inputs: List::with_capacity(0),
        project: PROJECT,
        requester: c::Requester::Person(73),
        lineage: List::with_capacity(0),
        spec: c::SpecificationDocument::complete(c::Specification {
            asks: false,
            words: text(""),
            parameters: List::with_capacity(0),
            inputs: List::with_capacity(0),
        }),
        spec_revision: c::Revision { value: 1 },
        person_revision: None,
        taken_by: None,
        instructions: None,
        wake,
        subscriptions: c::Subscriptions { entries: List::with_capacity(0) },
        result_contract: c::TaskContractDocument::Report { words: limits().client.domain.text },
        dependencies: List::with_capacity(0),
        dependency_facts: List::with_capacity(0),
        references: List::with_capacity(0),
        cut: false,
        resources: List::with_capacity(0),
        budget,
        authority: c::AuthorityDocument::complete(c::Authority {
            models: None,
            tools: 0,
            grants: List::with_capacity(0),
            delegation: c::Delegation { kinds: List::with_capacity(0), tasks: 0, depth: 0 },
            spend: 0,
            deadline: None,
            notes: 0,
            note_resources: List::with_capacity(0),
        }),
        hold: None,
        proposals: List::with_capacity(0),
        activity: 0,
        changed: Wall::EPOCH,
        acts: offers(&[c::ActKind::Message, c::ActKind::Stop, c::ActKind::Cancel]),
        result: None,
        delegates: List::with_capacity(0),
        context: None,
        llm: None,
        kept_account: None,
        run_wait: None,
    }
}
fn task_window(w: World, window: u32) -> c::Tasks {
    let all = chat_rows(w, false);
    let mut rows = List::with_capacity(window);
    for row in &all {
        if rows.len() < window {
            rows.push(row.clone()).expect("window");
        }
    }
    c::Tasks { more: all.len() > rows.len(), rows }
}
fn ended_rows(w: World) -> List<c::TaskLine> {
    let all = chat_rows(w, true);
    let mut rows = List::with_capacity(limits().client.domain.rows);
    for row in &all {
        if rows.len() < rows.capacity() {
            rows.push(row.clone()).expect("ended window");
        }
    }
    rows
}
fn reply_line(w: World, task: u64) -> c::TaskLine {
    let mut line = task_line(document(w, task));
    line.conversation = Some(c::ConversationRead { seen: 0, unread: c::Count { value: 1, limited: false } });
    line
}
fn inbox_window(w: World, window: u32) -> c::Inbox {
    // The engine allocates needs and updates independently, each at the requested window.
    let source = inbox_document(w);
    let mut needs = List::with_capacity(window);
    let mut updates = List::with_capacity(window);
    let mut more = None;
    for row in source.needs.as_slice().iter().chain(source.updates.as_slice()) {
        let task = if matches!(row.object, c::InboxObject::Proposal { .. }) { 10 } else { row.task_number() };
        let object = row.object;
        let cursor = c::InboxCursor::Inbox { task, object, at: row.at };
        let reply = if row.group() == c::InboxGroup::Replies { Some(Box::new(reply_line(w, task))) } else { None };
        let entry = c::InboxEntry {
            projects: List::with_capacity(0),
            reply,
            task: c::TaskLink::Live(chip(w, task)),
            project: Some(PROJECT),
            object,
            activity: row.activity,
            at: row.at,
            unread: Some(true),
            cursor,
        };
        let group = match row.group() {
            c::InboxGroup::Proposals
            | c::InboxGroup::Questions
            | c::InboxGroup::Escalations
            | c::InboxGroup::PersonTasks => &mut needs,
            c::InboxGroup::Replies | c::InboxGroup::Results => &mut updates,
        };
        if group.len() < window {
            group.push(entry).expect("inbox window");
        } else {
            more = Some(cursor);
        }
    }
    c::Inbox {
        party: 73,
        needs,
        updates,
        next: more,
        cached_needs: source.cached_needs,
        cached_updates: source.cached_updates,
        results_seen: source.results_seen,
        updates_time_complete: true,
        selection: None,
    }
}
// A page of `rows` lines, newest first, from the cursor's row offset; the
// cursor carries the next page's offset, so a chat filling its window ends.
fn activity_window(w: World, task: u64, before: Option<&c::Cursor>) -> c::ActivityPage {
    let mut document = activity(w, task);
    let all = document.rows;
    let from = match before {
        Some(cursor) => usize::from(*cursor.value.bytes().first().expect("gallery cursor")),
        None => 0,
    };
    document.rows = List::with_capacity(limits().client.domain.rows);
    for row in all.iter().skip(from) {
        if document.rows.len() < document.rows.capacity() {
            document.rows.push(row.clone()).expect("activity window");
        }
    }
    let next = from.checked_add(document.rows.as_slice().len()).expect("gallery offset");
    if all.as_slice().len() > next {
        let offset = u8::try_from(next).expect("gallery activity fits a byte");
        document.before =
            Some(c::Cursor::new(Box::from([offset].as_slice()), limits().client.domain.text).expect("cursor"));
    }
    document
}
#[expect(clippy::too_many_lines, reason = "the task fixture holds each confirmation's run, delegate and effect facts")]
fn shown_task(w: World, task: u64, then: Then) -> c::Task {
    let mut t = document(w, task);
    if matches!(then, Then::Mode(_) | Then::DraftEmpty) && task == 1 {
        t.chip.title = doc("Away mode, draft");
        t.chip.phase = c::Phase::Idle;
        let (phase, details) = owner(&mut t.chip);
        *phase = c::ChipPhase::Active(c::RunStage::Idle);
        details.ending = None;
        details.ended_at = None;
        t.result = None;
        t.acts = offers(&[c::ActKind::Message, c::ActKind::Stop, c::ActKind::Cancel]);
        t.waits_on = c::WaitsOn::Party { whom: c::Whom::Person(73), entries: 1, task: 1, kind: c::WaitedFor::Question };
        t.activity = if then == Then::DraftEmpty { 0 } else { 2 };
        t.spec = c::SpecificationDocument::complete(c::Specification {
            asks: false,
            words: text("Let’s draft away mode: a setting, limits I set, and decided for you with an overturn."),
            parameters: List::with_capacity(0),
            inputs: List::with_capacity(0),
        });
    }
    if matches!(then, Then::Mode(_) | Then::DraftEmpty) && task == 10 {
        t.chip.title = doc("Ask temper");
        t.activity = 1;
        t.spec = c::SpecificationDocument::complete(c::Specification {
            asks: false,
            words: text("I write a design draft first; agents plan its tasks and land them together once checked."),
            parameters: List::with_capacity(0),
            inputs: List::with_capacity(0),
        });
    }
    if then == Then::Paused {
        let hold = c::ChipHold {
            placed: Some(c::PlacedHold {
                number: 1,
                owner: c::HoldOwner::Party { person: frame(w, false, false).party.number },
                mode: c::HoldMode::Stop,
                scope: c::HoldScope::Subtree { task },
                reason: Box::new([]),
                placed_at: now(w),
                revision: c::Revision { value: 2 },
                lifted: false,
                may_lift: true,
            }),
            words: None,
            circle: None,
            account: None,
            source: None,
            reason: c::HoldCategory::Stopped,
            revision: 7,
            before: c::PriorPhase::Active(c::RunStage::Idle),
            escalation: None,
            tries: c::FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0 },
            hold_wait_since: None,
        };
        t.hold = Some(Box::new(hold.clone()));
        t.chip.phase = c::Phase::Held;
        let (phase, details) = owner(&mut t.chip);
        *phase = c::ChipPhase::Held { reason: hold.reason, revision: hold.revision };
        details.hold = Some(hold);
        t.acts = offers(&[c::ActKind::Release, c::ActKind::Stop, c::ActKind::Message, c::ActKind::Cancel]);
    }
    if then == Then::Working {
        let (phase, _) = owner(&mut t.chip);
        *phase = c::ChipPhase::Active(c::RunStage::Due);
        t.waits_on = c::WaitsOn::Nothing;
    }
    if then == Then::DelegatedGoal && task == 2 {
        t.priority = Some(1);
        t.chip.executor = c::ChipExecutor::Agent { charter: 2 };
        t.chip.title = doc("Keep widget's CSV export reliable");
        t.flags.conversation = false;
        t.flags.listed = true;
    }
    if then == Then::Held
        || then == Then::Releasing
        || then == Then::Sheet(Confirmation::ReleaseUncertain)
        || (then == Then::HeldChange && task == 2)
    {
        t.priority = None;
        t.flags.conversation = false;
        t.flags.listed = true;
        let hold = c::ChipHold {
            placed: None,
            words: None,
            circle: None,
            account: None,
            source: None,
            reason: c::HoldCategory::Failures { class: c::HoldFailure::Run },
            revision: 7,
            before: c::PriorPhase::Active(c::RunStage::Idle),
            escalation: None,
            tries: c::FailureTries { transient: 0, permanent: 0, run: 2, agent: 0, lost: 0, invalid: 0 },
            hold_wait_since: None,
        };
        t.hold = Some(Box::new(hold.clone()));
        t.chip.phase = c::Phase::Held;
        let (phase, details) = owner(&mut t.chip);
        *phase = c::ChipPhase::Held { reason: hold.reason, revision: hold.revision };
        details.hold = Some(hold);
        t.acts = List::with_capacity(1);
        t.acts
            .push(c::Offered {
                act: c::ActKind::Release,
                admission: c::Admission::Allowed,
                effect: Some(c::Effect::Release(c::ReleaseEffect::RunAgain)),
            })
            .expect("release");
    }
    if then == Then::TaskPaused {
        t.priority = None;
        t.flags.conversation = false;
        t.flags.listed = true;
        t.chip.executor = c::ChipExecutor::Agent { charter: 1 };
        let hold = c::ChipHold {
            placed: Some(c::PlacedHold {
                number: 1,
                owner: c::HoldOwner::Party { person: 73 },
                mode: c::HoldMode::Stop,
                scope: c::HoldScope::Subtree { task },
                reason: Box::default(),
                placed_at: now(w),
                revision: c::Revision { value: 1 },
                lifted: false,
                may_lift: true,
            }),
            words: None,
            circle: None,
            account: None,
            source: None,
            reason: c::HoldCategory::Stopped,
            revision: 1,
            before: c::PriorPhase::Active(c::RunStage::Idle),
            escalation: None,
            tries: c::FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0 },
            hold_wait_since: None,
        };
        t.hold = Some(Box::new(hold.clone()));
        t.chip.phase = c::Phase::Held;
        let (phase, details) = owner(&mut t.chip);
        *phase = c::ChipPhase::Held { reason: hold.reason, revision: hold.revision };
        details.hold = Some(hold);
        t.acts = offers(&[c::ActKind::Amend, c::ActKind::Release, c::ActKind::Cancel]);
    }
    if let Then::Sheet(kind) = then {
        match kind {
            Confirmation::CloseWork | Confirmation::CloseFailed | Confirmation::Closed if task == 6 => {
                t.chip.phase = c::Phase::Running;
                t.chip.details.phase = c::ChipPhase::Active(c::RunStage::Running { attempt: 1 });
                t.delegates = List::with_capacity(2);
                for (number, title) in [(4, "Fix the parser"), (5, "Review the parser")] {
                    let mut chip = t.chip.clone();
                    chip.task = number;
                    chip.title = doc(title);
                    t.delegates.push(c::LiveDelegate { chip, takeover_budget: 0 }).expect("two delegates");
                }
                t.acts = offers(&[c::ActKind::Cancel]);
                t.acts.get_mut(0).expect("cancel").effect = Some(c::Effect::Cancel(c::Impact {
                    tasks: c::ImpactCounts {
                        waiting: 2,
                        running: 0,
                        held: 0,
                        closing: 0,
                        ended: 0,
                        done: 7,
                        failed: 3,
                        cancelled: 1,
                    },
                    live_runs: 1,
                    proposals: 2,
                }));
                if kind == Confirmation::Closed {
                    let at = Wall::from_nanos(
                        now(w).as_nanos() / 86_400_000_000_000 * 86_400_000_000_000 + 75_900_000_000_000,
                    );
                    t.chip.phase = c::Phase::Cancelled;
                    t.chip.details.phase = c::ChipPhase::Ended;
                    t.chip.details.details.ended_at = Some(at);
                    t.result = Some(c::TaskResult::Ending(Box::new(c::ChipEnding::Cancelled {
                        reason: doc("Chat closed"),
                        by: t.requester,
                        result: None,
                        at,
                    })));
                }
            }
            Confirmation::CloseTask if task == 3 => {
                t.priority = None;
                t.flags.conversation = false;
                t.flags.listed = true;
                t.chip.executor = c::ChipExecutor::Agent { charter: 1 };
                t.chip.phase = c::Phase::Running;
                t.chip.details.phase = c::ChipPhase::Active(c::RunStage::Running { attempt: 1 });
                t.acts = offers(&[c::ActKind::Stop, c::ActKind::Cancel]);
                t.acts.get_mut(1).expect("cancel").effect = Some(c::Effect::Cancel(c::Impact {
                    tasks: c::ImpactCounts {
                        waiting: 1,
                        running: 0,
                        held: 0,
                        closing: 0,
                        ended: 0,
                        done: 0,
                        failed: 0,
                        cancelled: 0,
                    },
                    live_runs: 1,
                    proposals: 0,
                }));
            }
            Confirmation::CloseBelow | Confirmation::CloseProposal if task == 6 => {
                t.chip.phase = c::Phase::Idle;
                t.chip.details.phase = c::ChipPhase::Active(c::RunStage::Idle);
                t.delegates.clear();
                t.acts = offers(&[c::ActKind::Cancel]);
                t.acts.get_mut(0).expect("cancel").effect = Some(c::Effect::Cancel(c::Impact {
                    tasks: c::ImpactCounts {
                        waiting: u32::from(kind == Confirmation::CloseBelow),
                        running: 0,
                        held: 0,
                        closing: 0,
                        ended: 0,
                        done: 0,
                        failed: 0,
                        cancelled: 0,
                    },
                    live_runs: 0,
                    proposals: u32::from(kind == Confirmation::CloseProposal),
                }));
            }
            Confirmation::ReleaseUncertain if task == 3 => {
                t.acts.get_mut(0).expect("release").effect = Some(c::Effect::Release(c::ReleaseEffect::Uncertain));
            }
            Confirmation::CloseWork
            | Confirmation::CloseFailed
            | Confirmation::CloseBelow
            | Confirmation::CloseProposal
            | Confirmation::Closed
            | Confirmation::StartOver
            | Confirmation::CloseTask
            | Confirmation::Accept
            | Confirmation::Reject
            | Confirmation::ReleaseUncertain => {}
        }
    }

    t
}
fn plan_tree(w: World, task: u64, window: u32, then: Then) -> c::Delivery {
    let root = shown_task(w, task, then).chip;
    let mut descendants = Vec::new();
    let mut pending = vec![(task, 0_u32)];
    while let Some((parent, depth)) = pending.pop() {
        for number in facts(w, parent).children {
            let f = facts(w, *number);
            // The owner tree contains live tasks; T5's retained result is
            // not a plan row (client/pages/goal.md, 2).
            if f.phase != c::Phase::Done && f.phase != c::Phase::Ended {
                descendants.push((*number, parent, depth + 1));
                pending.push((*number, depth + 1));
            }
        }
    }
    // The engine's live task view supplies number order, not mockup order.
    descendants.sort_unstable_by_key(|(number, _, _)| *number);
    let mut rows = List::with_capacity(window);
    let mut phases = c::PhaseCounts::ZERO;
    for (number, parent, depth) in descendants {
        if rows.len() < window {
            rows.push(c::TreeRow {
                chip: chip(w, number),
                parent,
                depth,
                proposals: List::with_capacity(0),
                spent: round_mockup_spend(facts(w, number).spent_tenths),
                dependencies: List::with_capacity(0),
                cut: false,
            })
            .expect("tree window");
        } else {
            // The solo goal's omitted live descendants are running.
            assert_eq!(facts(w, number).phase, c::Phase::Running);
            phases.running += 1;
        }
    }
    let mut folded = List::with_capacity(1);
    let next = if phases.running > 0 {
        folded.push(c::TreeFold { parent: task, phases, held_below: 0 }).expect("root fold");
        Some(c::Cursor::new(Box::from(b"next".as_slice()), limits().client.domain.text).expect("opaque cursor"))
    } else {
        None
    };
    c::Delivery::Tree(Box::new(c::Tree { root, rows, folded, held_below: 0, next }))
}

fn deployment(w: World) -> c::Deployment {
    c::Deployment {
        store: c::StoreFill { bytes: 0, mark: 0 },
        hosts: List::with_capacity(0),
        attempts: List::with_capacity(0),
        accounts: List::with_capacity(0),
        tasks: c::Capacity { used: 0, limit: 0 },
        workers: c::Capacity { used: 0, limit: 0 },
        attempts_capacity: c::Capacity { used: 0, limit: 0 },
        waiting_attempts: 0,
        proposals: c::Capacity { used: 0, limit: 0 },
        projects: List::with_capacity(0),
        grace_bound: Duration::from_secs(30),
        hello_refusals: List::with_capacity(0),
        hosts_forgotten: 0,
        hello_refusals_forgotten: 0,
        person_room_refusals: List::with_capacity(0),
        person_room_refusals_forgotten: 0,
        person_answer_retention: Duration::from_secs(1000),
        task_room_refusals: List::with_capacity(0),
        task_room_refusals_forgotten: 0,
        cut: false,
        connectors: List::with_capacity(0),
        connectors_cut: false,
        providers: providers(w),
        providers_cut: false,
        key_bytes: 4096,
        readiness: List::with_capacity(0),
        readiness_cut: false,
    }
}

/// Mockup world clocks, without reading the host's clock (client/web/gallery.md, 2).
pub(super) fn now(w: World) -> Wall {
    let minutes = match w {
        FirstRun { step } => match step {
            // Synthetic connection frames have no relative timestamps on screen.
            0 | 1 | 10 | 20..=23 => 0,
            2 => 2,
            3 => 4,
            4 => 6,
            5 => 15,
            6 | 9 | 24 | 25 => 16,
            7 | 8 => 19,
            _ => unreachable!("mockup first-run step"),
        },
        Reference | Solo | World::Bare | World::WholeStory => 0,
    };
    offset(w, minutes)
}

fn offset(w: World, minutes: i64) -> Wall {
    let hour0 = match w {
        Reference => 1_792_656_000_000_000_000_u64,
        Solo => 1_793_093_400_000_000_000,
        FirstRun { .. } => 1_792_702_800_000_000_000,
        World::Bare => 0,
        World::WholeStory => whole_story::now("all-home").as_nanos(),
    };
    let delta = minutes.unsigned_abs().checked_mul(60_000_000_000).expect("mockup offset fits");
    Wall::from_nanos(
        if minutes < 0 { hour0.checked_sub(delta) } else { hour0.checked_add(delta) }.expect("mockup time fits"),
    )
}

fn sample_attempt(root: &mut Domain, out: &mut Queue<Request>) -> Token {
    step(root, Event::Client { event: jig_client::Event::Stored }, out);
    while let Some(request) = out.pop() {
        if let Some(jig_client::Down::Send { attempt, .. }) = crate::driver::down(request) {
            return attempt;
        }
    }
    unreachable!("sample sent its request")
}
fn sample_answer(root: &mut Domain, attempt: Token, answer: jig_client::Answered, out: &mut Queue<Request>) {
    step(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(answer),
                    why: None,
                },
            },
        },
        out,
    );
}
fn sample_forge_answer(
    root: &mut Domain,
    attempt: Token,
    outcome: temper_client_protocol_wire::v1::Outcome,
    out: &mut Queue<Request>,
) {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    sample_answer(
        root,
        attempt,
        jig_client::Answered::Family(jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).expect("sample answer")),
        }),
        out,
    );
}
fn adopt_owned_sample(root: &mut Domain, watch: Token, out: &mut Queue<Request>) {
    use temper_client_protocol_wire::v1 as w;
    step(
        root,
        Event::Act {
            action: temper_client_domain::Action::Forge(temper_client_domain::forge::Act::SetUpAtOnce { forge: 3 }),
        },
        out,
    );
    let attempt = sample_attempt(root, out);
    sample_forge_answer(root, attempt, w::Outcome::SettingUp, out);
    // The engine creates the project and adopts the repository before publishing Made.
    forges_watched(root, FirstRun { step: 7 }, watch, out);
}

/// Sample clocks keep the mock’s dates and relative ages (client/web/gallery.md, 2).
pub(super) fn sample_now(world: World, name: &str) -> Wall {
    match name {
        name if name.starts_with("all-home") || name == "night-all-home" || name.starts_with("home-step-") => {
            whole_story::now(name)
        }
        "rail-new-change" | "rail-all-projects" | "rail-across" | "work" | "work-pair" | "work-drafts"
        | "work-goals" | "work-changes" | "work-empty" | "members" | "members-refused" | "limits"
        | "limits-not-owner" | "night-work" | "night-limits" | "team-role" | "night-team-role" => rail_work::NOW,
        name if name.starts_with("approve-") => Wall::from_nanos(120_000_000_000),
        name if name.starts_with("chat-work-") || name == "agent-work" => {
            Wall::from_nanos(chat_work::AT + 60_000_000_000)
        }
        "home-morning"
        | "home-decided"
        | "needs-decided"
        | "needs-decided-project"
        | "home-away"
        | "home-away-one"
        | "home-away-overturned"
        | "home-away-quiet" => Wall::from_nanos(1_792_483_440_000_000_000),
        "rail-away-on" | "rail-away-off" => Wall::from_nanos(1_792_479_840_000_000_000),
        name if name.starts_with("goal-") => goal_wall(world, 0),
        "changes" | "changes-one" | "changes-empty" => offset(Reference, -173),
        "changes-landed-only" => offset(Reference, 2),
        _ => now(world),
    }
}

/// The changes subject is supplied by the sample, independently of the engine (client/forge/records.md).
pub(super) fn changes_world(name: &str) -> Domain {
    use jig_client::{Down, Readable, Up, Watchable};
    use temper_client_protocol_wire::{self as wire, v1 as v};
    let limits = limits();
    let (mut root, mut out) = started(Address::Changes { project: PROJECT });
    let panel = changes_panel(name);
    for _ in 0..256 {
        let Some(request) = out.pop() else { break };
        let Some(down) = crate::driver::down(request) else { continue };
        match down {
            Down::Open { watch, subject: Watchable::Jig(c::Subject::Me) } => {
                let project = if name == "changes-landed-only" { "hello" } else { "fabrig" };
                let mut me = base_frame("Owner", project);
                me.needs_total = if name == "changes-landed-only" { 1 } else { 3 };
                me.needs = c::Needs { questions: me.needs_total, proposals: 0, escalations: 0, person_tasks: 0 };
                deliver(&mut root, watch, c::Delivery::Me(Box::new(me)), &mut out);
            }
            Down::Open { watch, subject: Watchable::Jig(c::Subject::Accounts) } => {
                deliver(&mut root, watch, c::Delivery::Accounts(Box::new(accounts(Reference, Then::Shown))), &mut out);
            }
            Down::Open { watch, subject: Watchable::Family { record, .. } } => {
                let watching = wire::decode_watching(&record.bytes, &v::CEILINGS).expect("gallery watch");
                if let v::Watch::Changes(_) = watching.watch() {
                    for up in [
                        Up::Watched { watch, event: c::WatchEvent::Opened },
                        Up::FamilyDelivered {
                            watch,
                            record: jig_client::Family {
                                family: wire::FORGE,
                                version: wire::CLIENT_VERSION,
                                bytes: wire::changes_panel(&panel),
                            },
                        },
                    ] {
                        step(&mut root, Event::Client { event: jig_client::Event::Up { up } }, &mut out);
                    }
                } else if let v::Watch::Forges = watching.watch() {
                    forges_watched(&mut root, Reference, watch, &mut out);
                }
            }
            Down::Open { watch, subject: Watchable::Jig(c::Subject::Settings) } => {
                deliver(
                    &mut root,
                    watch,
                    c::Delivery::Settings(Box::new(c::SettingsDocument {
                        newest: 0,
                        settings: List::with_capacity(0),
                        held: List::with_capacity(0),
                        pending: List::with_capacity(0),
                    })),
                    &mut out,
                );
            }
            Down::Read { read, what: Readable::Family { record, .. } } => {
                if let v::Read::Setting(setting) =
                    wire::decode_reading(&record.bytes, &v::CEILINGS).expect("gallery setting read").read()
                {
                    match setting.kind() {
                        7 => {
                            project_ways(&mut root, read, false, false, name != "changes-landed-only", &mut out);
                        }
                        8 => project_classes(&mut root, read, &mut out),
                        _ => panic!("gallery setting read"),
                    }
                }
            }
            Down::Read { read, what: Readable::Jig(c::Read::Task { task }) } => {
                let mut document = base_task(task, "Change");
                document.project = PROJECT;
                document.lineage_chips = List::with_capacity(1);
                let (goal, title) = match task {
                    100 | 103 | 104 | 105 => (Some(80), "first-run-projects"),
                    101 => (Some(81), "forge-cards"),
                    106 => (Some(82), "pane-chat-line"),
                    102 => (None, "chat T6"),
                    107 => (None, "chat T2"),
                    _ => unreachable!("listed sample task"),
                };
                document.goal = goal;
                let mut chip = document.chip.clone();
                chip.task = goal.unwrap_or(if task == 107 { 2 } else { 6 });
                chip.title = doc(title);
                chip.executor = c::ChipExecutor::Agent { charter: 1 };
                document.lineage_chips.push(chip).expect("one lineage chip");
                answer(&mut root, read, c::ReadValue::Task(Box::new(document)), &mut out);
            }
            Down::Read { .. }
            | Down::Open { .. }
            | Down::Continue { .. }
            | Down::Close { .. }
            | Down::Save { .. }
            | Down::Erase { .. }
            | Down::Send { .. } => {}
        }
    }
    assert!(out.is_empty(), "changes sample settled");
    assert!(limits.client.domain.reads >= 7, "the sample's shown rows fit");
    root
}
pub(super) fn project_ways(
    root: &mut Domain,
    read: Token,
    drafts: bool,
    goals: bool,
    changes: bool,
    out: &mut Queue<Request>,
) {
    use temper_client_protocol_wire::{self as wire, v1 as v};
    let ways = v::ProjectWays::new(&v::CEILINGS, v::ProjectWaysParts { drafts, goals, changes, way: 0 })
        .expect("gallery project ways");
    let mut writer = skein_lib::Writer::new(usize::try_from(ways.measure()).expect("ways fit"));
    ways.encode(&mut writer).expect("measured ways");
    let value = v::SettingAnswer::new(&v::CEILINGS, v::SettingAnswerParts { value: Some(writer.finish()) })
        .expect("gallery setting answer");
    step(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family { family: FORGE, record: Some(wire::setting_answer(&value)) },
                },
            },
        },
        out,
    );
}

pub(super) fn project_classes(root: &mut Domain, read: Token, out: &mut Queue<Request>) {
    use temper_client_protocol_wire::{self as wire, v1 as v};
    let value =
        v::SettingAnswer::new(&v::CEILINGS, v::SettingAnswerParts { value: None }).expect("stock project classes");
    step(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family { family: FORGE, record: Some(wire::setting_answer(&value)) },
                },
            },
        },
        out,
    );
}

fn changes_panel(name: &str) -> temper_client_protocol_wire::v1::ChangesPanel {
    use temper_client_protocol_wire::v1 as v;
    let mut queues = List::with_capacity(1);
    let mut changes = List::with_capacity(3);
    if name == "changes" || name == "changes-one" {
        for (task, title) in
            [(100, "Store: policy record"), (101, "Two welcome cards"), (102, "Help typo in server.md")]
        {
            if name == "changes-one" && task != 100 {
                continue;
            }
            changes
                .push(
                    v::ChangesQueued::new(
                        &v::CEILINGS,
                        v::ChangesQueuedParts {
                            task,
                            title: title.as_bytes().into(),
                            ready_since: offset(Reference, -179).as_nanos(),
                            checks_since: if task == 100 { Some(offset(Reference, -179).as_nanos()) } else { None },
                        },
                    )
                    .expect("queued change"),
                )
                .expect("three changes");
        }
        queues
            .push(
                v::ChangesQueue::new(
                    &v::CEILINGS,
                    v::ChangesQueueParts {
                        repository: b"jeremy/fabrig".as_slice().into(),
                        branch: b"main".as_slice().into(),
                        red: false,
                        changes,
                    },
                )
                .expect("queue"),
            )
            .expect("one queue");
    }
    let mut landed = List::with_capacity(4);
    if name == "changes" || name == "changes-one" {
        for (task, title, minute) in [
            (103, "Pane’s playground line", -179),
            (104, "Banner’s set-up link", -230),
            (105, "Playground project", -270),
            (106, "Journey print", -408),
        ] {
            landed
                .push(
                    v::ChangesLanded::new(
                        &v::CEILINGS,
                        v::ChangesLandedParts {
                            signed: skein_lib::List::with_capacity(8),
                            commit: [0xab; 32],
                            default_branch: true,
                            task,
                            title: title.as_bytes().into(),
                            at: offset(Reference, minute).as_nanos(),
                        },
                    )
                    .expect("landed change"),
                )
                .expect("four landings");
        }
    } else if name == "changes-landed-only" {
        landed
            .push(
                v::ChangesLanded::new(
                    &v::CEILINGS,
                    v::ChangesLandedParts {
                        signed: skein_lib::List::with_capacity(8),
                        commit: [0xab; 32],
                        default_branch: true,
                        task: 107,
                        title: b"Hello in Rust".as_slice().into(),
                        at: offset(Reference, -58).as_nanos(),
                    },
                )
                .expect("landed change"),
            )
            .expect("one landing");
    }
    v::ChangesPanel::new(
        &v::CEILINGS,
        v::ChangesPanelParts {
            landed_default: List::with_capacity(0),
            landed_default_more: 0,
            project: PROJECT,
            landing: b"main".as_slice().into(),
            queues,
            more: 0,
            landed,
            landed_more: 0,
        },
    )
    .expect("changes panel")
}

/// Home's computed facts (client/pages/README.md), delivered by its status watch.
pub(super) fn home_status(name: &str) -> Option<temper_client_protocol_wire::v1::ProjectStatus> {
    use temper_client_protocol_wire::v1 as w;
    let (landed, seen, queued, red, previous, last) = match name {
        "home-last-landed" => (1, 1, 0, false, Some(540), Some(6)),
        "home-computed" => (3, 1, 3, false, Some(540), Some(6)),
        "home-last-looked-none" => (3, 1, 3, false, None, Some(6)),
        "home-landing-none" => (0, 0, 0, false, Some(540), None),
        "home-landing-red" => (3, 1, 0, true, Some(540), Some(60)),
        "home-chats-sentence" => (0, 0, 0, false, None, None),
        _ => return None,
    };
    let now = now(FirstRun { step: 7 });
    Some(
        w::ProjectStatus::new(
            &w::CEILINGS,
            w::ProjectStatusParts {
                project: PROJECT,
                landed,
                seen,
                queued,
                red,
                previous: previous.map(|minutes: u64| {
                    now.as_nanos()
                        .checked_sub(minutes.checked_mul(60_000_000_000).expect("Home span fits"))
                        .expect("Home previous follows epoch")
                }),
                last_landed: last.map(|minutes: u64| {
                    now.as_nanos()
                        .checked_sub(minutes.checked_mul(60_000_000_000).expect("Home span fits"))
                        .expect("Home landing follows epoch")
                }),
                brief: None,
                branch: b"main".as_slice().into(),
            },
        )
        .expect("Home status fits"),
    )
}

/// Settings' delivered kinds 6, 7 and 8 (temper/docs/design/client/web/gallery.md, 2).
fn settings_bytes(name: &str, kind: u32) -> Box<[u8]> {
    use temper_client_protocol_wire::v1 as v;
    if kind == 8 {
        use temper_protocol_reaches::{Classes, Way, classes_record};
        if matches!(name, "how-reaches" | "how-rules" | "how-rules-card") {
            return ways::reaches_setting(name);
        }
        let classes =
            classes_record(&Classes::stock(if name == "how-team" { Way::AgentsReview } else { Way::YouApprove }))
                .expect("project classes");
        let mut writer = skein_lib::Writer::new(usize::try_from(classes.measure()).expect("classes bytes"));
        classes.encode(&mut writer).expect("measured classes");
        writer.finish()
    } else if kind == 7 {
        let shown = name != "settings-all-hidden";
        let all = shown && name != "settings-goals";
        let ways = v::ProjectWays::new(
            &v::CEILINGS,
            v::ProjectWaysParts {
                drafts: all && name != "settings-drafts-off",
                goals: shown && name != "settings-goals-off",
                changes: all && name != "settings-changes-off",
                way: if shown { 4 } else { 0 },
            },
        )
        .expect("settings ways");
        let mut writer = skein_lib::Writer::new(usize::try_from(ways.measure()).expect("ways bytes"));
        ways.encode(&mut writer).expect("measured ways");
        writer.finish()
    } else {
        assert_eq!(kind, 6);
        let route = if name == "settings-all-hidden" { v::QuestionsRoute::None } else { v::QuestionsRoute::Up };
        let questions =
            v::ProjectQuestions::new(&v::CEILINGS, v::ProjectQuestionsParts { route }).expect("settings questions");
        let mut writer = skein_lib::Writer::new(usize::try_from(questions.measure()).expect("questions bytes"));
        questions.encode(&mut writer).expect("measured questions");
        writer.finish()
    }
}
pub(super) fn settings_world(name: &str) -> Domain {
    use temper_client_domain::forge::{
        self,
        ways::{Act, Asked, Ways},
    };
    let (mut root, _) = world_in_layout(
        Solo,
        Address::Settings { project: PROJECT },
        Then::Shown,
        name == "settings-chat-open",
        None,
        Some(name),
        false,
    );
    if name == "settings-refused" {
        let mut out = Queue::uncapped();
        let shown = root.forge().ways(PROJECT).expect("delivered ways");
        step(
            &mut root,
            Event::Act {
                action: temper_client_domain::Action::Forge(forge::Act::Ways(Act::Set {
                    project: PROJECT,
                    asked: Asked::Ways(Ways { drafts: false, ..shown }),
                })),
            },
            &mut out,
        );
        step(&mut root, Event::Client { event: jig_client::Event::Stored }, &mut out);
        for _ in 0..64 {
            let Some(request) = out.pop() else { break };
            if let Some(jig_client::Down::Send { attempt, .. }) = crate::driver::down(request) {
                step(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Answer {
                                attempt,
                                envelope: skein_lib::RequestEnvelope::Final,
                                answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                                    refusal: c::Refusal::NotOffered,
                                    at: None,
                                })),
                                why: None,
                            },
                        },
                    },
                    &mut out,
                );
            }
        }
        assert!(root.forge().ways_refused(forge::ways::Control::Ways).is_some());
    }
    root
}

const SETTINGS_NEXT: c::ProposalKey = c::ProposalKey { proposer: c::Proposer::Task(10), number: 2 };
pub(super) fn settings_chat_world(name: &str) -> Domain {
    world_in_layout(Solo, Address::Chat { task: 10 }, Then::Shown, true, None, Some(name), false).0
}
fn settings_proposal(key: c::ProposalKey, name: &str) -> c::ReadValue {
    let c::ReadValue::Proposal(mut proposal) = proposal() else { unreachable!("proposal fixture") };
    proposal.key = key;
    proposal.project = 0;
    proposal.projects = List::with_capacity(2);
    if name != "ask-chat-proposal-deployment" {
        proposal.projects.push(PROJECT).expect("first project");
    }
    if name == "ask-chat-proposal-two-projects" {
        proposal.projects.push(2).expect("second project");
    }
    proposal.reason = doc("## temper enforces\nI work alone on small changes.\nKeep drafts beside the code.");
    let mut items = List::with_capacity(3);
    let settings = if name == "ask-chat-proposal-deployment" {
        List::from_boxed(Box::new([c::Setting::Charter(c::SettingNumber { number: 1 })]))
    } else if name == "ask-chat-proposal-two-projects" {
        List::from_boxed(Box::new([
            c::Setting::Application { kind: 7, scope: PROJECT, number: 1 },
            c::Setting::Application { kind: 7, scope: 2, number: 1 },
        ]))
    } else {
        List::from_boxed(Box::new([
            c::Setting::Application { kind: 7, scope: PROJECT, number: 1 },
            c::Setting::Application { kind: 6, scope: PROJECT, number: 1 },
        ]))
    };
    let mut review = List::with_capacity(3);
    for setting in &settings {
        if let c::Setting::Charter(_) = setting {
            items
                .push(c::RevisionItemHead { setting: *setting, edit: c::RevisionItemEdit::Set })
                .expect("settings item");
        }
        review
            .push(c::ReviewItem {
                setting: *setting,
                edit: c::RevisionItemEdit::Set,
                now: c::ReviewValue::None,
                origin: c::SettingOrigin::NotOffered,
                would_be: c::ReviewValue::None,
                application: None,
            })
            .expect("review item");
    }
    proposal.action = c::ProposalAction::Settings(Box::new(c::ProposalActionSettings {
        against: 0,
        items,
        review: Some(c::SettingsReview { key: Some(9001), newest: 0, items: review, cut: false }),
        made: None,
    }));
    c::ReadValue::Proposal(proposal)
}
fn settings_proposal_values(root: &mut Domain, read: Token, out: &mut Queue<Request>) {
    use temper_client_protocol_wire::{self as wire, v1 as v};
    let mut items = List::with_capacity(2);
    for kind in [7, 6] {
        let value =
            v::SettingValue::new(&v::CEILINGS, v::SettingValueParts { value: settings_bytes("settings", kind) })
                .expect("setting value");
        let item = v::ProposalItem::new(
            &v::CEILINGS,
            v::ProposalItemParts { kind, scope: PROJECT, number: 1, edit: v::SettingEdit::Set(value) },
        )
        .expect("proposal item");
        items.push(item).expect("proposal items");
    }
    let record =
        v::ProposalItemsAnswer::new(&v::CEILINGS, v::ProposalItemsAnswerParts { items, owned_approval: false })
            .expect("proposal values");
    step(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family {
                        family: FORGE,
                        record: Some(wire::proposal_items_answer(&record)),
                    },
                },
            },
        },
        out,
    );
}
fn settings_activity(name: &str) -> c::ActivityPage {
    // Each proposal has its making line, tool completion and following reply.
    let mut rows = List::with_capacity(6);
    let keys: &[c::ProposalKey] = if name == "ask-chat-replaced" { &[PROPOSAL, SETTINGS_NEXT] } else { &[PROPOSAL] };
    for key in keys {
        let made = documents::line(
            1,
            Wall::EPOCH,
            c::Activity::Decided { revision: 1, proposal: Some(c::DecidedProposal { number: key.number, made: true }) },
        );
        let call = c::Block::Call(c::Call {
            number: key.number,
            tool: text("propose_settings"),
            input: text(""),
            outcome: None,
            failed: false,
            objects: List::with_capacity(0),
            parts: List::with_capacity(0),
        });
        let blocks = List::from_boxed(Box::new([call]));
        let number = u32::try_from(key.number.checked_mul(2).expect("small turn")).expect("small turn");
        let turn = turn_line(&blocks, number.checked_sub(1).expect("nonzero turn"), None, false);
        let reply = agent_line("Tell me what you would change.", number, None);
        for mut line in [made, turn, reply] {
            line.source.position = u64::from(rows.len()).checked_add(1).expect("small activity");
            line.source.task = 10;
            if let Some(decoded) = &mut line.decoded {
                decoded.task = 10;
                decoded.cumulative = line.source.position;
            }
            rows.push(line).expect("settings activity");
        }
    }
    c::ActivityPage { rows, cut: false, before: None, after: None }
}
fn finish_settings_proposal(root: &mut Domain, watch: Token, name: &str, out: &mut Queue<Request>) {
    let mut task = root.pages().chat().expect("chat").task(root.client()).expect("task").clone();
    task.proposals = List::with_capacity(1);
    if name == "ask-chat-replaced" {
        task.proposals.push(SETTINGS_NEXT).expect("pending proposal");
    }
    deliver(root, watch, c::Delivery::Task(Box::new(task)), out);
    for _ in 0..64 {
        let Some(request) = out.pop() else { break };
        if let Some(jig_client::Down::Read {
            read,
            what: jig_client::Readable::Jig(c::Read::Proposal { proposal: key }),
        }) = crate::driver::down(request)
        {
            assert_eq!(key, PROPOSAL);
            let c::ReadValue::Proposal(proposal) = settings_proposal(key, name) else { unreachable!("settings") };
            let c::ProposalAction::Settings(settings) = proposal.action else { unreachable!("settings") };
            let state = if name == "ask-chat-accepted" {
                c::ProposalHistoryState::Accepted { by: c::Requester::Person(73) }
            } else if name == "ask-chat-rejected" {
                c::ProposalHistoryState::Rejected {
                    by: c::Requester::Person(73),
                    reason: doc("Keep the current settings."),
                }
            } else {
                c::ProposalHistoryState::Withdrawn
            };
            answer(
                root,
                read,
                c::ReadValue::Ended(Box::new(c::Ended {
                    key: c::ObjectKey::Proposal(key),
                    value: c::EndedValue::Proposal(c::ProposalHistory {
                        reason: proposal.reason,
                        action: c::RetainedAction::Settings(c::RetainedActionSettings {
                            projects: proposal.projects,
                            against: settings.against,
                            items: settings.items,
                            made: if name == "ask-chat-accepted" { Some(1) } else { None },
                        }),
                        number: key.number,
                        proposer: 10,
                        as_holder: true,
                        state,
                    }),
                    at: None,
                    by: None,
                })),
                out,
            );
        }
    }
    assert!(out.is_empty(), "settings decision settles");
}

fn settings_chat_frame(me: &mut c::Me) {
    let member = me.projects.get_mut(0).expect("project");
    let mut templates = List::with_capacity(3);
    for template in &member.templates {
        templates.push(template.clone()).expect("chat templates");
    }
    let mut way = member.templates.get(0).expect("chat template").clone();
    way.number = 4;
    way.name = doc("goal for a draft");
    way.tracked = Some(1);
    way.flags.conversation = false;
    way.flags.listed = true;
    templates.push(way).expect("default way");
    member.templates = templates;
}
/// The Builders sample's three supplied connection states (client/web/gallery.md, 2).
fn builders() -> List<c::Host> {
    let mut rows = List::with_capacity(3);
    for (channel, name, connected, draining) in
        [(7, "Workshop", true, false), (8, "Finishing", true, true), (9, "Away", false, false)]
    {
        rows.push(c::Host {
            name: Some(name.as_bytes().into()),
            channel,
            kind: c::HostKind::Worker,
            slots: 2,
            used: 1,
            free: 1,
            draining,
            connected,
            awaiting_logins: false,
            rule: c::HostRule::Open,
            programs: List::with_capacity(0),
            grace_until: None,
            grace_remaining: None,
            runs: List::with_capacity(0),
            cut: false,
        })
        .expect("three builders");
    }
    rows
}

/// Home's copies share Ready's intent; the watch supplies progress and adoptions (client/pages/README.md).
pub(super) fn home_copies(
    mut row: temper_client_protocol_wire::v1::ForgeRowParts,
    state: &str,
) -> temper_client_protocol_wire::v1::ForgeRowParts {
    use temper_client_protocol_wire::v1 as w;
    let addresses: [&[u8]; 2] = [b"https://github.com/acme/widget", b"https://github.com/acme/other"];
    let addresses = if state == "home-copy-one" { &addresses[..1] } else { &addresses[..] };
    let mut intent = List::with_capacity(2);
    for address in addresses {
        intent.push((*address).into()).expect("copy intent");
    }
    row.first = None;
    row.starts_with = w::StartsWith::Copies(
        w::CopyAddresses::new(&w::CEILINGS, w::CopyAddressesParts { addresses: intent }).expect("copy intent"),
    );
    row.makings = List::with_capacity(2);
    row.repositories = List::with_capacity(2);
    // The watch's key order is other, widget, while Ready asked widget first.
    for address in addresses.iter().rev() {
        let (org, name) = temper_client_domain::forge::making::target(address).expect("copy target");
        if state == "home-copy-offer" {
            continue;
        }
        if matches!(state, "home-copied" | "home-copy-one") || state == "home-copy-failed" && *address == addresses[0] {
            row.repositories
                .push(
                    w::Adoption::new(&w::CEILINGS, w::AdoptionParts { owner: org, name, project: PROJECT })
                        .expect("copy adopted"),
                )
                .expect("two copies");
        } else {
            row.makings
                .push(
                    w::RepositoryMaking::new(
                        &w::CEILINGS,
                        w::RepositoryMakingParts {
                            org,
                            name,
                            from: w::MakeFrom::Copy(
                                w::MakeCopy::new(&w::CEILINGS, w::MakeCopyParts { address: (*address).into() })
                                    .expect("copy source"),
                            ),
                            into: Some(
                                w::MakeInto::new(
                                    &w::CEILINGS,
                                    w::MakeIntoParts { project: PROJECT, home: *address == addresses[0] },
                                )
                                .expect("copy into"),
                            ),
                            state: if state == "home-copy-failed" {
                                w::MakingState::Failed(
                                    w::MakingFailed::new(
                                        &w::CEILINGS,
                                        w::MakingFailedParts {
                                            step: w::MakeStep::Repository,
                                            reason: w::MakeFailure::Taken,
                                            words: b"name taken".as_slice().into(),
                                        },
                                    )
                                    .expect("copy failure"),
                                )
                            } else if *address == addresses[0] {
                                w::MakingState::Making
                            } else {
                                w::MakingState::Waiting
                            },
                        },
                    )
                    .expect("copy making"),
                )
                .expect("two copies");
        }
    }
    row
}

/// Home’s Add sheet is page state reached through its own button (client/pages/README.md).
pub(super) fn home_add_sheet(root: &mut Domain, copies: bool) {
    step(
        root,
        Event::Act {
            action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::AddOpen {
                start: if copies {
                    temper_client_domain::forge::forges::Start::Copies
                } else {
                    temper_client_domain::forge::forges::Start::Empty
                },
            }),
        },
        &mut Queue::uncapped(),
    );
}

fn chat_note(scope: &c::NoteScope) -> c::NotePage {
    let mut page = asking_note(scope);
    let note = page.rows.get_mut(0).expect("one document");
    note.description = doc("Away mode");
    note.revision = c::Revision { value: 2 };
    note.writer = c::NoteAuthor::Task { task: 1, attempt: 1 };
    note.body = c::DocumentText::complete(c::Text::new(Box::from("# Away mode

Draft, 2026-11-03, with the user.

## 1. In one page

- A setting, per project or for all: away until a time, or until turned off.
- While away, agents decide what they would ask, within the kinds the project allows (section 3).
- Each decision shows as decided for you, with Overturn while it is cheap (section 4).
- Coming back opens on what was decided (section 5).

## 2. The setting

The switch sits beside the person’s name, on the laptop and on the phone. On, it reads “Away until 08:00”; the time comes from the quiet hours, or one the person picks.

## 3. What agents may decide

- Implementation choices within a task’s spec.
- Retries, and splitting a failing task.
- A design document’s wording, not its decisions.
- Adding a dependency: off unless turned on.
- Always wait, fixed: budgets, sign-ins, machines, what is hard to reverse.

## 4. Decided for you

Each decision keeps the question, the choice taken and why. It shows in Needs you as decided, with Overturn while overturning is cheap: before its change lands. Overturning asks the question again.

## 5. Coming back

Home opens on what was decided, then what still needs you. Turning away off is the person’s; it never turns itself off.

## Open questions

1. How long can a decision be overturned after its change landed?
".as_bytes()), limits().client.domain.long_text).expect("chat document"));
    page
}

fn chat_activity() -> c::ActivityPage {
    let mut rows = List::with_capacity(2);
    for (number, reply, outcome) in [
        (
            1,
            "Outline: in one page, the setting, what agents may decide, decided for you, coming back. Writing it now.",
            "Note 1 revision 1 written · 64 lines",
        ),
        (
            2,
            "Done: section 3 is a list of kinds, with the fixed waits underneath. One question remains open in section 6.",
            "Note 1 revision 2 written · +18 −6 · line 17",
        ),
    ] {
        let mut blocks = List::with_capacity(2);
        blocks.push(c::Block::Text(text(reply))).expect("reply");
        blocks
            .push(c::Block::Call(c::Call {
                number,
                tool: text("note"),
                input: text(""),
                outcome: Some(text(outcome)),
                failed: false,
                objects: List::with_capacity(0),
                parts: List::with_capacity(0),
            }))
            .expect("write");
        let mut line = turn_line(&blocks, u32::try_from(number).expect("two turns"), None, false);
        line.source.task = 1;
        line.source.position = number;
        if let Some(decoded) = &mut line.decoded {
            decoded.task = 1;
        }
        rows.push(line).expect("two turns");
    }
    c::ActivityPage { rows, before: None, after: None, cut: false }
}

/// Away's watch supplies the row, while the request's answer supplies its toast (client/web/gallery.md, 2).
pub(super) fn away_record(on: bool) -> Box<[u8]> {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let wall = sample_now(FirstRun { step: 7 }, "rail-away-on");
    let until = temper_client_domain::forge::away::next_eight(wall, 0);
    let row = w::AwayRow::new(
        &w::CEILINGS,
        w::AwayRowParts {
            project: PROJECT,
            since: wall.as_nanos(),
            until: Some(until.as_nanos()),
            landed: 0,
            off: if on { None } else { Some(wall.as_nanos()) },
        },
    )
    .expect("Away row");
    wire::aways(
        &w::Aways::new(&w::CEILINGS, w::AwaysParts { rows: List::from_boxed(Box::new([row])) }).expect("Away rows"),
    )
}
pub(super) fn rail_away_toggle(root: &mut Domain, watch: Token, on: bool) {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut out = Queue::uncapped();
    temper_client_domain::step(
        root,
        &skein_lib::Env {
            now: skein_lib::Time::ZERO,
            wall: sample_now(FirstRun { step: 7 }, "rail-away-on"),
            limits: limits(),
        },
        Event::Act {
            action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Away(
                temper_client_domain::pages::away::Act::Toggle { project: PROJECT },
            )),
        },
        &mut out,
    );
    let attempt = sample_attempt(root, &mut out);
    sample_forge_answer(root, attempt, w::Outcome::Away, &mut out);
    step(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: away_record(on),
                    },
                },
            },
        },
        &mut out,
    );
}

/// Away's hand-made Home deliveries (client/web/gallery.md, 2).
pub(super) fn home_away_record(name: &str) -> Box<[u8]> {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let now = sample_now(FirstRun { step: 7 }, name).as_nanos();
    let minute = 60_000_000_000;
    let row = w::AwayRow::new(
        &w::CEILINGS,
        w::AwayRowParts {
            project: PROJECT,
            since: now - 534 * minute,
            until: None,
            landed: 0,
            off: if name == "home-away-one" { Some(now - minute) } else { None },
        },
    )
    .expect("Home away spell");
    if matches!(name, "home-decided" | "needs-decided" | "needs-decided-project") {
        return wire::aways(
            &w::Aways::new(&w::CEILINGS, w::AwaysParts { rows: List::with_capacity(0) }).expect("no Away spell"),
        );
    }
    wire::aways(
        &w::Aways::new(&w::CEILINGS, w::AwaysParts { rows: List::from_boxed(Box::new([row])) })
            .expect("Home away rows"),
    )
}
pub(super) fn home_away_status() -> temper_client_protocol_wire::v1::ProjectStatus {
    use temper_client_protocol_wire::v1 as w;
    let now = sample_now(FirstRun { step: 7 }, "home-away").as_nanos();
    let lede = "first-run-projects stopped at 05:50 for funds, which always wait for you. welcome-stall landed its fix in temper serve.";
    w::ProjectStatus::new(
        &w::CEILINGS,
        w::ProjectStatusParts {
            project: PROJECT,
            landed: 57,
            seen: 0,
            queued: 3,
            red: false,
            previous: None,
            last_landed: Some(now - 6 * 60_000_000_000),
            branch: b"main".as_slice().into(),
            brief: Some(
                w::StatusBrief::new(
                    &w::CEILINGS,
                    w::StatusBriefParts {
                        sentence: b"57 changes landed.".as_slice().into(),
                        lede: lede.as_bytes().into(),
                        at: now - 9 * 60_000_000_000,
                        landed: 57,
                    },
                )
                .expect("Home away brief"),
            ),
        },
    )
    .expect("Home away status")
}
fn home_away_task(name: &str, task: u64, project: u32) -> c::Task {
    let mut document = base_task(
        task,
        match task {
            101 => "welcome-stall",
            103 => "first-run-projects",
            _ => "forge-cards",
        },
    );
    document.project = project;
    document.chip.details.details.project = project;
    if name == "home-decided" && task == 104 {
        document.priority = Some(1);
        document.goal = Some(task);
    }
    if name == "home-away-one" || (name == "home-away-overturned" && task == 101) {
        document.chip.details.phase = c::ChipPhase::Ended;
        document.chip.details.details.ended_at =
            Some(Wall::from_nanos(sample_now(FirstRun { step: 7 }, name).as_nanos() - 292 * 60_000_000_000));
        document.chip.details.details.ending = Some(c::ChipEnding::Done(c::ResultValue::Report(doc("Landed."))));
    }
    document
}
pub(super) fn home_away_decisions(name: &str, project: u32) -> Box<[u8]> {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let now = sample_now(FirstRun { step: 7 }, name).as_nanos();
    let mut rows = List::with_capacity(4);
    for (number, task, minutes, question, took, why) in [
        (
            1,
            101,
            419,
            "Fix in jig’s store or in temper serve?",
            "In temper serve",
            "jig’s fix changes a stored record, which always waits for you; serve’s fix doesn’t. The jig fix can follow.",
        ),
        (
            2,
            102,
            324,
            "Link the GitHub help page, or hide the link?",
            "Link the GitHub page",
            "The agent’s own recommendation; the welcome’s design names the link a placeholder.",
        ),
        (
            3,
            103,
            144,
            "The store task failed twice on size. Retry, or split it?",
            "Split it in two",
            "Splitting a failing task is allowed while you’re away.",
        ),
        (
            4,
            104,
            109,
            "The welcome card says “Run on this machine”. Keep it, or “Set up here”?",
            "Keep “Run on this machine”",
            "A wording choice inside the task’s spec.",
        ),
    ] {
        if name == "home-away-quiet" {
            continue;
        }
        if matches!(name, "home-decided" | "needs-decided-project") && number != 1 && number != 4 {
            continue;
        }
        if name == "needs-decided" && ((project == 1 && number != 1) || (project == 2 && number != 4)) {
            continue;
        }
        if name == "home-away-one" && number != 1 {
            continue;
        }
        rows.push(
            w::Decision::new(
                &w::CEILINGS,
                w::DecisionParts {
                    number,
                    task,
                    at: now - minutes * 60_000_000_000,
                    question: question.as_bytes().into(),
                    took: took.as_bytes().into(),
                    why: why.as_bytes().into(),
                    overturned: name == "home-away-overturned" && number == 2,
                    reach: w::Reach::new(
                        &w::CEILINGS,
                        w::ReachParts {
                            kind: match number {
                                3 => w::ReachKind::Retry,
                                4 => w::ReachKind::Wording,
                                _ => w::ReachKind::Implementation,
                            },
                            class: if number == 4 { w::ReachClass::Decide } else { w::ReachClass::Ask },
                            reason: w::ReachReason::Kind,
                            under: 0,
                            sets: 0,
                            matched: 0,
                            away: number != 4,
                        },
                    )
                    .expect("fixture reach fits"),
                },
            )
            .expect("Home decision"),
        )
        .expect("four Home decisions");
    }
    wire::decisions(
        &w::Decisions::new(
            &w::CEILINGS,
            w::DecisionsParts { project, rows, asked: List::with_capacity(0), cut: false },
        )
        .expect("Home decisions"),
    )
}

fn home_away_step(root: &mut Domain, name: &str, event: Event, out: &mut Queue<Request>) {
    temper_client_domain::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: sample_now(FirstRun { step: 7 }, name), limits: limits() },
        event,
        out,
    );
}
/// Decision reads use the sample's wall so the shown rows remain within their day.
pub(super) fn home_away_deliveries(root: &mut Domain, watch: Token, name: &str, project: u32) {
    use jig_client::{Down, Readable, Readout, Up};
    let mut out = Queue::uncapped();
    home_away_step(
        root,
        name,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: FORGE,
                        version: temper_client_protocol_wire::CLIENT_VERSION,
                        bytes: home_away_decisions(name, project),
                    },
                },
            },
        },
        &mut out,
    );
    for _ in 0..64 {
        let Some(request) = out.pop() else { break };
        if let Some(Down::Read { read, what: Readable::Jig(c::Read::Task { task }) }) = crate::driver::down(request) {
            home_away_step(
                root,
                name,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: Up::ReadDone {
                            read,
                            result: Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(home_away_task(
                                name, task, project,
                            ))))),
                        },
                    },
                },
                &mut out,
            );
        }
    }
    assert!(out.is_empty(), "Home decisions settle");
}

/// The Models samples use only the accounts' supplied offer (client/pages/README.md, The `Models` tab).
fn models_frame(me: &mut c::Me, one_account: bool, state: super::samples::ModelState) {
    use super::samples::ModelState;
    if state == ModelState::Member {
        me.deployment_role = Some(c::HeldRole::Policy { role: 1 });
    }
    let member = me.projects.get_mut(0).expect("models project");
    let sample = member.models.get(0).expect("sample model").clone();
    member.models = List::with_capacity(7);
    for (number, (provider, name, shown, effort)) in (1u32..).zip([
        (17, "claude-opus-5-5", "Opus 5.5", "high"),
        (17, "claude-sonnet-5-5", "Sonnet 5.5", "high"),
        (17, "claude-fable-5-1", "Fable 5.1", "high"),
        (18, "gpt-6.1-sol", "GPT-6.1 Sol", "high"),
        (18, "gpt-6-astra", "GPT-6 Astra", "high"),
        (18, "gpt-6.1-terra", "GPT-6.1 Terra", "medium"),
        (18, "gpt-6-luna", "GPT-6 Luna", "medium"),
    ]) {
        if one_account && provider == 18 {
            continue;
        }
        let mut model = sample.clone();
        model.number = number;
        model.provider = Some(provider);
        model.name = text(name);
        model.shown = Some(text(shown));
        model.efforts =
            List::from_boxed(Box::new([text("low"), text("medium"), text("high"), text("xhigh"), text("max")]));
        model.effort = Some(text(effort));
        if state == ModelState::New && name == "gpt-6-luna" {
            model.source = c::ModelSource::List;
            model.new_until = Some(offset(Reference, 7 * 24 * 60));
        }
        model.picker = name != "gpt-6-luna" && !(state == ModelState::Off && name == "claude-sonnet-5-5");
        if state == ModelState::Added && name == "claude-sonnet-5-5" {
            model.source = c::ModelSource::Person;
            model.estimated = true;
        }
        member.models.push(model).expect("seven supplied models");
    }
    member.defaults = List::from_boxed(Box::new([c::MeDefault {
        template: jig_client::shapes::CHAT,
        choice: c::LlmChoice {
            later: None,
            account: None,
            main: c::ModelEffort { model: text("claude-opus-5-5"), effort: Some(text("high")) },
            subagents: List::with_capacity(0),
        },
    }]));
}
