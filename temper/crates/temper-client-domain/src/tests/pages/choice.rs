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
                prices: c::ModelPrices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1000 },
                description: None,
                effort: None,
                shown: None,
                accounts: List::with_capacity(0),
                picker: true,
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
    draft.edit(&me(), Act::Model { index: 1 });
    original.main.model = fixtures::acts::text(b"second");
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
    draft.edit(&me, Act::Model { index: 0 });
    (me, draft)
}
fn amended(mut draft: Draft, task: &c::Task) -> Queue<crate::pages::Request> {
    draft.edit(&me(), Act::Model { index: 1 });
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
        draft.edit(&me, Act::Model { index: 0 });
        assert!(draft.valid(&me));
        let asks = start(&draft, &me);
        let [c::Ask::Create { tasks, .. }] = asks.as_ref() else { panic!("create") };
        assert_eq!(tasks.get(0).unwrap().llm.as_ref().unwrap().main.effort, Some(fixtures::acts::text(b"high")));
    }
}

#[test]
fn a_project_change_discards_the_previous_choice() {
    let mut draft = draft();
    draft.edit(&me(), Act::Model { index: 1 });
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
    draft.edit(&me(), Act::Model { index: 1 });
    let original = choice(b"first");
    draft.reset(draft.context, Some(&original));
    assert_eq!(draft.requested, Some(original));
}
#[test]
fn an_amendment_names_the_shown_spec_revision() {
    let mut draft = draft();
    let task = fixtures::chat::task();
    let revision = c::Revision { value: task.spec_revision.value + 1 };
    draft.reset(Some(Context::Task { project: PROJECT, task: task.chip.task, revision, model_index: None }), None);
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
        Some(Context::Task { project: PROJECT, task: task.chip.task, revision: task.spec_revision, model_index: None }),
        None,
    );
    assert!(amended(draft, &task).is_empty());
}

#[test]
fn an_absent_amendment_offer_sends_nothing() {
    let mut draft = draft();
    let task = fixtures::chat::task();
    draft.reset(
        Some(Context::Task { project: PROJECT, task: task.chip.task, revision: task.spec_revision, model_index: None }),
        None,
    );
    assert!(amended(draft, &task).is_empty());
}

#[test]
fn a_suggested_request_carries_the_drafted_model() {
    let limits = fixtures::limits();
    let me = me();
    let mut draft = draft();
    draft.edit(&me, Act::Model { index: 1 });
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
    draft.edit(&me(), Act::Model { index: 1 });
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
                prices: c::ModelPrices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1000 },
                description: None,
                effort: efforts.get(0).cloned(),
                shown: None,
                accounts: List::with_capacity(0),
                picker: true,
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
    draft.edit(&me, Act::Model { index: 1 });
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
        Some(Context::Task { project: PROJECT, task: task.chip.task, revision: task.spec_revision, model_index: None }),
        Some(&choice(b"unoffered")),
    );
    assert!(draft.valid(&me()));
}
