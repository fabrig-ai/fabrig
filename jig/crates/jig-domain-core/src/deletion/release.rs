//! Project rows and connector release (jig/docs/design/domain/core.md, 6.1;
//! jig/docs/design/domain/people.md, 5.2).
//!
//! | State | Event | Next | Writes |
//! | --- | --- | --- | --- |
//! | absent | Releasing saved or start | notes | none |
//! | notes | page | notes | entry and index erases |
//! | notes | empty final page | connectors | policy, roles, ledgers, uses, holds |
//! | notes | busy | notes, new load | none |
//! | connectors | asked connector answers | remaining connectors | deletion erase if last |
//! | any | stale load or unsolicited answer | same | none |
//! A start discards the scan and acknowledgements and asks every connector again.
use crate::reads::{Loaded, Position, Range, Scope};
use crate::routing::children::{append_account, append_people, append_tasks};
use crate::{Ask, Core, Held, Key, Limits, Record, Request, Requests, Write};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, Token};

// Two erases per note, with the final release in its own empty-page decision.
const NOTE_ROWS: u32 = 32;

#[derive(Debug)]
pub(crate) struct Scan {
    project: u32,
    load: Token,
    after: Option<Position>,
}

/// Only this scan's current load; stale core loads go to the erasure no-op route.
pub(crate) fn owns(core: &Core, load: Token) -> bool {
    match &core.releasing {
        Some(scan) => scan.load == load,
        None => false,
    }
}

pub(crate) fn begin(core: &mut Core, _env: &Env<Limits>, out: &mut Queue<Request>) {
    if core.releasing.is_some() || !core.people.ready() {
        return;
    }
    let mut project = None;
    for (&number, &stage) in core.people.deletions() {
        if stage == people::Stage::Releasing && !core.release_pending.contains_key(&number) {
            project = Some(number);
            break;
        }
    }
    let Some(project) = project else { return };
    let mut scan = Scan { project, load: Token::new(0), after: None };
    ask(core, &mut scan, out);
    core.releasing = Some(scan);
}

fn ask(core: &mut Core, scan: &mut Scan, out: &mut Queue<Request>) {
    core.erase_load = core.erase_load.checked_add(1).expect("transient deletion load space");
    scan.load = Token::new((u64::from(u32::MAX) << 32) | u64::from(core.erase_load));
    let range = Range {
        party: 0,
        scope: Scope::Notes { project: scan.project },
        after: scan.after.clone(),
        backwards: false,
        rows: NOTE_ROWS,
        bytes: u64::MAX,
    };
    out.push(Request::Held(Box::new(Held::ReadLoad { load: scan.load, range: Box::new(range) })));
}

pub(crate) fn loaded(core: &mut Core, env: &Env<Limits>, load: Token, result: Loaded) -> Requests {
    let mut out = Queue::uncapped();
    out.push(Request::Decided);
    let Some(mut scan) = core.releasing.take() else { return Requests::Out(out) };
    if scan.load != load {
        core.releasing = Some(scan);
        return Requests::Out(out);
    }
    let (rows, next) = match result {
        Loaded::Page { rows, next } => (rows, next),
        Loaded::Busy => {
            ask(core, &mut scan, &mut out);
            core.releasing = Some(scan);
            return Requests::Out(out);
        }
        Loaded::Gone | Loaded::TooLinked { .. } | Loaded::TurnProblem { .. } => {
            unreachable!("release loads raw note rows")
        }
    };
    assert!(rows.len() <= usize::try_from(NOTE_ROWS).expect("page bound fits"), "bounded notes page");
    if rows.is_empty() && next.is_none() {
        release(core, env, scan.project, &mut out);
        begin(core, env, &mut out);
        return Requests::Out(out);
    }
    for row in rows {
        let (scope, name) = match row.value {
            Record::Notes(notes::Record::Entry(row)) => (row.scope, row.name),
            Record::Notes(notes::Record::Line(row)) => (row.scope, row.name),
            Record::Core(_) | Record::People(_) | Record::Tasks(_) | Record::Accounts(_) => {
                unreachable!("notes scan returns entries or index lines")
            }
        };
        let project = match &scope {
            notes::Scope::Project { project }
            | notes::Scope::Goal { project, .. }
            | notes::Scope::Resources { project, .. } => *project,
            notes::Scope::Deployment => unreachable!("project scan excludes deployment notes"),
        };
        assert!(project == scan.project, "notes scan project");
        out.push(Request::Write(Write::Erase(Key::Notes(notes::Key::Entry { name }))));
        out.push(Request::Write(Write::Erase(Key::Notes(notes::Key::Line { scope, name }))));
    }
    // A final nonempty page is followed by an empty scan from the beginning,
    // keeping the child cleanup decision independent of the page's writes.
    scan.after = next;
    ask(core, &mut scan, &mut out);
    core.releasing = Some(scan);
    Requests::Out(out)
}

fn release(core: &mut Core, env: &Env<Limits>, project: u32, out: &mut Queue<Request>) {
    crate::projections::forget_project(core, project, out);
    out.push(Request::Write(Write::Erase(Key::People(people::Key::Policy(project)))));
    let mut projects = List::with_capacity(core.projects.capacity());
    for &number in &core.projects {
        if number != project {
            projects.push(number).expect("subset of projects");
        }
    }
    core.projects = projects;
    drop(core.names.remove(&project));
    drop(core.permission_roles.remove(&project));
    drop(core.template_overrides.remove(&project));
    let mut resources = List::with_capacity(core.resource_roles.len());
    for (key, _) in &core.resource_roles {
        if key.project == project {
            resources.push(key.clone()).expect("subset of resource roles");
        }
    }
    for key in resources.into_boxed() {
        core.resource_roles.remove(&key);
    }
    let mut facts = Queue::uncapped();
    authority::step(&mut core.authority, authority::Event::Dropped { project }, &mut facts);
    append_people(core, env, people::Event::DropProject { project }, out);
    append_tasks(core, env, tasks::Event::ForgetProject { project }, out);
    append_account(core, env, accounts::Event::ForgetProject { project }, out);
    core.touch_accounts(None);
    core.touch_deployment();
    if core.connectors.is_empty() {
        append_people(core, env, people::Event::Deletion { project, stage: None }, out);
    } else {
        core.release_pending.insert(project, core.connectors.clone()).expect("one cohort per deletion");
        for &connector in &core.connectors {
            out.push(Request::Ask { connector, ask: Ask::ReleaseProject { project } });
        }
    }
}

pub(crate) fn answered(core: &mut Core, env: &Env<Limits>, project: u32, connector: u16) -> Requests {
    let mut out = Queue::uncapped();
    out.push(Request::Decided);
    if core.people.deletion(project) != Some(people::Stage::Releasing) {
        return Requests::Out(out);
    }
    let Some(pending) = core.release_pending.get_mut(&project) else { return Requests::Out(out) };
    if !pending.contains(&connector) {
        return Requests::Out(out);
    }
    let mut remaining = List::with_capacity(u32::try_from(pending.len()).expect("bounded connectors"));
    for &number in pending.as_ref() {
        if number != connector {
            remaining.push(number).expect("connector subset");
        }
    }
    *pending = remaining.into_boxed();
    if pending.is_empty() {
        core.release_pending.remove(&project);
        append_people(core, env, people::Event::Deletion { project, stage: None }, &mut out);
    }
    Requests::Out(out)
}
