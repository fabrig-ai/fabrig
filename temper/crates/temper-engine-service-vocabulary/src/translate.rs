//! Stateless application parts between the root and its channel (protocol/channel.md, 7).
use crate::{generated::worker_wire, settings};
use alloc::boxed::Box;
use skein_codec::Reason;
use skein_lib::{List, Secret, Writer};
use temper_engine_domain as engine;
use temper_protocol_worker_wire::{
    self as wire,
    v1::{self, Record},
};

/// Application limits from the engine's forge and checkout bounds.
#[must_use]
pub fn limits(own: &crate::limits::Limits) -> Option<v1::Limits> {
    wire::limits(&wire::Bounds {
        repositories: own.checkout.repositories,
        name_bytes: own.checkout.name_bytes.min(own.forge.name_bytes),
        remote_bytes: own.checkout.name_bytes,
        conflicts: own.checkout.conflicts,
        path_bytes: own.checkout.path_bytes,
        token_bytes: v1::CEILINGS.token_token,
        entries: own.forges.installations,
        entry_token_bytes: own.forge_component.token_bytes,
        text_bytes: own.checkout.name_bytes.min(v1::CEILINGS.identity_name),
        reds: own.forge.branch_reds,
        red_key_bytes: own.forge.red_key_bytes,
    })
}

/// Decode the whole application ask before the root reads it.
#[must_use]
pub fn ask(bytes: &[u8], limits: &v1::Limits) -> Option<engine::wire::Ask> {
    let mut reader = skein_lib::Reader::new(bytes);
    let ask = v1::Ask::decode(limits, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    worker_wire::ask_to_domain(ask).ok()
}

pub(crate) fn head_to_domain(head: Box<[u8]>) -> Result<Box<[u8]>, Reason> {
    if !wire::red_head(&head) {
        return Err(Reason::Bound);
    }
    Ok(head)
}

pub(crate) fn head_to_record(head: &[u8]) -> Result<Box<[u8]>, Reason> {
    if !wire::red_head(head) {
        return Err(Reason::Bound);
    }
    Ok(head.into())
}

/// Encode the root's goal-local snapshot within the engine's bounds.
pub fn answer(answer: engine::wire::Answer, limits: &v1::Limits) -> Result<Box<[u8]>, v1::Problem> {
    let value = match answer {
        engine::wire::Answer::Admitted => v1::Answer::Admitted,
        engine::wire::Answer::Kept => v1::Answer::Kept,
        engine::wire::Answer::Landed => v1::Answer::Landed,
        engine::wire::Answer::Unlanded => v1::Answer::Unlanded,
        engine::wire::Answer::Unadmitted { words } => v1::Answer::Unadmitted(v1::Unadmitted { words }),
        engine::wire::Answer::Reds { reds } => {
            let mut rows = List::with_capacity(limits.reds_reds);
            // A row is checked before it takes room; no later row is checked once the list refuses one.
            for red in &reds {
                let row = worker_wire::red_to_record(red)?;
                rows.push(row).or(Err(problem(v1::paths::REDS_REDS)))?;
            }
            v1::Answer::Reds(v1::Reds { reds: rows })
        }
    };
    value.encode(limits)
}

/// Return a typed application answer through the fleet's ask fence.
#[must_use]
pub fn asked(asking: engine::wire::Asking, value: engine::wire::Answer, limits: &v1::Limits) -> engine::WorkerInput {
    let engine::wire::Asking { channel, run, attempt, number } = asking;
    let answer = match answer(value, limits) {
        Ok(bytes) => jig_domain_link::AskAnswer::Bytes(bytes),
        Err(_) => jig_domain_link::AskAnswer::Refused,
    };
    engine::WorkerInput::Asked { channel, run, attempt, number, answer }
}

/// A workspace's remotes and programs, made from each configured forge.
pub fn items(
    assignment: &engine::Assignment,
    forges: &crate::forges::Forges,
    tokens: &crate::forge::tokens::Tokens,
    limits: &v1::Limits,
) -> Result<Option<v1::Items>, v1::Problem> {
    workspace_items(&assignment.workspace, assignment.run.policy.resume, forges, tokens, limits)
}

pub(crate) fn workspace_items(
    workspace: &engine::ForgeWorkspace,
    save: bool,
    forges: &crate::forges::Forges,
    tokens: &crate::forge::tokens::Tokens,
    limits: &v1::Limits,
) -> Result<Option<v1::Items>, v1::Problem> {
    if workspace.repositories.is_empty() {
        return Ok(None);
    }
    let mut repositories = List::with_capacity(limits.items_repositories);
    // Refuse at the first failing lookup, item conversion or list insertion, in assignment order.
    for repository in &workspace.repositories {
        let value = repository_item(repository, save, forges, tokens, limits)?;
        repositories.push(value).or(Err(problem(v1::paths::ITEMS_REPOSITORIES)))?;
    }
    Ok(Some(v1::Items { repositories }))
}
fn repository_item(
    repository: &engine::ForgeRepository,
    save: bool,
    forges: &crate::forges::Forges,
    tokens: &crate::forge::tokens::Tokens,
    limits: &v1::Limits,
) -> Result<v1::Repository, v1::Problem> {
    let forge = forges.get(repository.provider.forge).ok_or(problem(v1::paths::REPOSITORY_REMOTE))?;
    let base = forge.git.as_ref();
    let length = base.len().checked_add(repository.owner.len()).ok_or(problem(v1::paths::REPOSITORY_REMOTE))?;
    let length = length.checked_add(repository.name.len()).ok_or(problem(v1::paths::REPOSITORY_REMOTE))?;
    let length = length.checked_add(6).ok_or(problem(v1::paths::REPOSITORY_REMOTE))?;
    if length > skein_lib::usize_of(limits.repository_remote) {
        return Err(problem(v1::paths::REPOSITORY_REMOTE));
    }
    let mut remote = Writer::new(length);
    for segment in [base, b"/", &repository.owner, b"/", &repository.name, b".git"] {
        remote.put(segment).expect("measured remote");
    }
    let access = if repository.prefixes.is_empty() {
        match &repository.push {
            Some(push) => v1::Access::Writable(v1::Writable {
                push: push.clone(),
                expected: None,
                landing: repository.landing.clone(),
                delivered: repository.delivered,
                pin: repository.pin,
            }),
            None => v1::Access::ReadOnly,
        }
    } else {
        let mut branches = List::with_capacity(u32::try_from(repository.prefixes.len()).expect("bounded prefixes"));
        for branch in &repository.prefixes {
            branches.push(branch.clone()).expect("measured branches");
        }
        v1::Access::Prefixes(v1::Prefixes { branches })
    };
    Ok(v1::Repository {
        admit: repository.push.is_some() || !repository.prefixes.is_empty(),
        save: if save { repository.save.clone() } else { None },
        tag: repository.tag,
        name: repository.name.clone(),
        remote: remote.finish(),
        git: forge.git_program.clone(),
        start: start(&repository.start, limits)?,
        access,
        identity: repository.identity,
        account: match forge.kind {
            settings::Kind::GitHub => crate::forge::tokens::account(tokens, forge.number, &repository.owner),
            settings::Kind::Forgejo => None,
        },
    })
}
fn start(start: &engine::ForgeStart, limits: &v1::Limits) -> Result<v1::Start, v1::Problem> {
    use engine::ForgeStart;
    match start {
        ForgeStart::Merged { heads, .. } if heads.len() > skein_lib::usize_of(limits.merged_heads) => {
            return Err(problem(v1::paths::MERGED_HEADS));
        }
        ForgeStart::Commit(_)
        | ForgeStart::Base(_)
        | ForgeStart::Branch(_)
        | ForgeStart::Saved(_)
        | ForgeStart::Merge { .. }
        | ForgeStart::Merged { .. } => (),
    }
    worker_wire::start_to_record(start)
}
fn problem(path: v1::Path) -> v1::Problem {
    path.problem(Reason::Bound)
}

/// Each decoded optional part reaches its matching root field.
#[must_use]
pub fn parts(left: Option<v1::Left>, saved: Option<v1::Saved>, preparation: Option<v1::Preparation>) -> engine::Parts {
    let mut pushed = List::with_capacity(match &left {
        Some(left) => left.landed.len(),
        None => 0,
    });
    if let Some(left) = left {
        for head in left.landed.into_boxed() {
            pushed.push(worker_wire::landed_to_domain(head)).expect("bounded heads");
        }
    }
    let saved = match saved {
        Some(saved) => {
            let mut tags = List::with_capacity(saved.repositories.len());
            for repository in saved.repositories.into_boxed() {
                let kept = match repository.landing {
                    v1::Landing::Landed(_) => true,
                    v1::Landing::Conflicted(_)
                    | v1::Landing::Explained(_)
                    | v1::Landing::Moved
                    | v1::Landing::Failed
                    | v1::Landing::Refused
                    | v1::Landing::Unchanged => repository.from_saved,
                };
                if kept {
                    tags.push(repository.tag).expect("bounded tags");
                }
            }
            Some(tags.into_boxed())
        }
        None => None,
    };
    let preparation = match preparation {
        Some(v1::Preparation::Missing(missed)) => {
            let missing = worker_wire::missing_to_domain(missed.missing);
            Some(engine::Preparation::Missing { tag: missed.tag, missing })
        }
        Some(v1::Preparation::Refused(tag)) => Some(engine::Preparation::Refused { tag: tag.tag }),
        Some(v1::Preparation::Conflicted(value)) => {
            Some(engine::Preparation::Conflicted { tag: value.tag, head: value.head })
        }
        Some(v1::Preparation::Transient) | None => None,
    };
    engine::Parts { pushed: pushed.into_boxed(), saved, preparation }
}

/// Encode one configured identity as the single secret the table holds.
pub fn identity(identity: &settings::Identity, limits: &v1::Limits) -> Result<Secret, v1::Problem> {
    let value = v1::Identity {
        kind: v1::Kind::Forgejo(v1::Token { token: identity.token.clone() }),
        login: identity.login.clone(),
        name: identity.author.name.clone(),
        email: identity.author.email.clone(),
    };
    Ok(Secret::new(value.encode(limits)?))
}

/// Encode a GitHub value with configuration's login and author or the App bot's defaults.
pub fn github_identity(
    entries: List<v1::Entry>,
    login: &[u8],
    author: &settings::Author,
    slug: &[u8],
    id: u64,
    limits: &v1::Limits,
) -> Result<Secret, v1::Problem> {
    let name = if author.name.is_empty() {
        let mut name = Writer::new(slug.len().checked_add(5).expect("bot name"));
        name.put(slug).expect("slug room");
        name.put(b"[bot]").expect("suffix room");
        name.finish()
    } else {
        author.name.clone()
    };
    let email = if author.email.is_empty() {
        let id = skein_lib::Decimal::of(id);
        let suffix = b"[bot]@users.noreply.github.com";
        let mut email = Writer::new(
            id.as_bytes()
                .len()
                .checked_add(1)
                .expect("separator")
                .checked_add(slug.len())
                .expect("slug")
                .checked_add(suffix.len())
                .expect("email"),
        );
        for part in [id.as_bytes(), b"+", slug, suffix] {
            email.put(part).expect("measured email");
        }
        email.finish()
    } else {
        author.email.clone()
    };
    let value = v1::Identity {
        kind: v1::Kind::Github(v1::Entries { entries }),
        login: if login.is_empty() { b"x-access-token".as_slice().into() } else { login.into() },
        name,
        email,
    };
    Ok(Secret::new(value.encode(limits)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reds_check_the_first_row_that_exceeds_capacity_but_no_later_rows() {
        const FIRST_ROW_ONLY: u32 = 1;
        const SHA1_BYTES: usize = 20;
        const SHORT_HEAD: usize = SHA1_BYTES - 1;
        let limits = v1::Limits { reds_reds: FIRST_ROW_ONLY, ..v1::CEILINGS };
        for (head, path) in [
            (Box::<[u8]>::from([0; SHORT_HEAD]), v1::paths::RED_HEAD),
            (Box::<[u8]>::from([0; SHA1_BYTES]), v1::paths::REDS_REDS),
        ] {
            let reds = Box::new([
                engine::RunRed { key: Box::from([1]), head: Box::from([0; SHA1_BYTES]) },
                engine::RunRed { key: Box::from([2]), head },
                engine::RunRed { key: Box::from([3]), head: Box::from([0; SHORT_HEAD]) },
            ]);
            assert_eq!(answer(engine::wire::Answer::Reds { reds }, &limits), Err(problem(path)));
        }
    }

    #[test]
    fn items_look_up_the_first_repository_that_exceeds_capacity_but_no_later_ones() {
        use crate::tests::fixtures;
        const FIRST_REPOSITORY_ONLY: u32 = 1;
        const UNKNOWN_FORGE: u16 = 99;
        let forges = fixtures::forge_table(&fixtures::forges());
        let tokens = crate::forge::tokens::Tokens::empty();
        let first = fixtures::repository(1, b"acme", b"first");
        let limits = v1::Limits { items_repositories: FIRST_REPOSITORY_ONLY, ..fixtures::wire_limits() };
        for (forge, path) in
            [(UNKNOWN_FORGE, v1::paths::REPOSITORY_REMOTE), (first.provider.forge, v1::paths::ITEMS_REPOSITORIES)]
        {
            let mut second = fixtures::repository(2, b"acme", b"second");
            second.provider.forge = forge;
            let mut third = fixtures::repository(3, b"acme", b"third");
            third.provider.forge = UNKNOWN_FORGE;
            let workspace = engine::ForgeWorkspace { repositories: Box::new([first.clone(), second, third]) };
            assert_eq!(workspace_items(&workspace, false, &forges, &tokens, &limits), Err(problem(path)));
        }
    }

    #[test]
    fn merged_start_refuses_a_head_past_the_configured_bound() {
        const FIRST_HEAD_ONLY: u32 = 1;
        let limits = v1::Limits { merged_heads: FIRST_HEAD_ONLY, ..v1::CEILINGS };
        let merged =
            engine::ForgeStart::Merged { branch: b"base".as_slice().into(), heads: Box::new([[1; 32], [2; 32]]) };
        assert_eq!(start(&merged, &limits), Err(problem(v1::paths::MERGED_HEADS)));
    }

    #[test]
    fn repository_items_encode_the_tasks_own_delivery_without_an_expected_head() {
        use crate::tests::fixtures;
        let limits = fixtures::wire_limits();
        let forges = fixtures::forge_table(&fixtures::forges());
        let tokens = crate::forge::tokens::Tokens::empty();
        for delivered in [None, Some([7; 32])] {
            let mut repository = fixtures::repository(7, b"acme", b"shop");
            repository.delivered = delivered;
            let item = repository_item(&repository, false, &forges, &tokens, &limits).expect("repository item");
            let bytes = item.encode(&limits).expect("repository within bounds");
            let decoded = v1::Repository::decode(&limits, &mut skein_lib::Reader::new(&bytes)).expect("wire item");
            let v1::Access::Writable(value) = decoded.access else { panic!("writable repository") };
            assert_eq!(value.delivered, delivered);
            assert_eq!(value.expected, None);
        }
    }

    #[test]
    fn repository_items_encode_a_named_pin_or_none() {
        use crate::tests::fixtures;
        let limits = fixtures::wire_limits();
        let forges = fixtures::forge_table(&fixtures::forges());
        let tokens = crate::forge::tokens::Tokens::empty();
        for pin in [None, Some([8; 32])] {
            let mut repository = fixtures::repository(7, b"acme", b"shop");
            repository.landing = None;
            repository.pin = pin;
            let item = repository_item(&repository, false, &forges, &tokens, &limits).expect("repository item");
            let bytes = item.encode(&limits).expect("repository within bounds");
            let decoded = v1::Repository::decode(&limits, &mut skein_lib::Reader::new(&bytes)).expect("wire item");
            let v1::Access::Writable(value) = decoded.access else { panic!("writable repository") };
            assert_eq!(value.pin, pin);
            assert_eq!(value.expected, None);
            assert_eq!(value.landing, None);
        }
    }

    #[test]
    fn repository_items_ask_admission_only_for_writable_access() {
        use crate::tests::fixtures;
        let limits = fixtures::wire_limits();
        let forges = fixtures::forge_table(&fixtures::forges());
        let tokens = crate::forge::tokens::Tokens::empty();
        let mut repository = fixtures::repository(7, b"acme", b"shop");
        let writable = repository_item(&repository, false, &forges, &tokens, &limits).unwrap();
        assert!(writable.admit);
        repository.push = None;
        let readonly = repository_item(&repository, false, &forges, &tokens, &limits).unwrap();
        assert!(!readonly.admit);
        repository.prefixes = Box::new([b"landing/7".as_slice().into()]);
        let prefixes = repository_item(&repository, false, &forges, &tokens, &limits).unwrap();
        assert!(prefixes.admit);
    }

    #[test]
    fn pushed_heads_keep_the_workers_check_status() {
        for checked in [false, true] {
            let mut landed = List::with_capacity(1);
            landed.push(v1::Landed { tag: 7, commit: [3; 32], checked }).expect("room");
            let left = v1::Left { landed };
            assert_eq!(
                parts(Some(left), None, None).pushed.as_ref(),
                &[temper_engine_domain_forge::Pushed { tag: 7, commit: [3; 32], checked }]
            );
        }
    }
}
