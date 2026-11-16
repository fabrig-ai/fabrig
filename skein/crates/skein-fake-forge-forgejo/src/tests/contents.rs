//! The person's API requests use the default branch by name (temper/docs/design/client/drive.md, 3.5).
use super::fixtures::{self, Harness, boxed};
use skein_fake_forge_domain as fake;

fn empty() -> Harness {
    let mut h = Harness::creating();
    h.fake_env.limits.limits.content_bytes = 256;
    // This fixture's protocol clock and domain clock advance on the same millisecond.
    h.fake_env.limits.skew = fake::Skew::None;
    h.fake_env.limits.resolution = skein_lib::Duration::from_millis(1);
    h.fake_env.limits.made_checks.contexts = Box::new([boxed(b"check")]);
    h.fake_env.limits.made_checks.cue = Some(fake::api::Cue::Declared { path: boxed(b".fake-forge/ci") });
    fake::create_empty_repository(&mut h.domain, &h.fake_env.limits, b"work/hello", fixtures::ADMIN)
        .expect("empty repository");
    h.service
        .config
        .repositories
        .push(crate::Repository { id: 3, full: boxed(b"work/hello"), protected: None })
        .expect("repository identity room");
    h
}
#[test]
fn contents_at_an_unborn_or_missing_branch_is_404() {
    let mut h = empty();
    h.ask(b"GET", b"/api/v1/repos/work/hello/contents/README.md?ref=main", b"");
    assert_eq!(h.status(), 404);
    h.ask(b"GET", b"/api/v1/repos/org/repo/contents/README.md?ref=missing", b"");
    assert_eq!(h.status(), 404);
}
#[test]
fn combined_status_resolves_a_branch_and_missing_refs_are_404() {
    let mut h = Harness::new(fixtures::config());
    let head = h.push(b"work/topic");
    h.ask(b"GET", b"/api/v1/repos/org/repo/commits/work%2Ftopic/status", b"");
    assert_eq!(h.status(), 200);
    let sha = skein_forge::forgejo::id::write(&crate::routes::object(head));
    assert!(skein_lib::bytes::find(&h.body(), &sha).is_some());
    h.ask(b"GET", b"/api/v1/repos/org/repo/commits/missing/status", b"");
    assert_eq!(h.status(), 404);
}

const CONTENTS: &[u8] = b"/api/v1/repos/work/hello/contents";
fn commit(h: &mut Harness, files: &[skein_forge::forgejo::request::FileChange]) {
    let request = skein_forge::forgejo::request::Request::ChangeFiles {
        at: skein_forge::forgejo::request::Repo { owner: boxed(b"work"), name: boxed(b"hello") },
        branch: boxed(b"main"),
        message: boxed(b"Seed hello"),
        files: files.into(),
    };
    h.send(&request);
}
fn change(path: &[u8], content: &[u8], sha: Option<Box<[u8]>>) -> skein_forge::forgejo::request::FileChange {
    skein_forge::forgejo::request::FileChange { path: boxed(path), content: boxed(content), sha }
}
#[test]
fn contents_post_creates_an_unborn_main_and_its_declared_check_succeeds() {
    let mut h = empty();
    let workflow = b"on:\n  push:\n    branches: [main]\n  pull_request:\njobs:\n  check:\n    runs-on: temper-host\n    steps:\n      - run: echo \"temper's starter check passed; replace this step with your own checks\"\n";
    for path in [
        b"/api/v1/repos/work/hello/contents/README.md?ref=main".as_slice(),
        b"/api/v1/repos/work/hello/contents/.forgejo/workflows/check.yml?ref=main",
    ] {
        h.ask(b"GET", path, b"");
        assert_eq!(h.status(), 404);
    }
    commit(
        &mut h,
        &[change(b"README.md", b"# Hello\n", None), change(b".forgejo/workflows/check.yml", workflow, None)],
    );
    assert_eq!(h.status(), 201);
    let head = h.domain.branch(b"work/hello", b"main").expect("first main");
    let object = h.domain.object(head).unwrap();
    assert_eq!(object.parent, None);
    assert_eq!(object.tree.len(), 2);
    assert_eq!(object.tree.get(b"README.md".as_slice()).unwrap().as_ref(), b"# Hello\n");
    assert_eq!(object.tree.get(b".forgejo/workflows/check.yml".as_slice()).unwrap().as_ref(), workflow);
    h.ask(b"GET", b"/api/v1/repos/work/hello/commits/main/status", b"");
    assert_eq!(h.status(), 200);
    let request = skein_forge::forgejo::request::Request::CombinedStatus {
        at: skein_forge::forgejo::request::Repo { owner: boxed(b"work"), name: boxed(b"hello") },
        commit: crate::routes::object(head),
        page: fixtures::PAGE,
    };
    let decoded = h.decoded(&request);
    assert_eq!(
        decoded.document,
        skein_forge::forgejo::document::Document::CombinedStatus(skein_forge::forgejo::document::CombinedStatus {
            state: Some(skein_forge::forgejo::document::StatusState::Success),
            total_count: 1
        })
    );
    assert_eq!(decoded.rows.len(), 1);
    let skein_forge::forgejo::document::Row::Status(status) = &decoded.rows[0] else { panic!("check status") };
    assert_eq!(status.context.as_ref(), b"check");
    assert_eq!(status.status, skein_forge::forgejo::document::StatusState::Success);
}
#[test]
fn contents_post_accepts_the_reported_json_and_present_file_metadata_allows_an_update() {
    let mut h = empty();
    h.ask(
        b"POST",
        CONTENTS,
        br#"{"branch":"main","message":"x","files":[{"operation":"create","path":"README.md","content":"aGVsbG8="}]}"#,
    );
    assert_eq!(h.status(), 201);
    let first = h.domain.branch(b"work/hello", b"main").unwrap();
    h.ask(b"GET", b"/api/v1/repos/work/hello/contents/README.md?ref=main", b"");
    assert_eq!(h.status(), 200);
    let sha = skein_forge::forgejo::id::write(&crate::routes::tree_object(fake::blob_id(b"hello")));
    assert_eq!(
        h.body().as_ref(),
        skein_lib::bytes::concat(&[
            br#"{"type":"file","path":"README.md","sha":""#,
            &sha,
            br#"","size":5,"encoding":"base64","content":"aGVsbG8="}"#,
        ])
        .as_ref()
    );
    commit(&mut h, &[change(b"README.md", b"updated", Some(sha)), change(b"second", b"second", None)]);
    assert_eq!(h.status(), 201);
    let head = h.domain.branch(b"work/hello", b"main").unwrap();
    assert_eq!(h.domain.object(head).unwrap().parent, Some(first));
    assert_eq!(h.domain.object(head).unwrap().tree.get(b"README.md".as_slice()).unwrap().as_ref(), b"updated");
    let room = h.domain.room().commits;
    commit(
        &mut h,
        &[
            change(b"third", b"third", None),
            change(b"README.md", b"stale", Some(boxed(b"0000000000000000000000000000000000000000"))),
        ],
    );
    assert_eq!(h.status(), 409);
    assert_eq!(h.domain.branch(b"work/hello", b"main"), Some(head));
    assert_eq!(h.domain.room().commits, room);
}

#[test]
fn contents_post_on_an_initialized_repository_keeps_the_parent() {
    let mut h = Harness::new(fixtures::config());
    let first = h.domain.branch(b"org/repo", b"main").unwrap();
    h.ask(
        b"POST",
        b"/api/v1/repos/org/repo/contents",
        br#"{"branch":"main","message":"x","files":[{"operation":"create","path":"README.md","content":"aGVsbG8="}]}"#,
    );
    assert_eq!(h.status(), 201);
    let head = h.domain.branch(b"org/repo", b"main").unwrap();
    assert_eq!(h.domain.object(head).unwrap().parent, Some(first));
    assert_eq!(h.domain.object(head).unwrap().tree.get(b"README.md".as_slice()).unwrap().as_ref(), b"hello");
}
#[test]
fn contents_commit_ci_reads_the_new_heads_declaration() {
    let mut h = empty();
    commit(&mut h, &[change(b".fake-forge/ci", b"fail\n", None)]);
    assert_eq!(h.status(), 201);
    let first = h.domain.branch(b"work/hello", b"main").unwrap();
    h.ask(b"GET", b"/api/v1/repos/work/hello/commits/main/status", b"");
    assert_eq!(h.status(), 200);
    let request = skein_forge::forgejo::request::Request::CombinedStatus {
        at: skein_forge::forgejo::request::Repo { owner: boxed(b"work"), name: boxed(b"hello") },
        commit: crate::routes::object(first),
        page: fixtures::PAGE,
    };
    assert_eq!(
        h.decoded(&request).document,
        skein_forge::forgejo::document::Document::CombinedStatus(skein_forge::forgejo::document::CombinedStatus {
            state: Some(skein_forge::forgejo::document::StatusState::Failure),
            total_count: 1
        })
    );
    let sha = skein_forge::forgejo::id::write(&crate::routes::tree_object(fake::blob_id(b"fail\n")));
    commit(&mut h, &[change(b".fake-forge/ci", b"pass\n", Some(sha))]);
    assert_eq!(h.status(), 201);
    h.ask(b"GET", b"/api/v1/repos/work/hello/commits/main/status", b"");
    assert_eq!(h.status(), 200);
    assert_eq!(
        h.decoded(&request).document,
        skein_forge::forgejo::document::Document::CombinedStatus(skein_forge::forgejo::document::CombinedStatus {
            state: Some(skein_forge::forgejo::document::StatusState::Success),
            total_count: 1
        })
    );
    let next = h.domain.branch(b"work/hello", b"main").unwrap();
    assert_ne!(next, first);
    let sha = skein_forge::forgejo::id::write(&crate::routes::object(next));
    assert!(skein_lib::bytes::find(&h.body(), &sha).is_some());
}
