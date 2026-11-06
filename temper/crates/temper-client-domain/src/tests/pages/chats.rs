use super::fixtures;
use crate::ChatsFilter;
use crate::pages::{Request, chats::Chats};
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
fn configure_and_new_chat_start_their_template_and_open_the_chat() {
    use crate::{Address, pages::Act};
    for (me, charter, address) in [
        (configuring_me(), CONFIGURING, Address::Configure { project: fixtures::PROJECT }),
        (fixtures::chats::me(), 1, Address::Configure { project: fixtures::PROJECT }),
        (configuring_me(), 1, Address::NewChat { draft: false }),
    ] {
        let mut page = Page::new(me, address);
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
        let address = Address::Chat { task: fixtures::acts::TASK };
        let expected = if let Address::NewChat { .. } = page.pages.address() {
            Request::Replace { address }
        } else {
            Request::Show { address }
        };
        assert_eq!(page.out.pop(), Some(expected));
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
        Address::Chats { filter: ChatsFilter::All },
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
                effort: None,
                shown: None,
                accounts: List::with_capacity(0),
                picker: true,
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
        for (edited, requested, valid) in [
            (false, Some(expected.clone()), true),
            (true, Some(llm(b"other")), true),
            (true, None, true),
            (true, Some(llm(b"unoffered")), false),
        ] {
            for act in [Act::StartChat, Act::Starter { words: b"My workflow".as_slice().into() }] {
                let mut page = Page::new(me.clone(), Address::Configure { project: fixtures::PROJECT });
                page.words(b"My workflow");
                assert_eq!(page.pages.choice.requested, Some(expected.clone()));
                if edited {
                    page.pages.choice.requested = requested.clone();
                }
                assert_eq!(page.pages.choice.valid(&me), valid);
                page.act(act);
                if valid {
                    assert_eq!(page.task().llm, requested);
                } else {
                    assert!(fixtures::acts::sent(&mut page.out).is_empty(), "an unoffered draft sends no Create");
                }
            }
        }
    }
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
        false,
        &mut serial,
        &mut out,
    );
    chats.start(
        &fixtures::env(&limits),
        Some(&me),
        Some(fixtures::PROJECT),
        &crate::pages::choice::Draft::default(),
        false,
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
        &crate::Address::Chats { filter: ChatsFilter::All },
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
        false,
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
    limits.client.domain.long_text = 2048;
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
                    false,
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
fn new_chat_and_filter_moves_keep_the_draft_and_the_closed_page() {
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
        Event::Went { address: crate::Address::NewChat { draft: false } },
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
    for (address, filter) in [
        (crate::Address::Chats { filter: Filter::Live }, Filter::Live),
        (crate::Address::NewChat { draft: false }, Filter::Live),
        (crate::Address::Chats { filter: Filter::Closed }, Filter::Closed),
        (crate::Address::NewChat { draft: false }, Filter::Closed),
        (crate::Address::Chats { filter: Filter::All }, Filter::All),
        (crate::Address::NewChat { draft: false }, Filter::All),
    ] {
        pages::step(
            &mut pages,
            &fixtures::env(&limits),
            root.client(),
            Event::Act { act: Act::ChatFilter { filter } },
            &mut out,
        );
        assert_eq!(out.pop(), Some(Request::Show { address: crate::Address::Chats { filter } }));
        assert!(out.is_empty());
        pages::step(&mut pages, &fixtures::env(&limits), root.client(), Event::Went { address }, &mut out);
        let chats = pages.chats().expect("chats");
        assert_eq!(chats.filter, filter);
        assert_eq!(&*chats.words, b"draft");
        assert_eq!((chats.live, chats.closed), handles);
        assert_eq!(chats.page.as_ref(), Some(&closed));
        while let Some(request) = out.pop() {
            match request {
                Request::Client(jig_client::Event::Read { handle, read: Readable::Jig(Read::Notes { .. }) }) => {
                    assert_eq!(handle, chats.notes);
                }
                Request::Client(jig_client::Event::Unwatch { handle }) => {
                    assert!([chats.notes, chats.goals].contains(&handle));
                }
                other @ (Request::Client(_) | Request::Show { .. } | Request::Replace { .. }) => {
                    panic!("the live and closed windows stay: {other:?}")
                }
            }
        }
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
        crate::Address::Chats { filter: ChatsFilter::All },
        crate::Address::Accounts,
        crate::Address::Chats { filter: ChatsFilter::All },
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
            assert_eq!(out.pop(), Some(Request::Show { address: crate::Address::Chats { filter: Filter::Live } }));
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
    chats.told(Told::Changed { handle: chats.live }, &crate::Address::Chats { filter: ChatsFilter::All }, &mut out);
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
        delegates_done: 0,
        delegates_total: 0,
        held_below: 0,
        conversation: None,
    }
}

fn opened_chats() -> (crate::Domain, skein_lib::Token, skein_lib::Token) {
    let (root, watch, read, _) =
        open_chat_rows(fixtures::chats::me(), crate::Address::Chats { filter: ChatsFilter::All });
    (root, watch, read)
}

fn open_chat_rows(
    frame: Me,
    address: crate::Address,
) -> (crate::Domain, skein_lib::Token, skein_lib::Token, Option<skein_lib::Token>) {
    use crate::Place;
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
                up: jig_client::Up::Watched { watch: me, event: WatchEvent::Delivered(Delivery::Me(Box::new(frame))) },
            },
        },
        &mut out,
    );
    crate::step(&mut root, &env, crate::Event::Went { place: Place::Address(address) }, &mut out);
    let mut watch = None;
    let mut read = None;
    let mut facts = None;
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
        } else if let crate::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject } },
        } = request
            && Some(subject) == crate::forge::dismissed::watch(&limits.forge)
        {
            facts = Some(watch);
        }
    }
    (root, watch.expect("live watch"), read.expect("first closed page"), facts)
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
        Some(crate::Request::Show { address: crate::Address::Chats { filter: ChatsFilter::Closed }, replace: false })
    );
    assert!(out.is_empty());
}

#[test]
fn a_suggestion_replaces_new_chat_with_its_conversation() {
    let mut page = Page::new(fixtures::chats::me(), crate::Address::NewChat { draft: false });
    page.pages.suggest(&fixtures::env(&fixtures::limits()), page.root.client(), b"My suggestion", &mut page.out);
    let task = page.task();
    assert_eq!(task.spec.words.bytes(), b"My suggestion");
    let key = page.pages.chats().expect("chats").start.expect("pending suggestion");
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
    assert_eq!(page.out.pop(), Some(Request::Replace { address: crate::Address::Chat { task: fixtures::acts::TASK } }));
}

#[test]
fn settings_selects_the_newest_open_configuring_chat() {
    const OTHER_PROJECT: u32 = 91;
    const NEWER: u64 = 42;
    let mut open = chat_line();
    open.chip.executor = ChipExecutor::Agent { charter: CONFIGURING };
    let mut newer = open.clone();
    newer.chip.task = NEWER;
    // Activity ordering is independent of the task number's creation ordering.
    newer.last_activity = skein_lib::Wall::EPOCH;
    open.last_activity = skein_lib::Wall::from_nanos(1);
    let mut ordinary = open.clone();
    ordinary.chip.executor = ChipExecutor::Agent { charter: 1 };
    let mut other = newer.clone();
    other.project = OTHER_PROJECT;
    let mut closed = newer.clone();
    closed.ended_at = Some(skein_lib::Wall::EPOCH);
    let mut facts_ended = newer.clone();
    facts_ended.chip.details.phase = ChipPhase::Ended;
    let mut ended_rows = [closed, facts_ended, newer.clone(), newer.clone(), newer.clone(), newer.clone()];
    for (row, phase) in ended_rows.iter_mut().skip(2).zip([Phase::Done, Phase::Ended, Phase::Failed, Phase::Cancelled])
    {
        row.chip.phase = phase;
    }
    // Six closed/ended rows plus the ordinary, other-project and two open rows.
    let mut mixed = List::with_capacity(10);
    for row in ended_rows.iter().chain([ordinary.clone(), other.clone(), newer.clone(), open.clone()].iter()) {
        mixed.push(row.clone()).expect("ten candidates");
    }
    for (me, lines, expected) in [
        (configuring_me(), [].as_slice(), None),
        (configuring_me(), core::slice::from_ref(&ordinary), None),
        (configuring_me(), core::slice::from_ref(&other), None),
        (configuring_me(), ended_rows.as_slice(), None),
        (configuring_me(), core::slice::from_ref(&open), Some(open.chip.task)),
        (configuring_me(), [newer.clone(), open.clone()].as_slice(), Some(NEWER)),
        (configuring_me(), [open.clone(), newer.clone()].as_slice(), Some(NEWER)),
        (configuring_me(), mixed.as_slice(), Some(NEWER)),
        (fixtures::chats::me(), core::slice::from_ref(&newer), None),
    ] {
        for from_watch in [true, false] {
            let (mut root, watch, read, _) =
                open_chat_rows(me.clone(), crate::Address::Settings { project: fixtures::PROJECT });
            let select = |root: &crate::Domain, project| {
                root.pages().chats().expect("Settings holds chats").open_configuring(
                    root.client(),
                    root.pages().me(root.client()),
                    project,
                )
            };
            assert_eq!(select(&root, fixtures::PROJECT), None, "undelivered");
            let mut rows = List::with_capacity(u32::try_from(lines.len()).unwrap());
            for row in lines {
                rows.push(row.clone()).unwrap();
            }
            let up = if from_watch {
                jig_client::Up::Watched {
                    watch,
                    event: WatchEvent::Delivered(Delivery::Tasks(Box::new(Tasks { rows, more: false }))),
                }
            } else {
                jig_client::Up::ReadDone {
                    read,
                    result: Readout::Jig(ReadResult::Value(ReadValue::Tasks(Box::new(TaskPage {
                        live: rows,
                        live_after: None,
                        ended: EndedGroup::None,
                    })))),
                }
            };
            crate::step(
                &mut root,
                &fixtures::env(&fixtures::limits()),
                crate::Event::Client { event: Event::Up { up } },
                &mut Queue::uncapped(),
            );
            assert_eq!(select(&root, fixtures::PROJECT), expected, "watch: {from_watch}, rows: {lines:?}");
            assert_eq!(select(&root, OTHER_PROJECT), None, "query belongs to this project");
            assert_eq!(root.pages().chats().unwrap().open_configuring(root.client(), None, fixtures::PROJECT), None);
            assert_eq!(Chats::new(FIRST).open_configuring(root.client(), Some(&me), fixtures::PROJECT), None);
        }
    }
}

fn start_answer(page: &mut Page) {
    let key = page.pages.chats().unwrap().start.unwrap();
    let mut tasks = List::with_capacity(1);
    tasks.push(fixtures::acts::TASK).unwrap();
    page.event(crate::pages::Event::Told {
        told: Told::Answered {
            key,
            outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(Answer::Done(Outcome::Made {
                what: Made::Tasks { tasks },
            }))),
        },
    });
    while page.out.pop().is_some() {}
    page.event(crate::pages::Event::Went { address: crate::Address::Chat { task: fixtures::acts::TASK } });
}
#[test]
fn the_first_message_carries_the_words() {
    let mut page = Page::new(fixtures::chats::me(), crate::Address::NewChat { draft: true });
    page.words(b"A way to work");
    page.act(crate::pages::Act::StartChat);
    assert_eq!(page.task().spec.words.bytes(), b"A way to work\n\nWrite this up as a draft beside us.");
}
#[test]
fn the_chat_opens_in_split() {
    let mut page = Page::new(fixtures::chats::me(), crate::Address::NewChat { draft: true });
    page.words(b"Draft");
    page.act(crate::pages::Act::StartChat);
    start_answer(&mut page);
    let chat = page.pages.chat().unwrap();
    assert_eq!(chat.mode, crate::pages::chat::document::Mode::Split);
    assert!(chat.writing);
}
#[test]
fn a_new_chat_with_no_draft_opens_in_chat() {
    let mut page = Page::new(fixtures::chats::me(), crate::Address::NewChat { draft: false });
    page.words(b"Chat");
    page.act(crate::pages::Act::StartChat);
    assert_eq!(page.task().spec.words.bytes(), b"Chat");
    start_answer(&mut page);
    assert_eq!(page.pages.chat().unwrap().mode, crate::pages::chat::document::Mode::Chat);
    assert!(!page.pages.chat().unwrap().writing);
}
#[test]
fn moving_to_a_draft_keeps_the_words() {
    let mut page = Page::new(fixtures::chats::me(), crate::Address::Chats { filter: ChatsFilter::Live });
    page.words(b"Kept words");
    page.act(crate::pages::Act::StartChat);
    let key = page.pages.chats().unwrap().start;
    let handles = (page.pages.chats().unwrap().live, page.pages.chats().unwrap().closed);
    for address in [
        crate::Address::NewChat { draft: false },
        crate::Address::NewChat { draft: true },
        crate::Address::Chats { filter: ChatsFilter::All },
    ] {
        page.event(crate::pages::Event::Went { address });
        let chats = page.pages.chats().unwrap();
        assert_eq!(chats.words.as_ref(), b"Kept words");
        assert_eq!(chats.start, key);
        assert_eq!((chats.live, chats.closed), handles);
    }
}
#[test]
fn a_suggested_request_takes_no_draft() {
    let mut page = Page::new(fixtures::chats::me(), crate::Address::NewChat { draft: true });
    page.pages.suggest(&fixtures::env(&fixtures::limits()), page.root.client(), b"Suggestion", &mut page.out);
    assert_eq!(page.task().spec.words.bytes(), b"Suggestion");
    start_answer(&mut page);
    assert_eq!(page.pages.chat().unwrap().mode, crate::pages::chat::document::Mode::Chat);
    assert!(!page.pages.chat().unwrap().writing);
}
#[test]
fn a_drafts_first_message_stays_within_the_cue_bound() {
    let bound = usize::try_from(temper_client_protocol_wire::CUE_BYTES).unwrap();
    let suffix = crate::pages::chat::document::WRITE_THIS_UP.len() + 2;
    let mut page = Page::new(fixtures::chats::me(), crate::Address::NewChat { draft: true });
    let mut words = skein_lib::bytes::zeroed(bound);
    words.fill(b'a');
    page.words(words.get(..bound - suffix + 1).unwrap());
    page.act(crate::pages::Act::StartChat);
    assert!(page.pages.chats().unwrap().start.is_none());
    assert!(fixtures::acts::sent(&mut page.out).is_empty());
    page.words(words.get(..bound - suffix).unwrap());
    page.act(crate::pages::Act::StartChat);
    assert_eq!(page.task().spec.words.bytes().len(), bound);
}

#[test]
fn adding_a_draft_replaces_the_new_chat_address() {
    let mut page = Page::new(fixtures::chats::me(), crate::Address::NewChat { draft: false });
    let address = crate::Address::NewChat { draft: true };
    page.act(crate::pages::Act::Go { address: address.clone() });
    assert_eq!(page.out.pop(), Some(Request::Replace { address }));
    assert!(page.out.is_empty());
}

#[test]
fn reads_tasks_as_it_opens() {
    let mut page = Page::new(fixtures::chats::me(), crate::Address::NewChat { draft: false });
    page.event(crate::pages::Event::Went { address: crate::Address::Chats { filter: ChatsFilter::All } });
    let notes = page.pages.chats().unwrap().notes;
    assert_eq!(
        page.out.pop(),
        Some(Request::Client(Event::Read {
            handle: notes,
            read: Readable::Jig(Read::Notes {
                scope: NoteScope::Tasks { project: fixtures::PROJECT },
                description: None,
                after: None,
            }),
        }))
    );
    assert!(page.out.is_empty());
    page.event(crate::pages::Event::Went { address: crate::Address::Chats { filter: ChatsFilter::Drafts } });
    assert!(page.out.is_empty(), "filtering has no read");
}

fn list_note(number: u64, task: u64, title: &[u8]) -> Note {
    let mut note = super::chat::draft_note(number, 1, b"");
    note.scope = NoteScope::Task { project: fixtures::PROJECT, task };
    note.description = DocumentText::complete(Text::new(title.into(), 32).unwrap());
    note
}
fn note_page(notes: &[Note], after: Option<Cursor>) -> Told {
    let Readout::Jig(ReadResult::Value(ReadValue::Notes(mut page))) = super::chat::notes_page(notes) else {
        panic!("notes page")
    };
    page.after = after;
    Told::Page { handle: Chats::new(FIRST).notes, page: Readout::Jig(ReadResult::Value(ReadValue::Notes(page))) }
}
#[test]
fn pages_follow_to_the_rows_bound() {
    const ROWS: u32 = 4;
    let mut chats = Chats::new(FIRST);
    let root = crate::unlinked(&fixtures::limits(), fixtures::SEED);
    let mut out = Queue::uncapped();
    chats.open_documents(fixtures::PROJECT, ROWS, &mut out);
    out.pop();
    let cursor = Cursor::new(b"next".as_slice().into(), CURSOR_BYTES).unwrap();
    let first = note_page(&[list_note(9, 4, b"Later"), list_note(5, 4, b"Earlier")], Some(cursor.clone()));
    assert!(chats.notes_told(skein_lib::Time::ZERO, root.client(), &first, &mut out));
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Read {
            handle: chats.notes,
            read: Readable::Jig(Read::Notes {
                scope: NoteScope::Tasks { project: fixtures::PROJECT },
                description: None,
                after: Some(cursor.clone())
            }),
        }))
    );
    let second = note_page(
        &[list_note(2, 4, b"Lowest"), list_note(1, 8, b"Another"), list_note(1, 7, b"Past bound")],
        Some(cursor),
    );
    chats.notes_told(skein_lib::Time::ZERO, root.client(), &second, &mut out);
    assert!(out.is_empty(), "no page past four notes");
    assert_eq!(chats.documents.get(&4).unwrap().bytes(), b"Lowest");
    assert_eq!(chats.documents.get(&8).unwrap().bytes(), b"Another");
    assert!(!chats.documents.contains_key(&7));
}
#[test]
fn document_reads_retry_transient_ends_and_clear_a_refusal() {
    use skein_lib::{Duration, Time};
    let mut chats = Chats::new(FIRST);
    let root = crate::unlinked(&fixtures::limits(), fixtures::SEED);
    let mut out = Queue::uncapped();
    chats.open_documents(fixtures::PROJECT, 2, &mut out);
    let read = out.pop().unwrap();
    chats.notes_told(
        Time::ZERO,
        root.client(),
        &Told::Page { handle: chats.notes, page: Readout::Jig(ReadResult::None(ReadNone::Busy)) },
        &mut out,
    );
    let due = Time::ZERO.saturating_add(Duration::from_secs(1));
    assert_eq!(chats.next_deadline(), Some(due));
    chats.fire(Time::ZERO, &mut out);
    assert!(out.is_empty());
    chats.fire(due, &mut out);
    assert_eq!(out.pop(), Some(read));
    chats.notes_told(due, root.client(), &note_page(&[list_note(1, 4, b"Draft")], None), &mut out);
    assert!(chats.documents.contains_key(&4));
    chats.notes_told(
        due,
        root.client(),
        &Told::Page { handle: chats.notes, page: Readout::Jig(ReadResult::None(ReadNone::Refused)) },
        &mut out,
    );
    assert!(chats.documents.is_empty());
    assert_eq!(chats.next_deadline(), None);
    let closed = chats.leave(&mut out);
    let mut cancelled = false;
    while let Some(request) = out.pop() {
        cancelled |= request == Request::Client(Event::Unwatch { handle: chats.notes });
    }
    assert!(cancelled);
    assert!(closed > chats.goals.number);
}

#[test]
fn goal_chips_cover_only_the_newest_rows_across_both_lists() {
    use jig_client::{Down, Up};
    use skein_lib::Wall;
    use temper_client_protocol_wire::{self as wire, v1 as w};
    // Two live and two ended chats compete for the fixture's two-row window.
    const CHATS: [(u64, u64, bool); 4] = [(10, 4, false), (20, 5, true), (30, 3, true), (40, 4, false)];
    let limits = fixtures::limits();
    let (mut root, watch, read, facts) =
        open_chat_rows(fixtures::chats::me(), crate::Address::Chats { filter: ChatsFilter::All });
    let mut live = List::with_capacity(2);
    let mut closed = List::with_capacity(2);
    let mut dismissed = List::with_capacity(4);
    for (chat, at, ended) in CHATS {
        let mut row = chat_line();
        row.chip.task = chat;
        row.last_activity = Wall::from_nanos(if ended { 99 } else { at });
        row.ended_at = ended.then_some(Wall::from_nanos(at));
        if ended {
            closed.push(row).unwrap();
        } else {
            live.push(row).unwrap();
        }
        dismissed
            .push(w::PersonFact::GoalFrom(
                w::GoalFrom::new(
                    &w::CEILINGS,
                    w::GoalFromParts { project: fixtures::PROJECT, chat, goal: chat + 100, at: 1 },
                )
                .unwrap(),
            ))
            .unwrap();
    }
    let mut out = Queue::uncapped();
    for up in [
        Up::ReadDone {
            read,
            result: Readout::Jig(ReadResult::Value(ReadValue::Tasks(Box::new(TaskPage {
                live: List::with_capacity(0),
                live_after: None,
                ended: EndedGroup::Rows { rows: closed, after: None, cut: false },
            })))),
        },
        Up::Watched {
            watch,
            event: WatchEvent::Delivered(Delivery::Tasks(Box::new(Tasks { rows: live, more: false }))),
        },
        Up::FamilyDelivered {
            watch: facts.expect("person facts watch"),
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::dismissals(&w::Dismissals::new(&w::CEILINGS, w::DismissalsParts { dismissed }).unwrap()),
            },
        },
    ] {
        crate::step(&mut root, &fixtures::env(&limits), crate::Event::Client { event: Event::Up { up } }, &mut out);
    }
    let mut watched = None;
    let mut count = 0_u32;
    while let Some(request) = out.pop() {
        if let crate::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Open { subject: Watchable::Jig(Subject::Chips { tasks }), .. },
                },
        } = request
        {
            count += 1;
            watched = Some(tasks);
        }
    }
    assert_eq!(count, 1, "one chips watch for both lists");
    assert_eq!(watched.unwrap().as_slice(), [120, 110], "newest chat, then lower-numbered tie");
}

#[test]
#[expect(clippy::disallowed_macros, reason = "ordinary test snapshots page state apart from its render counter")]
fn reference_opens_only_at_configure_without_a_request() {
    use crate::{
        Address,
        pages::{Act, chat::document::Mode},
    };
    for address in [
        Address::Configure { project: fixtures::PROJECT },
        Address::NewChat { draft: false },
        Address::Chats { filter: ChatsFilter::All },
        Address::Home { project: fixtures::PROJECT },
        Address::Chat { task: fixtures::acts::TASK },
    ] {
        let mut page = Page::new(configuring_me(), address.clone());
        let reference = match page.pages.chats() {
            Some(chats) => chats.reference,
            None => None,
        };
        assert_eq!(reference, None);
        let before = format!("{:?}", (page.pages.chats(), page.pages.chat(), page.pages.task()));
        if let Address::Configure { .. } = address {
            page.act(Act::Mode { mode: Mode::Draft });
            assert_eq!(page.pages.chats().unwrap().reference, None);
            assert!(page.out.is_empty());
        }
        page.act(Act::ShowReference);
        assert!(page.out.is_empty(), "{address:?}");
        if let Address::Configure { .. } = address {
            assert_eq!(page.pages.chats().unwrap().reference, Some(Mode::Split));
            page.act(Act::ShowReference);
            assert_eq!(page.pages.chats().unwrap().reference, Some(Mode::Split));
            assert!(page.out.is_empty());
            for mode in [Mode::Draft, Mode::Chat, Mode::Split] {
                page.act(Act::Mode { mode });
                let chats = page.pages.chats().unwrap();
                assert_eq!(chats.reference, Some(mode));
                assert!(chats.start.is_none());
                assert!(page.out.is_empty());
            }
            page.event(crate::pages::Event::Went { address: Address::NewChat { draft: false } });
            page.event(crate::pages::Event::Went { address });
            assert_eq!(page.pages.chats().unwrap().reference, None);
        } else {
            assert_eq!(
                format!("{:?}", (page.pages.chats(), page.pages.chat(), page.pages.task())),
                before,
                "{address:?}"
            );
        }
    }
}

#[test]
fn the_chats_list_excludes_working_notes_from_documents() {
    let limits = fixtures::limits();
    let root = crate::unlinked(&limits, fixtures::SEED);
    let mut chats = Chats::new(FIRST);
    let mut out = Queue::uncapped();
    chats.open_documents(fixtures::PROJECT, 4, &mut out);
    while out.pop().is_some() {}
    chats.notes_told(
        skein_lib::Time::ZERO,
        root.client(),
        &note_page(
            &[
                list_note(1, 4, b"work: settings"),
                list_note(2, 4, b"Visible document"),
                list_note(3, 8, b"work: only memory"),
            ],
            None,
        ),
        &mut out,
    );
    assert_eq!(chats.documents.get(&4).unwrap().bytes(), b"Visible document");
    assert!(!chats.documents.contains_key(&8), "a working note is not a draft");
}
