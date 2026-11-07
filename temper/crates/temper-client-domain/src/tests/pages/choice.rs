use super::fixtures;
use crate::pages::{
    chats::Chats,
    choice::{Act, Context, Draft},
};
use jig_client::contract as c;
use skein_lib::{List, Queue};
const PROJECT: u32 = fixtures::PROJECT;
const SERIAL: u64 = 13;
const OUTPUTS: u32 = 16;
fn choice(model: &[u8]) -> c::LlmChoice {
    c::LlmChoice {
        later: None,
        account: Some(7),
        main: c::ModelEffort { model: fixtures::acts::text(model), effort: None },
        subagents: List::with_capacity(0),
    }
}
fn me() -> c::Me {
    let mut me = fixtures::chats::me();
    let mut models = List::with_capacity(2);
    for name in [b"first".as_slice(), b"second"] {
        models
            .push(c::MeModel {
                new_until: None,
                number: 1,
                source: c::ModelSource::Stock,
                estimated: false,
                effort: None,
                shown: None,
                accounts: List::with_capacity(0),
                picker: true,
                prices: c::ModelPrices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1000 },
                description: None,
                name: fixtures::acts::text(name),
                provider: Some(17),
                window: 1024,
                output: 128,
                efforts: List::with_capacity(0),
            })
            .unwrap();
    }
    me.projects.get_mut(0).unwrap().models = models;
    me
}
fn draft() -> Draft {
    let mut draft = Draft::default();
    draft.reset(
        Some(Context::Template {
            project: PROJECT,
            template: jig_client::shapes::CHAT,
            executor: c::TemplateExecutor::Charter(1),
            model_index: None,
        }),
        None,
    );
    draft
}
fn start(draft: &Draft, me: &c::Me) -> Box<[c::Ask]> {
    let limits = fixtures::limits();
    let mut chats = Chats::new(1);
    let mut out = Queue::with_capacity(OUTPUTS);
    let mut serial = SERIAL;
    chats.write(Box::from(*b"Hello"), &limits);
    chats.start(&fixtures::env(&limits), Some(me), Some(PROJECT), draft, false, &mut serial, &mut out);
    fixtures::acts::sent(&mut out)
}
#[test]
fn an_absent_template_choice_keeps_the_engine_default() {
    let asks = start(&draft(), &me());
    let [c::Ask::Create { tasks, .. }] = asks.as_ref() else { panic!("create") };
    assert_eq!(tasks.get(0).unwrap().llm, None);
}
#[test]
fn a_main_model_edit_preserves_the_remaining_choice() {
    let mut draft = draft();
    let mut original = choice(b"first");
    original.subagents = List::with_capacity(1);
    original.subagents.push(original.main.clone()).unwrap();
    draft.reset(draft.context, Some(&original));
    draft.edit(&me(), Act::Model { index: 1, account: None });
    original.main.model = fixtures::acts::text(b"second");
    original.account = None;
    let asks = start(&draft, &me());
    let [c::Ask::Create { tasks, .. }] = asks.as_ref() else { panic!("create") };
    assert_eq!(tasks.get(0).unwrap().llm, Some(original));
}
fn me_requiring_effort() -> c::Me {
    let mut me = me();
    let model = me.projects.get_mut(0).unwrap().models.get_mut(0).unwrap();
    model.efforts = List::with_capacity(2);
    model.efforts.push(fixtures::acts::text(b"low")).unwrap();
    model.efforts.push(fixtures::acts::text(b"high")).unwrap();
    model.effort = Some(fixtures::acts::text(b"high"));
    me
}
fn effort_draft() -> (c::Me, Draft) {
    let me = me_requiring_effort();
    let mut draft = draft();
    draft.edit(&me, Act::Model { index: 0, account: None });
    (me, draft)
}
fn amended(mut draft: Draft, task: &c::Task) -> Queue<crate::pages::Request> {
    draft.edit(&me(), Act::Model { index: 1, account: None });
    let mut out = Queue::with_capacity(OUTPUTS);
    let mut serial = SERIAL;
    draft.amend(&me(), task, &mut serial, &mut out);
    out
}
#[test]
fn a_missing_required_effort_prevents_start() {
    let (me, mut draft) = effort_draft();
    draft.requested.as_mut().unwrap().main.effort = None;
    assert!(start(&draft, &me).is_empty());
}
#[test]
fn a_model_press_sets_its_effort_and_starts_at_once() {
    let me = me_requiring_effort();
    for original in [None, Some(choice(b"second"))] {
        let mut draft = draft();
        draft.reset(draft.context, original.as_ref());
        draft.edit(&me, Act::Model { index: 0, account: None });
        assert!(draft.valid(&me));
        let asks = start(&draft, &me);
        let [c::Ask::Create { tasks, .. }] = asks.as_ref() else { panic!("create") };
        assert_eq!(tasks.get(0).unwrap().llm.as_ref().unwrap().main.effort, Some(fixtures::acts::text(b"high")));
    }
}

#[test]
fn a_project_change_discards_the_previous_choice() {
    let mut draft = draft();
    draft.edit(&me(), Act::Model { index: 1, account: None });
    draft.reset(
        Some(Context::Template {
            project: PROJECT + 1,
            template: jig_client::shapes::CHAT,
            executor: c::TemplateExecutor::Charter(1),
            model_index: None,
        }),
        None,
    );
    assert_eq!(draft.requested, None);
}
#[test]
fn a_template_change_replaces_its_previous_default() {
    let mut draft = draft();
    draft.edit(&me(), Act::Model { index: 1, account: None });
    let original = choice(b"first");
    draft.reset(draft.context, Some(&original));
    assert_eq!(draft.requested, Some(original));
}
#[test]
fn an_amendment_names_the_shown_spec_revision() {
    let mut draft = draft();
    let task = fixtures::chat::task();
    let revision = c::Revision { value: task.spec_revision.value + 1 };
    draft.reset(
        Some(Context::Task {
            project: PROJECT,
            task: task.chip.task,
            template: jig_client::shapes::CHAT,
            revision,
            model_index: None,
        }),
        None,
    );
    let mut task = task;
    task.acts = fixtures::acts::offered(c::ActKind::Amend, c::Admission::Allowed);
    let mut out = amended(draft, &task);
    let asks = fixtures::acts::sent(&mut out);
    let [c::Ask::Amend { revision: shown, .. }] = asks.as_ref() else { panic!("amend") };
    assert_eq!(*shown, Some(revision));
}
#[test]
fn a_refused_amendment_sends_nothing() {
    let mut draft = draft();
    let mut task = fixtures::chat::task();
    task.acts = fixtures::acts::offered(c::ActKind::Amend, c::Admission::Refused(c::Refusal::NotOffered));
    draft.reset(
        Some(Context::Task {
            project: PROJECT,
            task: task.chip.task,
            template: jig_client::shapes::CHAT,
            revision: task.spec_revision,
            model_index: None,
        }),
        None,
    );
    assert!(amended(draft, &task).is_empty());
}

#[test]
fn an_absent_amendment_offer_sends_nothing() {
    let mut draft = draft();
    let task = fixtures::chat::task();
    draft.reset(
        Some(Context::Task {
            project: PROJECT,
            task: task.chip.task,
            template: jig_client::shapes::CHAT,
            revision: task.spec_revision,
            model_index: None,
        }),
        None,
    );
    assert!(amended(draft, &task).is_empty());
}

#[test]
fn a_suggested_request_carries_the_drafted_model() {
    let limits = fixtures::limits();
    let me = me();
    let mut draft = draft();
    draft.edit(&me, Act::Model { index: 1, account: None });
    let mut chats = Chats::new(1);
    chats.write(Box::from(*b"Not the suggested request"), &limits);
    let mut out = Queue::with_capacity(OUTPUTS);
    let mut serial = SERIAL;
    chats.suggest(&fixtures::env(&limits), Some(&me), Some(PROJECT), b"Suggested", &draft, &mut serial, &mut out);
    let asks = fixtures::acts::sent(&mut out);
    let [c::Ask::Create { tasks, .. }] = asks.as_ref() else { panic!("create") };
    let mut expected = jig_client::shapes::chat(&me, PROJECT, fixtures::acts::text(b"Suggested")).unwrap();
    let c::Ask::Create { tasks: wanted, .. } = &mut expected else { panic!("create") };
    wanted.get_mut(0).unwrap().llm = draft.requested;
    assert_eq!(tasks, &*wanted);
}

#[test]
fn a_model_edit_from_engine_default_keeps_account_selection_in_the_engine() {
    let mut draft = draft();
    draft.edit(&me(), Act::Model { index: 1, account: None });
    let asks = start(&draft, &me());
    let [c::Ask::Create { tasks, .. }] = asks.as_ref() else { panic!("create") };
    assert_eq!(tasks.get(0).unwrap().llm.as_ref().unwrap().account, None);
}

#[test]
fn the_selected_duplicate_models_effort_reaches_the_request() {
    let mut me = me();
    let mut models = List::with_capacity(2);
    for (provider, effort) in [(17, b"first-effort".as_slice()), (18, b"second-effort".as_slice())] {
        let mut efforts = List::with_capacity(1);
        efforts.push(fixtures::acts::text(effort)).unwrap();
        models
            .push(c::MeModel {
                new_until: None,
                number: 1,
                source: c::ModelSource::Stock,
                estimated: false,
                effort: efforts.get(0).cloned(),
                shown: None,
                accounts: List::with_capacity(0),
                picker: true,
                prices: c::ModelPrices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1000 },
                description: None,
                name: fixtures::acts::text(b"shared"),
                provider: Some(provider),
                window: 1024,
                output: 128,
                efforts,
            })
            .unwrap();
    }
    me.projects.get_mut(0).unwrap().models = models;
    let mut draft = draft();
    draft.edit(&me, Act::Model { index: 1, account: None });
    draft.edit(&me, Act::Effort { index: 0 });
    let asks = start(&draft, &me);
    let [c::Ask::Create { tasks, .. }] = asks.as_ref() else { panic!("create") };
    assert_eq!(tasks.get(0).unwrap().llm.as_ref().unwrap().main.effort, Some(fixtures::acts::text(b"second-effort")));
}

#[test]
fn a_templates_unoffered_choice_still_starts() {
    let mut draft = draft();
    let original = choice(b"unoffered");
    let mut me = me();
    me.projects.get_mut(0).unwrap().templates.get_mut(0).unwrap().llm = Some(original.clone());
    draft.reset(draft.context, Some(&original));
    let asks = start(&draft, &me);
    let [c::Ask::Create { tasks, .. }] = asks.as_ref() else { panic!("create") };
    assert_eq!(tasks.get(0).unwrap().llm, Some(original));
}

#[test]
fn a_tasks_unoffered_choice_stays_valid() {
    let mut draft = draft();
    let task = fixtures::chat::task();
    draft.reset(
        Some(Context::Task {
            project: PROJECT,
            task: task.chip.task,
            template: jig_client::shapes::CHAT,
            revision: task.spec_revision,
            model_index: None,
        }),
        Some(&choice(b"unoffered")),
    );
    assert!(draft.valid(&me()));
}

fn preview(me: &mut c::Me, model: &[u8], effort: Option<&[u8]>) {
    let mut chosen = choice(model);
    chosen.account = None;
    chosen.main.effort = match effort {
        Some(effort) => Some(fixtures::acts::text(effort)),
        None => None,
    };
    me.projects.get_mut(0).unwrap().defaults =
        List::from_boxed(Box::new([c::MeDefault { template: jig_client::shapes::CHAT, choice: chosen }]));
}
fn task_page() -> (crate::pages::Pages, jig_client::Client, c::Task) {
    let mut f = fixtures::chat::opened();
    let mut task = fixtures::chat::task();
    task.llm = Some(choice(b"first"));
    task.acts = fixtures::acts::offered(c::ActKind::Amend, c::Admission::Allowed);
    fixtures::chat::deliver(&mut f.client, f.watch, task.clone());
    f.client.step_unlinked(
        skein_lib::Time::ZERO,
        skein_lib::Wall::EPOCH,
        jig_client::Event::Watch {
            handle: crate::pages::me_handle(),
            subject: jig_client::Watchable::Jig(c::Subject::Me),
        },
        &mut Queue::uncapped(),
    );
    f.client.step_unlinked(
        skein_lib::Time::ZERO,
        skein_lib::Wall::EPOCH,
        jig_client::Event::Up {
            up: jig_client::Up::Watched {
                watch: f.me,
                event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me()))),
            },
        },
        &mut Queue::uncapped(),
    );
    let mut pages = crate::pages::Pages::new();
    crate::pages::step(
        &mut pages,
        &fixtures::env(&fixtures::limits()),
        &f.client,
        crate::pages::Event::Went { address: crate::Address::Chat { task: task.chip.task } },
        &mut Queue::uncapped(),
    );
    f.client.step_unlinked(
        skein_lib::Time::ZERO,
        skein_lib::Wall::EPOCH,
        jig_client::Event::Watch {
            handle: pages.chat().unwrap().document,
            subject: jig_client::Watchable::Jig(c::Subject::Task { task: task.chip.task }),
        },
        &mut Queue::uncapped(),
    );
    crate::pages::choice::refresh(&mut pages, &f.client);
    (pages, f.client, task)
}
fn press(
    pages: &mut crate::pages::Pages,
    client: &jig_client::Client,
    index: u32,
    out: &mut Queue<crate::pages::Request>,
) {
    crate::pages::choice::act(pages, client, Act::Model { index, account: None }, out);
}
#[test]
fn one_press_on_a_task_sends_one_amendment() {
    let (mut pages, client, task) = task_page();
    let mut out = Queue::uncapped();
    press(&mut pages, &client, 1, &mut out);
    let asks = fixtures::acts::sent(&mut out);
    let [c::Ask::Amend { task: number, change, .. }] = asks.as_ref() else { panic!("one amendment: {asks:?}") };
    assert_eq!(*number, task.chip.task);
    assert_eq!(change.llm.as_ref().unwrap().main.model.bytes(), b"second");
}
#[test]
fn the_same_press_sends_nothing() {
    let (mut pages, client, _) = task_page();
    // The task already has this model with no preferred account.
    pages.choice.original.as_mut().unwrap().account = None;
    pages.choice.requested = pages.choice.original.clone();
    let mut out = Queue::uncapped();
    press(&mut pages, &client, 0, &mut out);
    assert!(out.is_empty());
}
#[test]
fn the_default_row_goes_back_to_no_choice() {
    let mut me = me();
    preview(&mut me, b"first", None);
    let mut draft = draft();
    draft.edit(&me, Act::Model { index: 1, account: None });
    draft.edit(&me, Act::Model { index: 0, account: None });
    assert_eq!(draft.requested, None);
    assert_eq!(selected_index(&draft), None);
}
#[test]
fn effort_with_no_request_starts_from_the_default() {
    let mut me = me_requiring_effort();
    preview(&mut me, b"first", Some(b"high"));
    let mut draft = draft();
    draft.edit(&me, Act::Effort { index: 0 });
    assert_eq!(draft.requested.unwrap().main.effort.unwrap().bytes(), b"low");
}
fn answer_choice(answer: c::Answer) -> Draft {
    let (mut pages, client, _) = task_page();
    press(&mut pages, &client, 1, &mut Queue::uncapped());
    let key = pages.choice.pending.unwrap();
    pages.choice.told(&jig_client::Told::Answered {
        key,
        outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(answer)),
    });
    pages.choice
}
#[test]
fn a_refusal_returns_the_chats_own_choice() {
    let draft = answer_choice(c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None });
    assert_eq!(draft.requested, draft.original);
    assert_eq!(selected_index(&draft), None);
    assert!(draft.outcome.is_some());
}
#[test]
fn a_proposal_returns_the_chats_own_choice() {
    let draft =
        answer_choice(c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Proposed { proposal: 1, to: 1 })));
    assert_eq!(draft.requested, draft.original);
    assert_eq!(selected_index(&draft), None);
}
#[test]
fn a_second_press_while_changing_does_nothing() {
    let (mut pages, client, _) = task_page();
    press(&mut pages, &client, 1, &mut Queue::uncapped());
    let mut out = Queue::uncapped();
    press(&mut pages, &client, 0, &mut out);
    assert!(out.is_empty());
    assert_eq!(pages.choice.requested.as_ref().unwrap().main.model.bytes(), b"second");
}
#[test]
fn the_menu_holds_across_a_revision() {
    let (mut pages, _, mut task) = task_page();
    pages.choice.edit(&me(), Act::All);
    pages.choice.edit(&me(), Act::Find { words: Box::from(b"FIRST".as_slice()) });
    task.spec_revision.value += 1;
    pages.choice.reset(
        Some(Context::Task {
            project: PROJECT,
            task: task.chip.task,
            template: jig_client::shapes::CHAT,
            revision: task.spec_revision,
            model_index: None,
        }),
        task.llm.as_ref(),
    );
    assert!(pages.choice.all);
    assert_eq!(pages.choice.find.as_ref(), b"FIRST");
}

fn selected_index(draft: &Draft) -> Option<u8> {
    match draft.context {
        Some(Context::Template { model_index, .. } | Context::Task { model_index, .. }) => model_index,
        None => None,
    }
}
#[test]
fn find_admits_its_bound_and_refuses_one_more_byte() {
    let me = me();
    for (size, expected) in [(crate::pages::choice::FIND, true), (crate::pages::choice::FIND + 1, false)] {
        let mut draft = draft();
        let words = skein_lib::bytes::zeroed(usize::try_from(size).unwrap());
        draft.edit(&me, Act::Find { words: words.clone() });
        assert_eq!(draft.find == words, expected);
    }
}
