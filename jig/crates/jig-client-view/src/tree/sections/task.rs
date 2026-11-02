//! The universal task page and executor sections (client/domain/pages.md, 5).
use crate::words::Words;
use crate::words::phrases::jig::JigPhrase;
use crate::{JigBindings, tree::cards};
use alloc::boxed::Box;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role};

use crate::{
    Borrowed,
    state::{
        action::Act,
        address::{Address, TaskSection},
        objects::Body,
    },
};
use jig_client::contract::{ChipExecutor, ObjectKey, Task};
pub(crate) mod history;
pub(crate) mod plan;
/// The task's shared card and context, while any of its sections shows.
pub fn header(b: &mut Builder, client: &Borrowed<'_>, words: &Words, bindings: &mut JigBindings) -> bool {
    let Some(number) = shown(client) else {
        return false;
    };
    b.open(Role::Region);
    b.region(crate::regions::JigRegion::TaskHeader.region());
    b.name(&words.say(JigPhrase::TaskPage, &[Hole::Number(number)]));
    let Some(id) = client.task_page().header else {
        words.status(b, JigPhrase::LoadingObject, &[]);
        b.close();
        return true;
    };
    cards::card(b, client, id, &cards::Place::TaskPage, words, bindings);
    let Some(doc) = document(client, number) else {
        b.close();
        return true;
    };
    if doc.flags.conversation
        && let Some(me) = client.frame().document
    {
        match &doc.waits_on {
            jig_client::contract::WaitsOn::Dependencies { .. } => {
                words.text(b, JigPhrase::Word(jig_client::words::Word::waiting(&doc.waits_on)), &[]);
            }
            jig_client::contract::WaitsOn::Party { .. }
            | jig_client::contract::WaitsOn::Delegates
            | jig_client::contract::WaitsOn::News
            | jig_client::contract::WaitsOn::Nothing => {
                words.chat_status(
                    b,
                    &jig_client::facts::chat_status(&doc.chip, &doc.waits_on, doc.run_wait.as_ref(), doc.project, me),
                    client.wall(),
                );
            }
        }
    }
    words.text(b, JigPhrase::ContextProject, &[Hole::Number(u64::from(doc.project))]);
    if let Some(goal) = doc.goal {
        if goal == doc.chip.task {
            words.text(b, JigPhrase::Goal, &[]);
        } else {
            crate::tree::frame::control(
                b,
                words,
                bindings,
                Role::Link,
                JigPhrase::ServedGoal,
                &[Hole::Number(goal)],
                Act::Go { address: Address::task(goal) },
            );
        }
    }
    if let Some(priority) = doc.priority {
        words.text(b, JigPhrase::Priority, &[Hole::Number(u64::from(priority))]);
    }
    for chip in &doc.lineage_chips {
        cards::chip::chip(b, client, chip, words, bindings);
    }
    b.close();
    true
}
fn shown(client: &Borrowed<'_>) -> Option<u64> {
    for section in client.sections() {
        if client.shows(*section) {
            match section {
                crate::state::page::Section::Task { task, .. } => return Some(*task),
                crate::state::page::Section::Inbox { .. }
                | crate::state::page::Section::Conversations { .. }
                | crate::state::page::Section::Project { .. }
                | crate::state::page::Section::Deployment => {}
            }
        }
    }
    None
}
fn document(client: &Borrowed<'_>, number: u64) -> Option<Box<Task>> {
    let objects = client.objects();
    let id = objects.find(ObjectKey::Task(number))?;
    match &objects.get(id)?.body {
        Some(Body::Task(doc)) => Some(doc.clone()),
        Some(
            Body::Ended(_)
            | Body::Chip(_)
            | Body::Proposal(_)
            | Body::Escalation(_)
            | Body::Question(_)
            | Body::PersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => None,
    }
}
/// Every offered section, linked to its address with the current address marked.
pub fn tabs(b: &mut Builder, client: &Borrowed<'_>, words: &Words, bindings: &mut JigBindings) -> bool {
    let Some(number) = shown(client) else {
        return false;
    };
    let Some(doc) = document(client, number) else {
        return false;
    };
    let sections = crate::state::page::task::offers(&doc);
    b.open(Role::Tabs);
    b.region(crate::regions::JigRegion::TaskTabs.region());
    b.name(&words.say(JigPhrase::TaskPage, &[Hole::Number(number)]));
    for offered in &sections.offered {
        let address = Address::Task { task: number, section: *offered };
        crate::tree::pages::work::tab(
            b,
            words,
            bindings,
            section_phrase(*offered),
            Act::Go { address },
            match client.page() {
                crate::state::page::Page::Work { address: own, .. } => own == &address,
                crate::state::page::Page::Starting
                | crate::state::page::Page::SigningIn
                | crate::state::page::Page::Elsewhere
                | crate::state::page::Page::Missing
                | crate::state::page::Page::OutOfDate
                | crate::state::page::Page::SomeoneElse => false,
            },
        );
    }
    b.close();
    true
}
pub(crate) fn all_shown(client: &Borrowed<'_>) -> bool {
    let Some(number) = shown(client) else {
        return false;
    };
    let Some(doc) = document(client, number) else {
        return false;
    };
    for section in &crate::state::page::task::offers(&doc).offered {
        if !client.shows(crate::state::page::Section::Task { task: number, section: *section }) {
            return false;
        }
    }
    true
}
pub(crate) fn section(
    b: &mut Builder,
    client: &Borrowed<'_>,
    words: &Words,
    bindings: &mut JigBindings,
    task: u64,
    section: TaskSection,
) -> bool {
    if !client.shows(crate::state::page::Section::Task { task, section }) {
        return false;
    }
    crate::tree::pages::work::section(b, words, section_phrase(section), region(section));
    crate::tree::pages::mark_current(b, client, Address::Task { task, section });
    super::status(b, client, words, crate::state::page::Section::Task { task, section });
    match section {
        TaskSection::Plan => plan::plan(b, client, words, bindings),
        TaskSection::History => history::history(b, client, words, bindings),
        TaskSection::Overview | TaskSection::Activity | TaskSection::Budget | TaskSection::Panels => {
            if let Some(doc) = document(client, task) {
                match section {
                    TaskSection::Overview => overview(b, client, &doc, words, bindings),
                    TaskSection::Activity => activity(b, client, &doc, words, bindings),
                    TaskSection::Budget => budget(b, &doc, words, bindings, client.client.domain_limits().rows),
                    TaskSection::Panels => slots(b, &doc, section, words, bindings),
                    TaskSection::Plan | TaskSection::History => unreachable!("independent section"),
                }
            }
        }
    }
    b.close();
    true
}
const fn region(section: TaskSection) -> crate::regions::JigRegion {
    use crate::regions::JigRegion;
    match section {
        TaskSection::Overview => JigRegion::TaskOverview,
        TaskSection::Activity => JigRegion::TaskActivity,
        TaskSection::Plan => JigRegion::TaskPlan,
        TaskSection::History => JigRegion::TaskHistory,
        TaskSection::Budget => JigRegion::TaskBudget,
        TaskSection::Panels => JigRegion::TaskPanels,
    }
}
pub(super) const fn section_phrase(section: TaskSection) -> JigPhrase {
    match section {
        TaskSection::Overview => JigPhrase::Overview,
        TaskSection::Activity => JigPhrase::Activity,
        TaskSection::Plan => JigPhrase::Plan,
        TaskSection::History => JigPhrase::History,
        TaskSection::Budget => JigPhrase::BudgetAuthority,
        TaskSection::Panels => JigPhrase::Panels,
    }
}
fn slots(b: &mut Builder, doc: &Task, section: TaskSection, words: &Words, bindings: &mut JigBindings) {
    let context_parts = match &doc.context {
        Some(context) => context.parts.len(),
        None => 0,
    };
    let count = doc
        .resources
        .len()
        .checked_add(context_parts)
        .expect("bounded task parts")
        .checked_add(doc.authority.authority.grants.len().checked_mul(2).expect("bounded grants"))
        .expect("bounded task parts")
        .checked_add(1)
        .expect("result slot fits");
    for part in &crate::state::page::task::parts(doc, section, count) {
        cards::common::part(b, part, words, bindings);
    }
    match section {
        TaskSection::Overview | TaskSection::Panels => {
            for resource in &doc.resources {
                crate::tree::fragments::chip::resource(b, resource, words, bindings);
            }
        }
        TaskSection::Activity | TaskSection::Plan | TaskSection::History | TaskSection::Budget => {}
    }
}
pub(crate) fn overview(b: &mut Builder, client: &Borrowed<'_>, doc: &Task, words: &Words, bindings: &mut JigBindings) {
    let objects = client.objects();
    {
        let jig_client::contract::ChipPhaseFacts { details, .. } = doc.chip.details.as_ref();
        crate::tree::fragments::chip::render(b, details, client, words, bindings);
    }
    crate::tree::fragments::specification::document(b, &doc.spec, words);
    crate::tree::fragments::instructions::document(b, doc.instructions.as_ref(), words);
    words.text(b, JigPhrase::ResultExpected, &[]);
    crate::tree::fragments::proposed_task::contract(b, &doc.result_contract, words, bindings);
    crate::tree::fragments::result::requester(b, doc.requester, words);
    if doc.cut {
        words.text(b, JigPhrase::TaskListsCut, &[]);
    }
    crate::tree::fragments::wake::policy(b, &doc.wake, words.book(), words.language());
    crate::tree::fragments::wake::subscriptions(b, &doc.subscriptions, words.book(), words.language());
    for dependency in &doc.dependency_facts {
        cards::chip::dependency_words(b, dependency, words);
    }
    for link in &doc.dependencies {
        task_link(b, client, link, words, bindings);
    }
    for link in &doc.references {
        task_link(b, client, link, words, bindings);
    }
    for link in &doc.inputs {
        task_link(b, client, link, words, bindings);
        if let Some(id) = objects.find(ObjectKey::Result(link.number()))
            && !client.owes(id)
        {
            cards::card(b, client, id, &cards::Place::TaskPage, words, bindings);
        }
    }
    if let Some(result) = &doc.result {
        crate::tree::fragments::task_result::render(b, client, result, words, bindings);
    }
    if let Some(context) = &doc.context {
        for slot in &context.writer_slots {
            cards::common::part(b, &slot.resource, words, bindings);
            crate::tree::frame::control(
                b,
                words,
                bindings,
                Role::Link,
                JigPhrase::ContextTask,
                &[Hole::Number(slot.task)],
                Act::Go { address: Address::task(slot.task) },
            );
            if let Some(attempt) = slot.attempt {
                words.text(b, JigPhrase::Attempt, &[Hole::Number(attempt)]);
            }
        }
    }
    slots(b, doc, TaskSection::Overview, words, bindings);
    match doc.chip.executor {
        ChipExecutor::Job { kind } => words.text(b, JigPhrase::Job, &[Hole::Number(u64::from(kind))]),
        ChipExecutor::Agent { charter } => words.text(b, JigPhrase::Charter, &[Hole::Number(u64::from(charter))]),
        ChipExecutor::Party { .. } | ChipExecutor::Role { .. } => {
            if let Some(id) = objects.find(ObjectKey::PersonTask(doc.chip.task))
                && !client.owes(id)
            {
                cards::card(b, client, id, &cards::Place::TaskPage, words, bindings);
            }
        }
        ChipExecutor::Procedure { .. } => {}
    }
}
fn task_link(
    b: &mut Builder,
    client: &Borrowed<'_>,
    link: &jig_client::contract::TaskLink,
    words: &Words,
    bindings: &mut JigBindings,
) {
    crate::tree::fragments::result::link(b, client, link, words, bindings);
    match link {
        jig_client::contract::TaskLink::Live(_) | jig_client::contract::TaskLink::Retained { .. } => {}
        jig_client::contract::TaskLink::Stored { number, .. } => crate::tree::frame::control(
            b,
            words,
            bindings,
            Role::Link,
            JigPhrase::ContextTask,
            &[Hole::Number(*number)],
            Act::Go { address: Address::task(*number) },
        ),
    }
}
fn activity(b: &mut Builder, client: &Borrowed<'_>, doc: &Task, words: &Words, bindings: &mut JigBindings) {
    use jig_client::contract::ChipPhaseFacts;
    let objects = client.objects();
    match doc.chip.executor {
        ChipExecutor::Agent { .. } => {
            {
                let ChipPhaseFacts { details, .. } = doc.chip.details.as_ref();

                if let Some(observation) = &details.observation {
                    crate::tree::fragments::chip::observation(b, observation, words);
                }
            }
            if let Some(context) = &doc.context {
                super::conversations::runtime(b, client, &context.runtime, words, bindings);
            }
            crate::tree::turns::conversation(b, client, words, bindings);
            for proposal in &client.task_page().proposals {
                let key = ObjectKey::Proposal(*proposal);
                if let Some(id) = objects.find(key)
                    && !client.conversation().shows(key)
                    && !client.owes(id)
                {
                    cards::card(b, client, id, &cards::Place::TaskPage, words, bindings);
                }
            }
        }
        ChipExecutor::Job { .. } | ChipExecutor::Procedure { .. } => {
            slots(b, doc, TaskSection::Activity, words, bindings);
        }
        ChipExecutor::Party { .. } | ChipExecutor::Role { .. } => {
            if let Some(id) = objects.find(ObjectKey::PersonTask(doc.chip.task))
                && !client.owes(id)
            {
                cards::card(b, client, id, &cards::Place::TaskPage, words, bindings);
            }
            if let Some(result) = &doc.result {
                crate::tree::fragments::task_result::render(b, client, result, words, bindings);
            }
        }
    }
}
pub(crate) fn budget(b: &mut Builder, doc: &Task, words: &Words, bindings: &mut JigBindings, rows: u32) {
    let text_limit = bindings.text_limit();
    crate::tree::fragments::budget::render(b, &doc.budget, words);
    if let Some(context) = &doc.context {
        for spend in &context.spend {
            match spend.attempt {
                Some(attempt) => words.text(
                    b,
                    JigPhrase::SpentRun,
                    &[Hole::Number(spend.task), Hole::Number(attempt), Hole::Amount(spend.spent)],
                ),
                None => words.text(b, JigPhrase::SpentDelegate, &[Hole::Number(spend.task), Hole::Amount(spend.spent)]),
            }
        }
    }
    match doc.chip.executor {
        ChipExecutor::Agent { .. } | ChipExecutor::Job { .. } | ChipExecutor::Procedure { .. } => {
            crate::tree::fragments::authority::document_words(b, &doc.authority, words, text_limit, rows);
        }
        ChipExecutor::Party { .. } | ChipExecutor::Role { .. } => {}
    }
}
