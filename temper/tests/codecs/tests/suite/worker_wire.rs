//! Identity kinds and repository accounts (protocol/channel.md, sections 4 and 6).
use skein_lib::{List, Reader, Secret, Writer};
use temper_protocol_worker_wire::{self as wire, v1};

fn identities() -> [v1::Identity; 2] {
    let limits = &v1::CEILINGS;
    let forgejo = v1::Kind::Forgejo(
        v1::Token::new(limits, v1::TokenParts { token: Secret::new(Box::from(&b"forge-token-1234"[..])) })
            .expect("within ceilings"),
    );
    let mut entries = List::with_capacity(2);
    for (account, login, token) in [
        (19, b"owner-z".as_slice(), b"installation-token-first".as_slice()),
        (7, b"owner-a".as_slice(), b"installation-token-second-longer".as_slice()),
    ] {
        entries
            .push(
                v1::Entry::new(
                    limits,
                    v1::EntryParts { account, account_login: login.into(), token: Secret::new(token.into()) },
                )
                .expect("within ceilings"),
            )
            .expect("within ceilings");
    }
    let github = v1::Kind::Github(v1::Entries::new(limits, v1::EntriesParts { entries }).expect("within ceilings"));
    [forgejo, github].map(|kind| {
        v1::Identity::new(
            limits,
            v1::IdentityParts {
                kind,
                login: b"temper".as_slice().into(),
                name: b"Temper".as_slice().into(),
                email: b"t@forge.example".as_slice().into(),
            },
        )
        .expect("within ceilings")
    })
}

#[test]
fn an_identitys_debug_holds_no_byte_of_its_token() {
    for identity in identities() {
        let printed = format!("{identity:?}");
        let tokens = match identity.kind() {
            v1::Kind::Forgejo(value) => vec![value.token()],
            v1::Kind::Github(value) => value.entries().iter().map(v1::Entry::token).collect(),
        };
        for token in tokens {
            let text = core::str::from_utf8(token.expose()).expect("ASCII fixture token");
            assert!(!printed.contains(text), "{printed}");
            assert!(!printed.contains(&format!("{:?}", token.expose())), "no byte array: {printed}");
            assert!(printed.contains(&format!("{token:?}")), "only the token's length: {printed}");
        }
    }
}

#[test]
fn each_identity_kind_round_trips() {
    for identity in identities() {
        let bytes = wire::identity(&identity);
        let decoded = v1::Identity::decode(&v1::CEILINGS, &mut Reader::new(bytes.expose())).expect("within ceilings");
        assert_eq!(decoded, identity, "including entries in order");
    }
}

#[test]
fn an_unknown_identity_kind_is_refused() {
    let mut bytes =
        include_bytes!("../../../../crates/temper-protocol-worker-wire/golden/record_identity_full.bin").to_vec();
    bytes[0] = 2; // One past github's tag, in Identity.kind.
    let problem = v1::Identity::decode(&v1::CEILINGS, &mut Reader::new(&bytes)).expect_err("unknown kind tag");
    assert_eq!(problem.path, v1::Path::KindTag);
}

#[test]
fn a_repositorys_account_round_trips() {
    let golden = include_bytes!("../../../../crates/temper-protocol-worker-wire/golden/record_repository_full.bin");
    let original = v1::Repository::decode(&v1::CEILINGS, &mut Reader::new(golden)).expect("within ceilings");
    for account in [Some(0x1234_5678_9abc_def0), None] {
        let parts = v1::RepositoryParts { account, ..original.clone().into_parts() };
        let repository = v1::Repository::new(&v1::CEILINGS, parts).expect("within ceilings");
        let mut writer = Writer::new(usize::try_from(repository.measure()).expect("measured length fits usize"));
        repository.encode(&mut writer).expect("measured room");
        let decoded =
            v1::Repository::decode(&v1::CEILINGS, &mut Reader::new(&writer.finish())).expect("within ceilings");
        assert_eq!(decoded, repository);
        assert_eq!(*decoded.account(), account);
    }
}
