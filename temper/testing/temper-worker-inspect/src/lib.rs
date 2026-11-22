//! The worker's frozen waits and tables (temper/docs/design/testing/worker.md, 2).

use std::collections::{BTreeMap, BTreeSet};

use skein_fake_checkout::Checkout;
use skein_lib::Token;
use skein_world::inspect::{Entry, Inspection, Refused, Release, Table, Wait, Wake};
use temper_fake_checkout::layout;
use temper_worker_domain_checkout::{Domain, Limits};

pub mod faults;

pub const COMPONENT: &str = "worker";
pub const KINDS: &[&str] =
    &["a prepare queued for a permit", "a prepare waiting for another's store", "an operation in flight"];

#[must_use]
pub fn inspection() -> Inspection {
    Inspection::new(COMPONENT, KINDS)
}

/// The whole worker and its disk, inspected between steps.
pub fn inspect(
    domain: &temper_worker_domain::Domain,
    limits: &temper_worker_domain::Limits,
    disk: &Checkout,
    into: &mut Inspection,
) {
    inspect_root(domain, limits, into);
    inspect_checkout(domain.checkout(), &limits.checkout, into);
    inspect_disk(domain.checkout(), &limits.checkout, disk, into);
}

/// Hosted slots include answers retained until acknowledgement.
pub fn inspect_root(
    domain: &temper_worker_domain::Domain,
    limits: &temper_worker_domain::Limits,
    into: &mut Inspection,
) {
    let mut slots = table("run slots", u64::from(limits.host.slots));
    for index in 0..domain.host().hosted() {
        if let Some((run, attempt)) = domain.host().hosted_at(index) {
            slots.entries.push(entry(
                format!("run {} attempt {}", run.raw(), attempt.raw()),
                "the run's end, its answer acknowledged",
                Release::Pending,
            ));
        }
    }
    into.table(slots);
}

/// Holds, workspace ownership and preparation admission.
pub fn inspect_checkout(checkout: &Domain, limits: &Limits, into: &mut Inspection) {
    let mut workspaces = table("workspaces", u64::from(limits.workspaces));
    for (id, workspace) in checkout.cached_workspaces() {
        let owner = checkout.held().iter().find(|(_, hold)| hold.workspace() == id.token() && !hold.is_ended());
        // A just-ended hold is released at this iteration's reclaim point.
        let release = if workspace.is_held() && owner.is_some() { Release::Pending } else { Release::Kept };
        workspaces.entries.push(entry(workspace_name(id.token()), "its hold's release", release));
    }
    into.table(workspaces);

    let mut permits = table("prepare permits", u64::from(limits.preparations));
    let active: Vec<_> = checkout.held().iter().filter(|(_, hold)| hold.is_preparing()).map(|(id, _)| id).collect();
    for id in &active {
        permits.entries.push(entry(hold_name(id.token()), "the prepare's settling", Release::Pending));
    }
    for id in checkout.queued() {
        if let Some(hold) = checkout.held().get(*id)
            && let Some(since) = hold.queued_since()
        {
            let waiter = hold_name(id.token());
            permits.refused.push(Refused { key: waiter.clone(), since, bound: None });
            into.wait(Wait {
                waiter,
                kind: "a prepare queued for a permit".into(),
                wakes: active.iter().map(|id| Wake::Waiter(hold_name(id.token()))).collect(),
                bound: None,
            });
        }
    }
    into.table(permits);
    for (id, hold) in checkout.held() {
        if let Some(deadline) = hold.operation_deadline() {
            into.wait(Wait {
                waiter: hold_name(id.token()),
                kind: "an operation in flight".into(),
                wakes: vec![Wake::Timer(deadline), Wake::Party("git".into())],
                bound: None,
            });
        }
    }
    for (_, store) in checkout.stores() {
        for id in store.waiters() {
            let wake = match store.maker() {
                Some(maker) if checkout.held().iter().any(|(id, hold)| id.token() == maker && !hold.is_ended()) => {
                    Wake::Waiter(hold_name(maker))
                }
                Some(maker) => Wake::Ended(hold_name(maker)),
                None => Wake::Ended(format!("store {}'s maker", store.token().raw())),
            };
            into.wait(Wait {
                waiter: hold_name(id.token()),
                kind: "a prepare waiting for another's store".into(),
                wakes: vec![wake],
                bound: None,
            });
        }
    }
}

/// Per-run git metadata, pack counts and locks on the domain-face fake disk.
pub fn inspect_disk(checkout: &Domain, limits: &Limits, disk: &Checkout, into: &mut Inspection) {
    let repositories = u64::from(limits.workspaces) * u64::from(limits.repositories);
    let mut scratch = table("per-run scratch", repositories);
    let mut packs = table("Git stores' packs", repositories.saturating_mul(2));
    let mut locks = table("git's locks", u64::MAX);
    let mut runs = BTreeSet::new();
    let mut counts = BTreeMap::<String, u64>::new();
    let busy: BTreeSet<_> = checkout
        .held()
        .iter()
        .filter(|(_, hold)| hold.is_preparing() || hold.queued_since().is_some() || hold.operation_deadline().is_some())
        .map(|(_, hold)| hold.workspace())
        .collect();
    let cached: BTreeMap<_, _> = checkout
        .cached_workspaces()
        .iter()
        .map(|(id, ws)| (id.token(), ws.is_trusted() && !busy.contains(&id.token())))
        .collect();
    let occupied_slots: BTreeSet<_> = cached.keys().map(|token| token.raw() >> 32).collect();
    for (path, _) in disk.files() {
        if let Some((workspace, relative)) = layout::workspace_file(path) {
            if relative.ends_with(b".lock") {
                locks.entries.push(lock(path, cached.get(&workspace).copied().unwrap_or(false)));
            }
            if let Some((repository, git_path)) = repository_git(relative) {
                let name = format!("{} {}", workspace_name(workspace), String::from_utf8_lossy(repository));
                runs.insert((workspace, name.clone()));
                if is_pack(git_path) {
                    *counts.entry(name).or_default() += 1;
                }
            }
        } else if let Some((store, relative)) = layout::store_file(path) {
            if is_pack(relative) {
                *counts.entry(format!("store {}", store.raw())).or_default() += 1;
            }
            if relative.ends_with(b".lock")
                && let Some(workspace) = layout::reference_workspace(relative)
            {
                let trusted =
                    cached.get(&workspace).copied().unwrap_or(false) && checkout.workspace_uses_store(workspace, store);
                locks.entries.push(lock(path, trusted));
            }
        }
    }
    for (workspace, name) in runs {
        let slot = workspace.raw() >> 32;
        let release = if cached.contains_key(&workspace) {
            Release::Pending
        } else if slot < u64::from(limits.workspaces) && !occupied_slots.contains(&slot) {
            Release::Kept
        } else {
            Release::Due
        };
        scratch.entries.push(entry(name, "the workspace's release, or its slot's next taking", release));
    }
    for (name, count) in counts {
        let release = if count <= 75 { Release::Pending } else { Release::Due };
        packs.entries.push(entry(name, "the packing its own count triggers", release));
    }
    into.table(scratch);
    into.table(packs);
    into.table(locks);
}

fn hold_name(token: Token) -> String {
    format!("hold {}", token.raw())
}

fn workspace_name(token: Token) -> String {
    format!("workspace {}", token.raw())
}

fn table(name: &'static str, capacity: u64) -> Table {
    Table { name, capacity, entries: Vec::new(), refused: Vec::new() }
}

fn entry(key: String, rule: &'static str, release: Release) -> Entry {
    Entry { key, rule, release }
}

fn lock(path: &[u8], trusted: bool) -> Entry {
    entry(
        String::from_utf8_lossy(path).into_owned(),
        "the workspace's making again",
        if trusted { Release::Due } else { Release::Pending },
    )
}

fn repository_git(path: &[u8]) -> Option<(&[u8], &[u8])> {
    let marker = b"/.git/";
    let at = path.windows(marker.len()).position(|part| part == marker)?;
    Some((path.get(..at)?, path.get(at + marker.len()..)?))
}

fn is_pack(path: &[u8]) -> bool {
    path.strip_prefix(b"objects/pack/pack-")
        .and_then(|name| name.strip_suffix(b".pack"))
        .is_some_and(|number| !number.is_empty() && number.iter().all(u8::is_ascii_digit))
}
