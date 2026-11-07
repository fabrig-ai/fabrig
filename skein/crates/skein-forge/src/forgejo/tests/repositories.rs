use super::fixtures;
use crate::forgejo::{
    client::{self, Refusal},
    decode::Malformed,
    document::{CreatedRepository, Document},
    request::{Method, Repo, Request},
    server::{self, Route},
};
use alloc::boxed::Box;

fn request(org: &[u8], name: &[u8], private: bool) -> Request {
    Request::CreateRepository { org: Box::from(org), name: Box::from(name), private }
}
#[test]
fn creating_a_repository_fixes_auto_init_and_main_for_both_visibilities() {
    for (private, body) in [
        (true, br#"{"name":"starter","private":true,"auto_init":true,"default_branch":"main"}"#.as_slice()),
        (false, br#"{"name":"starter","private":false,"auto_init":true,"default_branch":"main"}"#.as_slice()),
    ] {
        let encoded = client::encode(&request(b"creation", b"starter", private), &fixtures::limits()).unwrap();
        assert_eq!(encoded.method, Method::Post);
        assert_eq!(&*encoded.target, b"/api/v1/orgs/creation/repos");
        assert_eq!(encoded.body.as_deref(), Some(body));
        assert_eq!(
            fixtures::request(Method::Post, &encoded.target, body, &fixtures::limits()).unwrap(),
            request(b"creation", b"starter", private)
        );
    }
}
#[test]
fn creating_a_repository_encodes_the_org_as_one_segment_and_the_name_as_json() {
    let encoded = client::encode(&request("a/b é".as_bytes(), b"a\"b\\c", true), &fixtures::limits()).unwrap();
    assert_eq!(&*encoded.target, b"/api/v1/orgs/a%2Fb%20%C3%A9/repos");
    assert_eq!(
        encoded.body.as_deref(),
        Some(br#"{"name":"a\"b\\c","private":true,"auto_init":true,"default_branch":"main"}"#.as_slice())
    );
}
#[test]
fn creating_a_repository_checks_the_exact_target_and_body_bounds() {
    let request = request(b"creation", b"starter", true);
    let encoded = client::encode(&request, &fixtures::limits()).unwrap();
    let mut limits = fixtures::limits();
    limits.target_bytes = u32::try_from(encoded.target.len()).unwrap();
    client::encode(&request, &limits).expect("exact room");
    limits.target_bytes -= 1;
    assert_eq!(client::encode(&request, &limits), Err(Refusal::TooLong));
    let mut limits = fixtures::limits();
    limits.writer.length = u32::try_from(encoded.body.unwrap().len()).unwrap();
    client::encode(&request, &limits).expect("exact room");
    limits.writer.length -= 1;
    assert_eq!(client::encode(&request, &limits), Err(Refusal::TooLong));
}
#[test]
fn the_fake_refuses_creation_without_auto_init_or_main() {
    for (body, field) in [
        (
            br#"{"name":"starter","private":true,"auto_init":false,"default_branch":"main"}"#.as_slice(),
            b"auto_init".as_slice(),
        ),
        (
            br#"{"name":"starter","private":true,"auto_init":true,"default_branch":"master"}"#.as_slice(),
            b"default_branch".as_slice(),
        ),
    ] {
        assert_eq!(
            fixtures::request(Method::Post, b"/api/v1/orgs/creation/repos", body, &fixtures::limits()),
            Err(Malformed::Value { field })
        );
    }
}
#[test]
fn the_fake_refuses_wrong_shapes_and_missing_creation_fields() {
    for (body, error) in [
        (
            br#"{"name":"starter","private":"true","auto_init":true,"default_branch":"main"}"#.as_slice(),
            Malformed::Shape { field: b"private" },
        ),
        (
            br#"{"name":"starter","private":true,"default_branch":"main"}"#.as_slice(),
            Malformed::Missing { field: b"auto_init" },
        ),
    ] {
        assert_eq!(
            fixtures::request(Method::Post, b"/api/v1/orgs/creation/repos", body, &fixtures::limits()),
            Err(error)
        );
    }
}

fn migration() -> Request {
    Request::Migrate {
        at: Repo { owner: Box::from(*b"creation"), name: Box::from(*b"copy") },
        from: Box::from(*b"https://source.invalid/a/repo.git"),
    }
}
const MIGRATION: &[u8] = br#"{"clone_addr":"https://source.invalid/a/repo.git","repo_owner":"creation","repo_name":"copy","service":"git","private":true,"mirror":false,"issues":false,"labels":false,"milestones":false,"pull_requests":false,"releases":false,"wiki":false,"lfs":false}"#;
#[test]
fn migrate_encodes_its_fixed_body_and_round_trips() {
    let request = migration();
    let limits = fixtures::limits();
    let encoded = client::encode(&request, &limits).unwrap();
    assert_eq!(encoded.method, Method::Post);
    assert_eq!(encoded.target.as_ref(), b"/api/v1/repos/migrate");
    assert_eq!(encoded.body.as_deref(), Some(MIGRATION));
    assert_eq!(fixtures::request(Method::Post, b"/api/v1/repos/migrate", MIGRATION, &limits), Ok(request.clone()));
    let answer = br#"{"id":7,"full_name":"creation/copy","default_branch":"trunk","clone_url":"https://forge.invalid/creation/copy.git","ssh_url":"ssh://git@forge.invalid/creation/copy.git"}"#;
    let decoded = fixtures::decode(client::decoder(&request, &limits).unwrap(), answer, &limits);
    assert_eq!(
        decoded.document,
        Ok(Document::CreatedRepository(CreatedRepository {
            id: 7,
            full_name: Box::from(*b"creation/copy"),
            default_branch: Box::from(*b"trunk"),
            clone_url: Box::from(*b"https://forge.invalid/creation/copy.git"),
            ssh_url: Box::from(*b"ssh://git@forge.invalid/creation/copy.git"),
        }))
    );
}
#[test]
fn migrate_refuses_a_body_that_asks_for_more() {
    for (body, field) in [
        (
            br#"{"clone_addr":"https://source.invalid/a/repo.git","repo_owner":"creation","repo_name":"copy","service":"github","private":true,"mirror":false,"issues":false,"labels":false,"milestones":false,"pull_requests":false,"releases":false,"wiki":false,"lfs":false}"#.as_slice(),
            b"service".as_slice(),
        ),
        (
            br#"{"clone_addr":"https://source.invalid/a/repo.git","repo_owner":"creation","repo_name":"copy","service":"git","private":false,"mirror":false,"issues":false,"labels":false,"milestones":false,"pull_requests":false,"releases":false,"wiki":false,"lfs":false}"#.as_slice(),
            b"private".as_slice(),
        ),
        (
            br#"{"clone_addr":"https://source.invalid/a/repo.git","repo_owner":"creation","repo_name":"copy","service":"git","private":true,"mirror":true,"issues":false,"labels":false,"milestones":false,"pull_requests":false,"releases":false,"wiki":false,"lfs":false}"#.as_slice(),
            b"mirror".as_slice(),
        ),
        (
            br#"{"clone_addr":"https://source.invalid/a/repo.git","repo_owner":"creation","repo_name":"copy","service":"git","private":true,"mirror":false,"issues":true,"labels":false,"milestones":false,"pull_requests":false,"releases":false,"wiki":false,"lfs":false}"#.as_slice(),
            b"issues".as_slice(),
        ),
        (
            br#"{"clone_addr":"https://source.invalid/a/repo.git","repo_owner":"creation","repo_name":"copy","service":"git","private":true,"mirror":false,"issues":false,"labels":true,"milestones":false,"pull_requests":false,"releases":false,"wiki":false,"lfs":false}"#.as_slice(),
            b"labels".as_slice(),
        ),
        (
            br#"{"clone_addr":"https://source.invalid/a/repo.git","repo_owner":"creation","repo_name":"copy","service":"git","private":true,"mirror":false,"issues":false,"labels":false,"milestones":true,"pull_requests":false,"releases":false,"wiki":false,"lfs":false}"#.as_slice(),
            b"milestones".as_slice(),
        ),
        (
            br#"{"clone_addr":"https://source.invalid/a/repo.git","repo_owner":"creation","repo_name":"copy","service":"git","private":true,"mirror":false,"issues":false,"labels":false,"milestones":false,"pull_requests":true,"releases":false,"wiki":false,"lfs":false}"#.as_slice(),
            b"pull_requests".as_slice(),
        ),
        (
            br#"{"clone_addr":"https://source.invalid/a/repo.git","repo_owner":"creation","repo_name":"copy","service":"git","private":true,"mirror":false,"issues":false,"labels":false,"milestones":false,"pull_requests":false,"releases":true,"wiki":false,"lfs":false}"#.as_slice(),
            b"releases".as_slice(),
        ),
        (
            br#"{"clone_addr":"https://source.invalid/a/repo.git","repo_owner":"creation","repo_name":"copy","service":"git","private":true,"mirror":false,"issues":false,"labels":false,"milestones":false,"pull_requests":false,"releases":false,"wiki":true,"lfs":false}"#.as_slice(),
            b"wiki".as_slice(),
        ),
        (
            br#"{"clone_addr":"https://source.invalid/a/repo.git","repo_owner":"creation","repo_name":"copy","service":"git","private":true,"mirror":false,"issues":false,"labels":false,"milestones":false,"pull_requests":false,"releases":false,"wiki":false,"lfs":true}"#.as_slice(),
            b"lfs".as_slice(),
        ),
    ] {
        assert_eq!(
            fixtures::request(Method::Post, b"/api/v1/repos/migrate", body, &fixtures::limits()),
            Err(Malformed::Value { field })
        );
    }
}
#[test]
fn delete_repository_is_one_segment_each_and_round_trips() {
    let request =
        Request::DeleteRepository { at: Repo { owner: Box::from("a/b é".as_bytes()), name: Box::from(*b"c/d ?#%") } };
    let limits = fixtures::limits();
    let encoded = client::encode(&request, &limits).unwrap();
    assert_eq!(encoded.method, Method::Delete);
    assert_eq!(encoded.target.as_ref(), b"/api/v1/repos/a%2Fb%20%C3%A9/c%2Fd%20%3F%23%25");
    assert_eq!(encoded.body, None);
    assert!(client::decoder(&request, &limits).is_none());
    let Route::Ready(decoded) =
        server::route(Method::Delete, b"/api/v1/repos/a%2Fb%20%C3%A9/c%2Fd%20%3F%23%25", &limits).unwrap()
    else {
        panic!("repository deletion has no body");
    };
    assert_eq!(decoded, request);
}
