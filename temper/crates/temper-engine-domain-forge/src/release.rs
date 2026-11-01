//! Project release (temper/docs/plan/playground-reset-r4/notes/contracts.md, 4;
//! temper/docs/design/draft/playground-reset.md, 5).
use crate::{ApplicationOutcome, Domain, Key, Name, Request, Topic};
use skein_forge::api::Repository;
use skein_lib::{Queue, Token};

pub(crate) fn release(d: &mut Domain, request: Token, project: u32, out: &mut Queue<Request>) {
    landings(d, project, None, out);
    receipts(d, project, None, out);
    // Descending ordinals let each family drop its rows without retaining a second key set.
    for index in (0..d.repositories.iter().len()).rev() {
        let (_, row) = d.repositories.iter().nth(index).expect("bounded repository index");
        if row.project != project {
            continue;
        }
        let repository = row.provider;
        landings(d, project, Some(repository), out);
        receipts(d, project, Some(repository), out);
        holds(d, repository, project, out);
        names(d, repository, project, out);
        subscriptions(d, repository, project, out);
        heads(d, repository, out);
        pulls(d, repository, out);
        ci(d, repository, out);
        changes(d, repository, project, out);
        issues(d, repository, project, out);
        proposed_effects(d, repository, project, out);
        entries(d, repository, project, out);
        releases(d, repository, project, out);
        applications(d, repository, out);
        d.repositories.remove(&repository);
        out.push(Request::Erase { key: Key::Repository(repository) });
    }
    d.landing.projects.remove(&project);
    out.push(Request::ProjectReleased { request, project });
}

fn on(name: &Name, repository: Repository) -> bool {
    name.forge == repository.forge && name.repository == repository.repository
}

// The caller has already erased the project's tasks. Keep this a diagnostic:
// release builds must still erase and acknowledge a repeated ask.
#[expect(clippy::disallowed_macros, reason = "contracts 4 requires debug-only checks of jig's task-erasure promise")]
fn task(d: &mut Domain, task: u64, project: u32, out: &mut Queue<Request>) {
    for ((owner, _), ()) in &d.flights {
        debug_assert!(*owner != task, "flight of task {task} while releasing project {project}");
    }
    debug_assert!(!d.releases.contains_key(&task), "unfinished release of task {task} in project {project}");
    for index in (0..d.flights.iter().len()).rev() {
        let (&(owner, entry), ()) = d.flights.iter().nth(index).expect("bounded flight index");
        if owner == task {
            d.flights.remove(&(owner, entry));
            out.push(Request::Erase { key: Key::Flight { task, entry } });
        }
    }
    if d.releases.remove(&task).is_some() {
        out.push(Request::Erase { key: Key::Release(task) });
    }
    for index in (0..d.named_calls.iter().len()).rev() {
        let (&key, _) = d.named_calls.iter().nth(index).expect("bounded call index");
        if key.task == task {
            d.named_calls.remove(&key);
            out.push(Request::Erase { key: Key::Call(key) });
        }
    }
    for index in (0..d.landed.iter().len()).rev() {
        let (&commit, &owner) = d.landed.iter().nth(index).expect("bounded landed index");
        if owner == task {
            d.landed.remove(&commit);
            out.push(Request::Erase { key: Key::Landed(commit) });
        }
    }
    d.change_due.remove(&task);
    d.projection_due.remove(&task);
    d.procedure_wakes.remove(&task);
    d.panel_ci.remove(&task);
    d.goal_files.remove(&task);
    d.settling.remove(&task);
}

fn holds(d: &mut Domain, repository: Repository, project: u32, out: &mut Queue<Request>) {
    for index in (0..d.holds.iter().len()).rev() {
        let (name, row) = d.holds.iter().nth(index).expect("bounded hold index");
        if on(name, repository) {
            let name = name.clone();
            let owner = row.task;
            task(d, owner, project, out);
            d.holds.remove(&name);
            out.push(Request::Erase { key: Key::Hold(name) });
        }
    }
}

fn names(d: &mut Domain, repository: Repository, project: u32, out: &mut Queue<Request>) {
    for index in (0..d.names.iter().len()).rev() {
        let (&owner, resources) = d.names.iter().nth(index).expect("bounded names index");
        let mut found = false;
        for name in resources {
            found |= on(name, repository);
        }
        if found {
            task(d, owner, project, out);
            d.names.remove(&owner);
            out.push(Request::Erase { key: Key::Names(owner) });
        }
    }
}

fn subscriptions(d: &mut Domain, repository: Repository, project: u32, out: &mut Queue<Request>) {
    for index in (0..d.subscriptions.iter().len()).rev() {
        let ((owner, topic), _) = d.subscriptions.iter().nth(index).expect("bounded subscription index");
        let target = match topic {
            Topic::Landings { repository, .. }
            | Topic::Ci { repository, .. }
            | Topic::Pull { repository, .. }
            | Topic::Participation { repository, .. } => *repository,
        };
        if target == repository {
            let owner = *owner;
            let topic = topic.clone();
            task(d, owner, project, out);
            d.subscriptions.remove(&(owner, topic.clone()));
            out.push(Request::Erase { key: Key::Subscription { task: owner, topic } });
        }
    }
}

fn heads(d: &mut Domain, repository: Repository, out: &mut Queue<Request>) {
    for index in (0..d.heads.iter().len()).rev() {
        let (name, _) = d.heads.iter().nth(index).expect("bounded head index");
        if on(name, repository) {
            let name = name.clone();
            d.heads.remove(&name);
            out.push(Request::Erase { key: Key::BranchHead(name) });
        }
    }
}

fn pulls(d: &mut Domain, repository: Repository, out: &mut Queue<Request>) {
    for index in (0..d.pulls.iter().len()).rev() {
        let (name, _) = d.pulls.iter().nth(index).expect("bounded pull index");
        if on(name, repository) {
            let name = name.clone();
            d.pulls.remove(&name);
            out.push(Request::Erase { key: Key::PullState(name) });
        }
    }
    for index in (0..d.pull_mergeable.iter().len()).rev() {
        let (name, _) = d.pull_mergeable.iter().nth(index).expect("bounded mergeable index");
        if on(name, repository) {
            let name = name.clone();
            d.pull_mergeable.remove(&name);
        }
    }
}

fn ci(d: &mut Domain, repository: Repository, out: &mut Queue<Request>) {
    for index in (0..d.ci.iter().len()).rev() {
        let (&(target, head), _) = d.ci.iter().nth(index).expect("bounded CI index");
        if target == repository {
            d.ci.remove(&(target, head));
            out.push(Request::Erase { key: Key::Ci { repository, head } });
        }
    }
}

fn changes(d: &mut Domain, repository: Repository, project: u32, out: &mut Queue<Request>) {
    for index in (0..d.changes.iter().len()).rev() {
        let (&owner, row) = d.changes.iter().nth(index).expect("bounded change index");
        if row.repository == repository {
            task(d, owner, project, out);
            d.changes.remove(&owner);
            out.push(Request::Erase { key: Key::Change(owner) });
        }
    }
}

fn issues(d: &mut Domain, repository: Repository, project: u32, out: &mut Queue<Request>) {
    for index in (0..d.issues.iter().len()).rev() {
        let (&owner, row) = d.issues.iter().nth(index).expect("bounded issue index");
        if row.repository == repository {
            task(d, owner, project, out);
            d.issues.remove(&owner);
            out.push(Request::Erase { key: Key::Issue(owner) });
        }
    }
}

fn proposed_effects(d: &mut Domain, repository: Repository, project: u32, out: &mut Queue<Request>) {
    for index in (0..d.proposed_effects.iter().len()).rev() {
        let (&number, row) = d.proposed_effects.iter().nth(index).expect("bounded proposal index");
        if row.entry.repository == repository {
            let owner = row.entry.task;
            task(d, owner, project, out);
            d.proposed_effects.remove(&number);
            out.push(Request::Erase { key: Key::ProposedEffect(number) });
        }
    }
}

fn entries(d: &mut Domain, repository: Repository, project: u32, out: &mut Queue<Request>) {
    for index in (0..d.entries.iter().len()).rev() {
        let (&number, row) = d.entries.iter().nth(index).expect("bounded entry index");
        if row.entry.repository == repository {
            let owner = row.entry.task;
            task(d, owner, project, out);
            d.entries.remove(&number);
            out.push(Request::Erase { key: Key::Entry(number) });
        }
    }
}

fn releases(d: &mut Domain, repository: Repository, project: u32, out: &mut Queue<Request>) {
    // A pending cleanup can be the task's only remaining repository link.
    for index in (0..d.releases.iter().len()).rev() {
        let (&owner, row) = d.releases.iter().nth(index).expect("bounded release index");
        if let Some((_, Some(name))) = &row.pending
            && on(name, repository)
        {
            task(d, owner, project, out);
        }
    }
}

fn applications(d: &mut Domain, repository: Repository, out: &mut Queue<Request>) {
    for index in (0..d.application.kept.iter().len()).rev() {
        let (&token, (_, outcome)) = d.application.kept.iter().nth(index).expect("bounded application index");
        match outcome {
            ApplicationOutcome::Adopted(row) if row.repository == repository => {
                d.application.kept.remove(&token);
                out.push(Request::Erase { key: Key::Application(token.raw()) });
            }
            ApplicationOutcome::Adopted(_)
            | ApplicationOutcome::CoreRefused(_)
            | ApplicationOutcome::ForgeRefused(_) => {}
        }
    }
}

fn landings(d: &mut Domain, project: u32, repository: Option<Repository>, out: &mut Queue<Request>) {
    for index in (0..d.landing.rows.iter().len()).rev() {
        let (&owner, row) = d.landing.rows.iter().nth(index).expect("bounded landing index");
        let mut found = row.landing.project == project;
        for member in &row.landing.members {
            found |= Some(Repository { forge: member.repository.forge, repository: member.repository.repository })
                == repository;
        }
        if found {
            task(d, owner, project, out);
            d.landing.rows.remove(&owner);
            d.landing.due.remove(&owner);
            out.push(Request::Erase { key: Key::Landing(owner) });
        }
    }
}

fn receipts(d: &mut Domain, project: u32, repository: Option<Repository>, out: &mut Queue<Request>) {
    for index in (0..d.landing.receipts.iter().len()).rev() {
        let (key, _) = d.landing.receipts.iter().nth(index).expect("bounded receipt index");
        if key.project == project || Some(key.repository) == repository {
            let key = key.clone();
            d.landing.receipts.remove(&key);
            out.push(Request::Erase { key: Key::Receipt(key) });
        }
    }
}

#[cfg(test)]
#[path = "tests/release.rs"]
mod tests;
