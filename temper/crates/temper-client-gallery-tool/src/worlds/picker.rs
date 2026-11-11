//! Composer picker states (client/web/gallery.md; client/pages/README.md).
use super::{
    Address, CHAT_FLAGS, CHAT_LABEL, Domain, Event, List, PROJECT, Queue, Request, Then, World, c, chat_work,
    providers, step, text, world_in_layout,
};
use temper_client_domain::{
    Action,
    pages::{Act, choice},
};

pub(super) fn is_state(name: &str) -> bool {
    name.starts_with("chat-model-") || name.starts_with("chat-started-model-") || name == "ask-model-open"
}
pub(super) fn world(name: &str) -> Domain {
    let address = if name.starts_with("chat-started-") || name == "chat-model-fixed" {
        Address::Chat { task: 1 }
    } else if name == "ask-model-open" {
        Address::Ask { project: PROJECT }
    } else {
        Address::NewChat { draft: false }
    };
    let (mut root, _) = world_in_layout(World::Bare, address, Then::Shown, false, Some(name), None, false);
    let mut out = Queue::uncapped();
    if matches!(name, "chat-model-all" | "chat-model-new") {
        step(&mut root, Event::Act { action: Action::Pages(Act::Choice(choice::Act::All)) }, &mut out);
    }
    if matches!(name, "chat-started-model-changing" | "chat-started-model-refused") {
        step(
            &mut root,
            Event::Act { action: Action::Pages(Act::Choice(choice::Act::Model { index: 3, account: None })) },
            &mut out,
        );
        step(&mut root, Event::Client { event: jig_client::Event::Stored }, &mut out);
        if name == "chat-started-model-refused" {
            let attempt = out
                .iter()
                .find_map(|r| match r {
                    Request::Unlinked {
                        request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
                    } => Some(*attempt),
                    _ => None,
                })
                .expect("model amendment sent");
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
    root
}
fn both(name: &str) -> bool {
    matches!(
        name,
        "chat-model-both"
            | "chat-model-all"
            | "chat-model-new"
            | "chat-started-model-changing"
            | "chat-started-model-refused"
    )
}
fn chosen() -> c::LlmChoice {
    c::LlmChoice {
        later: None,
        account: None,
        main: c::ModelEffort { model: text("claude-opus-5-5"), effort: Some(text("high")) },
        subagents: List::with_capacity(0),
    }
}
pub(super) fn frame(me: &mut c::Me, name: &str) {
    me.providers = providers(World::Reference);
    let member = me.projects.get_mut(0).expect("picker project");
    for template in member.templates.iter_mut() {
        template.llm = None;
    }
    member.defaults = List::from_boxed(Box::new([
        c::MeDefault { template: 1, choice: chosen() },
        c::MeDefault { template: 5, choice: chosen() },
    ]));
    let sample = super::frame(World::Solo, false, false)
        .projects
        .get(0)
        .expect("project")
        .models
        .get(0)
        .expect("model sample")
        .clone();
    member.models = List::with_capacity(8);
    for (model, shown, provider, picker) in [
        ("claude-opus-5-5", "Opus 5.5", 17, true),
        ("claude-sonnet-5-5", "Sonnet 5.5", 17, true),
        ("claude-fable-5-1", "Fable 5.1", 17, true),
        ("gpt-6.1-sol", "GPT-6.1 Sol", 18, true),
        ("gpt-6-astra", "GPT-6 Astra", 18, true),
        ("gpt-6.1-terra", "GPT-6.1 Terra", 18, true),
        ("gpt-6-luna", "GPT-6 Luna", 18, true),
        ("claude-haiku-5-5", "Haiku 5.5", 17, false),
    ] {
        if provider == 18 && !both(name) {
            continue;
        }
        if !picker && !matches!(name, "chat-model-both" | "chat-model-all" | "chat-model-new") {
            continue;
        }
        let mut row = sample.clone();
        row.name = text(model);
        row.shown = Some(text(shown));
        row.provider = Some(provider);
        row.picker = picker;
        if name == "chat-model-new" && !picker {
            row.source = c::ModelSource::List;
            row.new_until = Some(skein_lib::Wall::from_nanos(604_800_000_000_000));
        }
        row.accounts = if provider == 17 && name == "chat-model-two-accounts" {
            List::from_boxed(Box::new([1, 3]))
        } else {
            List::from_boxed(Box::new([if provider == 17 { 1 } else { 2 }]))
        };
        row.efforts =
            List::from_boxed(Box::new([text("low"), text("medium"), text("high"), text("xhigh"), text("max")]));
        row.effort = Some(text("high"));
        member.models.push(row).expect("picker catalogue");
    }
}
pub(super) fn accounts(doc: &mut c::Accounts, name: &str) {
    *doc = super::accounts(World::Reference, Then::Shown);
    if !both(name) && name != "chat-model-sign-in-again" {
        doc.accounts.swap_remove(1).expect("OpenAI account");
    }
    if name == "chat-model-sign-in-again" {
        let openai = doc.accounts.get_mut(1).expect("OpenAI account");
        for source in openai.sources.iter_mut() {
            source.usable = false;
            source.state = c::AccountState::SignInAgain;
        }
    }
    if name == "chat-model-two-accounts" {
        let mut other = doc.accounts.get(0).expect("Claude account").clone();
        other.number = 3;
        other.display = super::doc("Work");
        doc.accounts.push(other).expect("second Claude account");
    }
}
pub(super) fn task(task: &mut c::Task, name: &str) {
    *task = chat_work::task("chat-work-running");
    task.llm = Some(chosen());
    task.label = CHAT_LABEL;
    task.flags = CHAT_FLAGS;
    task.result = None;
    task.acts = List::with_capacity(2);
    task.acts
        .push(c::Offered { act: c::ActKind::Message, admission: c::Admission::Allowed, effect: None })
        .expect("message");
    if name != "chat-model-fixed" {
        task.acts
            .push(c::Offered { act: c::ActKind::Amend, admission: c::Admission::Allowed, effect: None })
            .expect("amend");
    }
}
