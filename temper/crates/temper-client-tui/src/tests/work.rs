//! Working chats read by the front (temper/docs/design/client/README.md, 6.17).
use super::{
    fixtures as f,
    foot::{TaskState, root_state},
    launchpad::{home, line},
};
use crate::{Tui, view::work, work_runs};
use jig_client::contract as c;

fn working(task: u64, title: &[u8], activity: u64) -> c::TaskLine {
    let mut row = line(task, title, activity, false);
    row.chip.details.phase = c::ChipPhase::Active(c::RunStage::Due);
    row.waits_on = c::WaitsOn::Nothing;
    row.chip.details.details.live_delegates = 0;
    row
}

enum Read {
    None,
    NoProject,
    Home(Box<[c::TaskLine]>),
    Open(TaskState),
}
fn root(read: Read) -> temper_client_domain::Domain {
    match read {
        Read::None => f::signed_out(),
        Read::NoProject => f::live(Some(f::me(&[]))).0,
        Read::Home(lines) => home(None, &lines),
        Read::Open(state) => root_state(true, true, true, false, false, state),
    }
}

#[test]
fn work_runs_reads_the_projects_working_chats() {
    let mut idle = line(9, b"idle", 1, false);
    idle.chip.details.details.live_delegates = 0;
    let active = working(9, b"working", 1);
    let mut delegate = idle.clone();
    delegate.chip.details.details.live_delegates = 1;
    let mut ended = active.clone();
    ended.chip.details.phase = c::ChipPhase::Ended;
    let mut other = active.clone();
    other.project = 2;
    let open = root(Read::Open(TaskState::Working));
    let title = open.pages().chat().unwrap().task(open.client()).unwrap().chip.title.bytes();
    let rows = [
        ("none read", Read::None, 0, None),
        ("no project", Read::NoProject, 0, None),
        ("waiting on the person", Read::Home(Box::new([idle])), 0, None),
        ("working", Read::Home(Box::new([active.clone()])), 1, Some(b"working".as_slice())),
        ("live delegate", Read::Home(Box::new([delegate])), 1, Some(b"idle".as_slice())),
        ("ended", Read::Home(Box::new([ended])), 0, None),
        ("another project", Read::Home(Box::new([other])), 0, None),
        (
            "two working",
            Read::Home(Box::new([active.clone(), working(10, b"latest", 2)])),
            2,
            Some(b"latest".as_slice()),
        ),
        (
            "latest first",
            Read::Home(Box::new([working(10, b"latest", 2), active.clone()])),
            2,
            Some(b"latest".as_slice()),
        ),
        ("counted once", Read::Home(Box::new([active.clone(), active])), 1, Some(b"working".as_slice())),
        ("open working", Read::Open(TaskState::Working), 1, Some(title)),
        ("open result", Read::Open(TaskState::Result), 0, None),
        ("open ended", Read::Open(TaskState::Ended), 0, None),
        ("open idle with delegates", Read::Open(TaskState::Idle), 1, Some(title)),
        ("open paused with delegates", Read::Open(TaskState::Paused), 1, Some(title)),
    ];
    let mut failures = skein_lib::List::with_capacity(u32::try_from(rows.len()).unwrap());
    let tui = Tui::new(f::ADDRESS);
    for (name, input, count, title) in rows {
        let root = root(input);
        let read = work::running(&root);
        if read.count != count || read.title.as_deref() != title || work_runs(&root, &tui) != (count > 0) {
            failures.push(name).unwrap();
        }
    }
    assert!(failures.is_empty(), "working chat rows: {failures:?}");
}
