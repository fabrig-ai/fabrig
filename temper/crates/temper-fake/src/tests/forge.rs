//! Source catalogue admission and seeding (protocol/setup/box.md, 5).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary tests edit bounded JSON fixtures with serde"
)]
use crate::forge::Document;
use serde_json::{Value, json};

fn fixture() -> Value {
    json!({
        "listen":{"socket":"127.0.0.1:0"}, "admin_token":"/fixture/admin.token", "owner":"person",
        "repositories":[{"name":"existing","landing":"main","ci":null}],
        "sources":[{
            "address":"https://code.example.invalid/acme/widget",
            "repository":{"owner":"acme","name":"widget","landing":"trunk","branches":["topic"],
                "tree":[{"path":".github/workflows/ci.yml","content":"name: ci\n"}]}
        }]
    })
}
fn decode(value: &Value) -> Result<Document, crate::Refusal> {
    Document::decode(&serde_json::to_vec(value).expect("fixture JSON"))
}
fn sources(count: u32) -> Value {
    let source = fixture()["sources"][0].clone();
    (0..count)
        .map(|i| {
            let mut source = source.clone();
            source["address"] = json!(format!("https://code.example.invalid/acme/source-{i}"));
            source["repository"]["name"] = json!(format!("source-{i}"));
            source
        })
        .collect()
}

#[test]
fn source_hosts_outside_invalid_are_refused_by_name() {
    let mut value = fixture();
    value["sources"][0]["address"] = json!("https://code.example.org/acme/widget");
    assert_eq!(
        decode(&value).expect_err("real host refused").line().as_ref(),
        b"sources[0].address.code.example.org: expected a host ending in .invalid\n"
    );
}

#[test]
fn sources_seed_their_trees_branches_and_catalogue() {
    for private in [None, Some(false), Some(true)] {
        let mut value = fixture();
        if let Some(private) = private {
            value["sources"][0]["private"] = json!(private);
        }
        let document = decode(&value).expect("source document");
        let (mut forge, config) = document.seed(7);
        let [source] = config.sources.as_ref() else { panic!("one source") };
        assert_eq!(source.address.as_ref(), b"https://code.example.invalid/acme/widget");
        assert_eq!(source.repository.as_ref(), b"acme/widget");
        assert_eq!(source.private, private.unwrap_or(false));
        assert_eq!(source.fill, skein_fake_forge_domain::api::Fill::Whole);
        assert_eq!(
            forge.domain.permission(&source.repository, 1),
            Ok(skein_fake_forge_domain::api::Permission::Read),
            "the site admin can fetch the catalogue source for migration"
        );
        assert!(skein_fake_forge_forgejo::valid(&crate::wire::forge_limits(), &config));
        assert_eq!(
            forge.domain.metadata(&source.repository).expect("seeded repository").default_branch.as_ref(),
            b"trunk"
        );
        let first = forge.domain.branch(&source.repository, b"trunk").expect("default branch");
        assert_eq!(forge.domain.branch(&source.repository, b"topic"), Some(first));
        let object = forge.domain.object(first).expect("source commit");
        assert_eq!(
            object.tree.get(b".github/workflows/ci.yml".as_slice()).expect("source workflow").as_ref(),
            b"name: ci\n"
        );
        assert!(object.parent.is_none());
        assert!(forge.domain.pop_observation().is_none());
    }
}

#[test]
fn seeded_repository_owners_have_distinct_wire_identities() {
    let mut value = fixture();
    value["sources"] = sources(3);
    value["sources"][2]["repository"]["owner"] = json!("another");
    let document = decode(&value).expect("source document");
    let (_, config) = document.seed(7);
    assert_eq!(config.users.len(), 3, "repeated owners share one identity");
    let identities = [b"person".as_slice(), b"acme", b"another"].map(|owner| {
        config.users.iter().find(|(_, login)| login.as_ref() == owner).expect("repository owner has a wire identity").0
    });
    for (at, id) in identities.iter().enumerate() {
        assert_ne!(*id, 0);
        assert!(!identities[..at].contains(id), "distinct owners have distinct identities");
    }
    assert_eq!(config.users, document.seed(7).1.users, "owner identities survive a fresh seed");
}

#[test]
fn sources_fill_the_remaining_repository_capacity() {
    // The fixture seeds one ordinary repository before its sources.
    const SOURCE_COUNT: u32 = crate::ENTRIES - 1;
    let mut value = fixture();
    value["sources"] = sources(SOURCE_COUNT);
    let document = decode(&value).expect("sources fit remaining repository slots");
    let (forge, config) = document.seed(7);
    assert_eq!(config.sources.len(), usize::try_from(SOURCE_COUNT).expect("count fits"));
    assert_eq!(config.repositories.len(), crate::ENTRIES);
    for source in &config.sources {
        assert!(forge.domain.branch(&source.repository, b"trunk").is_some());
        assert!(forge.domain.branch(&source.repository, b"topic").is_some());
    }
}

#[test]
fn invalid_source_fields_and_excess_sources_are_refused() {
    let source = fixture()["sources"][0].clone();
    for (pointer, replacement, expected) in [
        ("/sources/0/private", json!("yes"), "sources[0].private: wrong value kind\n"),
        (
            "/sources/0/address",
            json!("http://code.example.invalid/acme/widget"),
            "sources[0].address: expected an https source address with a DNS host\n",
        ),
        (
            "/sources/0/address",
            json!("https://-code.invalid/acme/widget"),
            "sources[0].address: expected an https source address with a DNS host\n",
        ),
        (
            "/sources/0/address",
            json!("https://code.invalid:0/acme/widget"),
            "sources[0].address: expected an https source address with a DNS host\n",
        ),
        ("/sources/0/repository/owner", json!("bad/name"), "sources[0].repository.owner: expected a Forgejo login\n"),
        (
            "/sources/0/repository/name",
            json!("bad/name"),
            "sources[0].repository.name: expected a Forgejo repository name\n",
        ),
        (
            "/sources/0/repository/landing",
            json!("bad..branch"),
            "sources[0].repository.landing: expected a branch name\n",
        ),
        (
            "/sources/0/repository/branches",
            json!(["bad..branch"]),
            "sources[0].repository.branches[0]: expected a branch name\n",
        ),
        (
            "/sources/0/repository/branches",
            json!(["trunk"]),
            "sources[0].repository.branches[0]: must be unique and other than landing\n",
        ),
        (
            "/sources/0/repository/branches",
            json!(["topic", "topic"]),
            "sources[0].repository.branches[1]: must be unique and other than landing\n",
        ),
        (
            "/sources/0/repository/tree",
            json!([{"path":"../outside","content":""}]),
            "sources[0].repository.tree[0].path: expected a relative file path with no . or .. component\n",
        ),
        (
            "/sources/0",
            json!({"address":"https://code.invalid/acme/widget","repository":source["repository"],"extra":true}),
            "sources[0].extra: unknown member\n",
        ),
        ("/sources", json!([source, source]), "sources[1]: source address and repository must be unique\n"),
        ("/sources", sources(crate::ENTRIES), "sources: sources and repositories together are past their bound\n"),
        ("/sources", sources(crate::ENTRIES + 1), "sources: past its bound\n"),
        (
            "/repositories/0",
            json!({"id":u32::MAX,"name":"existing","landing":"main","ci":null}),
            "sources: no repository ids left for sources\n",
        ),
    ] {
        let mut value = fixture();
        value["sources"][0]["private"] = json!(false);
        *value.pointer_mut(pointer).expect(pointer) = replacement;
        assert_eq!(decode(&value).expect_err(pointer).line().as_ref(), expected.as_bytes(), "{pointer}");
    }
}
