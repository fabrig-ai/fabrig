#![cfg(test)]
use crate::common::forgejo;
use skein_forge::forgejo::{
    client,
    document::{Body, Document, ErrorBody},
    request::{Repo, Request},
};
fn at(name: &[u8]) -> Repo {
    Repo { owner: Box::from(*b"copies"), name: name.into() }
}
fn migrate() -> Request {
    Request::Migrate { at: at(b"whole"), from: Box::from(*b"http://127.0.0.1:3001/fixture/source.git") }
}
#[test]
fn migration_and_deletion_requests_match_the_recording_and_server_routes() {
    for (case, request) in [
        ("setup-migrate-created", migrate()),
        ("setup-repository-delete", Request::DeleteRepository { at: at(b"whole") }),
    ] {
        let transcript = forgejo::transcript(case);
        let encoded = client::encode(&request, &forgejo::limits()).unwrap();
        assert_eq!(encoded.method, transcript.method);
        assert_eq!(encoded.target.as_ref(), transcript.target);
        assert_eq!(encoded.body.as_deref().unwrap_or_default(), transcript.request_body);
        assert_eq!(
            forgejo::decode_route(transcript.method, &transcript.target, &transcript.request_body, &forgejo::limits())
                .unwrap()
                .0,
            request
        );
    }
}
#[test]
fn a_whole_migration_decodes_as_created_repository_in_one_seven_and_whole_byte_chunks() {
    let transcript = forgejo::transcript("setup-migrate-created");
    assert_eq!(transcript.status, 201);
    for chunk in [1, 7, transcript.body.len()] {
        let decoded = forgejo::decode_chunks(&migrate(), &transcript.body, chunk);
        let Ok(Document::CreatedRepository(repository)) = decoded.document else {
            panic!("migration's created repository: {:?}", decoded.document);
        };
        assert_eq!(repository.id, 1);
        assert_eq!(&*repository.full_name, b"copies/whole");
        assert_eq!(&*repository.default_branch, b"other");
        assert_eq!(&*repository.clone_url, b"http://127.0.0.1:3000/copies/whole.git");
        assert!(repository.ssh_url.starts_with(b"ssh://"));
        assert!(repository.ssh_url.ends_with(b"@localhost/copies/whole.git"));
        assert_eq!(decoded.rows.len(), 0);
    }
}
#[test]
fn a_migration_preserves_name_taken_and_missing_source_messages() {
    for (case, status, message) in [
        ("setup-migrate-name-taken", 409, "The repository with the same name already exists."),
        (
            "setup-migrate-missing-source",
            422,
            "Authentication failed: Clone: exit status 128 - fatal: could not read Username for 'http://127.0.0.1:3001': terminal prompts disabled\n.",
        ),
    ] {
        let transcript = forgejo::transcript(case);
        assert_eq!(transcript.status, status);
        let decoded =
            forgejo::decode_answer(client::error_decoder(&forgejo::limits()), &transcript.body, &forgejo::limits());
        assert_eq!(decoded.document, Ok(Document::Error(ErrorBody { message: Body::Kept(message.as_bytes().into()) })));
    }
}
