//! The launchpad's states and hand-written frames (temper/docs/design/client/tui/pages/launchpad.md and temper/docs/design/client/tui/theme.md).
#[path = "launchpad/team.rs"]
pub(super) mod team;
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
use temper_client_view::shared::NeedMark;

const PROJECT: u32 = 1;
const OTHER_PROJECT: u32 = 2;
const PARTY: u64 = 73;
const CHAT: u64 = 9;
const ENDED_CHAT: u64 = 10;
const OLDER_CHAT: u64 = 11;
// Read and draw states share one bounded failure list.
const FAILURES: u32 = 25;
// The longest state includes the header, rows, Start and notes.
const LINES: u32 = 16;
// Nine review frames; six needs are one past the design's five-line bound.
const FRAMES: u32 = 9;
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
    me.party.number = PARTY;
    me.deployment_role = Some(c::HeldRole::Owner);
    let mut goal = super::foot::template(jig_client::shapes::GOAL);
    goal.label = 8;
    me.projects.get_mut(0).unwrap().templates =
        List::from_boxed(Box::new([super::foot::template(jig_client::shapes::CHAT), goal, {
            let mut draft = super::foot::template(d::pages::chats::DRAFTING_TEMPLATE);
            draft.executor = c::TemplateExecutor::Charter(12);
            draft
        }]));
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
fn home_at(
    brief: Option<bool>,
    chats: &[c::TaskLine],
    needs: bool,
    playground: Option<(Option<u32>, w::PlaygroundState)>,
    address: Address,
) -> Domain {
    home_rows(brief, chats, &[], needs, playground, address, LEDE.as_bytes())
}
fn home_rows(
    brief: Option<bool>,
    chats: &[c::TaskLine],
    goals: &[c::TaskLine],
    needs: bool,
    playground: Option<(Option<u32>, w::PlaygroundState)>,
    address: Address,
    lede: &[u8],
) -> Domain {
    home_projects(brief, chats, goals, needs, playground, address, lede, false)
}
#[expect(clippy::too_many_arguments, reason = "Home's independent reads")]
fn home_projects(
    brief: Option<bool>,
    chats: &[c::TaskLine],
    goals: &[c::TaskLine],
    needs: bool,
    playground: Option<(Option<u32>, w::PlaygroundState)>,
    address: Address,
    lede: &[u8],
    across: bool,
) -> Domain {
    home_team(brief, chats, goals, needs, playground, address, lede, across, None)
}
#[expect(clippy::too_many_arguments, reason = "the fixture supplies independent public reads for two projects")]
#[expect(clippy::too_many_lines, reason = "the fixture supplies Home's independent public reads and brief")]
fn home_team(
    brief: Option<bool>,
    chats: &[c::TaskLine],
    goals: &[c::TaskLine],
    needs: bool,
    playground: Option<(Option<u32>, w::PlaygroundState)>,
    address: Address,
    lede: &[u8],
    across: bool,
    team: Option<team::Fixture<'_>>,
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
    f::event(&mut root, Event::Launchpad);
    let mut out = f::event(
        &mut root,
        Event::Start { address, kept: List::with_capacity(0), with: Some(jig_client::With::Transport) },
    );
    append(&mut out, f::client(&mut root, jig_client::Event::Up { up: Up::Served { served: true, versions: None } }));
    while let Some(request) = out.pop() {
        let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else { continue };
        let up = match down {
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Me) } => {
                let mut me = me();
                if across {
                    me.projects.get_mut(1).unwrap().name = c::DocumentText::complete(
                        c::Text::new(b"shop".as_slice().into(), f::limits().client.domain.text).unwrap(),
                    );
                }
                Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) }
            }
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Tasks { query, .. }) }
                if query.label == Some(3) =>
            {
                let Some(team) = &team else { continue };
                let Some(roles) = team.roles else { continue };
                Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(team::roles(roles)))) }
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Project { .. }) }
                if team.is_some() =>
            {
                Up::ReadDone {
                    read,
                    result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Project(Box::new(
                        team::project(),
                    )))),
                }
            }
            jig_client::Down::Open { watch, subject }
                if team.is_some()
                    && Some(&subject) == d::forge::findings::watch(PROJECT, &f::limits().forge).as_ref() =>
            {
                Up::FamilyDelivered { watch, record: family(wire::findings(&team::findings(team.as_ref().unwrap()))) }
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Notes { scope, .. }) }
                if team.is_some() =>
            {
                Up::ReadDone {
                    read,
                    result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(Box::new(team::now(
                        scope,
                    ))))),
                }
            }
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Tasks { query, .. }) } => {
                let mut rows = List::with_capacity(u32::try_from(chats.len().max(goals.len())).unwrap());
                for row in if query.tracked == Some(true) { goals } else { chats } {
                    if query.project.is_none() || query.project == Some(row.project) {
                        rows.push(row.clone()).unwrap();
                    }
                }
                Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks { rows, more: false }))),
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
                if brief.is_none() && !chat {
                    continue;
                }
                let mut document = inbox();
                if needs {
                    document.needs = if team.is_some() { team::entries() } else { entries() };
                    if across {
                        for (index, entry) in document.needs.iter_mut().enumerate() {
                            if !chat || index == 1 {
                                entry.project = Some(OTHER_PROJECT);
                            }
                        }
                    }
                    document.cached_needs = document.needs.len();
                }
                Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(document))) }
            }
            jig_client::Down::Open { watch, subject }
                if Some(&subject) == d::forge::status::watch(PROJECT, &f::limits().forge).as_ref()
                    || (across
                        && Some(&subject) == d::forge::status::watch(OTHER_PROJECT, &f::limits().forge).as_ref()) =>
            {
                let Some(brief) = brief else { continue };
                let status = status(brief);
                let mut parts = status.into_parts();
                let other = Some(&subject) == d::forge::status::watch(OTHER_PROJECT, &f::limits().forge).as_ref();
                if other {
                    parts.project = OTHER_PROJECT;
                    parts.landed = 1;
                    parts.seen = u64::from(!brief);
                }
                if let Some(brief) = parts.brief {
                    let mut brief = brief.into_parts();
                    brief.lede =
                        if other { b"Coupons are two of three in. One awaits review.".as_slice() } else { lede }.into();
                    if other {
                        brief.sentence = b"One change landed in shop.".as_slice().into();
                        brief.at = 480 * MINUTE;
                        brief.landed = 1;
                    }
                    parts.brief = Some(w::StatusBrief::new(&w::CEILINGS, brief).unwrap());
                }
                let record = family(wire::project_status(&w::ProjectStatus::new(&w::CEILINGS, parts).unwrap()));
                assert!(d::forge::status::document(&record, f::limits().forge).is_some(), "status decodes");
                Up::FamilyDelivered { watch, record }
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
            jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } }
                if f::ways_read(&record.bytes) =>
            {
                Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family { family: wire::FORGE, record: Some(f::draft_setting(true)) },
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
                                let mut entries = if team.is_some() { team::entries() } else { entries() };
                                if across {
                                    for entry in entries.iter_mut() {
                                        entry.project = Some(OTHER_PROJECT);
                                    }
                                }
                                entries
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
                if Some(&request) == d::forge::status::ask(PROJECT, &f::limits().forge).as_ref()
                    || (across
                        && Some(&request) == d::forge::status::ask(OTHER_PROJECT, &f::limits().forge).as_ref()) =>
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
                let text = f::text(tree, node);
                if !text.is_empty() {
                    result.push(text).unwrap();
                }
            }
        }
    }
    result
}
#[expect(
    clippy::disallowed_methods,
    reason = "read-state rows ignore padding; the room table and goldens check its exact columns"
)]
fn same_words(a: &[u8], b: &[u8]) -> bool {
    a.split(|byte| *byte == b' ')
        .filter(|word| !word.is_empty())
        .eq(b.split(|byte| *byte == b' ').filter(|word| !word.is_empty()))
}
fn start_lines(lines: &mut List<&'static [u8]>, project: Option<&[u8]>, drafts: bool) {
    if let Some(project) = project {
        lines.push(b"Start").unwrap();
        if drafts {
            lines.push(b"/draft    a design session that ends in a draft and a goal").unwrap();
        }
        lines
            .push(if project == b"Demo" {
                b"/change   one change, run as Demo's way says"
            } else {
                b"/change   one change, run as Hello's way says"
            })
            .unwrap();
        lines.push(b"/ask      Ask temper").unwrap();
    }
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
    let mut session_line = line(CHAT, b"Away mode", 1, false);
    session_line.chip.executor = c::ChipExecutor::Agent { charter: 12 };
    let session = home(None, &[session_line]);
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
    noted.step(Told::Notice { notice: Some(crate::Notice::Unknown { words: b"/unknown".as_slice().into() }) });
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
            "design session",
            &session,
            &ordinary,
            header,
            &["Where you left off", "Hello   Away mode, a design session · today 00:00 · waiting on you  /chats 1"][..],
            Some(Wall::from_nanos(1440 * MINUTE)),
        ),
        (
            "the playground running in the project",
            &playground,
            &ordinary,
            "temper · Helloplayground · 127.0.0.1:8367",
            &[
                "This is the playground: a scripted model and in-memory forge. Set temper up in your browser when you're ready to leave it.",
                "Where you left off",
                "Hello   Away mode, a chat · today 00:00 · waiting on you  /chats 1",
                "Needs you",
                "? Hello   csv-export: comma or semicolon? · asked 9 h 42 min ago",
                "• Hello   Held task: Held task  n walks them",
            ][..],
            Some(Wall::from_nanos(583 * MINUTE)),
        ),
        (
            "the playground in another project",
            &other_playground,
            &ordinary,
            header,
            &[
                "3 changes landed in Hello. Nothing needs you.",
                "Hello: csv-export is two of three tasks in; one waits on you.",
                "written by the status agent 07:42 · /brief",
            ][..],
            Some(Wall::from_nanos(1440 * MINUTE)),
        ),
        (
            "the playground starting",
            &starting_playground,
            &ordinary,
            header,
            &[
                "3 changes landed in Hello. Nothing needs you.",
                "Hello: csv-export is two of three tasks in; one waits on you.",
                "written by the status agent 07:42 · /brief",
            ][..],
            Some(Wall::from_nanos(1440 * MINUTE)),
        ),
        (
            "needs read",
            &needs,
            &ordinary,
            header,
            &[
                "The last change landed 2 hours ago. Two things need you.",
                "Needs you",
                "? Hello   csv-export: comma or semicolon? · asked 9 h 42 min ago",
                "• Hello   Held task: Held task  n walks them",
            ][..],
            Some(Wall::from_nanos(583 * MINUTE)),
        ),
        ("Home project takes precedence", &second, &ordinary, "temper · Demo127.0.0.1:8367", &[][..], None),
        (
            "notes follow sections",
            &left,
            &noted,
            header,
            &["Where you left off", "Hello   Away mode, a chat · today 00:00 · waiting on you  /chats 1"][..],
            Some(Wall::from_nanos(1440 * MINUTE)),
        ),
        (
            "new brief",
            &brief,
            &ordinary,
            header,
            &[
                "3 changes landed in Hello. Nothing needs you.",
                "Hello: csv-export is two of three tasks in; one waits on you.",
                "written by the status agent 07:42 · /brief",
            ][..],
            Some(Wall::from_nanos(1440 * MINUTE)),
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
            &["Where you left off", "Hello   Away mode, a chat · today 00:00 · waiting on you  /chats 1"][..],
            Some(Wall::from_nanos(1440 * MINUTE)),
        ),
        (
            "left off sorts activity",
            &sorted,
            &ordinary,
            header,
            &[
                "Where you left off",
                "Hello   Away mode, a chat · today 00:00 · waiting on you  /chats 1",
                "Hello   Older, a chat · today 00:00 · waiting on you  /chats 2",
            ][..],
            Some(Wall::from_nanos(1440 * MINUTE)),
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
        let part = launchpad::read(root, tui, &f::words(), NOW);
        if name == "start address" {
            assert!(!part.drafts, "drafts off");
        } else if name == "new brief" {
            assert!(part.drafts, "drafts on");
        }
        for line in lines {
            expected.push(line.as_bytes()).unwrap();
        }
        start_lines(&mut expected, part.project.as_deref(), part.drafts);
        if tui.notice().is_some() {
            let notice = f::find(view.tree(), Role::Text, "Unknown command /unknown · /help lists them".as_bytes());
            assert_eq!(notice.styles.as_slice(), &[look::SIGNAL]);
        }
        let mut good =
            text.len() == expected.len() && view.tick() == tick && skein_ui::outline::check(view.tree()).is_none();
        for (text, expected) in text.iter().zip(&expected) {
            good &= same_words(text, expected);
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
    let mut tui = Tui::new(f::ADDRESS);
    tui.step(Told::Brief);
    let part = launchpad::read(&brief, &tui, &f::words(), NOW);
    assert_eq!(part.lines.get(2).unwrap().style, Some(look::BYLINE));
    assert_eq!(part.brief.len(), 3);
    for (line, expected) in part.brief.iter().zip([
        b"Hello: 3 changes landed overnight.".as_slice(),
        LEDE.as_bytes(),
        b"written by the status agent 07:42",
    ]) {
        assert_eq!(line.words.as_ref(), expected);
        assert_eq!(line.style, Some(look::NOTE));
    }
    draw_states(&mut failures);
    assert!(failures.is_empty(), "launchpad rows: {failures:?}");
}
fn draw(mut part: launchpad::Launchpad, columns: u16, needs: u32) -> crate::View {
    part.columns = columns;
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
    // Money, sign-off and proposal cards, and the six-card bound, are drawn from data.
    for (name, count, expected) in [
        ("zero needs", 0_u32, &[][..]),
        ("question", 1, &["? Hello need 0 n walks them"][..]),
        ("money", 2, &["? Hello need 0", "$ Hello need 1 n walks them"][..]),
        (
            "six needs",
            SIX_NEEDS,
            &["? Hello need 0", "$ Hello need 1", "✓ Hello need 2", "• Hello need 3", "• Hello need 4 n walks them"][..],
        ),
    ] {
        let root = home(None, &[]);
        let mut part = launchpad::read(&root, &Tui::new(f::ADDRESS), &f::words(), NOW);
        let mut needs = List::with_capacity(count);
        for number in 0..count {
            needs
                .push(launchpad::NeedRow {
                    since: None,
                    mark: match number {
                        0 => NeedMark::Question,
                        1 => NeedMark::Money,
                        2 => NeedMark::SignOff,
                        _ => NeedMark::Other,
                    },
                    project: b"Hello".as_slice().into(),
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
            if node.role == Role::Description
                && !f::text(view.tree(), node).starts_with(b"/draft")
                && !f::text(view.tree(), node).starts_with(b"/change")
                && !f::text(view.tree(), node).starts_with(b"/ask")
            {
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
            && heads == usize::from(count > 0).checked_add(1).unwrap()
            && signals == expected.len();
        for (line, expected) in lines.iter().zip(expected) {
            good &= same_words(line, expected.as_bytes());
        }
        if !good {
            failures.push(name).unwrap();
        }
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the golden table embeds its hand-written frames")]
#[expect(clippy::too_many_lines, reason = "one table holds six launchpad review frames and their shared inputs")]
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
            "launchpad-110x32-dark",
            Scheme::Dark,
            Size { columns: 110, rows: 32 },
            false,
            include_bytes!("golden/launchpad-110x32-dark.txt").as_slice(),
        ),
        (
            "launchpad-brief-80x24-dark",
            Scheme::Dark,
            FULL,
            false,
            include_bytes!("golden/launchpad-brief-80x24-dark.txt").as_slice(),
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
        (
            "launchpad-team-80x24-dark",
            Scheme::Dark,
            FULL,
            false,
            include_bytes!("golden/launchpad-team-80x24-dark.txt").as_slice(),
        ),
        (
            "launchpad-team-80x24-light",
            Scheme::Light,
            FULL,
            false,
            include_bytes!("golden/launchpad-team-80x24-light.txt").as_slice(),
        ),
        (
            "launchpad-team-110x32-dark",
            Scheme::Dark,
            Size { columns: 110, rows: 32 },
            false,
            include_bytes!("golden/launchpad-team-110x32-dark.txt").as_slice(),
        ),
    ];
    let mut goal = goal(7, BRIEF_AT, PARTY, false);
    goal.waits_on = c::WaitsOn::Nothing;
    goal.chip.phase = c::Phase::Running;
    goal.chip.details.phase = c::ChipPhase::Active(c::RunStage::Running { attempt: 1 });
    goal.project = OTHER_PROJECT;
    let mut chat = line(CHAT, b"Away mode", 492 * MINUTE, false);
    chat.project = OTHER_PROJECT;
    let root = home_projects(
        Some(true),
        &[chat],
        &[goal],
        false,
        None,
        Address::Home { project: PROJECT },
        LEDE.as_bytes(),
        true,
    );
    let team_root = team::root(team::Fixture { roles: Some(&[b"Prover", b"Triager"]), areas: true, you: true });
    let (none, _) = f::live(Some(f::me(&[])));
    let mut views = List::with_capacity(FRAMES);
    for (name, _, size, empty, _) in rows {
        let mut tui = Tui::new(f::ADDRESS);
        if name == "launchpad-brief-80x24-dark" {
            tui.step(Told::Brief);
        }
        let mut part = launchpad::read(
            if empty {
                &none
            } else if name.starts_with("launchpad-team-") {
                &team_root
            } else {
                &root
            },
            &tui,
            &f::words(),
            NOW,
        );
        if !empty && !name.starts_with("launchpad-team-") {
            part.needs = Box::new([launchpad::NeedRow {
                since: Some(Wall::from_nanos(562 * MINUTE)),
                mark: NeedMark::Question,
                project: b"Hello".as_slice().into(),
                words: b"csv-export: comma or semicolon?".as_slice().into(),
            }]);
            if size.columns == 110 {
                part.needs = Box::new([
                    launchpad::NeedRow {
                        since: Some(Wall::from_nanos(562 * MINUTE)),
                        mark: NeedMark::Question,
                        project: b"Hello".as_slice().into(),
                        words: b"csv-export: comma or semicolon?".as_slice().into(),
                    },
                    launchpad::NeedRow {
                        since: Some(Wall::from_nanos(562 * MINUTE)),
                        mark: NeedMark::Money,
                        project: b"shop".as_slice().into(),
                        words: b"Porter: raise its weekly budget?".as_slice().into(),
                    },
                    launchpad::NeedRow {
                        since: Some(Wall::from_nanos(562 * MINUTE)),
                        mark: NeedMark::SignOff,
                        project: b"Hello".as_slice().into(),
                        words: b"Monthly totals: ready for review".as_slice().into(),
                    },
                    launchpad::NeedRow {
                        since: Some(Wall::from_nanos(562 * MINUTE)),
                        mark: NeedMark::Other,
                        project: b"shop".as_slice().into(),
                        words: b"Proposal: review the new way".as_slice().into(),
                    },
                ]);
            }
        }
        let count = u32::try_from(part.needs.len()).unwrap();
        if !empty && !name.starts_with("launchpad-team-") {
            let mut facts = d::pages::all_home::facts(&root, NOW);
            facts.needs = d::pages::home::Needs::Count { count, cut: false };
            part.lines.get_mut(0).unwrap().words =
                temper_client_view::shared::all_home_headline(f::words().view(), &facts, NOW).0;
        }
        views.push(draw(part, size.columns, count)).unwrap();
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

fn goal(task: u64, activity: u64, requester: u64, ended: bool) -> c::TaskLine {
    let mut row = line(task, b"csv-export", activity, ended);
    row.requester = c::Requester::Person(requester);
    row.tracked = Some(1);
    row.label = 8;
    row.flags.conversation = false;
    row
}
fn goals_home(chats: &[c::TaskLine], goals: &[c::TaskLine]) -> Domain {
    home_rows(None, chats, goals, false, None, Address::Home { project: PROJECT }, LEDE.as_bytes())
}

#[test]
#[expect(clippy::disallowed_methods, reason = "row diagnostics count UTF-8 columns and check the painted frame")]
fn rows_keep_their_hint_in_the_room() {
    use skein_ui_screen::{Focus, Frame, Size, render};
    for (columns, title, hint, cut) in [
        (80, "Away mode", true, false),
        (80, "A long title with many words that would cross the end of the row and hide its hint", true, true),
        (40, "Away mode", false, true),
        (110, "A longer title", true, false),
        (80, "éééééééééééééééééééééééééééééééééééééééééééééééééééééééééééé", true, true),
    ] {
        let root = home(None, &[line(CHAT, b"Away mode", BRIEF_AT, false)]);
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Size { columns });
        let mut part = launchpad::read(&root, &tui, &f::words(), NOW);
        part.left_off.get_mut(0).unwrap().title = title.as_bytes().into();
        part.needs = Box::new([launchpad::NeedRow {
            since: None,
            mark: NeedMark::Question,
            project: b"shop".as_slice().into(),
            words: title.as_bytes().into(),
        }]);
        let view = draw(part, columns, 1);
        let left = left_off_lines(&view).swap_remove(0).unwrap();
        assert_eq!(left.ends_with(b"/chats 1"), hint, "{columns} {title}");
        assert_eq!(core::str::from_utf8(&left).unwrap().contains('…'), cut, "{columns} {title}");
        assert!(core::str::from_utf8(&left).unwrap().chars().count() <= usize::from(columns - 4));
        if hint {
            assert_eq!(core::str::from_utf8(&left).unwrap().chars().count(), usize::from(columns - 4));
        }
        let need = view.tree().nodes().iter().find(|node| node.role == Role::Description).unwrap();
        assert_eq!(f::text(view.tree(), need).ends_with(b"n walks them"), hint, "need at {columns}");
        let size = Size { columns, rows: 32 };
        let mut frame = Frame::new(size);
        render(view.tree(), &crate::theme(Scheme::Dark, None), size, &mut Focus::default(), &mut frame);
        let printed = frame.print();
        let printed = core::str::from_utf8(&printed).unwrap();
        if hint {
            assert!(printed.lines().any(|line| line.trim_end().ends_with("/chats 1")));
        }
    }
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
#[expect(clippy::too_many_lines, reason = "one table checks chat ordering and the goal at each boundary")]
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
            &[("Newest", "today 00:00"), ("Second", "today 00:00"), ("Older", "today 00:00")][..],
        ),
        (
            "tie and bound",
            &[
                line(12, b"Fourth", 0, false),
                line(11, b"Third", 1, false),
                line(9, b"Higher", 2, false),
                line(8, b"Lower", 2, false),
            ][..],
            &[("Lower", "today 00:00"), ("Higher", "today 00:00"), ("Third", "today 00:00")][..],
        ),
        ("clock", &[line(CHAT, b"Morning", BRIEF_AT, false)][..], &[("Morning", "today 07:42")][..]),
    ] {
        let root = home(None, chats);
        let part = launchpad::read(&root, &Tui::new(f::ADDRESS), &f::words(), NOW);
        assert_eq!(usize::try_from(part.left_off.len()).unwrap(), expected.len(), "{name} read bound");
        for (index, (row, (_, at))) in part.left_off.iter().zip(expected).enumerate() {
            assert_eq!(
                row.kind,
                launchpad::LeftKind::Chat { number: u32::try_from(index + 1).unwrap(), session: false },
                "{name}"
            );
            assert_eq!(row.project.as_ref(), b"Hello", "{name}");
            assert_eq!(row.at.as_ref(), at.as_bytes(), "{name}");
        }
        let view = build(&root, &Tui::new(f::ADDRESS));
        let lines = left_off_lines(&view);
        assert_eq!(usize::try_from(lines.len()).unwrap(), expected.len(), "{name}");
        for (line, (title, _)) in lines.iter().zip(expected) {
            assert!(
                f::text(view.tree(), f::find(view.tree(), Role::Text, title.as_bytes())).as_ref() == title.as_bytes(),
                "{name}"
            );
            assert!(line.starts_with(b"Hello   "), "{name}");
        }
        assert_eq!(f::count(view.tree(), Role::List), 1, "{name}");
        for node in view.tree().nodes() {
            if node.role == Role::List {
                assert_eq!(node.styles.as_slice(), &[look::LEFT_OFF], "{name}");
            }
        }
    }
    for (name, activity, expected) in [
        (
            "goal newest",
            5,
            &[
                (7, launchpad::LeftKind::Goal),
                (9, launchpad::LeftKind::Chat { number: 1, session: false }),
                (8, launchpad::LeftKind::Chat { number: 2, session: false }),
            ][..],
        ),
        (
            "goal between chats",
            3,
            &[
                (9, launchpad::LeftKind::Chat { number: 1, session: false }),
                (7, launchpad::LeftKind::Goal),
                (8, launchpad::LeftKind::Chat { number: 2, session: false }),
            ][..],
        ),
        (
            "goal oldest",
            1,
            &[
                (9, launchpad::LeftKind::Chat { number: 1, session: false }),
                (8, launchpad::LeftKind::Chat { number: 2, session: false }),
                (7, launchpad::LeftKind::Goal),
            ][..],
        ),
        (
            "goal fourth",
            0,
            &[
                (9, launchpad::LeftKind::Chat { number: 1, session: false }),
                (8, launchpad::LeftKind::Chat { number: 2, session: false }),
                (11, launchpad::LeftKind::Chat { number: 3, session: false }),
            ][..],
        ),
        (
            "goal tie",
            4,
            &[
                (7, launchpad::LeftKind::Goal),
                (9, launchpad::LeftKind::Chat { number: 1, session: false }),
                (8, launchpad::LeftKind::Chat { number: 2, session: false }),
            ][..],
        ),
    ] {
        let chats = [line(9, b"Newest", 4, false), line(8, b"Second", 2, false), line(11, b"Third", 1, false)];
        let root = goals_home(
            &chats,
            &[
                goal(7, activity, PARTY, false),
                goal(6, 9, PARTY + 1, false),
                goal(5, 10, PARTY, true),
                goal(20, 0, PARTY, false),
            ],
        );
        assert!(root.pages().goals().is_some(), "fixture reads goals");
        let part = launchpad::read(&root, &Tui::new(f::ADDRESS), &f::words(), NOW);
        assert_eq!(usize::try_from(part.left_off.len()).unwrap(), expected.len(), "{name}");
        for (row, (task, kind)) in part.left_off.iter().zip(expected) {
            assert_eq!((row.task, row.kind), (*task, *kind), "{name}");
            assert_eq!(row.project.as_ref(), b"Hello");
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
            if cells[11].glyph == 'N' {
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
            row[11].paint.fore,
            if selected == Some(CHAT) { Some(Rgb { red: 0xe0, green: 0x8a, blue: 0x68 }) } else { None }
        );
        for cell in &row[1..79] {
            assert_eq!(cell.paint.back, band);
        }
        assert_eq!(row[0].paint.back, None);
        assert_eq!(row[79].paint.back, None);
        assert_eq!(row[18].paint.fore, Some(Rgb { red: 0x9a, green: 0x93, blue: 0x88 }));
    }
}

#[test]
fn the_launchpad_surface_follows_the_displayed_facts() {
    use crate::{Event as E, Own};
    let mut ordinary = Tui::new(f::ADDRESS);
    for (name, page, words, list, expected) in [
        ("need", SurfacePage::Home { brief: true, needs: true }, "", false, Some(E::Own { command: Own::Needs })),
        ("words", SurfacePage::Home { brief: true, needs: true }, "words", false, None),
        ("list", SurfacePage::Home { brief: true, needs: true }, "", true, None),
        ("no need", SurfacePage::Home { brief: false, needs: false }, "", false, None),
        ("chat", SurfacePage::Chat, "", false, None),
    ] {
        let mut root = surface_root(page);
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
        let view = build(&root, &ordinary);
        assert_eq!(crate::walk_key(&view), expected, "{name}");
    }
    let root = home(None, &[line(OLDER_CHAT, b"Older", 1, false), line(CHAT, b"Newest", 2, false)]);
    let view = build(&root, &ordinary);
    for (number, expected) in [(0, None), (1, Some(CHAT)), (2, Some(OLDER_CHAT)), (3, None)] {
        assert_eq!(crate::chat_number(&view, number), expected, "chat {number}");
    }
    for (name, page, selected, expected) in [
        (
            "chat selected",
            SurfacePage::Home { brief: false, needs: false },
            Some(CHAT),
            Some(b"http://127.0.0.1:8367/chats/9".as_slice()),
        ),
        (
            "home",
            SurfacePage::Home { brief: false, needs: false },
            None,
            Some(b"http://127.0.0.1:8367/projects/1".as_slice()),
        ),
        (
            "stale selection",
            SurfacePage::Home { brief: false, needs: false },
            Some(ENDED_CHAT),
            Some(b"http://127.0.0.1:8367/projects/1".as_slice()),
        ),
        ("chat page", SurfacePage::Chat, None, Some(b"http://127.0.0.1:8367/chats/9".as_slice())),
        ("signed out", SurfacePage::SignedOut, None, None),
    ] {
        let root = surface_root(page);
        ordinary.step(Told::LeftOff { task: selected });
        assert_eq!(crate::focus(&build(&root, &ordinary)), expected, "{name}");
    }
    // The reader does not read goals yet; a goal is supplied as launchpad data.
    let mut tui = Tui::new(f::ADDRESS);
    tui.step(Told::LeftOff { task: Some(CHAT) });
    let mut part = launchpad::read(&root, &tui, &f::words(), NOW);
    part.left_off.get_mut(0).unwrap().kind = launchpad::LeftKind::Goal;
    let mut view = build(&root, &tui);
    let foot = foot::read(&root, &tui, &f::words(), NOW);
    view::signed(&mut view, &view::Signed { body: view::Body::Launchpad(part), foot }, &mut Queue::uncapped());
    assert_eq!(crate::focus(&view), Some(b"http://127.0.0.1:8367/tasks/9".as_slice()), "goal selected");
    assert_eq!(crate::chat_number(&view, 1), None, "goals have no chat number");
    let field = f::find(view.tree(), Role::TextArea, b"Ask temper");
    assert_eq!(
        crate::event(&view, field.id, skein_ui::Act::Submit),
        Some(E::Open { address: Box::from(*b"http://127.0.0.1:8367/tasks/9") })
    );
    for (name, page, expected) in [
        ("brief and need", SurfacePage::Home { brief: true, needs: true }, &[Own::Draft, Own::Change, Own::Ask, Own::Brief, Own::Needs, Own::Open, Own::Tell, Own::Theme][..]),
        (
            "neither",
            SurfacePage::Home { brief: false, needs: false },
            &[Own::Draft, Own::Change, Own::Ask, Own::Open, Own::Tell, Own::Theme][..],
        ),
        ("chat", SurfacePage::Chat, &[Own::Open, Own::Tell, Own::Theme][..]),
        ("no project", SurfacePage::NoProject, &[Own::Theme][..]),
        ("signed out", SurfacePage::SignedOut, &[Own::Theme][..]),
    ] {
        let root = surface_root(page);
        assert_eq!(crate::offered_own(&root, &tui).as_ref(), expected, "{name}");
    }
    // Facts used by the keys and focus are replaced when a view is reused.
    crate::build(&mut view, &tui, &f::signed_out(), NOW, &mut Queue::uncapped());
    assert_eq!(crate::focus(&view), None);
    assert_eq!(crate::walk_key(&view), None);
    assert_eq!(crate::chat_number(&view, 2), None);
}

#[derive(Clone, Copy)]
enum SurfacePage {
    Home { brief: bool, needs: bool },
    Chat,
    NoProject,
    SignedOut,
}
fn surface_root(page: SurfacePage) -> Domain {
    match page {
        SurfacePage::Home { brief, needs } => home_cards(Some(brief), &[line(CHAT, b"Chat", BRIEF_AT, false)], needs),
        SurfacePage::Chat => question_root(),
        SurfacePage::NoProject => f::live(Some(f::me(&[]))).0,
        SurfacePage::SignedOut => f::signed_out(),
    }
}

#[test]
fn needs_age_is_a_hint_cut_from_the_end_and_keeps_the_soonest_tick() {
    let root = home_cards(Some(true), &[line(CHAT, b"Away mode", 492 * MINUTE, false)], true);
    let part = launchpad::read(&root, &Tui::new(f::ADDRESS), &f::words(), NOW);
    assert_eq!(part.needs[0].since, Some(Wall::EPOCH));
    assert_eq!(part.tick, Some(Wall::from_nanos(583 * MINUTE)));
    assert_eq!(part.left_off.get(0).unwrap().at.as_ref(), b"today 08:12");
    for (columns, since, expected) in [
        (110, Some(Wall::from_nanos(562 * MINUTE)), "csv-export: comma or semicolon? · asked 20 min ago"),
        (110, None, "csv-export: comma or semicolon?"),
        (69, Some(Wall::from_nanos(562 * MINUTE)), "csv-export: comma or semicolon? · asked…"),
    ] {
        let mut part = launchpad::read(&root, &Tui::new(f::ADDRESS), &f::words(), NOW);
        part.needs = Box::new([launchpad::NeedRow {
            mark: NeedMark::Question,
            project: b"Hello".as_slice().into(),
            words: b"csv-export: comma or semicolon?".as_slice().into(),
            since,
        }]);
        let view = draw(part, columns, 1);
        let mut line = None;
        let mut age_hint = false;
        for node in view.tree().nodes() {
            if node.role == Role::Description && line.is_none() {
                line = Some(f::text(view.tree(), node));
            }
            if node.styles.as_slice() == [look::HINT]
                && f::text(view.tree(), node).as_ref() == b" \xc2\xb7 asked 20 min ago"
            {
                age_hint = true;
            }
        }
        let line = line.unwrap();
        assert!(skein_lib::bytes::find(&line, expected.as_bytes()).is_some(), "{line:?}");
        assert_eq!(age_hint, since.is_some() && columns == 110);
    }
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "the frame assertion locates the header's bottom and the next drawn section"
)]
fn the_headers_frame_has_one_blank_row_before_a_section_or_note() {
    use skein_ui_screen::{Focus, Frame, Size, render};
    let root = home(None, &[]);
    for section in [true, false] {
        let mut part = launchpad::read(&root, &Tui::new(f::ADDRESS), &f::words(), NOW);
        if !section {
            part.project = None;
            part.notes.lines = Box::new([b"A note".as_slice().into()]);
        }
        let view = draw(part, 80, 0);
        let size = Size { columns: 80, rows: 24 };
        let mut frame = Frame::new(size);
        render(view.tree(), &crate::theme(Scheme::Dark, None), size, &mut Focus::default(), &mut frame);
        let printed = frame.print();
        let printed = core::str::from_utf8(&printed).unwrap();
        let mut lines = printed.lines();
        assert!(lines.find(|line| line.starts_with('╰')).is_some());
        assert_eq!(lines.next().unwrap().trim(), "");
        assert_eq!(lines.next().unwrap().trim(), if section { "Start" } else { "A note" });
    }
}

#[test]
fn the_header_joins_new_briefs_across_projects() {
    for (brief, expected) in [
        (
            Some(true),
            &[
                "4 changes landed across two projects. Nothing needs you.",
                "shop: Coupons are two of three in. Hello: csv-export is two of three tasks in; one waits on you.",
                "written by the status agent 08:00 · /brief",
            ][..],
        ),
        (Some(false), &["The last change landed 2 hours ago. Nothing needs you."][..]),
        (None, &[][..]),
    ] {
        let root =
            home_projects(brief, &[], &[], false, None, Address::Home { project: PROJECT }, LEDE.as_bytes(), true);
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Brief);
        let part = launchpad::read(&root, &tui, &f::words(), NOW);
        assert_eq!(
            usize::try_from(part.lines.len()).unwrap(),
            expected.len(),
            "{:?}",
            d::pages::all_home::facts(&root, NOW)
        );
        for (line, expected) in part.lines.iter().zip(expected) {
            assert_eq!(line.words.as_ref(), expected.as_bytes(), "{:?}", d::pages::all_home::facts(&root, NOW));
        }
        if brief == Some(true) {
            let expected = [
                "shop: One change landed in shop.",
                "Coupons are two of three in. One awaits review.",
                "Hello: 3 changes landed overnight.",
                LEDE,
                "written by the status agent 08:00",
            ];
            assert_eq!(part.brief.len(), 5);
            for (line, expected) in part.brief.iter().zip(expected) {
                assert_eq!(line.words.as_ref(), expected.as_bytes());
                assert_eq!(line.style, Some(look::NOTE));
            }
            assert_eq!(part.tick, Some(Wall::from_nanos(1440 * MINUTE)));
            tui.step(Told::Kept { written: false });
            assert_eq!(launchpad::read(&root, &tui, &f::words(), NOW).lines.len(), 4);
        } else {
            assert!(part.brief.is_empty());
        }
    }
}

#[test]
fn left_off_reads_the_other_projects_chat_and_newest_goal() {
    let mut chat = line(CHAT, b"Shop chat", 492 * MINUTE, false);
    chat.project = OTHER_PROJECT;
    let mut newest_goal = goal(7, 480 * MINUTE, PARTY, false);
    newest_goal.project = OTHER_PROJECT;
    let root = home_projects(
        None,
        &[line(OLDER_CHAT, b"Hello chat", BRIEF_AT, false), chat],
        &[goal(8, BRIEF_AT, PARTY, false), newest_goal],
        false,
        None,
        Address::Home { project: PROJECT },
        LEDE.as_bytes(),
        true,
    );
    let mut tui = Tui::new(f::ADDRESS);
    let part = launchpad::read(&root, &tui, &f::words(), NOW);
    assert_eq!(part.left_off.len(), 3);
    for (row, (task, project, kind)) in part.left_off.iter().zip([
        (CHAT, b"shop".as_slice(), launchpad::LeftKind::Chat { number: 1, session: false }),
        (7, b"shop".as_slice(), launchpad::LeftKind::Goal),
        (OLDER_CHAT, b"Hello".as_slice(), launchpad::LeftKind::Chat { number: 2, session: false }),
    ]) {
        assert_eq!((row.task, row.project.as_ref(), row.kind), (task, project, kind));
    }
    assert_eq!(part.left_off.get(0).unwrap().at.as_ref(), b"today 08:12");
    let view = build(&root, &tui);
    assert_eq!(crate::arrow(&view, crate::Arrow::Down), Some(crate::Event::LeftOff { task: Some(CHAT) }));
    tui.step(Told::LeftOff { task: Some(CHAT) });
    let view = build(&root, &tui);
    let field = f::find(view.tree(), Role::TextArea, b"Ask temper");
    assert_eq!(
        crate::event(&view, field.id, skein_ui::Act::Submit),
        Some(crate::Event::Act { act: d::pages::Act::Go { address: Address::Chat { task: CHAT } } })
    );
}

#[test]
fn needs_and_the_quiet_line_cross_projects_only_on_the_launchpad() {
    let root =
        home_projects(Some(false), &[], &[], true, None, Address::Home { project: PROJECT }, LEDE.as_bytes(), true);
    let mut tui = Tui::new(f::ADDRESS);
    let part = launchpad::read(&root, &tui, &f::words(), NOW);
    assert_eq!(part.needs.len(), 2);
    for row in &part.needs {
        assert_eq!(row.project.as_ref(), b"shop");
    }
    assert_eq!(foot::read(&root, &tui, &f::words(), NOW).needs.count, 2);
    tui.step(Told::Walk { by_key: true });
    let question = foot::lists::walk(&root, &tui, &f::words()).unwrap();
    assert_eq!(question.head.as_deref(), Some(b"comma or semicolon?".as_slice()));
    tui.step(Told::Walked { object: c::ObjectKey::Question { task: CHAT, message: 1 } });
    let web = foot::lists::walk(&root, &tui, &f::words()).unwrap();
    assert_eq!(
        web.items.first().unwrap().event,
        crate::Event::Walk {
            object: c::ObjectKey::Escalation(ENDED_CHAT),
            step: crate::WalkStep::Open { address: b"http://127.0.0.1:8367/?project=2".as_slice().into() },
        }
    );
    let chat = home_projects(Some(false), &[], &[], true, None, Address::Chat { task: CHAT }, LEDE.as_bytes(), true);
    assert_eq!(foot::read(&chat, &Tui::new(f::ADDRESS), &f::words(), NOW).needs.count, 1);
}
