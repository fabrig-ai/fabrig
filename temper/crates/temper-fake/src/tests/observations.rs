//! The process case's additive documents preserve seed and query identity.
use super::{forge_document, query};
use crate::{
    forge, limits,
    llm::observations::{self, Observation},
};
use serde_json::json;
use skein_fake_llm_domain::api::{Message, Part, Role};
use skein_lib::List;

#[test]
fn explicit_repository_ids_preserve_the_first_draft_tree() {
    let mut doc = forge_document();
    doc["repositories"] = json!([
        {"id":2,"name":"app","landing":"main","ci":null,"tree":[{"path":"draft.txt","content":"draft\n"}]},
        {"id":1,"name":"spare","landing":"main","ci":null}
    ]);
    let doc = forge::Document::decode(&serde_json::to_vec(&doc).unwrap()).unwrap();
    let (forge, config) = doc.seed(23);
    assert_eq!(config.repositories.iter().map(|r| r.id).collect::<Vec<_>>(), [2, 1]);
    let head = forge.domain.branch(b"fake/app", b"main").unwrap();
    assert_eq!(head, 1, "first commit is independent of repository id");
    let object = forge.domain.object(head).unwrap();
    assert_eq!(
        object.tree.iter().map(|(p, c)| (p.as_ref(), c.as_ref())).collect::<Vec<_>>(),
        [(b"draft.txt".as_slice(), b"draft\n".as_slice())]
    );
}

#[test]
fn ambiguous_ids_and_unsafe_or_duplicate_paths_are_refused() {
    for repositories in [
        json!([{"id":0,"name":"app","landing":"main","ci":null}]),
        json!([{"id":2,"name":"app","landing":"main","ci":null},{"name":"spare","landing":"main","ci":null}]),
        json!([{"id":2,"name":"app","landing":"main","ci":null},{"id":2,"name":"spare","landing":"main","ci":null}]),
        json!([{"name":"app","landing":"main","ci":null,"tree":[{"path":"../escape","content":""}]}]),
        json!([{"name":"app","landing":"main","ci":null,"tree":[{"path":"/absolute","content":""}]}]),
        json!([{"name":"app","landing":"main","ci":null,"tree":[{"path":"same","content":"a"},{"path":"same","content":"b"}]}]),
    ] {
        let mut doc = forge_document();
        doc["repositories"] = repositories;
        assert!(forge::Document::decode(&serde_json::to_vec(&doc).unwrap()).is_err(), "{doc}");
    }
}

#[test]
fn query_records_preserve_bytes_roles_and_part_order() {
    let messages = Box::new([
        Message {
            role: Role::User,
            parts: Box::new([
                Part::Text { text: b"text\n\0\xff".as_slice().into() },
                Part::Opaque { bytes: b"\0\xfe".as_slice().into() },
                Part::ToolOutput {
                    id: b"id".as_slice().into(),
                    output: b"\xff output".as_slice().into(),
                    is_error: true,
                },
            ]),
        },
        Message {
            role: Role::Assistant,
            parts: Box::new([
                Part::ToolCall {
                    id: b"id".as_slice().into(),
                    name: b"tool".as_slice().into(),
                    arguments: b"{ \"a\": 1 }".as_slice().into(),
                },
                Part::ToolOutput { id: b"second".as_slice().into(), output: b"ok".as_slice().into(), is_error: false },
            ]),
        },
    ]);
    for credential in [None, Some(0), Some(u32::MAX)] {
        let mut query = query(b"system\0\xff", b"");
        query.credential = credential;
        query.model = b"model\xfe".as_slice().into();
        query.messages = messages.clone();
        let mut out = List::with_capacity(limits::OBSERVATION_BYTES);
        observations::encode(&query, &mut out).unwrap();
        let Observation::Query(decoded) = observations::decode(out.as_slice()).unwrap() else { panic!("query") };
        assert_eq!(decoded.credential, credential);
        assert_eq!(decoded.model, query.model);
        assert_eq!(decoded.system, query.system);
        assert_eq!(decoded.messages, query.messages);
        // The stream buffer admits exactly a complete JSONL row, never a partial row.
        let length = out.len();
        let mut exact = List::with_capacity(length);
        observations::encode(&query, &mut exact).unwrap();
        assert_eq!(exact.as_slice(), out.as_slice());
        let mut short = List::with_capacity(length - 1);
        assert!(observations::encode(&query, &mut short).is_err());
        assert!(short.is_empty());
    }
}

#[test]
fn installation_snapshots_record_creation_and_deletion_once_and_stop_at_the_bound() {
    use crate::forge::observations::{Installations, Observation, decode};
    use skein_fake_forge_github::installations::{Installations as Store, Kind, Limits, Reach};
    use skein_lib::Wall;
    let mut store = Store::new(&Limits { installations: 2, tokens: 2, listed: 2 }, 0);
    let mut observer = Installations::new(2, 2);
    let mut out = List::with_capacity(4096);
    assert!(!observer.drain(&store, &mut out).unwrap());
    assert!(out.is_empty());
    let id = store.install(10, b"acme", Kind::Organization, Reach::All, Wall::EPOCH).unwrap();
    assert!(!observer.drain(&store, &mut out).unwrap());
    let length = out.len();
    assert!(!observer.drain(&store, &mut out).unwrap());
    assert_eq!(out.len(), length, "unchanged snapshots add no acceptance");
    assert!(store.remove(id));
    assert!(observer.drain(&store, &mut out).unwrap());
    let rows: Vec<_> = out
        .as_slice()
        .split(|b| *b == b'\n')
        .filter(|s| !s.is_empty())
        .map(|row| decode(row).unwrap().unwrap())
        .collect();
    assert_eq!(
        rows,
        [
            Observation::InstallationCreated { installation: id },
            Observation::InstallationDeleted { installation: id },
            Observation::InstallationsFull { capacity: 2 },
        ]
    );
    let length = out.len();
    assert!(observer.drain(&store, &mut out).unwrap());
    assert_eq!(out.len(), length, "full is terminal");
    for row in [
        br#"{"observation":"installation_created","installation":1,"token":"secret"}"#.as_slice(),
        br#"{"observation":"installation_deleted","installation":"ghs_secret"}"#,
        br#"{"observation":"installations_full","capacity":4294967296}"#,
    ] {
        assert!(decode(row).is_err(), "closed, bounded installation records");
    }
}
