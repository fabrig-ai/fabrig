use super::{ENGINE, MAIN, STRANGER, calm};
use crate::{
    self as fake, ContentsChange, Domain,
    api::{Error, What},
};
use skein_lib::{Env, Time, Wall};
const REPO: &[u8] = b"work/hello";
fn change(path: &[u8], content: &[u8], expected: Option<[u8; 20]>) -> ContentsChange {
    ContentsChange { path: path.into(), content: content.into(), expected }
}
fn empty() -> (Domain, Env<crate::Config>) {
    let mut config = calm();
    config.limits.files = 2;
    config.limits.name_bytes = 64;
    config.limits.branches = 1;
    let mut domain = Domain::new(&config, 3);
    fake::create_empty_repository(&mut domain, &config, REPO, ENGINE).expect("repository");
    (domain, Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config })
}
#[test]
fn first_contents_commit_creates_one_head_with_every_file_and_no_parent() {
    let (mut domain, env) = empty();
    let files = [change(b"README.md", b"hello", None), change(b".forgejo/workflows/check.yml", b"workflow", None)];
    let commit =
        fake::change_files(&mut domain, &env, REPO, MAIN, None, b"Seed hello", &files, ENGINE).expect("first commit");
    assert_eq!(domain.branch(REPO, MAIN), Some(commit));
    let object = domain.object(commit).expect("stored commit");
    assert_eq!(object.parent, None);
    assert_eq!(object.tree.len(), 2);
    assert_eq!(object.message.as_ref(), b"Seed hello");
    for file in files {
        assert_eq!(object.tree.get(&file.path), Some(&file.content));
    }
}
#[test]
fn contents_batch_refusals_leave_no_commit_and_no_partial_tree() {
    let (mut domain, env) = empty();
    let head = fake::change_files(
        &mut domain,
        &env,
        REPO,
        MAIN,
        None,
        b"seed",
        &[change(b"README.md", b"hello", None)],
        ENGINE,
    )
    .unwrap();
    let room = domain.room().commits;
    for (files, user, expected) in [
        (
            Box::<[ContentsChange]>::from([
                change(b"new", b"new", None),
                change(b"README.md", b"changed", Some([0; 20])),
            ]),
            ENGINE,
            Error::Conflict,
        ),
        (
            Box::<[ContentsChange]>::from([change(b"new", b"new", None), change(b"README.md", b"changed", None)]),
            ENGINE,
            Error::Exists,
        ),
        (
            Box::<[ContentsChange]>::from([change(b"new", b"new", None), change(b"absent", b"changed", Some([0; 20]))]),
            ENGINE,
            Error::Missing(What::File),
        ),
        (
            Box::<[ContentsChange]>::from([change(b"new", b"new", None), change(b"another", b"another", None)]),
            ENGINE,
            Error::Full,
        ),
        (Box::<[ContentsChange]>::from([change(b"new", b"new", None)]), STRANGER, Error::Forbidden),
        (
            Box::<[ContentsChange]>::from([change(b"new", b"new", None), change(b"new", b"again", None)]),
            ENGINE,
            Error::Refused,
        ),
        (Box::<[ContentsChange]>::from([change(b"../bad", b"bad", None)]), ENGINE, Error::Refused),
    ] {
        assert_eq!(fake::change_files(&mut domain, &env, REPO, MAIN, None, b"refused", &files, user), Err(expected));
        assert_eq!(domain.branch(REPO, MAIN), Some(head));
        assert_eq!(domain.room().commits, room);
        assert_eq!(domain.object(head).unwrap().tree.len(), 1);
    }
    let files = [change(b"new", b"new", None), change(b"README.md", b"changed", Some(fake::blob_id(b"hello")))];
    let next = fake::change_files(&mut domain, &env, REPO, MAIN, None, b"update", &files, ENGINE).unwrap();
    assert_eq!(domain.object(next).unwrap().parent, Some(head));
    assert_eq!(domain.object(next).unwrap().tree.len(), 2);
    assert_eq!(domain.object(next).unwrap().tree.get(b"README.md".as_slice()).unwrap().as_ref(), b"changed");
}
#[test]
fn contents_cannot_create_a_missing_branch_in_a_populated_repository() {
    let (mut domain, env) = empty();
    let files = [change(b"README.md", b"hello", None)];
    fake::change_files(&mut domain, &env, REPO, MAIN, None, b"seed", &files, ENGINE).unwrap();
    let room = domain.room().commits;
    assert_eq!(
        fake::change_files(&mut domain, &env, REPO, b"missing", None, b"seed", &files, ENGINE),
        Err(Error::Missing(What::Branch))
    );
    assert_eq!(domain.room().commits, room);
}

#[test]
fn contents_new_branch_is_atomic_at_the_base_head_and_refuses_exists_or_missing_base() {
    let mut config = calm();
    config.limits.branches = 2;
    let mut domain = Domain::new(&config, 3);
    fake::create_empty_repository(&mut domain, &config, REPO, ENGINE).unwrap();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config };
    let head =
        fake::change_files(&mut domain, &env, REPO, MAIN, None, b"base", &[change(b"base", b"kept", None)], ENGINE)
            .unwrap();
    let room = domain.room().commits;
    assert_eq!(
        fake::change_files(
            &mut domain,
            &env,
            REPO,
            b"missing",
            Some(b"topic"),
            b"push",
            &[change(b"new", b"x", None)],
            ENGINE
        ),
        Err(Error::Missing(What::Branch))
    );
    assert_eq!(
        fake::change_files(&mut domain, &env, REPO, MAIN, Some(MAIN), b"push", &[change(b"new", b"x", None)], ENGINE),
        Err(Error::Exists)
    );
    assert_eq!(domain.room().commits, room);
    assert_eq!(domain.branch(REPO, b"topic"), None);
    let commit = fake::change_files(
        &mut domain,
        &env,
        REPO,
        MAIN,
        Some(b"topic"),
        b"push",
        &[change(b"new", b"x", None)],
        ENGINE,
    )
    .unwrap();
    assert_eq!(domain.branch(REPO, MAIN), Some(head));
    assert_eq!(domain.branch(REPO, b"topic"), Some(commit));
    let object = domain.object(commit).unwrap();
    assert_eq!(object.parent, Some(head));
    assert_eq!(object.tree.get(b"base".as_slice()).unwrap().as_ref(), b"kept");
    assert_eq!(object.tree.get(b"new".as_slice()).unwrap().as_ref(), b"x");
}
