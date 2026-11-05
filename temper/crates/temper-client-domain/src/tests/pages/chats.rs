use super::fixtures;
use crate::ChatsFilter;
use crate::pages::{
    Request,
    chats::{Chats, FIRST_CHAT_WORDS},
};
use jig_client::{Event, Readable, Readout, Told, Watchable, contract::*};
use skein_lib::{List, Queue};
const FIRST: u64 = 1;
const OUTPUTS: u32 = 16;
const CURSOR_BYTES: u32 = 8;
const SERIAL: u64 = 13;
const PRESSES: u32 = 2;
const CONFIGURING: u32 = 3;
// The stock chat and configuring chat supply two templates and two charters.
const TEMPLATES: u32 = 2;
// The model tests offer one selectable alternative to the template's models.
const MODELS: u32 = 1;
// Each chat start creates one task.
const CREATED_TASKS: u32 = 1;

struct Page {
    root: crate::Domain,
    pages: crate::pages::Pages,
    out: Queue<Request>,
}

impl Page {
    fn new(me: Me, address: crate::Address) -> Self {
        let limits = fixtures::limits();
        let mut root = crate::unlinked(&limits, fixtures::SEED);
        let watch = fixtures::start(&mut root, List::with_capacity(0));
        crate::step(
            &mut root,
            &fixtures::env(&limits),
            crate::Event::Client {
                event: Event::Up {
                    up: jig_client::Up::Watched { watch, event: WatchEvent::Delivered(Delivery::Me(Box::new(me))) },
                },
            },
            &mut Queue::uncapped(),
        );
        let mut page = Self { root, pages: crate::pages::Pages::new(), out: Queue::uncapped() };
        crate::pages::project_changed(&mut page.pages, page.root.client());
        page.event(crate::pages::Event::Went { address });
        while page.out.pop().is_some() {}
        page
    }

    fn event(&mut self, event: crate::pages::Event) {
        crate::pages::step(
            &mut self.pages,
            &fixtures::env(&fixtures::limits()),
            self.root.client(),
            event,
            &mut self.out,
        );
    }

    fn act(&mut self, act: crate::pages::Act) {
        self.event(crate::pages::Event::Act { act });
    }

    fn task(&mut self) -> NewTask {
        let sent = fixtures::acts::sent(&mut self.out);
        let [Ask::Create { project, tasks }] = sent.as_ref() else { panic!("one create: {sent:?}") };
        assert_eq!(*project, fixtures::PROJECT);
        let [task] = tasks.as_slice() else { panic!("one task: {tasks:?}") };
        task.clone()
    }

    fn words(&mut self, words: &[u8]) {
        self.act(crate::pages::Act::ChatWords { words: words.into() });
    }
}

fn configuring_me() -> Me {
    let mut me = fixtures::chats::me();
    let project = me.projects.get_mut(0).expect("project");
    let stock = project.templates.get(0).expect("stock chat").clone();
    let mut configuring = stock.clone();
    configuring.number = CONFIGURING;
    configuring.executor = TemplateExecutor::Charter(CONFIGURING);
    project.templates = List::with_capacity(TEMPLATES);
    project.templates.push(stock).unwrap();
    project.templates.push(configuring).unwrap();
    let charters = &mut project.creating.as_mut().expect("creating").within.charters;
    *charters = List::with_capacity(TEMPLATES);
    charters.push(1).unwrap();
    charters.push(CONFIGURING).unwrap();
    me
}

#[test]
fn the_configure_page_starts_template_three_or_one() {
    use crate::{Address, pages::Act};
    for (me, charter) in [(configuring_me(), CONFIGURING), (fixtures::chats::me(), 1)] {
        let mut page = Page::new(me, Address::Configure { project: fixtures::PROJECT });
        page.words(b"My workflow");
        for _ in 0..PRESSES {
            page.act(Act::StartChat);
        }
        let task = page.task();
        assert_eq!(task.executor, Executor::Agent { charter });
        assert_eq!(task.spec.words.bytes(), b"My workflow");
        assert!(task.symbols.is_empty(), "configuring names no repository");
        let chats = page.pages.chats().expect("configure chats");
        assert_eq!(chats.words.as_ref(), b"My workflow");
        let key = chats.start.expect("pending start");
        assert_eq!(key.asker, crate::owner::Owner::Pages.asker());
        let mut tasks = List::with_capacity(CREATED_TASKS);
        tasks.push(fixtures::acts::TASK).unwrap();
        page.event(crate::pages::Event::Told {
            told: Told::Answered {
                key,
                outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(Answer::Done(Outcome::Made {
                    what: Made::Tasks { tasks },
                }))),
            },
        });
        assert_eq!(page.out.pop(), Some(Request::Show { address: Address::Chat { task: fixtures::acts::TASK } }));
        assert!(page.out.is_empty());
        assert!(page.pages.chats().expect("chats").start.is_none());
    }
}

#[test]
fn a_starter_starts_once_with_its_words() {
    use crate::{Address, pages::Act};
    for (me, charter) in [(configuring_me(), CONFIGURING), (fixtures::chats::me(), 1)] {
        let mut page = Page::new(me, Address::Configure { project: fixtures::PROJECT });
        page.words(b"Unsent draft");
        page.act(Act::Starter { words: b"My starter".as_slice().into() });
        page.act(Act::Starter { words: b"Another starter".as_slice().into() });
        page.act(Act::StartChat);
        let task = page.task();
        assert_eq!(task.executor, Executor::Agent { charter });
        assert_eq!(task.spec.words.bytes(), b"My starter");
        assert_eq!(page.pages.chats().expect("chats").words.as_ref(), b"My starter");
        assert!(page.pages.chats().expect("chats").start.is_some());
    }
}

#[test]
fn starters_outside_configure_do_nothing() {
    use crate::{Address, pages::Act};
    for address in [
        Address::Home { project: fixtures::PROJECT },
        Address::Chats { filter: ChatsFilter::All, start: false },
        Address::Settings { project: fixtures::PROJECT },
    ] {
        let mut page = Page::new(configuring_me(), address);
        page.words(b"Unsent draft");
        page.act(Act::Starter { words: b"My starter".as_slice().into() });
        assert!(page.out.is_empty());
        if let Some(chats) = page.pages.chats() {
            assert_eq!(chats.words.as_ref(), b"Unsent draft");
            assert!(chats.start.is_none());
        }
    }
}

#[test]
fn a_refused_start_keeps_the_words() {
    use crate::{Address, pages::Act};
    for act in [Act::StartChat, Act::Starter { words: b"My workflow".as_slice().into() }] {
        let mut me = configuring_me();
        me.projects.get_mut(0).expect("project").creating = None;
        let mut page = Page::new(me, Address::Configure { project: fixtures::PROJECT });
        page.words(b"My workflow");
        page.act(act.clone());
        let chats = page.pages.chats().expect("chats");
        assert_eq!(chats.refusal, Some(jig_client::shapes::NotOffered::Creating));
        assert_eq!(chats.words.as_ref(), b"My workflow");
        assert!(chats.start.is_none());
        assert!(page.out.is_empty());

        let mut page = Page::new(configuring_me(), Address::Configure { project: fixtures::PROJECT });
        page.words(b"My workflow");
        page.act(act);
        let key = page.pages.chats().expect("chats").start.expect("start");
        while page.out.pop().is_some() {}
        let outcome = jig_client::Outcome::Answered(jig_client::Answered::Jig(Answer::Refused {
            refusal: Refusal::NotOffered,
            at: None,
        }));
        page.event(crate::pages::Event::Told { told: Told::Answered { key, outcome: outcome.clone() } });
        let chats = page.pages.chats().expect("chats");
        assert_eq!(chats.outcome, Some(outcome));
        assert_eq!(chats.words.as_ref(), b"My workflow");
        assert!(chats.start.is_none());
        assert!(page.out.is_empty());
    }
}

fn llm(model: &[u8]) -> LlmChoice {
    LlmChoice {
        account: Some(7),
        main: ModelEffort { model: fixtures::acts::text(model), effort: None },
        subagents: List::with_capacity(0),
    }
}

#[test]
fn a_start_from_configure_keeps_its_templates_models() {
    use crate::{Address, pages::Act};
    for (mut me, expected) in [(fixtures::chats::me(), llm(b"stock")), (configuring_me(), llm(b"configuring"))] {
        for row in me.projects.get_mut(0).expect("project").templates.iter_mut() {
            row.llm = Some(llm(if row.number == CONFIGURING { b"configuring" } else { b"stock" }));
        }
        let mut models = List::with_capacity(MODELS);
        models
            .push(MeModel {
                prices: ModelPrices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1000 },
                description: None,
                name: fixtures::acts::text(b"other"),
                provider: Some(17),
                window: 1024,
                output: 128,
                efforts: List::with_capacity(0),
            })
            .unwrap();
        me.projects.get_mut(0).expect("project").models = models;
        for requested in [None, Some(llm(b"other")), Some(llm(b"unoffered"))] {
            for act in [Act::StartChat, Act::Starter { words: b"My workflow".as_slice().into() }] {
                let mut page = Page::new(me.clone(), Address::Configure { project: fixtures::PROJECT });
                page.words(b"My workflow");
                page.pages.choice.requested = requested.clone();
                page.act(act);
                assert_eq!(page.task().llm, Some(expected.clone()));
            }
        }
    }
}
#[test]
fn the_first_chat_starts_the_stock_chat_while_no_configuring_template_is_offered() {
    first_chat_uses_template(fixtures::chats::me(), jig_client::shapes::CHAT);
}

#[test]
fn the_first_chat_starts_the_configuring_template_once_offered() {
    first_chat_uses_template(configuring_me(), CONFIGURING);
}

fn first_chat_uses_template(me: Me, template: u32) {
    use crate::{Address, pages::Act};
    let mut page = Page::new(me.clone(), Address::Chats { filter: ChatsFilter::All, start: false });
    page.words(b"unfinished draft");
    for _ in 0..PRESSES {
        page.act(Act::FirstChat);
    }
    let expected =
        jig_client::shapes::chat_from(&me, fixtures::PROJECT, template, fixtures::acts::text(FIRST_CHAT_WORDS))
            .expect("offered template");
    let chats = page.pages.chats().expect("chats");
    assert_eq!(&*chats.words, b"unfinished draft");
    let key = chats.start.expect("pending start");
    assert_eq!(key.asker, crate::owner::Owner::Pages.asker());
    assert_eq!(fixtures::acts::sent(&mut page.out).as_ref(), [expected]);
    let mut tasks = List::with_capacity(CREATED_TASKS);
    tasks.push(fixtures::acts::TASK).unwrap();
    page.event(crate::pages::Event::Told {
        told: Told::Answered {
            key,
            outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(Answer::Done(Outcome::Made {
                what: Made::Tasks { tasks },
            }))),
        },
    });
    assert_eq!(page.out.pop(), Some(Request::Show { address: Address::Chat { task: fixtures::acts::TASK } }));
    assert!(page.out.is_empty());
    assert!(page.pages.chats().expect("chats").start.is_none());
}

#[test]
fn starting_a_chat_sends_the_chat_template_filled_once() {
    let limits = fixtures::limits();
    let me = fixtures::chats::me();
    let mut chats = Chats::new(FIRST);
    let mut out = Queue::with_capacity(OUTPUTS);
    let mut serial = SERIAL;
    chats.write(Box::from(*b"Hello"), &limits);
    chats.start(
        &fixtures::env(&limits),
        Some(&me),
        Some(fixtures::PROJECT),
        &crate::pages::choice::Draft::default(),
        &mut serial,
        &mut out,
    );
    chats.start(
        &fixtures::env(&limits),
        Some(&me),
        Some(fixtures::PROJECT),
        &crate::pages::choice::Draft::default(),
        &mut serial,
        &mut out,
    );
    let expected = jig_client::shapes::chat(&me, fixtures::PROJECT, fixtures::acts::text(b"Hello")).expect("template");
    assert_eq!(fixtures::acts::sent(&mut out).as_ref(), [expected]);
}
#[test]
fn the_chats_watch_live_lines_by_last_activity() {
    let limits = fixtures::limits();
    let me = fixtures::chats::me();
    let mut chats = Chats::new(FIRST);
    let mut out = Queue::with_capacity(OUTPUTS);
    chats.configure(Some(&me), Some(fixtures::PROJECT), &limits, &mut out);
    let query = TaskQuery {
        project: Some(fixtures::PROJECT),
        requester: Some(Requester::Person(fixtures::PARTY)),
        label: Some(7),
        root: None,
        parent: None,
        phase: None,
        executor: None,
        tracked: None,
        order: QueryOrder::Activity,
    };
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Watch {
            handle: chats.live,
            subject: Watchable::Jig(Subject::Tasks { query, window: limits.client.domain.rows })
        }))
    );
}
#[test]
fn closed_chats_are_read_a_page_at_a_time() {
    let limits = fixtures::limits();
    let mut chats = Chats::new(FIRST);
    let mut out = Queue::with_capacity(OUTPUTS);
    chats.configure(Some(&fixtures::chats::me()), Some(fixtures::PROJECT), &limits, &mut out);
    out.pop();
    let Some(Request::Client(Event::Read { handle, read: Readable::Jig(Read::Tasks { query, live, ended }) })) =
        out.pop()
    else {
        panic!("closed read")
    };
    assert_eq!(
        (handle, query.phase, query.order, live, ended),
        (chats.closed, Some(PhaseClass::Ended), QueryOrder::Ended, None, None)
    );
    let cursor = Cursor::new(Box::from(*b"next"), CURSOR_BYTES).expect("cursor");
    chats.told(
        Told::Page {
            handle,
            page: Readout::Jig(ReadResult::Value(ReadValue::Tasks(Box::new(TaskPage {
                live: List::with_capacity(0),
                live_after: None,
                ended: EndedGroup::Rows { rows: List::with_capacity(0), after: Some(cursor.clone()), cut: false },
            })))),
        },
        &mut out,
    );
    chats.older(&mut out);
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Read {
            handle,
            read: Readable::Jig(Read::Tasks { query, live: None, ended: Some(cursor) })
        }))
    );
}
#[test]
fn starting_a_chat_waits_for_the_first_words() {
    let limits = fixtures::limits();
    let me = fixtures::chats::me();
    let mut chats = Chats::new(FIRST);
    let mut out = Queue::with_capacity(OUTPUTS);
    let mut serial = SERIAL;
    chats.start(
        &fixtures::env(&limits),
        Some(&me),
        Some(fixtures::PROJECT),
        &crate::pages::choice::Draft::default(),
        &mut serial,
        &mut out,
    );
    assert_eq!(fixtures::acts::sent(&mut out).len(), 0);
}
#[test]
fn a_started_chat_opens_its_conversation_activity() {}

#[test]
fn new_chats_and_cues_keep_the_send_bound_when_receiving_grows() {
    let mut limits = fixtures::limits();
    limits.client.domain.text = 2048;
    limits.client.domain.turn_text = 2048;
    let bound = usize::try_from(temper_client_protocol_wire::CUE_BYTES).unwrap();
    let me = fixtures::chats::me();
    for suggested in [false, true] {
        for length in [bound, bound + 1] {
            let mut words = skein_lib::bytes::zeroed(length);
            words.fill(b'a');
            let mut chats = Chats::new(FIRST);
            let mut out = Queue::with_capacity(OUTPUTS);
            let mut serial = SERIAL;
            if suggested {
                chats.suggest(
                    &fixtures::env(&limits),
                    Some(&me),
                    Some(fixtures::PROJECT),
                    &words,
                    &crate::pages::choice::Draft::default(),
                    &mut serial,
                    &mut out,
                );
            } else {
                chats.write(words.clone(), &limits);
                chats.start(
                    &fixtures::env(&limits),
                    Some(&me),
                    Some(fixtures::PROJECT),
                    &crate::pages::choice::Draft::default(),
                    &mut serial,
                    &mut out,
                );
            }
            let sent = fixtures::acts::sent(&mut out);
            assert_eq!(sent.len(), usize::from(length == bound), "suggested={suggested}, length={length}");
            if length == bound {
                let words = Text::new(words, temper_client_protocol_wire::CUE_BYTES).unwrap();
                assert_eq!(sent.as_ref(), [jig_client::shapes::chat(&me, fixtures::PROJECT, words).unwrap()]);
            }
        }
    }
}

#[test]
fn chats_open_with_all_selected() {
    assert_eq!(Chats::new(FIRST).filter, ChatsFilter::All);
}

#[test]
fn chat_filter_moves_keep_the_draft_and_the_closed_page() {
    use crate::{
        ChatsFilter as Filter,
        pages::{self, Act, Event, Pages},
    };
    let limits = fixtures::limits();
    let root = crate::unlinked(&limits, fixtures::SEED);
    let mut pages = Pages::new();
    let mut out = Queue::with_capacity(OUTPUTS);
    pages::step(
        &mut pages,
        &fixtures::env(&limits),
        root.client(),
        Event::Went { address: crate::Address::Chats { filter: ChatsFilter::All, start: false } },
        &mut out,
    );
    while out.pop().is_some() {}
    pages::step(
        &mut pages,
        &fixtures::env(&limits),
        root.client(),
        Event::Act { act: Act::ChatWords { words: Box::from(*b"draft") } },
        &mut out,
    );
    assert!(out.is_empty());
    let handles = {
        let chats = pages.chats().expect("chats");
        (chats.live, chats.closed)
    };
    let closed = TaskPage {
        live: List::with_capacity(0),
        live_after: None,
        ended: EndedGroup::Rows { rows: List::with_capacity(0), after: None, cut: false },
    };
    pages::step(
        &mut pages,
        &fixtures::env(&limits),
        root.client(),
        Event::Told {
            told: Told::Page {
                handle: handles.1,
                page: Readout::Jig(ReadResult::Value(ReadValue::Tasks(Box::new(closed.clone())))),
            },
        },
        &mut out,
    );
    assert!(out.is_empty());
    for filter in [Filter::Live, Filter::Closed, Filter::All] {
        let address = crate::Address::Chats { filter, start: false };
        pages::step(
            &mut pages,
            &fixtures::env(&limits),
            root.client(),
            Event::Act { act: Act::ChatFilter { filter } },
            &mut out,
        );
        assert_eq!(out.pop(), Some(Request::Show { address: address.clone() }));
        assert!(out.is_empty());
        pages::step(&mut pages, &fixtures::env(&limits), root.client(), Event::Went { address }, &mut out);
        let chats = pages.chats().expect("chats");
        assert_eq!(chats.filter, filter);
        assert_eq!(&*chats.words, b"draft");
        assert_eq!((chats.live, chats.closed), handles);
        assert_eq!(chats.page.as_ref(), Some(&closed));
        assert!(out.is_empty(), "a filter move reads nothing again");
    }
}

#[test]
fn chat_filters_outside_chats_are_ignored_and_reopening_resets_them() {
    use crate::{
        ChatsFilter as Filter,
        pages::{self, Act, Event, Pages},
    };
    let limits = fixtures::limits();
    let root = crate::unlinked(&limits, fixtures::SEED);
    let mut pages = Pages::new();
    let mut out = Queue::with_capacity(OUTPUTS);
    pages::step(
        &mut pages,
        &fixtures::env(&limits),
        root.client(),
        Event::Act { act: Act::ChatFilter { filter: Filter::Closed } },
        &mut out,
    );
    assert!(pages.chats().is_none());
    assert!(out.is_empty());
    for address in [
        crate::Address::Chats { filter: ChatsFilter::All, start: false },
        crate::Address::Accounts,
        crate::Address::Chats { filter: ChatsFilter::All, start: false },
    ] {
        pages::step(&mut pages, &fixtures::env(&limits), root.client(), Event::Went { address }, &mut out);
        while out.pop().is_some() {}
        if let Some(chats) = pages.chats() {
            assert_eq!(chats.filter, Filter::All);
        }
        pages::step(
            &mut pages,
            &fixtures::env(&limits),
            root.client(),
            Event::Act { act: Act::ChatFilter { filter: Filter::Live } },
            &mut out,
        );
        if pages.chats().is_some() {
            assert_eq!(
                out.pop(),
                Some(Request::Show { address: crate::Address::Chats { filter: Filter::Live, start: false } })
            );
        }
        assert!(out.is_empty());
    }
}

#[test]
fn live_updates_do_not_replace_an_older_closed_page() {
    let limits = fixtures::limits();
    let mut chats = Chats::new(FIRST);
    let mut out = Queue::with_capacity(OUTPUTS);
    chats.configure(Some(&fixtures::chats::me()), Some(fixtures::PROJECT), &limits, &mut out);
    while out.pop().is_some() {}
    chats.page = Some(TaskPage {
        live: List::with_capacity(0),
        live_after: None,
        ended: EndedGroup::Rows {
            rows: List::with_capacity(0),
            after: Some(Cursor::new(Box::from(*b"next"), CURSOR_BYTES).expect("cursor")),
            cut: false,
        },
    });
    chats.older(&mut out);
    out.pop();
    chats.told(Told::Changed { handle: chats.live }, &mut out);
    assert!(out.is_empty(), "an older page stays put");
}

pub(super) fn chat_line() -> TaskLine {
    let task = fixtures::chat::task();
    let ChipPhaseFacts { details, .. } = task.chip.details.as_ref();
    let budget = details.budget.clone();
    TaskLine {
        chip: task.chip,
        project: fixtures::PROJECT,
        requester: Requester::Person(fixtures::PARTY),
        tracked: None,
        label: fixtures::chat::LABEL,
        flags: TaskFlags { ended_by: EndedBy::Result, conversation: true, listed: false, retention: Retention::Kept },
        changed: skein_lib::Wall::EPOCH,
        last_activity: skein_lib::Wall::EPOCH,
        ended_at: None,
        budget,
        waits_on: WaitsOn::Nothing,
        phases: PhaseCounts::ZERO,
        held_below: 0,
        conversation: None,
    }
}

fn opened_chats() -> (crate::Domain, skein_lib::Token, skein_lib::Token) {
    use crate::{Address, Place};
    let limits = fixtures::limits();
    let env = fixtures::env(&limits);
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let me = fixtures::start(&mut root, List::with_capacity(0));
    let mut out = Queue::uncapped();
    crate::step(
        &mut root,
        &env,
        crate::Event::Client {
            event: Event::Up {
                up: jig_client::Up::Watched {
                    watch: me,
                    event: WatchEvent::Delivered(Delivery::Me(Box::new(fixtures::chats::me()))),
                },
            },
        },
        &mut out,
    );
    crate::step(
        &mut root,
        &env,
        crate::Event::Went { place: Place::Address(Address::Chats { filter: ChatsFilter::All, start: false }) },
        &mut out,
    );
    let mut watch = None;
    let mut read = None;
    while let Some(request) = out.pop() {
        if let crate::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Open { watch: token, subject: Watchable::Jig(Subject::Tasks { .. }) },
                },
        } = request
        {
            watch = Some(token);
        } else if let crate::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read: token, what: Readable::Jig(Read::Tasks { .. }) },
                },
        } = request
        {
            read = Some(token);
        }
    }
    (root, watch.expect("live watch"), read.expect("first closed page"))
}

#[test]
fn an_ending_chat_refreshes_the_first_closed_page_and_remains_listed() {
    use crate::{Action, pages::Act};
    let limits = fixtures::limits();
    let env = fixtures::env(&limits);
    let (mut root, watch, first_read) = opened_chats();
    let mut out = Queue::uncapped();
    crate::step(
        &mut root,
        &env,
        crate::Event::Client {
            event: Event::Up {
                up: jig_client::Up::ReadDone {
                    read: first_read,
                    result: Readout::Jig(ReadResult::Value(ReadValue::Tasks(Box::new(TaskPage {
                        live: List::with_capacity(0),
                        live_after: None,
                        ended: EndedGroup::Rows { rows: List::with_capacity(0), after: None, cut: false },
                    })))),
                },
            },
        },
        &mut out,
    );
    let mut rows = List::with_capacity(1);
    rows.push(chat_line()).expect("one live chat");
    for rows in [rows, List::with_capacity(0)] {
        crate::step(
            &mut root,
            &env,
            crate::Event::Client {
                event: Event::Up {
                    up: jig_client::Up::Watched {
                        watch,
                        event: WatchEvent::Delivered(Delivery::Tasks(Box::new(Tasks { rows, more: false }))),
                    },
                },
            },
            &mut out,
        );
        if !root.pages().chats().expect("chats").live(root.client()).expect("live group").rows.is_empty() {
            while out.pop().is_some() {}
        }
    }
    assert!(root.pages().chats().expect("chats").live(root.client()).expect("live group").rows.is_empty());
    let mut refreshed = None;
    while let Some(request) = out.pop() {
        if let crate::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: Readable::Jig(Read::Tasks { query, live, ended }) },
                },
        } = request
        {
            assert_eq!(
                (query.phase, query.order, live, ended),
                (Some(PhaseClass::Ended), QueryOrder::Ended, None, None)
            );
            refreshed = Some(read);
        }
    }
    let mut ended = chat_line();
    ended.ended_at = Some(skein_lib::Wall::EPOCH);
    let mut rows = List::with_capacity(1);
    rows.push(ended).expect("ended chat");
    crate::step(
        &mut root,
        &env,
        crate::Event::Client {
            event: Event::Up {
                up: jig_client::Up::ReadDone {
                    read: refreshed.expect("ending re-reads first page"),
                    result: Readout::Jig(ReadResult::Value(ReadValue::Tasks(Box::new(TaskPage {
                        live: List::with_capacity(0),
                        live_after: None,
                        ended: EndedGroup::Rows { rows, after: None, cut: false },
                    })))),
                },
            },
        },
        &mut out,
    );
    let EndedGroup::Rows { rows, .. } = &root.pages().chats().expect("chats").page.as_ref().expect("page").ended else {
        panic!("ended rows")
    };
    assert_eq!(rows.get(0).expect("ended chat").chip.task, fixtures::chat::TASK);
    // Selection is independent of the retained page and its cursors.
    crate::step(
        &mut root,
        &env,
        crate::Event::Act { action: Action::Pages(Act::ChatFilter { filter: ChatsFilter::Closed }) },
        &mut out,
    );
    assert_eq!(
        out.pop(),
        Some(crate::Request::Show {
            address: crate::Address::Chats { filter: ChatsFilter::Closed, start: false },
            replace: false,
        })
    );
    assert!(out.is_empty());
}
