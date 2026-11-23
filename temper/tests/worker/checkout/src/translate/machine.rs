//! The domain-face machine's disk faults and packs (temper/docs/design/testing/fakes.md, 3).
//! Timing and the choice of a fault belong to the calling world.

use super::{assert_cloned, commit, failed, fake, path, perform_unchecked, store_path};
use skein_lib::Token;
use temper_fake_checkout::{
    Checkout,
    git::{self, Remote},
};
use temper_worker_domain_checkout::git::{Done, Fault, Into, Op, Place, PushDiagnostic, Reds, Want};

/// Bytes in regular files, including git metadata and packs; links and directories count nothing.
#[must_use]
pub fn used(disk: &Checkout) -> u64 {
    disk.files().values().map(|bytes| u64::try_from(bytes.len()).expect("file size fits u64")).sum()
}

/// Run an operation with the world's optional byte capacity. Only the usage at admission matters.
pub fn perform_with_capacity(
    forge: &mut impl temper_fake_git::counts::Remote,
    disk: &mut Checkout,
    op: Op,
    capacity: Option<u64>,
) -> Done {
    if capacity.is_some_and(|capacity| used(disk) >= capacity) && writes(&op) {
        return Done::FailedWithOutput {
            fault: Fault::Broken,
            diagnostic: PushDiagnostic::new(b"No space left on device", 0),
        };
    }
    if locked(disk, &op) {
        return Done::Failed { fault: Fault::Broken };
    }
    perform_unchecked(forge, disk, op)
}

fn writes(op: &Op) -> bool {
    matches!(
        op,
        Op::Store { .. }
            | Op::Clone { .. }
            | Op::Fetch { .. }
            | Op::Counts { .. }
            | Op::Point { .. }
            | Op::CheckOut { .. }
            | Op::Merge { .. }
            | Op::Commit { .. }
            | Op::Reset { .. }
            | Op::Reds { action: Reds::Prepare { .. } | Reds::Write { .. }, .. }
    )
}

pub(super) fn reference(into: Into) -> &'static [u8] {
    match into {
        Into::Fetched => b"refs/temper/fetched",
        Into::Landing => b"refs/temper/landing",
    }
}

fn store_reference(at: &Place, into: Into) -> Vec<u8> {
    [
        b"refs/temper/".as_slice(),
        at.workspace.raw().to_string().as_bytes(),
        b"/",
        reference(into).strip_prefix(b"refs/temper/").expect("fixed reference"),
    ]
    .concat()
}

fn reference_locked(disk: &Checkout, directory: &[u8], reference: &[u8]) -> bool {
    disk.exists(&[directory, b"/", reference, b".lock"].concat())
}

fn locked(disk: &Checkout, op: &Op) -> bool {
    match op {
        Op::CheckOut { at, .. } | Op::Merge { at, .. } | Op::Commit { at, .. } => {
            disk.exists(&[path(at).as_slice(), b"/.git/index.lock"].concat())
        }
        Op::Fetch { at, store, into, .. } => {
            reference_locked(disk, &store_path(*store), &store_reference(at, *into))
                || reference_locked(disk, &[path(at).as_slice(), b"/.git"].concat(), reference(*into))
        }
        Op::Counts { at, store, .. } => {
            reference_locked(disk, &store_path(*store), &temper_fake_git::counts::store_reference(at.workspace))
                || reference_locked(disk, &[path(at).as_slice(), b"/.git"].concat(), temper_fake_git::counts::REFERENCE)
        }
        Op::Point { at, into, .. } => {
            reference_locked(disk, &[path(at).as_slice(), b"/.git"].concat(), reference(*into))
        }
        _ => false,
    }
}

/// Cut a fetch, checkout or merge partway. Any other operation is a caller error.
/// Tree writes retain the first ceil(n/2) regular files in path order, without deleting old files.
pub fn cut(forge: &mut impl temper_fake_git::counts::Remote, disk: &mut Checkout, op: Op) -> Done {
    assert!(
        matches!(op, Op::Fetch { .. } | Op::Counts { .. } | Op::CheckOut { .. } | Op::Merge { .. }),
        "only content fetches can be cut"
    );
    if locked(disk, &op) {
        return Done::Failed { fault: Fault::Broken };
    }
    match op {
        Op::Fetch { at, store, remote, want, into, .. } => {
            assert_cloned(disk, &at);
            fetch(forge, disk, &at, store, &remote, &want, into, true)
        }
        Op::Counts { at, store, remote, .. } => {
            assert_cloned(disk, &at);
            counts(forge, disk, &at, store, &remote, true)
        }
        Op::CheckOut { at, commit, .. } => {
            assert_cloned(disk, &at);
            assert!(git::object_present(disk, &path(&at), fake(commit)), "checkout commit fetched");
            partial_tree(disk, &at, forge.tree(fake(commit)));
            Done::Failed { fault: Fault::TimedOut }
        }
        Op::Merge { at, theirs, .. } => {
            assert_cloned(disk, &at);
            // Compute git's tree on a private disk, so no head or merge metadata leaks through the cut.
            let mut scratch = Checkout::new();
            for (name, content) in disk.files() {
                scratch.write(name, content);
            }
            git::merge(forge, &mut scratch, &path(&at), fake(theirs)).expect("merge parent is locally fetched");
            partial_tree(disk, &at, scratch.tree(&path(&at)));
            Done::Failed { fault: Fault::TimedOut }
        }
        _ => unreachable!("checked above"),
    }
}

fn partial_tree(disk: &mut Checkout, at: &Place, mut tree: git::Tree) {
    tree.retain(|name, _| !temper_fake_checkout::in_git(name));
    let count = tree.len().div_ceil(2);
    for (name, content) in tree.into_iter().take(count) {
        disk.write(&[path(at).as_slice(), b"/", &name].concat(), &content);
    }
    disk.write(&[path(at).as_slice(), b"/.git/index.lock"].concat(), b"");
    disk.write(&[path(at).as_slice(), b"/.git/index"].concat(), b"partial index\n");
}

#[expect(clippy::too_many_arguments, reason = "the Fetch operation's fields and whether it is cut")]
pub(super) fn fetch(
    forge: &mut impl Remote,
    disk: &mut Checkout,
    at: &Place,
    store: Token,
    remote: &[u8],
    want: &Want,
    into: Into,
    cut: bool,
) -> Done {
    let want = match want {
        Want::Branch { branch } => git::Want::Branch(branch),
        Want::Commit { commit } => git::Want::Commit(fake(*commit)),
        Want::Default => git::Want::Default,
    };
    match git::fetch(forge, disk, remote, &store_path(store), want) {
        Ok(fetched) => {
            let store_ref = store_reference(at, into);
            if cut {
                disk.write(&[store_path(store).as_slice(), b"/", &store_ref, b".lock"].concat(), b"");
                return Done::Failed { fault: Fault::TimedOut };
            }
            git::update_ref(disk, &store_path(store), &store_ref, fetched).expect("fetched into store");
            git::update_ref(disk, &path(at), reference(into), fetched).expect("borrowed from store");
            add_pack(disk, &store_path(store));
            Done::Fetched { commit: commit(fetched) }
        }
        Err(fault) => failed(fault),
    }
}

pub(super) fn counts(
    forge: &mut impl temper_fake_git::counts::Remote,
    disk: &mut Checkout,
    at: &Place,
    store: Token,
    remote: &[u8],
    cut: bool,
) -> Done {
    use temper_fake_git::counts::{REFERENCE, import, store_reference};
    let directory = store_path(store);
    let store_ref = store_reference(at.workspace);
    let local = path(at);
    // In this world the worker's metadata is collapsed into the run's .git, as for Fetch.
    let local_ref = [local.as_slice(), b"/.git/", REFERENCE].concat();
    match forge.fetch_counts(remote) {
        Ok(Some(fetched)) => {
            if cut {
                disk.write(&[directory.as_slice(), b"/", &store_ref, b".lock"].concat(), b"");
                return Done::Failed { fault: Fault::TimedOut };
            }
            let new = !git::object_present(disk, &directory, fetched);
            import(disk, &directory, &store_ref, fetched, &forge.tree(fetched));
            git::update_ref(disk, &local, REFERENCE, fetched).expect("borrowed from store");
            if new {
                add_pack(disk, &directory);
            }
            Done::Fetched { commit: commit(fetched) }
        }
        Ok(None) => {
            disk.remove(&local_ref);
            Done::Succeeded
        }
        Err(fault) => failed(fault),
    }
}

fn packs(disk: &Checkout, directory: &[u8]) -> Vec<Vec<u8>> {
    disk.tree(&[directory, b"/objects/pack"].concat())
        .into_keys()
        .filter(|name| name.starts_with(b"pack-") && name.ends_with(b".pack"))
        .collect()
}

pub(super) fn add_pack(disk: &mut Checkout, directory: &[u8]) {
    let next = packs(disk, directory)
        .iter()
        .map(|name| {
            std::str::from_utf8(
                name.strip_prefix(b"pack-").expect("pack prefix").strip_suffix(b".pack").expect("pack suffix"),
            )
            .expect("decimal pack name")
            .parse::<u64>()
            .expect("decimal pack number")
        })
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .expect("pack number fits u64");
    disk.write(&[directory, format!("/objects/pack/pack-{next}.pack").as_bytes()].concat(), b"pack\n");
}

pub(super) fn pack(disk: &mut Checkout, directory: &[u8]) {
    let packs = packs(disk, directory);
    if packs.len() > 50 {
        add_pack(disk, directory);
        for name in packs {
            disk.remove(&[directory, b"/objects/pack/", &name].concat());
        }
    }
}
