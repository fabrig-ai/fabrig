//! An agent's task's lineage, brief, run and conversation (client/pages/task.md, 2).
use crate::cards::CardWords as _;
use crate::{
    bindings::Bindings,
    look,
    words::{Shell, Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::{Latest, contract as c};
use skein_lib::bytes::copy_of;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::{Act, chat, task},
};

fn text(b: &mut Builder, role: Role, value: &[u8]) {
    b.open(role);
    b.name(value);
    b.text(Run::Plain, value);
    b.close();
}
#[expect(
    clippy::too_many_lines,
    reason = "the agent page keeps its taskbar, brief, run and conversation in page order"
)]
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    rows: u32,
    times: &mut Times,
) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::AGENT_PAGE);
    if let Some(page) = root.pages().task()
        && let Some(task) = page.document(root.client())
    {
        let me = root.pages().me(root.client());
        let paused = match me {
            Some(me) => chat::paused(&task.chip, me),
            None => false,
        };
        lineage(b, task, words, bindings);
        b.open(Role::Slot);
        b.style(look::TASKBAR);
        b.open(Role::Slot);
        text(
            b,
            Role::Heading { level: 1 },
            &words.say(Word::ChatTitle, &[Hole::Number(task.chip.task), Hole::Name(copy_of(task.chip.title.bytes()))]),
        );
        super::task::state(b, task::header_state(task, me), words);
        b.close();
        b.open(Role::Slot);
        super::task::executor(b, task.chip.executor, words);
        text(b, Role::Paragraph, &words.say(Word::RowSpent, &[Hole::Number(task.budget.numbers.spent)]));
        if (task.result.is_some() || super::chats::ended(&task.chip))
            && let Some(line) = crate::choice::ran_line(root, task, &page.thread, words)
        {
            text(b, Role::Paragraph, &line);
        }
        if root.pages().confirmation().is_none()
            && match &page.amending {
                Some(open) => open.done,
                None => true,
            }
        {
            if task::can_leave(task, me) {
                super::task::act(b, task, c::ActKind::Leave, Word::LeaveTalk, Act::LeaveTalk, words, bindings);
            }
            super::task::act(b, task, c::ActKind::Amend, Word::Amend, Act::Amend, words, bindings);
            if paused {
                super::task::act(b, task, c::ActKind::Release, Word::Resume, Act::Resume, words, bindings);
            } else {
                super::task::act(b, task, c::ActKind::Stop, Word::Pause, Act::Pause, words, bindings);
            }
            super::task::act(b, task, c::ActKind::Cancel, Word::Close, Act::Close, words, bindings);
        }
        b.close();
        b.close();
        b.open(Role::Slot);
        b.style(look::THREAD);
        super::task::amendment(b, root, words, bindings);
        // A browser's log is the one scroller, holding the column (client/pages/task.md, 2);
        // a terminal prints the column around its log.
        let scroller = words.shell == Shell::Browser;
        if scroller {
            super::chat::open_log(b, &page.thread, words, bindings);
        }
        brief(b, task, me, words, bindings);
        if let Some(run) = task::run(task) {
            run_heading(b, run, words);
        }
        if !paused {
            super::task::controls(b, &task.chip, root, words, bindings, rows);
        }
        super::chat::older(b, &page.thread, words);
        if !scroller {
            super::chat::open_log(b, &page.thread, words, bindings);
        }
        super::chat::conversation_lines(
            b,
            &page.thread,
            words,
            (rows, page.thread.limits.text),
            super::chat::speakers(task, me),
            (
                match task::run(task) {
                    Some(task::Run::Live { .. }) => super::chat::Drawn::AgentRun { current: page.current_call() },
                    Some(task::Run::Waiting { .. } | task::Run::Ended { .. } | task::Run::NoRun) | None => {
                        super::chat::Drawn::Lines
                    }
                },
                None,
            ),
            bindings,
        );
        if !scroller {
            b.close();
        }
        b.open(Role::Slot);
        b.style(look::AGENT_FOOT);
        b.key(super::chat::fixed_key(56));
        super::task::outcome(b, root, words);
        super::task::result(b, task, words, look::AGENT_RESULT);
        if let Some(document) = root.client().document(page.tree)
            && let Some(Latest::Jig(tree)) = document.latest
        {
            let delegates = match tree {
                c::Delivery::Tree(tree) => !tree.rows.is_empty(),

                c::Delivery::Project(_)
                | c::Delivery::Me(_)
                | c::Delivery::Task(_)
                | c::Delivery::Proposal(_)
                | c::Delivery::Chips(_)
                | c::Delivery::Inbox(_)
                | c::Delivery::Tasks(_)
                | c::Delivery::Run(_)
                | c::Delivery::RunState(_)
                | c::Delivery::Settings(_)
                | c::Delivery::Deployment(_)
                | c::Delivery::Terminal(_)
                | c::Delivery::Accounts(_) => false,
            };
            if delegates || !page.cards.is_empty() {
                super::task::plan(
                    b,
                    Some(tree),
                    root,
                    words,
                    bindings,
                    rows,
                    super::task::Plan { word: Word::Plan, ended: None },
                );
            }
        }
        b.close();
        if scroller {
            b.close();
        }
        super::chat::latest(b, &page.thread, words, bindings);
        b.close();
        composer(b, root, page, task, words, bindings, times);
    }
    crate::confirm::main(b, root, words, bindings, rows);
    b.close();
}
fn brief(b: &mut Builder, task: &c::Task, me: Option<&c::Me>, words: &Words, bindings: &mut Bindings) {
    b.open(Role::Region);
    b.style(look::AGENT_BRIEF);
    b.name(&words.say(Word::TaskBrief, &[]));
    text(b, Role::Heading { level: 2 }, &words.say(Word::TaskBrief, &[]));
    skein_ui::markdown::read(task.spec.specification.words.bytes(), 3, b);
    super::task::requester(b, task.requester, me, words);
    super::task::dependencies(
        b,
        task.chip.task,
        &task.dependencies,
        task.dependency_facts.as_slice(),
        false,
        me,
        words,
        bindings,
    );
    b.close();
}
fn lineage(b: &mut Builder, task: &c::Task, words: &Words, bindings: &mut Bindings) {
    b.open(Role::Slot);
    b.style(look::AGENT_LINEAGE);
    if let Some(parent) = task.lineage_chips.as_slice().last() {
        b.open(Role::Slot);
        super::task::task_link(b, parent.task, words, bindings);
        text(b, Role::Paragraph, parent.title.bytes());
        b.close();
    } else if let c::Requester::Task(parent) = task.requester {
        b.open(Role::Slot);
        super::task::task_link(b, parent, words, bindings);
        b.close();
    }
    b.open(Role::Slot);
    text(b, Role::Paragraph, &words.say(Word::Task, &[Hole::Number(task.chip.task)]));
    text(b, Role::Paragraph, task.chip.title.bytes());
    b.close();
    b.close();
}
fn run_heading(b: &mut Builder, run: task::Run, words: &Words) {
    let (word, attempt, since) = match run {
        task::Run::Live { attempt, since } => (Word::RunLive, Some(attempt), since),
        task::Run::Waiting { attempt, since } => (Word::RunWaiting, Some(attempt), since),
        task::Run::Ended { attempt } => (Word::RunOver, Some(attempt), None),
        task::Run::NoRun => (Word::NoRun, None, None),
    };
    let heading = match attempt {
        Some(attempt) => words.say(word, &[Hole::Number(attempt)]),
        None => words.say(word, &[]),
    };
    b.open(Role::Slot);
    b.style(look::AGENT_RUN);
    text(b, Role::Heading { level: 2 }, &heading);
    if let Some(at) = since {
        text(b, Role::Paragraph, &words.run_started(at));
    }
    b.close();
}

fn composer(
    b: &mut Builder,
    root: &Domain,
    page: &task::Task,
    task: &c::Task,
    words: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    let talking = task::talks(task);
    let kind = if talking { c::ActKind::Talk } else { c::ActKind::Message };
    let label = if talking { Word::Talk } else { Word::Message };
    let Some(admission) = jig_client::facts::offer(task.acts.as_slice(), kind) else {
        crate::choice::composer(b, root, Some(task), words, bindings, times);
        return;
    };
    let over = match root.pages().me(root.client()) {
        Some(me) => temper_client_domain::pages::composer::over(&page.words, page.beyond, me.words_bytes),
        None => 0,
    };
    b.open(Role::Slot);
    b.style(look::COMPOSER_AREA);
    match admission {
        c::Admission::Allowed | c::Admission::Bounded { .. } => {
            b.open(Role::Paragraph);
            b.style(look::AGENT_STEER);
            if talking {
                words.text(b, Word::TalkNote, &[Hole::Name(copy_of(task.chip.title.bytes()))]);
            } else {
                words.text(b, Word::SteerNote, &[]);
            }
            b.close();
            crate::choice::composer(b, root, Some(task), words, bindings, times);
            b.open(Role::Slot);
            b.style(look::ENTRY);
            b.open(Role::TextArea);
            b.name(&words.say(label, &[]));
            if talking {
                b.placeholder(&words.say(Word::TalkPlaceholder, &[Hole::Name(copy_of(task.chip.title.bytes()))]));
            }
            if page.composer_focus != 0 && root.pages().switcher().renaming().is_none() {
                b.focus(page.composer_focus);
            }
            if over == 0 {
                b.state(State::SubmitsOnEnter);
            } else {
                bindings.block_submissions();
            }
            b.style(look::COMPOSER);
            b.value(&page.words, temper_client_domain::pages::shown(root.pages()));
            b.bind(bindings.insert(Act::MessageWords { words: Box::default() }));
            b.close();
            b.open(Role::Paragraph);
            b.style(look::HINT);
            words.text(b, Word::ChatSendHint, &[]);
            b.close();
            b.open(Role::Button);
            b.name(&words.say(Word::Send, &[]));
            if over != 0 {
                b.state(State::Disabled);
            }
            b.bind(bindings.insert(Act::Send));
            b.close();
            super::session::over(b, words, over);
            b.close();
            if talking {
                b.open(Role::Paragraph);
                b.style(look::HINT);
                words.text(b, Word::TalkTerminal, &[Hole::Number(task.chip.task)]);
                b.close();
            }
        }
        // Why Message isn't offered is a note, as the steer note is, not an outcome (5.11).
        c::Admission::Refused(refusal) => {
            text(b, Role::Paragraph, &words.say(label, &[]));
            b.open(Role::Paragraph);
            b.style(look::AGENT_STEER);
            if !talking && *refusal == c::Refusal::NotWaiting {
                words.text(b, Word::MessageOnceRuns, &[]);
            } else {
                words.refusal(b, refusal, 0);
            }
            b.close();
            crate::choice::composer(b, root, Some(task), words, bindings, times);
        }
        c::Admission::Proposed { .. } => {
            super::task::act(b, task, kind, label, Act::Send, words, bindings);
            crate::choice::composer(b, root, Some(task), words, bindings, times);
        }
    }
    b.close();
}
