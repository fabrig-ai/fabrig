//! Chat template selection (temper/docs/design/client/tools.md, 5.3 and 6.3).
use crate::tests::fixtures as f;
use jig_client::{contract as c, shapes};
use skein_lib::List;

const TEMPLATE_CHAT: &[&str] =
    &["chat", "start", "--project", "shop", "--template", "3", "hello", "--key", "template-chat"];

fn template_me() -> c::Me {
    let mut me = f::choice_me(f::CHAT_BUDGET);
    let project = me.projects.get_mut(0).expect("project");
    let chat = project.templates.get(0).expect("chat template").clone();
    let mut goal = chat.clone();
    goal.number = 2;
    goal.name = f::document_text(b"Goal");
    goal.flags.conversation = false;
    let mut configuring = chat.clone();
    configuring.number = 3;
    configuring.name = f::document_text(b"Configuring");
    configuring.executor = c::TemplateExecutor::Charter(3);
    let mut templates = List::with_capacity(3);
    templates.push(chat).expect("chat");
    templates.push(goal).expect("goal");
    templates.push(configuring).expect("configuring chat");
    project.templates = templates;
    let mut charters = List::with_capacity(2);
    charters.push(1).expect("chat charter");
    charters.push(3).expect("configuring charter");
    project.creating.as_mut().expect("creating offer").within.charters = charters;
    me
}

#[test]
fn chat_start_names_a_chat_template() {
    let me = template_me();
    let expected = shapes::chat_from(&me, f::PROJECT, 3, f::call_words(TEMPLATE_CHAT, 2)).expect("named shape");
    let mut root = f::choice_root(me);
    let call = root.submit(TEMPLATE_CHAT);
    assert_eq!(f::act_ask(&root, call), &expected);
    assert_eq!(f::chat_task(&root, call).executor, c::Executor::Agent { charter: 3 });
}

#[test]
fn chat_start_without_a_template_takes_the_chat_template() {
    let me = template_me();
    let expected = shapes::chat(&me, f::PROJECT, f::call_words(f::CREATE_CHAT_CALL, 2)).expect("default shape");
    let mut root = f::choice_root(me);
    let call = root.submit(f::CREATE_CHAT_CALL);
    assert_eq!(f::act_ask(&root, call), &expected);
}

fn refused(me: c::Me, template: &str, expected: &[u8]) {
    let mut root = f::choice_root(me);
    let call =
        root.submit(&["chat", "start", "--project", "shop", "--template", template, "hello", "--key", "refused"]);
    assert!(root.sends.is_empty());
    let answer = root.answer(call).expect("template refusal");
    assert_eq!((answer.status, answer.stderr.as_ref()), (1, expected));
}

#[test]
fn chat_start_refuses_a_goal_template() {
    refused(
        template_me(),
        "2",
        b"template 2 is no chat template of this project; its chat templates: 1 chat, 3 Configuring\n",
    );
}

#[test]
fn chat_start_refuses_a_template_not_offered() {
    refused(
        template_me(),
        "9",
        b"template 9 is no chat template of this project; its chat templates: 1 chat, 3 Configuring\n",
    );
}

#[test]
fn chat_start_names_a_template_and_a_model() {
    let me = template_me();
    let mut expected = shapes::chat_from(&me, f::PROJECT, 3, f::call_words(TEMPLATE_CHAT, 2)).expect("named shape");
    let c::Ask::Create { tasks, .. } = &mut expected else { panic!("chat creates") };
    tasks.get_mut(0).expect("chat").llm.as_mut().expect("template choice").main =
        f::model_effort(f::MODEL_NAME, Some(f::EFFORT_NAME));
    let mut root = f::choice_root(me);
    let call = root.submit(&[
        "chat",
        "start",
        "--project",
        "shop",
        "--template",
        "3",
        "hello",
        "--model",
        "chosen-model",
        "--effort",
        "high",
        "--key",
        "template-model",
    ]);
    assert_eq!(f::act_ask(&root, call), &expected);
}

#[test]
fn chat_start_lists_no_templates_when_none_are_chats() {
    let mut me = template_me();
    let project = me.projects.get_mut(0).expect("project");
    for index in 0..project.templates.len() {
        project.templates.get_mut(index).expect("template").flags.conversation = false;
    }
    refused(me, "9", b"template 9 is no chat template of this project; its chat templates: none\n");
}

#[test]
fn chat_start_lists_chat_templates_in_me_order() {
    let mut me = template_me();
    let project = me.projects.get_mut(0).expect("project");
    let mut templates = List::with_capacity(2);
    templates.push(project.templates.get(2).expect("configuring").clone()).expect("first");
    templates.push(project.templates.get(0).expect("chat").clone()).expect("second");
    project.templates = templates;
    refused(me, "9", b"template 9 is no chat template of this project; its chat templates: 3 Configuring, 1 chat\n");
}

#[test]
fn chat_start_refuses_a_template_number_past_u32() {
    refused(
        template_me(),
        "4294967296",
        b"template 4294967296 is no chat template of this project; its chat templates: 1 chat, 3 Configuring\n",
    );
}

#[test]
fn chat_start_keeps_other_not_offered_failures() {
    let mut me = template_me();
    me.projects.get_mut(0).expect("project").creating = None;
    let mut root = f::choice_root(me);
    let call = root.submit(TEMPLATE_CHAT);
    assert!(root.sends.is_empty());
    let answer = root.answer(call).expect("not offered");
    assert_eq!((answer.status, answer.stderr.as_ref()), (1, b"chat not offered\n".as_slice()));
}
