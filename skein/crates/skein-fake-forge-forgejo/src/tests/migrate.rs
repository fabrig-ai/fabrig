use super::fixtures::{self, Harness, boxed};
use crate::{
    Server, Source,
    setup::migrate::{MISSING_SOURCE, PRIVATE_SOURCE},
};
use skein_fake_forge_domain::{
    self as fake,
    api::{self as a, Fill},
};
use skein_fake_forge_http::git;
use skein_forge::forgejo::{
    document::{Document, Row},
    request::{Repo, Request},
};
use skein_json::tokenizer;
use skein_lib::bytes::concat;
use skein_lib::{Duration, Secret, Time, Token};

const SOURCE: &[u8] = b"https://sources.invalid/upstream/repository";
const DESTINATION: &[u8] = b"creation/copied";
#[expect(clippy::disallowed_macros, reason = "the committed conformance answer is the fake's oracle")]
const MISSING_REFUSAL: &[u8] =
    include_bytes!("../../../../tests/forge/conformance/transcripts/setup-migrate-missing-source/response");
#[expect(clippy::disallowed_macros, reason = "the committed conformance read is the fake's oracle")]
const MISSING_READ: &[u8] =
    include_bytes!("../../../../tests/forge/conformance/transcripts/setup-migrate-missing-source-read/response");

fn json(h: &Harness, body: &[u8]) -> skein_json::Document {
    skein_json::parse(body, &h.env.limits.documents.json, &skein_json::document::Limits { tokens: 1024, text: 8192 })
        .expect("answer JSON")
}
fn version_and_status(line: &[u8]) -> (&[u8], &[u8]) {
    let version = skein_lib::bytes::find(line, b" ").expect("HTTP version");
    let tail = &line[version.saturating_add(1)..];
    let code = skein_lib::bytes::find(tail, b" ").expect("status code");
    (&line[..version], &tail[..code])
}
fn recorded(h: &Harness, transcript: &[u8], fields: &[&[u8]]) {
    let line = skein_lib::bytes::find(transcript, b"\r\n").expect("recorded status line");
    let output = h.output.as_slice();
    let end = skein_lib::bytes::find(output, b"\r\n").expect("fake status line");
    // Reason phrases are advisory: Forgejo uses "Unprocessable Entity", HTTP uses RFC 9110's "Unprocessable Content".
    assert_eq!(version_and_status(&output[..end]), version_and_status(&transcript[..line]));
    let body =
        skein_lib::bytes::find(transcript, b"\r\n\r\n").expect("recorded body").checked_add(4).expect("body offset");
    let expected = json(h, &transcript[body..]);
    let actual = json(h, &h.body());
    for field in fields {
        let expected_value =
            expected.token(expected.member(0, field).expect("recorded field")).expect("recorded value");
        let actual_value = actual.token(actual.member(0, field).expect("fake field")).expect("fake value");
        assert_eq!(actual_value.kind, expected_value.kind);
        assert_eq!(actual.text(actual_value), expected.text(expected_value));
    }
}
fn at() -> Repo {
    Repo { owner: boxed(b"creation"), name: boxed(b"copied") }
}
fn migrate() -> Request {
    Request::Migrate { at: at(), from: boxed(SOURCE) }
}
fn harness(fill: Fill) -> Harness {
    let mut h = Harness::creating();
    let tip = h.push(b"main");
    h.branch(b"trunk", tip);
    fake::settings(
        &mut h.domain,
        b"org/repo",
        a::Settings { default: boxed(b"trunk"), merge: true, squash: true, rebase: true },
    );
    h.service.config.repositories.clear();
    h.service.config.sources =
        Box::new([Source { address: boxed(SOURCE), repository: boxed(b"org/repo"), private: false, fill }]);
    h.service.setup_mut().org(b"creation", b"kit-admin");
    h
}
fn id(h: &Harness) -> u64 {
    for row in &h.service.config.repositories {
        if row.full.as_ref() == DESTINATION {
            return u64::from(row.id);
        }
    }
    panic!("reserved row")
}
fn git(h: &mut Harness, repository: &[u8], name: &[u8], body: &[u8]) {
    let clear = concat(&[b"engine:", fixtures::TOKEN]);
    let size = skein_lib::base64::encoded_len(
        skein_lib::base64::Alphabet::Standard,
        clear.len().try_into().expect("credential size"),
    )
    .expect("base64 size");
    let mut credential = skein_lib::Writer::new(size.try_into().expect("credential size"));
    skein_lib::base64::encode(skein_lib::base64::Alphabet::Standard, &clear, &mut credential).expect("credential");
    let credential = credential.finish();
    let length = skein_lib::Decimal::of(body.len().try_into().expect("body size"));
    let request = concat(&[
        b"POST /.fake/git/",
        name,
        b" HTTP/1.1\r\nHost: localhost\r\nAuthorization: Basic ",
        &credential,
        b"\r\nContent-Length: ",
        length.as_bytes(),
        b"\r\n\r\n",
        body,
    ]);
    assert!(skein_lib::bytes::find(body, repository).is_some());
    h.request(&request);
    h.run();
}
fn clone(h: &mut Harness, repository: &[u8]) {
    let body = concat(&[b"{\"repository\":\"", repository, b"\"}"]);
    git(h, repository, b"clone", &body);
}
fn decoded_clone(h: &Harness) -> git::Answer {
    let mut decoder = git::client::decoder(git::Name::Clone, h.status(), Time::ZERO, &h.env.limits.git());
    let body = h.body();
    let mut feed = tokenizer::Feed::new(&body, &h.env.limits.documents.json);
    let mut demand = tokenizer::Request::Next;
    loop {
        let event = feed.answer(demand);
        match git::client::step(&mut decoder, &h.env.limits.git(), &event) {
            git::client::Step::Demand(next) => demand = next,
            git::client::Step::Decoded(answer) => return answer.expect("git clone"),
            git::client::Step::Malformed => panic!("malformed git clone"),
        }
    }
}
fn assert_refused(h: &mut Harness, message: &[u8]) {
    h.send(&migrate());
    assert_eq!(h.status(), 422);
    let expected = skein_forge::forgejo::server::encode(
        &skein_forge::forgejo::server::Answer::Error(skein_forge::forgejo::document::ErrorBody {
            message: skein_forge::forgejo::document::Body::Kept(boxed(message)),
        }),
        &h.env.limits.documents,
    )
    .expect("refusal document");
    assert_eq!(h.body(), expected);
    recorded(h, MISSING_REFUSAL, &[b"message"]);
    assert_eq!(h.service.config.repositories.len(), 1);
    assert!(h.service.setup().is_private(id(h)));
    assert!(h.domain.branches(DESTINATION).is_empty());
    assert_eq!(h.domain.metadata(DESTINATION).expect("empty destination").default_branch.as_ref(), b"main");
    assert_eq!(h.domain.permission(DESTINATION, fixtures::ADMIN), Ok(a::Permission::Admin));
    for request in [Request::Repository { at: at() }, Request::RepositoryById { id: id(h) }] {
        h.send(&request);
        assert_eq!(h.status(), 200);
        // The fake's repository document carries no `empty`; branch absence pins that state.
        recorded(h, MISSING_READ, &[b"default_branch"]);
        let Document::Repository(repository) = h.decoded(&request).document else { panic!("repository") };
        assert_eq!(repository.id, id(h));
        assert_eq!(repository.owner.login.as_ref(), b"creation");
        assert_eq!(repository.name.as_ref(), b"copied");
    }
    h.send(&Request::Branch { at: at(), name: boxed(b"main") });
    assert_eq!(h.status(), 404);
    let search = Request::RepoSearch { page: fixtures::PAGE };
    h.send(&search);
    assert_eq!(h.status(), 200);
    let decoded = h.decoded(&search);
    let [Row::Found(repository)] = decoded.rows.as_ref() else { panic!("one search row") };
    assert_eq!(repository.repository, id(h));
    assert_eq!(repository.login.as_ref(), b"creation");
    assert!(repository.private);
    clone(h, DESTINATION);
    assert_eq!(h.status(), 200);
    let git::Answer::Cloned { default, branches } = decoded_clone(h) else { panic!("empty clone") };
    assert_eq!(default.as_ref(), b"main");
    assert!(branches.is_empty());
}
#[test]
fn a_migrate_answers_as_forgejo_with_the_sources_default_branch() {
    let mut h = harness(Fill::Whole);
    h.send(&migrate());
    assert_eq!(h.status(), 201);
    let Document::CreatedRepository(created) = h.decoded(&migrate()).document else { panic!("created repository") };
    assert_eq!(created.default_branch.as_ref(), b"trunk");
    assert_eq!(created.full_name.as_ref(), DESTINATION);
    assert_eq!(created.id, id(&h));
    assert_eq!(created.clone_url.as_ref(), b"http://127.0.0.1:3000/creation/copied.git");
    assert_eq!(created.ssh_url.as_ref(), b"ssh://git@localhost/creation/copied.git");
    assert!(h.service.setup().is_private(created.id));
    assert_eq!(h.domain.branches(DESTINATION).len(), h.domain.branches(b"org/repo").len());
    for (name, tip) in h.domain.branches(b"org/repo") {
        assert_eq!(h.domain.branch(DESTINATION, name), Some(*tip));
    }
    for request in [Request::Repository { at: at() }, Request::RepositoryById { id: created.id }] {
        h.send(&request);
        assert_eq!(h.status(), 200);
        let Document::Repository(repository) = h.decoded(&request).document else { panic!("repository") };
        assert_eq!(repository.id, created.id);
        assert_eq!(repository.default_branch.as_ref(), b"trunk");
    }
    for request in [Request::RepoSearch { page: fixtures::PAGE }, Request::UserRepos { page: fixtures::PAGE }] {
        h.send(&request);
        assert_eq!(h.status(), 200);
        let decoded = h.decoded(&request);
        let [row] = decoded.rows.as_ref() else { panic!("one repository") };
        let repository = match row {
            Row::Reachable(repository) | Row::Found(repository) => repository,
            Row::Entry { .. }
            | Row::Artifact(_)
            | Row::CommitMessage(_)
            | Row::Hook(_)
            | Row::Token(_)
            | Row::Team(_)
            | Row::User(_)
            | Row::Issue(_)
            | Row::Comment(_)
            | Row::Status(_)
            | Row::Review(_)
            | Row::ReviewComment(_)
            | Row::Branch(_)
            | Row::Protection(_)
            | Row::Pull(_)
            | Row::File(_)
            | Row::Commit(_)
            | Row::TreeEntry(_)
            | Row::Run(_)
            | Row::Job(_)
            | Row::Step(_) => panic!("repository row"),
        };
        assert_eq!(repository.repository, created.id);
        assert!(repository.private);
    }
    let branch = Request::Branch { at: at(), name: boxed(b"trunk") };
    h.send(&branch);
    assert_eq!(h.status(), 200);
    let Document::Branch(branch) = h.decoded(&branch).document else { panic!("branch") };
    h.send(&Request::Raw { at: at(), path: boxed(b"file"), reference: branch.commit });
    assert_eq!(h.status(), 200);
    assert_eq!(h.body().as_ref(), b"content");
    clone(&mut h, DESTINATION);
    assert_eq!(h.status(), 200);
    let git::Answer::Cloned { default, branches } = decoded_clone(&h) else { panic!("clone") };
    assert_eq!(default.as_ref(), b"trunk");
    assert_eq!(branches.len(), 2);
}
#[test]
fn a_migrate_under_way_reads_as_present_with_no_branch() {
    let mut h = harness(Fill::Slow(Duration::from_secs(30)));
    h.send(&migrate());
    assert!(h.output.is_empty(), "migration stays unanswered");
    let reserved = id(&h);
    assert!(h.service.setup().is_private(reserved));
    h.domain.metadata(DESTINATION).expect("destination present");
    assert!(h.domain.branch(DESTINATION, b"trunk").is_none());
    let reader = Server::new(Token::new(2), &h.env.limits).expect("reader");
    let migrating = core::mem::replace(&mut h.server, reader);
    let demand = h.demand.take();
    crate::start(&mut h.server, &mut h.service, &h.env, &mut h.above, &mut h.below);
    h.send(&Request::Repository { at: at() });
    assert_eq!(h.status(), 200);
    h.send(&Request::RepositoryById { id: reserved });
    assert_eq!(h.status(), 200);
    h.send(&Request::Branch { at: at(), name: boxed(b"trunk") });
    assert_eq!(h.status(), 404);
    h.server = migrating;
    h.demand = demand;
    h.output.clear();
    h.run_at(Time::from_nanos(40_000_000_000));
    assert_eq!(h.status(), 201);
}
#[test]
fn a_source_not_in_the_catalogue_is_refused_and_leaves_an_empty_repository() {
    // copy-facts: setup-migrate-missing-source records this 422 and message.
    let mut h = harness(Fill::Whole);
    h.service.config.sources = Box::new([]);
    assert_refused(&mut h, MISSING_SOURCE);
}
#[test]
fn a_private_source() {
    // Fallback: the missing-source authentication refusal; no private-source transcript is recorded.
    let mut h = harness(Fill::Whole);
    h.service.config.sources[0].private = true;
    assert_refused(&mut h, PRIVATE_SOURCE);
}
#[test]
fn a_name_taken_is_409() {
    let mut h = harness(Fill::Whole);
    h.send(&migrate());
    let before = id(&h);
    let metadata = h.domain.metadata(DESTINATION).expect("first copy");
    let mut branches = skein_lib::List::with_capacity(h.domain.branches(DESTINATION).len());
    for (name, tip) in h.domain.branches(DESTINATION) {
        branches.push((name.clone(), *tip)).expect("snapshot room");
    }
    h.service.config.sources = Box::new([]);
    h.send(&migrate());
    assert_eq!(h.status(), 409);
    assert_eq!(id(&h), before);
    assert_eq!(h.service.config.repositories.len(), 1);
    assert_eq!(h.domain.metadata(DESTINATION), Ok(metadata));
    assert_eq!(h.domain.branches(DESTINATION).len(), branches.len());
    for (name, tip) in &branches {
        assert_eq!(h.domain.branch(DESTINATION, name), Some(*tip));
    }
}
#[test]
fn checks_before_the_source_leave_no_repository() {
    for (owner, name, user, room, status) in [
        (b"creation".as_slice(), b"copied".as_slice(), fixtures::PERSON, true, 403),
        (b"absent", b"copied", fixtures::ADMIN, true, 404),
        (b"creation", b"bad name", fixtures::ADMIN, true, 422),
        (b"creation", b"copied", fixtures::ADMIN, false, 503),
    ] {
        let mut h = harness(Fill::Whole);
        h.service.config.sources = Box::new([]);
        if !room {
            h.service.config.repositories = skein_lib::List::with_capacity(0);
        }
        h.service.setup_mut().token(user, b"caller", &[b"all"], Secret::new(boxed(b"caller")));
        let request = Request::Migrate { at: Repo { owner: boxed(owner), name: boxed(name) }, from: boxed(SOURCE) };
        let encoded = skein_forge::forgejo::client::encode(&request, &h.env.limits.documents).expect("request");
        h.ask_as(b"POST", &encoded.target, encoded.body.as_deref().expect("body"), b"caller");
        assert_eq!(h.status(), status);
        assert!(h.service.config.repositories.is_empty());
        let full = concat(&[owner, b"/", name]);
        assert_eq!(h.domain.metadata(&full), Err(a::Error::Missing(a::What::Repository)));
    }
}
#[test]
fn a_slow_source_is_slow() {
    let mut h = harness(Fill::Slow(Duration::from_secs(5)));
    let start = h.fake_env.now;
    h.send(&migrate());
    assert!(h.output.is_empty());
    h.run_at(start.saturating_add(Duration::from_secs(4)));
    assert!(h.output.is_empty());
    h.run_at(start.saturating_add(Duration::from_secs(6)));
    assert_eq!(h.status(), 201);
    assert!(h.domain.branch(DESTINATION, b"trunk").is_some());
}
#[test]
fn a_cut_source_answers_504_or_as_a_timed_out_write_does() {
    let mut h = harness(Fill::Cut);
    h.send(&migrate());
    assert_eq!(h.status(), 500, "testing/fakes/forge.md, 4: a timed-out write answers 500");
    assert!(h.domain.branch(DESTINATION, b"trunk").is_none());
    h.send(&Request::Repository { at: at() });
    assert_eq!(h.status(), 200, "the timed-out copy remains readable");
    h.send(&Request::RepositoryById { id: id(&h) });
    assert_eq!(h.status(), 200);
}
#[test]
fn a_source_is_read_by_no_route() {
    let mut h = harness(Fill::Whole);
    for request in [
        Request::Repository { at: fixtures::at() },
        Request::Branch { at: fixtures::at(), name: boxed(b"trunk") },
        Request::RepositoryById { id: 2 },
    ] {
        h.send(&request);
        assert_eq!(h.status(), 404);
    }
    clone(&mut h, b"org/repo");
    assert_eq!(h.status(), 404);
}
#[test]
fn a_catalogue_with_a_host_outside_invalid_is_refused_by_valid() {
    let mut config = fixtures::config();
    for address in [b"https://sources.invalid/upstream/repo".as_slice(), b"https://sources.invalid:443/upstream/repo"] {
        config.sources = Box::new([Source {
            address: boxed(address),
            repository: boxed(b"org/repo"),
            private: false,
            fill: Fill::Whole,
        }]);
        assert!(crate::valid(&fixtures::limits(), &config));
    }
    for address in [
        b"https://outside.example/upstream/repo".as_slice(),
        b"https://sources.invalid.example/upstream/repo",
        b"https://sources.invalid@outside.example/upstream/repo",
        b"https://sources.invalid:443:80/upstream/repo",
    ] {
        config.sources[0].address = boxed(address);
        assert!(!crate::valid(&fixtures::limits(), &config), "{address:?}");
        assert_eq!(
            crate::Service::new(config, &fixtures::limits(), 3).expect_err("outside invalid refused"),
            crate::Error::Limits
        );
        config = fixtures::config();
        config.sources = Box::new([Source {
            address: boxed(SOURCE),
            repository: boxed(b"org/repo"),
            private: false,
            fill: Fill::Whole,
        }]);
    }
}
#[test]
fn a_delete_is_204_then_404_and_drops_collaborators_and_hooks() {
    let mut h = harness(Fill::Whole);
    h.send(&migrate());
    let original = id(&h);
    h.send(&Request::AddCollaborator {
        at: at(),
        login: boxed(b"kit"),
        permission: skein_forge::forgejo::request::Grant::Write,
    });
    assert_eq!(h.status(), 204);
    h.send(&Request::CreateRepoHook { at: at(), hook: fixtures::hook_spec() });
    assert_eq!(h.status(), 201);
    assert_eq!(h.service.setup().repository_hooks(original.try_into().expect("id")).count(), 1);
    h.send(&Request::DeleteRepository { at: at() });
    assert_eq!(h.status(), 204);
    assert!(h.body().is_empty());
    assert!(h.service.setup().collaborator(original.try_into().expect("id"), b"kit").is_none());
    assert_eq!(h.service.setup().repository_hooks(original.try_into().expect("id")).count(), 0);
    assert!(!h.service.setup().is_private(original));
    h.send(&Request::DeleteRepository { at: at() });
    assert_eq!(h.status(), 404);
    h.send(&Request::RepositoryById { id: original });
    assert_eq!(h.status(), 404);
    clone(&mut h, DESTINATION);
    assert_eq!(h.status(), 404);
    h.send(&Request::CreateRepository { org: boxed(b"creation"), name: boxed(b"copied"), private: true });
    assert_eq!(h.status(), 201);
    let recreated = id(&h);
    assert!(recreated > original);
    h.send(&Request::DeleteRepository { at: at() });
    assert_eq!(h.status(), 204);
    h.send(&migrate());
    assert_eq!(h.status(), 201);
    assert!(id(&h) > recreated);
}
#[test]
fn migrate_needs_write_repository_and_delete_needs_write_organization() {
    let mut h = harness(Fill::Whole);
    let token = b"limited-admin";
    h.service.setup_mut().token(
        fixtures::ADMIN,
        b"limited",
        &[b"read:repository", b"write:admin"],
        Secret::new(boxed(token)),
    );
    let encoded = skein_forge::forgejo::client::encode(&migrate(), &h.env.limits.documents).expect("request");
    for (method, path, body, scope) in [
        (
            b"POST".as_slice(),
            encoded.target.as_ref(),
            encoded.body.as_deref().expect("body"),
            b"write:repository".as_slice(),
        ),
        (b"DELETE", b"/api/v1/repos/creation/copied".as_slice(), b"".as_slice(), b"write:organization".as_slice()),
    ] {
        h.ask_as(method, path, body, token);
        assert_eq!(h.status(), 403);
        assert!(skein_lib::bytes::find(&h.body(), scope).is_some());
    }
    h.service.setup_mut().token(fixtures::ADMIN, b"writer", &[b"write:repository"], Secret::new(boxed(b"writer")));
    let encoded = skein_forge::forgejo::client::encode(&migrate(), &h.env.limits.documents).expect("request");
    h.ask_as(b"POST", &encoded.target, encoded.body.as_deref().expect("body"), b"writer");
    assert_eq!(h.status(), 201);
    h.ask_as(b"DELETE", b"/api/v1/repos/creation/copied", b"", b"writer");
    assert_eq!(h.status(), 403);
    assert!(skein_lib::bytes::find(&h.body(), b"write:organization").is_some());
    h.service.setup_mut().token(fixtures::ADMIN, b"deleter", &[b"write:organization"], Secret::new(boxed(b"deleter")));
    h.ask_as(b"POST", &encoded.target, encoded.body.as_deref().expect("body"), b"deleter");
    assert_eq!(h.status(), 403);
    assert!(skein_lib::bytes::find(&h.body(), b"write:repository").is_some());
    h.ask_as(b"DELETE", b"/api/v1/repos/creation/copied", b"", b"deleter");
    assert_eq!(h.status(), 204);
}
