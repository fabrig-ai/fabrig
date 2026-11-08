//! The launchpad's states and hand-written frames (client/README.md, 6.14 and 6.15).
use super::{fixtures as f, goldens};
use crate::{
    Local, Scheme, Told, Tui, look,
    view::{self, foot, launchpad},
};
use jig_client::{Up, contract as c};
use skein_lib::{List, Queue, Wall};
use skein_ui::Role;
use temper_client_domain::{self as d, Address, Domain, Event, Request};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_client_view::shared::{NeedKind, NeedLine};

const PROJECT: u32 = 1;
const OTHER_PROJECT: u32 = 2;
const PARTY: u64 = 73;
const CHAT: u64 = 9;
const ENDED_CHAT: u64 = 10;
const OLDER_CHAT: u64 = 11;
// Twenty read rows, four draw rows, and the headline's rebuild check.
const FAILURES: u32 = 25;
// The longest read row: heading, banner, two left-off lines, and three need lines.
const LINES: u32 = 7;
// Four review frames; six needs are one past the design's five-line bound.
const FRAMES: u32 = 4;
const SIX_NEEDS: u32 = 6;
// Two hours after the landing, at 09:42; a brief is written at 07:42.
const MINUTE: u64 = 60_000_000_000;
const NOW: Wall = Wall::from_nanos(582 * MINUTE);
const BRIEF_AT: u64 = 462 * MINUTE;
const SENTENCE: &str = "3 changes landed overnight.";
const LEDE: &str = "csv-export is two of three tasks in; one waits on you.";

fn status(brief: bool) -> w::ProjectStatus {
    w::ProjectStatus::new(
        &w::CEILINGS,
        w::ProjectStatusParts {
            project: PROJECT,
            landed: 3,
            seen: if brief { 0 } else { 3 },
            last_landed: Some(BRIEF_AT),
            previous: Some(0),
            queued: 0,
            red: false,
            branch: b"main".as_slice().into(),
            brief: if brief {
                Some({
                    w::StatusBrief::new(
                        &w::CEILINGS,
                        w::StatusBriefParts {
                            sentence: SENTENCE.as_bytes().into(),
                            lede: LEDE.as_bytes().into(),
                            at: BRIEF_AT,
                            landed: 3,
                        },
                    )
                    .unwrap()
                })
            } else {
                None
            },
        },
    )
    .unwrap()
}
fn family(bytes: Box<[u8]>) -> jig_client::Family {
    jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes }
}
pub(super) fn line(task: u64, title: &[u8], activity: u64, ended: bool) -> c::TaskLine {
    let bounds =
        jig_client_protocol_link::Limits::new(f::limits().client.domain, f::limits().client.exchanges).unwrap();
    let mut rows =
        jig_client_protocol_link::translate::documents::tasks(jig_client_test_records::tasks_rows_record(), &bounds)
            .unwrap()
            .rows;
    let mut row = rows.swap_remove(0).unwrap();
    row.chip.task = task;
    row.chip.title = c::DocumentText::complete(c::Text::new(title.into(), f::limits().client.domain.text).unwrap());
    row.chip.phase = if ended { c::Phase::Ended } else { c::Phase::Idle };
    row.chip.details.phase = if ended { c::ChipPhase::Ended } else { c::ChipPhase::Active(c::RunStage::Idle) };
    row.waits_on = c::WaitsOn::Party { whom: c::Whom::Person(PARTY), task, kind: c::WaitedFor::Question, entries: 1 };
    row.project = PROJECT;
    row.last_activity = Wall::from_nanos(activity);
    row
}
fn me() -> c::Me {
    let mut me = f::me(&[b"Hello", b"Demo"]);
    me.deployment_role = Some(c::HeldRole::Owner);
    me.projects.get_mut(0).unwrap().templates =
        List::from_boxed(Box::new([super::foot::template(jig_client::shapes::CHAT)]));
    me
}
fn inbox() -> c::Inbox {
    c::Inbox {
        party: PARTY,
        needs: List::with_capacity(0),
        updates: List::with_capacity(0),
        next: None,
        cached_needs: 0,
        cached_updates: c::Count { value: 0, limited: false },
        results_seen: 0,
        updates_time_complete: true,
        selection: None,
    }
}
// Home's watches and reads use the same public deliveries as the root's fixtures.
pub(super) fn home(brief: Option<bool>, chats: &[c::TaskLine]) -> Domain {
    home_cards(brief, chats, false)
}
fn home_cards(brief: Option<bool>, chats: &[c::TaskLine], needs: bool) -> Domain {
    home_read(brief, chats, needs, None)
}
pub(super) fn home_read(
    brief: Option<bool>,
    chats: &[c::TaskLine],
    needs: bool,
    playground: Option<(Option<u32>, w::PlaygroundState)>,
) -> Domain {
    home_at(brief, chats, needs, playground, Address::Home { project: PROJECT })
}
pub(super) fn question_root() -> Domain {
    home_at(Some(false), &[], true, None, Address::Chat { task: CHAT })
}
#[expect(clippy::too_many_lines, reason = "one fixture answers the page's watches and reads through public events")]
fn home_at(
    brief: Option<bool>,
    chats: &[c::TaskLine],
    needs: bool,
    playground: Option<(Option<u32>, w::PlaygroundState)>,
    address: Address,
) -> Domain {
    let chat = address == (Address::Chat { task: CHAT });
    let mut playground = playground;
    let mut root = f::signed_out();
    if chat {
        f::event(
            &mut root,
            Event::LookKept {
                look: d::pages::terminal::Look {
                    terminal: true,
                    theme: d::pages::terminal::Theme::Dark,
                    font: d::pages::terminal::Font::DejaVu,
                    night: false,
                },
            },
        );
    }
    let mut out = f::event(
        &mut root,
        Event::Start { address, kept: List::with_capacity(0), with: Some(jig_client::With::Transport) },
    );
    append(&mut out, f::client(&mut root, jig_client::Event::Up { up: Up::Served { served: true, versions: None } }));
    while let Some(request) = out.pop() {
        let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else { continue };
        let up = match down {
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Me) } => {
                Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me()))) }
            }
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Tasks { .. }) } => {
                Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
                        rows: List::from_boxed(chats.into()),
                        more: false,
                    }))),
                }
            }
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Task { task }) } => {
                let mut document = task_document();
                document.chip.task = task;
                document.project = PROJECT;
                document.result = None;
                document.flags.conversation = true;
                document.chip.details.phase = c::ChipPhase::Active(c::RunStage::Idle);
                document.waits_on =
                    c::WaitsOn::Party { whom: c::Whom::Person(PARTY), task, kind: c::WaitedFor::Question, entries: 1 };
                Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(document))) }
            }
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Inbox { .. }) } => {
                let mut document = inbox();
                if chat && needs {
                    document.needs = entries();
                    document.cached_needs = document.needs.len();
                }
                Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(document))) }
            }
            jig_client::Down::Open { watch, subject }
                if Some(&subject) == d::forge::status::watch(PROJECT, &f::limits().forge).as_ref() =>
            {
                let Some(brief) = brief else { continue };
                Up::FamilyDelivered { watch, record: family(wire::project_status(&status(brief))) }
            }
            jig_client::Down::Open { watch, .. } => Up::WatchEnded { watch, end: c::WatchEnd::Unknown },
            jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } }
                if wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read() == &w::Read::Playground =>
            {
                let Some((project, state)) = playground.take() else { continue };
                Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family {
                        family: wire::FORGE,
                        record: Some(wire::playground_answer(
                            &w::PlaygroundAnswer::new(
                                &w::CEILINGS,
                                w::PlaygroundAnswerParts {
                                    state,
                                    project,
                                    forge: wire::FORGE,
                                    provider: 17,
                                    owner: b"demo".as_slice().into(),
                                    name: b"demo".as_slice().into(),
                                    cues: List::with_capacity(0),
                                },
                            )
                            .unwrap(),
                        )),
                    },
                }
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Tasks { .. }) } => {
                Up::ReadDone {
                    read,
                    result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(Box::new(
                        c::TaskPage { live: List::with_capacity(0), live_after: None, ended: c::EndedGroup::None },
                    )))),
                }
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Inbox { group, .. }) } => {
                if brief.is_none() {
                    continue;
                }
                Up::ReadDone {
                    read,
                    result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(Box::new(
                        c::InboxPage {
                            rows: if needs && group == c::InboxReadGroup::Needs {
                                entries()
                            } else {
                                List::with_capacity(0)
                            },
                            cut: false,
                            before: None,
                            after: None,
                            selection: None,
                        },
                    )))),
                }
            }
            jig_client::Down::Send { attempt, request, .. }
                if Some(&request) == d::forge::status::ask(PROJECT, &f::limits().forge).as_ref() =>
            {
                Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Family(family(wire::answer(
                        &w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: w::Outcome::Looked }).unwrap(),
                    )))),
                }
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Question { task, message }) } => {
                let mut chip = task_document().chip;
                chip.task = task;
                chip.title = c::DocumentText::complete(
                    c::Text::new(b"csv-export".as_slice().into(), f::limits().client.domain.text).unwrap(),
                );
                Up::ReadDone {
                    read,
                    result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Question(Box::new(
                        c::Question {
                            task: chip,
                            from: None,
                            message,
                            words: c::DocumentText::complete(
                                c::Text::new(b"comma or semicolon?".as_slice().into(), f::limits().client.domain.text)
                                    .unwrap(),
                            ),
                            since: Wall::EPOCH,
                            acts: List::from_boxed(Box::new([c::Offered {
                                act: c::ActKind::Answer,
                                admission: c::Admission::Allowed,
                                effect: None,
                            }])),
                            ending: None,
                        },
                    )))),
                }
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Escalation { task }) } => {
                let mut document = task_document();
                document.chip.task = task;
                document.chip.title = c::DocumentText::complete(
                    c::Text::new(b"Held task".as_slice().into(), f::limits().client.domain.text).unwrap(),
                );
                document.hold = None;
                Up::ReadDone {
                    read,
                    result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Escalation(Box::new(
                        c::Escalation { task: Box::new(document), acts: List::with_capacity(0) },
                    )))),
                }
            }
            jig_client::Down::Save { .. }
            | jig_client::Down::Erase { .. }
            | jig_client::Down::Send { .. }
            | jig_client::Down::Continue { .. }
            | jig_client::Down::Close { .. }
            | jig_client::Down::Read { .. } => continue,
        };
        append(&mut out, f::client(&mut root, jig_client::Event::Up { up }));
    }
    root
}
fn task_document() -> c::Task {
    let bounds =
        jig_client_protocol_link::Limits::new(f::limits().client.domain, f::limits().client.exchanges).unwrap();
    jig_client_protocol_link::translate::documents::task(jig_client_test_records::task_record(), &bounds).unwrap()
}
fn entries() -> List<c::InboxEntry> {
    let mut entries = List::with_capacity(f::limits().client.domain.rows);
    for (number, object) in
        [(CHAT, c::InboxObject::Question { message: 1 }), (ENDED_CHAT, c::InboxObject::Escalation { revision: 1 })]
    {
        entries
            .push(c::InboxEntry {
                reply: None,
                task: c::TaskLink::Stored { number, status: None },
                project: Some(PROJECT),
                object,
                activity: Some(1),
                at: Some(Wall::EPOCH),
                unread: None,
                cursor: c::InboxCursor::Inbox { task: number, object, at: Some(Wall::EPOCH) },
                projects: List::with_capacity(0),
            })
            .unwrap();
    }
    entries
}
fn append(out: &mut Queue<Request>, mut more: Queue<Request>) {
    while let Some(request) = more.pop() {
        out.push(request);
    }
}
fn build(root: &Domain, tui: &Tui) -> crate::View {
    let mut view = f::view();
    crate::build(&mut view, tui, root, NOW, &mut Queue::uncapped());
    view
}
fn body(tree: &skein_ui::Tree) -> List<Box<[u8]>> {
    let mut result = List::with_capacity(tree.len());
    for region in tree.nodes() {
        if region.region != Some(look::TOP) && region.region != Some(look::SECTIONS) {
            continue;
        }
        let start = f::index(tree, region.id);
        let end = start.checked_add(usize::try_from(region.size).unwrap()).unwrap();
        for node in &tree.nodes()[start..end] {
            if node.role == (Role::Heading { level: 1 }) || node.role == Role::Paragraph || node.role == Role::Alert {
                result.push(f::text(tree, node)).unwrap();
            }
        }
    }
    result
}
#[test]
#[expect(clippy::too_many_lines, reason = "one table checks launchpad read states, their tick and bindings")]
fn the_launchpad_says_each_state() {
    let brief = home(Some(true), &[]);
    let headline = home(Some(false), &[]);
    let unanswered = home(None, &[]);
    let left = home(None, &[line(CHAT, b"Away mode", 1, false), line(ENDED_CHAT, b"Ended", 3, true)]);
    let sorted = home(None, &[line(OLDER_CHAT, b"Older", 0, false), line(CHAT, b"Away mode", 1, false)]);
    let ended = home(None, &[line(ENDED_CHAT, b"Ended", 3, true)]);
    let (start, _) = f::live(Some(f::me(&[b"Hello", b"Demo"])));
    let (none, _) = f::live(Some(f::me(&[])));
    let (before, _) = f::live(None);
    let (starting, _) = f::starting();
    let (mut offline, watch) = f::live(Some(f::me(&[b"Hello"])));
    f::client(&mut offline, jig_client::Event::Up { up: Up::WatchEnded { watch, end: c::WatchEnd::Dropped } });
    let (mut stale, _) = f::live(Some(f::me(&[b"Hello"])));
    f::client(&mut stale, jig_client::Event::Up { up: Up::Served { served: false, versions: None } });
    let signed_out = f::signed_out();
    let mut unkept = Tui::new(f::ADDRESS);
    unkept.step(Told::Kept { written: false });
    let mut signing = Tui::new(f::ADDRESS);
    signing.step(Told::Local { local: Local::Signing });
    let ordinary = Tui::new(f::ADDRESS);
    let needs = home_cards(Some(false), &[], true);
    let (mut second, _) = f::live(Some(f::me(&[b"Hello", b"Demo"])));
    f::event(&mut second, Event::Went { place: d::Place::Address(Address::Home { project: OTHER_PROJECT }) });
    let mut noted = Tui::new(f::ADDRESS);
    noted.step(Told::Unknown { words: Some(b"/unknown".as_slice().into()) });
    let playground = home_read(
        Some(true),
        &[line(CHAT, b"Away mode", 1, false)],
        true,
        Some((Some(PROJECT), w::PlaygroundState::Running)),
    );
    let other_playground = home_read(Some(true), &[], false, Some((Some(OTHER_PROJECT), w::PlaygroundState::Running)));
    let starting_playground = home_read(Some(true), &[], false, Some((Some(PROJECT), w::PlaygroundState::Starting)));
    let header = "temper · Hello127.0.0.1:8367";
    let rows = [
        (
            "the playground running in the project",
            &playground,
            &ordinary,
            "temper · Helloplayground · 127.0.0.1:8367",
            &[
                "This is the playground: a scripted model and in-memory forge. Set temper up in your browser when you're ready to leave it.",
                "Where you left off",
                "Away mode · waiting on you",
                "Needs you",
                "? csv-export: comma or semicolon?",
                "• Held task: Held task",
            ][..],
            None,
        ),
        (
            "the playground in another project",
            &other_playground,
            &ordinary,
            header,
            &[
                "3 changes landed overnight. csv-export is two of three tasks in; one waits on you.",
                "written by the status agent 07:42",
            ][..],
            None,
        ),
        (
            "the playground starting",
            &starting_playground,
            &ordinary,
            header,
            &[
                "3 changes landed overnight. csv-export is two of three tasks in; one waits on you.",
                "written by the status agent 07:42",
            ][..],
            None,
        ),
        (
            "needs read",
            &needs,
            &ordinary,
            header,
            &[
                "The last change landed 2 hours ago. Two things need you.",
                "Needs you",
                "? csv-export: comma or semicolon?",
                "• Held task: Held task",
            ][..],
            Some(Wall::from_nanos(583 * MINUTE)),
        ),
        ("Home project takes precedence", &second, &ordinary, "temper · Demo127.0.0.1:8367", &[][..], None),
        (
            "notes follow sections",
            &left,
            &noted,
            header,
            &["Where you left off", "Away mode · waiting on you", "Unknown command /unknown. /help lists them."][..],
            None,
        ),
        (
            "new brief",
            &brief,
            &ordinary,
            header,
            &[
                "3 changes landed overnight. csv-export is two of three tasks in; one waits on you.",
                "written by the status agent 07:42",
            ][..],
            None,
        ),
        (
            "no brief",
            &headline,
            &ordinary,
            header,
            &["The last change landed 2 hours ago. Nothing needs you."][..],
            Some(Wall::from_nanos(583 * MINUTE)),
        ),
        ("unanswered", &unanswered, &ordinary, header, &[][..], None),
        (
            "left off skips ended and sorts activity",
            &left,
            &ordinary,
            header,
            &["Where you left off", "Away mode · waiting on you"][..],
            None,
        ),
        (
            "left off sorts activity",
            &sorted,
            &ordinary,
            header,
            &["Where you left off", "Away mode · waiting on you", "Older · waiting on you"][..],
            None,
        ),
        ("no live chat", &ended, &ordinary, header, &[][..], None),
        ("start address", &start, &ordinary, header, &[][..], None),
        ("offline cached project", &offline, &ordinary, header, &[][..], None),
        ("before me", &before, &ordinary, "temper", &["Connecting to 127.0.0.1:8367"][..], None),
        ("starting", &starting, &ordinary, "temper", &["Connecting to 127.0.0.1:8367"][..], None),
        ("signing", &signed_out, &signing, "temper", &["Connecting to 127.0.0.1:8367"][..], None),
        (
            "out of date",
            &stale,
            &ordinary,
            "temper",
            &["This temper is out of date: run the temper the engine runs."][..],
            None,
        ),
        (
            "not kept",
            &start,
            &unkept,
            header,
            &["This sign-in could not be kept: the next temper asks for a link again."][..],
            None,
        ),
        (
            "no project",
            &none,
            &ordinary,
            "temper",
            &["No project yet", "Setting temper up makes your first project: open http://127.0.0.1:8367 in a browser."]
                [..],
            None,
        ),
    ];
    let mut failures = List::with_capacity(FAILURES);
    for (name, root, tui, heading, lines, tick) in rows {
        let view = build(root, tui);
        let text = body(view.tree());
        let mut expected = List::with_capacity(LINES);
        expected.push(heading.as_bytes()).unwrap();
        for line in lines {
            expected.push(line.as_bytes()).unwrap();
        }
        let mut good =
            text.len() == expected.len() && view.tick() == tick && skein_ui::outline::check(view.tree()).is_none();
        for (text, expected) in text.iter().zip(&expected) {
            good &= text.as_ref() == *expected;
        }
        let mut bound = 0_u32;
        for node in view.tree().nodes() {
            if node.binding.is_some() {
                bound = bound.checked_add(1).unwrap();
                good &= node.role == Role::TextArea;
            }
            if node.role == Role::Alert {
                good &= node.styles.as_slice() == [look::REFUSED];
            }
        }
        if !good || bound != 1 {
            failures.push(name).unwrap();
        }
    }
    let mut ticking = build(&headline, &ordinary);
    let mut patches = Queue::uncapped();
    crate::build(&mut ticking, &ordinary, &headline, NOW, &mut patches);
    assert!(patches.is_empty(), "an unchanged frame does not rebuild");
    crate::build(&mut ticking, &ordinary, &headline, Wall::from_nanos(583 * MINUTE), &mut patches);
    if ticking.tick() != Some(Wall::from_nanos(584 * MINUTE)) {
        failures.push("the headline did not rebuild at its minute").unwrap();
    }
    for (root, tui, state, style) in [
        (&brief, &ordinary, "● Live", look::LIVE),
        (&before, &ordinary, "● Connecting", look::WAITING),
        (&starting, &ordinary, "● Connecting", look::WAITING),
        (&signed_out, &signing, "● Connecting", look::WAITING),
        (&offline, &ordinary, "● Offline", look::DOWN),
        (&stale, &ordinary, "● Out of date", look::DOWN),
    ] {
        let view = build(root, tui);
        assert_eq!(f::find(view.tree(), Role::Text, state.as_bytes()).styles.as_slice(), &[style]);
    }
    draw_states(&mut failures);
    assert!(failures.is_empty(), "launchpad rows: {failures:?}");
}
fn draw(part: launchpad::Launchpad, columns: u16, needs: u32) -> crate::View {
    let (root, _) = f::live(Some(f::me(if part.project.is_some() { &[b"Hello"] } else { &[] })));
    let mut tui = Tui::new(f::ADDRESS);
    tui.step(Told::Went);
    let mut foot = foot::read(&root, &tui, &f::words(), NOW);
    foot.columns = columns;
    foot.needs = d::pages::needs::Count { count: needs, cut: false };
    foot.closed = if part.project.is_some() { Some("Claude Opus 5.5 · high".as_bytes().into()) } else { None };
    let mut view = f::view();
    view::signed(&mut view, &view::Signed { body: view::Body::Launchpad(part), foot }, &mut Queue::uncapped());
    view
}
fn draw_states(failures: &mut List<&'static str>) {
    // The fixture's two-row inbox cannot deliver six cards; draw that bound from data.
    for (name, count, expected) in [
        ("zero needs", 0_u32, &[][..]),
        ("question", 1, &["? need 0"][..]),
        ("held task", 2, &["? need 0", "• need 1"][..]),
        ("six needs", SIX_NEEDS, &["? need 0", "• need 1", "• need 2", "• need 3", "• need 4"][..]),
    ] {
        let root = home(None, &[]);
        let mut part = launchpad::read(&root, &Tui::new(f::ADDRESS), &f::words(), NOW);
        let mut needs = List::with_capacity(count);
        for number in 0..count {
            needs
                .push(NeedLine {
                    kind: if number == 0 { NeedKind::Question } else { NeedKind::Other },
                    words: skein_lib::bytes::concat(&[b"need ", skein_lib::Decimal::of(u64::from(number)).as_bytes()]),
                })
                .unwrap();
        }
        part.needs = needs.into_boxed();
        let view = draw(part, 80, count);
        let mut lines = List::with_capacity(count);
        let mut heads = 0_usize;
        let mut signals = 0_usize;
        for node in view.tree().nodes() {
            if node.role == Role::Description {
                lines.push(f::text(view.tree(), node)).unwrap();
            }
            if node.styles.as_slice() == [look::HEAD] {
                heads = heads.checked_add(1).unwrap();
            }
            if node.styles.as_slice() == [look::SIGNAL] && f::text(view.tree(), node).len() <= 3 {
                signals = signals.checked_add(1).unwrap();
            }
        }
        let mut good = usize::try_from(lines.len()).unwrap() == expected.len()
            && heads == usize::from(count > 0)
            && signals == expected.len();
        for (line, expected) in lines.iter().zip(expected) {
            good &= line.as_ref() == expected.as_bytes();
        }
        if !good {
            failures.push(name).unwrap();
        }
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the golden table embeds its hand-written frames")]
fn the_launchpads_are_their_goldens() {
    use skein_ui_screen::{Focus, Size};
    const FULL: Size = Size { columns: 80, rows: 24 };
    const NARROW: Size = Size { columns: 40, rows: 12 };
    let rows = [
        (
            "launchpad-80x24-dark",
            Scheme::Dark,
            FULL,
            false,
            include_bytes!("golden/launchpad-80x24-dark.txt").as_slice(),
        ),
        (
            "launchpad-80x24-light",
            Scheme::Light,
            FULL,
            false,
            include_bytes!("golden/launchpad-80x24-light.txt").as_slice(),
        ),
        (
            "launchpad-40x12-dark",
            Scheme::Dark,
            NARROW,
            false,
            include_bytes!("golden/launchpad-40x12-dark.txt").as_slice(),
        ),
        (
            "launchpad-none-80x24-dark",
            Scheme::Dark,
            FULL,
            true,
            include_bytes!("golden/launchpad-none-80x24-dark.txt").as_slice(),
        ),
    ];
    let root = home(Some(true), &[line(CHAT, b"Away mode", 1, false)]);
    let (none, _) = f::live(Some(f::me(&[])));
    let mut views = List::with_capacity(FRAMES);
    for (_, _, size, empty, _) in rows {
        let mut part = launchpad::read(if empty { &none } else { &root }, &Tui::new(f::ADDRESS), &f::words(), NOW);
        if !empty {
            part.project = Some(b"Hello project".as_slice().into());
            part.needs = Box::new([NeedLine {
                kind: NeedKind::Question,
                words: b"csv-export: comma or semicolon?".as_slice().into(),
            }]);
        }
        views.push(draw(part, size.columns, u32::from(!empty))).unwrap();
    }
    let mut checks = List::with_capacity(FRAMES);
    for ((name, scheme, size, _, want), view) in rows.iter().zip(&views) {
        checks
            .push(goldens::Golden {
                background: Some(crate::theme::palette(*scheme)[0].1),
                name,
                tree: view.tree(),
                scheme: *scheme,
                size: *size,
                focus: Focus::default(),
                want,
            })
            .unwrap();
    }
    goldens::check(checks.as_slice());
}

fn left_off_home() -> Domain {
    home(
        None,
        &[
            line(OLDER_CHAT, b"Older", 1, false),
            line(ENDED_CHAT, b"Ended", 5, true),
            line(8, b"Second", 3, false),
            line(CHAT, b"Newest", 4, false),
        ],
    )
}
fn left_off_lines(view: &crate::View) -> List<Box<[u8]>> {
    let mut lines = List::with_capacity(view.tree().len());
    for node in view.tree().nodes() {
        if node.role == Role::Item {
            lines.push(f::text(view.tree(), node)).unwrap();
        }
    }
    lines
}
#[test]
fn the_left_off_chats_are_ordered_and_bounded() {
    for (name, chats, expected) in [
        (
            "activity and ended",
            &[
                line(OLDER_CHAT, b"Older", 1, false),
                line(ENDED_CHAT, b"Ended", 5, true),
                line(8, b"Second", 3, false),
                line(CHAT, b"Newest", 4, false),
            ][..],
            &["Newest", "Second", "Older"][..],
        ),
        (
            "tie and bound",
            &[
                line(12, b"Fourth", 0, false),
                line(11, b"Third", 1, false),
                line(9, b"Higher", 2, false),
                line(8, b"Lower", 2, false),
            ][..],
            &["Lower", "Higher", "Third"][..],
        ),
    ] {
        let root = home(None, chats);
        let part = launchpad::read(&root, &Tui::new(f::ADDRESS), &f::words(), NOW);
        assert_eq!(usize::try_from(part.left_off.len()).unwrap(), expected.len(), "{name} read bound");
        let view = build(&root, &Tui::new(f::ADDRESS));
        let lines = left_off_lines(&view);
        assert_eq!(usize::try_from(lines.len()).unwrap(), expected.len(), "{name}");
        for (line, title) in lines.iter().zip(expected) {
            assert_eq!(
                line.as_ref(),
                skein_lib::bytes::concat(&[title.as_bytes(), " · waiting on you".as_bytes()]).as_ref(),
                "{name}"
            );
        }
        assert_eq!(f::count(view.tree(), Role::List), 1, "{name}");
        for node in view.tree().nodes() {
            if node.role == Role::List {
                assert_eq!(node.styles.as_slice(), &[look::LEFT_OFF], "{name}");
            }
        }
    }
}
#[test]
fn the_left_off_keys_follow_the_last_tree() {
    use crate::{Arrow, Event as E};
    use skein_ui::Act;
    let root = left_off_home();
    let mut tui = Tui::new(f::ADDRESS);
    let mut view = build(&root, &tui);
    for (arrow, task) in [
        (Arrow::Down, CHAT),
        (Arrow::Down, 8),
        (Arrow::Down, OLDER_CHAT),
        (Arrow::Down, OLDER_CHAT),
        (Arrow::Up, 8),
        (Arrow::Up, CHAT),
        (Arrow::Up, CHAT),
    ] {
        assert_eq!(crate::arrow(&view, arrow), Some(E::LeftOff { task: Some(task) }));
        tui.step(Told::LeftOff { task: Some(task) });
        assert_eq!(crate::arrow(&view, arrow), Some(E::LeftOff { task: Some(task) }), "state changes wait for a build");
        crate::build(&mut view, &tui, &root, NOW, &mut Queue::uncapped());
    }
    for (selected, words, list, first, submit, escape) in [
        (None, "", false, Some(CHAT), E::Send, None),
        (
            Some(CHAT),
            "",
            false,
            Some(8),
            E::Act { act: d::pages::Act::Go { address: Address::Chat { task: CHAT } } },
            Some(E::LeftOff { task: None }),
        ),
        (Some(99), "", false, Some(CHAT), E::Send, None),
        (Some(CHAT), "words", false, None, E::Send, Some(E::LeftOff { task: None })),
        (Some(CHAT), "/", false, None, E::Send, Some(E::Dismiss)),
        (
            Some(CHAT),
            "",
            true,
            None,
            E::Act { act: d::pages::Act::Go { address: Address::Chat { task: CHAT } } },
            Some(E::Dismiss),
        ),
    ] {
        let mut root = left_off_home();
        f::event(
            &mut root,
            Event::Act { action: d::Action::Pages(d::pages::Act::ChatWords { words: words.as_bytes().into() }) },
        );
        if list {
            f::event(
                &mut root,
                Event::Act {
                    action: d::Action::Pages(d::pages::Act::Terminal(d::pages::terminal::Act::Run {
                        command: d::pages::terminal::Command::Chats,
                    })),
                },
            );
        }
        tui.step(Told::LeftOff { task: selected });
        let view = build(&root, &tui);
        match first {
            Some(task) => assert_eq!(
                crate::arrow(&view, Arrow::Down),
                Some(E::LeftOff { task: Some(task) }),
                "{selected:?} {words} {list}"
            ),
            None => {
                assert_eq!(crate::arrow(&view, Arrow::Down), None, "{selected:?} {words} {list}");
                assert_eq!(crate::arrow(&view, Arrow::Up), None);
            }
        }
        if selected.is_none() || selected == Some(99) {
            assert_eq!(crate::arrow(&view, Arrow::Up), Some(E::LeftOff { task: Some(CHAT) }));
        }
        let field = f::find(view.tree(), Role::TextArea, b"Ask temper");
        assert_eq!(crate::event(&view, field.id, Act::Submit), Some(submit), "{selected:?} {words} {list}");
        assert_eq!(crate::escape(&view), escape, "{selected:?} {words} {list}");
    }
    // Reusing a view clears the launchpad's key facts when its next tree has none.
    let empty = home(None, &[]);
    crate::build(&mut view, &tui, &empty, NOW, &mut Queue::uncapped());
    assert_eq!(crate::arrow(&view, Arrow::Down), None);
    assert_eq!(crate::escape(&view), None);
    let mut chat = left_off_home();
    f::event(&mut chat, Event::Went { place: d::Place::Address(Address::Chat { task: CHAT }) });
    tui.step(Told::LeftOff { task: Some(CHAT) });
    crate::build(&mut view, &tui, &chat, NOW, &mut Queue::uncapped());
    assert_eq!(crate::arrow(&view, Arrow::Down), None);
    assert_eq!(crate::escape(&view), None);
    tui.step(Told::Went);
    crate::build(&mut view, &tui, &f::signed_out(), NOW, &mut Queue::uncapped());
    assert_eq!(crate::arrow(&view, Arrow::Up), None);
    assert_eq!(crate::escape(&view), None);
}
#[test]
fn the_picked_left_off_row_has_its_mark_and_paint() {
    use skein_ui_screen::{Focus, Frame, Rgb, Size, render};
    let root = left_off_home();
    for (selected, background) in
        [(Some(CHAT), Some(Rgb { red: 20, green: 30, blue: 40 })), (Some(CHAT), None), (None, None), (Some(99), None)]
    {
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::LeftOff { task: selected });
        let view = build(&root, &tui);
        let mut picked = 0_usize;
        for node in view.tree().nodes() {
            if node.styles.as_slice().contains(&look::PICKED) {
                assert_eq!(node.role, Role::Item);
                picked = picked.checked_add(1).unwrap();
            }
        }
        assert_eq!(picked, usize::from(selected == Some(CHAT)));
        let size = Size { columns: 80, rows: 24 };
        let mut frame = Frame::new(size);
        render(view.tree(), &crate::theme(Scheme::Dark, background), size, &mut Focus::default(), &mut frame);
        let mut picked_row = None;
        for row in 0..size.rows {
            let cells = frame.row(row).unwrap();
            if cells[3].glyph == 'N' {
                picked_row = Some(cells);
                break;
            }
        }
        let row = picked_row.unwrap();
        let band = match (selected, background) {
            (Some(CHAT), Some(background)) => Some(crate::theme::band(background)),
            (Some(_) | None, Some(_) | None) => None,
        };
        assert_eq!(row[1].glyph, if selected == Some(CHAT) { '❯' } else { ' ' });
        assert_eq!(
            row[3].paint.fore,
            if selected == Some(CHAT) { Some(Rgb { red: 0xe0, green: 0x8a, blue: 0x68 }) } else { None }
        );
        for cell in &row[1..79] {
            assert_eq!(cell.paint.back, band);
        }
        assert_eq!(row[0].paint.back, None);
        assert_eq!(row[79].paint.back, None);
        assert_eq!(row[12].paint.fore, Some(Rgb { red: 0x9a, green: 0x93, blue: 0x88 }));
    }
}
