//! Root-numbered task notices
//! (jig/docs/design/domain/engine.md, 4.4).

use crate::{Core, Limits};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue};

pub(crate) fn person_stall_due(core: &mut Core, env: &Env<Limits>, task: u64, pending: &mut Queue<tasks::Request>) {
    let message = crate::fresh(&mut core.counters, crate::Family::Message).expect("person stall number admitted");
    let mut follow = Queue::uncapped();
    crate::routing::step::step_tasks(core, env, tasks::Event::PersonStallNotice { task, message }, &mut follow);
    for _ in 0..follow.len() {
        pending.push(follow.pop().expect("person stall output count"));
    }
}

pub(crate) fn question_owed_due(core: &mut Core, env: &Env<Limits>, task: u64, pending: &mut Queue<tasks::Request>) {
    let message = crate::fresh(&mut core.counters, crate::Family::Message).expect("question reminder number admitted");
    let mut follow = Queue::uncapped();
    crate::routing::step::step_tasks(core, env, tasks::Event::QuestionOwedNotice { task, message }, &mut follow);
    for _ in 0..follow.len() {
        pending.push(follow.pop().expect("question notice output count"));
    }
}

pub(crate) fn question_ended_due(
    core: &mut Core,
    env: &Env<Limits>,
    task: u64,
    questions: Box<[u64]>,
    pending: &mut Queue<tasks::Request>,
) {
    let mut notices = List::uncapped();
    for question in questions {
        let message =
            crate::fresh(&mut core.counters, crate::Family::Message).expect("question notice number admitted");
        notices.push(tasks::QuestionNotice { question, message }).expect("bounded question batch");
    }
    let mut follow = Queue::uncapped();
    crate::routing::step::step_tasks(
        core,
        env,
        tasks::Event::QuestionEndedNotice { task, notices: notices.into_boxed() },
        &mut follow,
    );
    for _ in 0..follow.len() {
        pending.push(follow.pop().expect("question notice output count"));
    }
}

pub(crate) fn met_due(
    core: &mut Core,
    env: &Env<Limits>,
    task: u64,
    dependency: u64,
    removed: bool,
    by: tasks::Party,
    pending: &mut Queue<tasks::Request>,
) {
    let message = crate::fresh(&mut core.counters, crate::Family::Message).expect("met notice number admitted");
    let mut follow = Queue::uncapped();
    crate::routing::step::step_tasks(
        core,
        env,
        tasks::Event::MetNotice { task, dependency, removed, by, message },
        &mut follow,
    );
    for _ in 0..follow.len() {
        pending.push(follow.pop().expect("met notice output count"));
    }
}

pub(crate) fn notify(
    core: &mut Core,
    env: &Env<Limits>,
    task: u64,
    subscription: u64,
    target: u64,
    state: tasks::NoticeState,
    words: Box<[u8]>,
) -> Queue<tasks::Request> {
    let event = core.notice(task, subscription, target, state, words, env.wall);
    let mut follow = Queue::uncapped();
    crate::routing::step::step_tasks(core, env, event, &mut follow);
    follow
}

pub(crate) fn timer(
    core: &mut Core,
    env: &Env<Limits>,
    task: u64,
    subscription: u64,
    pending: &mut Queue<tasks::Request>,
) {
    let event = core.notice_timer(task, subscription, env.wall);
    let mut follow = Queue::uncapped();
    crate::routing::step::step_tasks(core, env, event, &mut follow);
    for _ in 0..follow.len() {
        pending.push(follow.pop().expect("timer output count"));
    }
}
