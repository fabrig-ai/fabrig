//! Stateless application parts between the root and its channel (protocol/channel.md, 7).
use crate::settings;
use alloc::boxed::Box;
use skein_lib::{List, Secret, Writer};
use temper_engine_domain as engine;
use temper_protocol_worker_wire::{self as wire, v1};

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
    Some(match ask {
        v1::Ask::Reds => engine::wire::Ask::Reds,
        v1::Ask::Results(value) => engine::wire::Ask::Results(results(value)?),
    })
}

fn results(value: v1::CheckResults) -> Option<engine::wire::CheckResults> {
    let parts = value.into_parts();
    if !wire::red_head(&parts.tree) {
        return None;
    }
    if let Some(base) = &parts.base
        && !wire::red_head(base)
    {
        return None;
    }
    let mut flakes = List::with_capacity(parts.flakes.len());
    for flake in parts.flakes.into_boxed() {
        let flake = flake.into_parts();
        flakes.push(engine::wire::Flake { key: flake.key, log: flake.log }).expect("one row per decoded flake");
    }
    Some(engine::wire::CheckResults {
        tag: parts.tag,
        check: parts.check_number,
        tree: parts.tree,
        base: parts.base,
        verdict: match parts.verdict {
            v1::Verdict::Passed => engine::wire::Verdict::Passed,
            v1::Verdict::Refused => engine::wire::Verdict::Refused,
        },
        passed: parts.passed.into_boxed(),
        failed: parts.failed.into_boxed(),
        flakes: flakes.into_boxed(),
    })
}

/// Encode the root's goal-local snapshot within the engine's bounds.
pub fn answer(answer: engine::wire::Answer, limits: &v1::Limits) -> Result<Box<[u8]>, v1::Problem> {
    let value = match answer {
        engine::wire::Answer::Kept => v1::Answer::Kept,
        engine::wire::Answer::Reds { reds } => {
            let mut rows = List::with_capacity(limits.reds_reds);
            for red in reds {
                if !wire::red_head(&red.head) {
                    return Err(problem(v1::Path::RedHead));
                }
                let row = v1::Red::new(limits, v1::RedParts { key: red.key, head: red.head })?;
                rows.push(row).or(Err(problem(v1::Path::RedsReds)))?;
            }
            v1::Answer::Reds(v1::Reds::new(limits, v1::RedsParts { reds: rows })?)
        }
    };
    Ok(wire::answer(&value))
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
    for repository in &workspace.repositories {
        let value = repository_item(repository, save, forges, tokens, limits)?;
        repositories.push(value).or(Err(problem(v1::Path::ItemsRepositories)))?;
    }
    Ok(Some(v1::Items::new(limits, v1::ItemsParts { repositories })?))
}
fn repository_item(
    repository: &engine::ForgeRepository,
    save: bool,
    forges: &crate::forges::Forges,
    tokens: &crate::forge::tokens::Tokens,
    limits: &v1::Limits,
) -> Result<v1::Repository, v1::Problem> {
    let forge = forges.get(repository.provider.forge).ok_or(problem(v1::Path::RepositoryRemote))?;
    let base = forge.git.as_ref();
    let length = base.len().checked_add(repository.owner.len()).ok_or(problem(v1::Path::RepositoryRemote))?;
    let length = length.checked_add(repository.name.len()).ok_or(problem(v1::Path::RepositoryRemote))?;
    let length = length.checked_add(6).ok_or(problem(v1::Path::RepositoryRemote))?;
    if length > skein_lib::usize_of(limits.repository_remote) {
        return Err(problem(v1::Path::RepositoryRemote));
    }
    let mut remote = Writer::new(length);
    for segment in [base, b"/", &repository.owner, b"/", &repository.name, b".git"] {
        remote.put(segment).expect("measured remote");
    }
    let access = if repository.prefixes.is_empty() {
        match &repository.push {
            Some(push) => v1::Access::Writable(v1::Writable::new(
                limits,
                v1::WritableParts { push: push.clone(), expected: None, landing: repository.landing.clone() },
            )?),
            None => v1::Access::ReadOnly,
        }
    } else {
        let mut branches = List::with_capacity(u32::try_from(repository.prefixes.len()).expect("bounded prefixes"));
        for branch in &repository.prefixes {
            branches.push(branch.clone()).expect("measured branches");
        }
        v1::Access::Prefixes(v1::Prefixes::new(limits, v1::PrefixesParts { branches })?)
    };
    v1::Repository::new(
        limits,
        v1::RepositoryParts {
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
        },
    )
}
fn start(start: &engine::ForgeStart, limits: &v1::Limits) -> Result<v1::Start, v1::Problem> {
    Ok(match start {
        engine::ForgeStart::Commit(commit) => {
            v1::Start::Commit(v1::Commit::new(limits, v1::CommitParts { commit: *commit })?)
        }
        engine::ForgeStart::Base(branch) => {
            v1::Start::Base(v1::Branch::new(limits, v1::BranchParts { branch: branch.clone() })?)
        }
        engine::ForgeStart::Branch(branch) => {
            v1::Start::Branch(v1::Branch::new(limits, v1::BranchParts { branch: branch.clone() })?)
        }
        engine::ForgeStart::Saved(branch) => {
            v1::Start::Saved(v1::Branch::new(limits, v1::BranchParts { branch: branch.clone() })?)
        }
        engine::ForgeStart::Merge { branch, base } => {
            v1::Start::Merge(v1::Merge::new(limits, v1::MergeParts { branch: branch.clone(), base: *base })?)
        }
        engine::ForgeStart::Merged { branch, heads } => {
            let mut ordered = List::with_capacity(limits.merged_heads);
            for head in heads {
                ordered.push(*head).or(Err(problem(v1::Path::MergedHeads)))?;
            }
            v1::Start::Merged(v1::Merged::new(limits, v1::MergedParts { branch: branch.clone(), heads: ordered })?)
        }
    })
}
fn problem(path: v1::Path) -> v1::Problem {
    v1::Problem { path, reason: skein_codec::Reason::Bound }
}

/// Each decoded optional part reaches its matching root field.
#[must_use]
pub fn parts(left: Option<v1::Left>, saved: Option<v1::Saved>, preparation: Option<v1::Preparation>) -> engine::Parts {
    let mut pushed = List::with_capacity(match &left {
        Some(left) => left.landed().len(),
        None => 0,
    });
    if let Some(left) = left {
        for head in left.into_parts().landed.into_boxed() {
            let head = head.into_parts();
            pushed
                .push(temper_engine_domain_forge::Pushed { tag: head.tag, commit: head.commit, checked: head.checked })
                .expect("bounded heads");
        }
    }
    let saved = match saved {
        Some(saved) => {
            let saved = saved.into_parts();
            let mut tags = List::with_capacity(saved.repositories.len());
            for repository in saved.repositories.into_boxed() {
                let repository = repository.into_parts();
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
            let missed = missed.into_parts();
            let missing = match missed.missing {
                v1::Missing::Repository => engine::Missing::Repository,
                v1::Missing::Branch => engine::Missing::Branch,
                v1::Missing::Commit => engine::Missing::Commit,
            };
            Some(engine::Preparation::Missing { tag: missed.tag, missing })
        }
        Some(v1::Preparation::Refused(tag)) => Some(engine::Preparation::Refused { tag: tag.tag() }),
        Some(v1::Preparation::Conflicted(value)) => {
            Some(engine::Preparation::Conflicted { tag: value.tag(), head: *value.head() })
        }
        Some(v1::Preparation::Transient) | None => None,
    };
    engine::Parts { pushed: pushed.into_boxed(), saved, preparation }
}

/// Encode one configured identity as the single secret the table holds.
pub fn identity(identity: &settings::Identity, limits: &v1::Limits) -> Result<Secret, v1::Problem> {
    let value = v1::Identity::new(
        limits,
        v1::IdentityParts {
            kind: v1::Kind::Forgejo(v1::Token::new(limits, v1::TokenParts { token: identity.token.clone() })?),
            login: identity.login.clone(),
            name: identity.author.name.clone(),
            email: identity.author.email.clone(),
        },
    )?;
    Ok(wire::identity(&value))
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
    let value = v1::Identity::new(
        limits,
        v1::IdentityParts {
            kind: v1::Kind::Github(v1::Entries::new(limits, v1::EntriesParts { entries })?),
            login: if login.is_empty() { b"x-access-token".as_slice().into() } else { login.into() },
            name,
            email,
        },
    )?;
    Ok(wire::identity(&value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pushed_heads_keep_the_workers_check_status() {
        for checked in [false, true] {
            let mut landed = List::with_capacity(1);
            landed
                .push(
                    v1::Landed::new(&v1::CEILINGS, v1::LandedParts { tag: 7, commit: [3; 32], checked }).expect("head"),
                )
                .expect("room");
            let left = v1::Left::new(&v1::CEILINGS, v1::LeftParts { landed }).expect("left");
            assert_eq!(
                parts(Some(left), None, None).pushed.as_ref(),
                &[temper_engine_domain_forge::Pushed { tag: 7, commit: [3; 32], checked }]
            );
        }
    }
}
