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
        let printed = match identity.kind() {
            v1::Kind::Forgejo(_) => vec![format!("{identity:?}")],
            v1::Kind::Github(entries) => {
                vec![format!("{identity:?}"), format!("{entries:?}"), format!("{:?}", entries.entries())]
            }
        };
        let tokens = match identity.kind() {
            v1::Kind::Forgejo(value) => vec![value.token()],
            v1::Kind::Github(value) => value.entries().iter().map(v1::Entry::token).collect(),
        };
        for printed in printed {
            for token in &tokens {
                let text = core::str::from_utf8(token.expose()).expect("ASCII fixture token");
                assert!(!printed.contains(text), "{printed}");
                assert!(!printed.contains(&format!("{:?}", token.expose())), "no byte array: {printed}");
                assert!(printed.contains(&format!("{token:?}")), "only the token's length: {printed}");
            }
        }
    }
}

#[test]
fn each_identity_kind_round_trips() {
    for identity in identities().into_iter().chain([
        identity(vec![]),
        identity(vec![entry(u64::MAX, b"acme", FIRST), entry(1_u64 << 53, b"umbra", SECOND)]),
    ]) {
        assert_eq!(decoded(&identity), identity, "including empty entries, full-width ids and installation order");
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
    for account in [Some(0x1234_5678_9abc_def0), None, Some(1_u64 << 53), Some(u64::MAX)] {
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

const FIRST: &[u8] = b"qa-installation-token-one-distinct";
const SECOND: &[u8] = b"qa-installation-token-two-distinct";

fn entry(account: u64, login: &[u8], token: &[u8]) -> v1::Entry {
    v1::Entry::new(
        &v1::CEILINGS,
        v1::EntryParts { account, account_login: login.into(), token: Secret::new(token.into()) },
    )
    .expect("fixture entry")
}
fn identity(entries: Vec<v1::Entry>) -> v1::Identity {
    let mut list = List::with_capacity(u32::try_from(entries.len()).expect("small fixture list"));
    for entry in entries {
        list.push(entry).expect("reserved entries");
    }
    let entries = v1::Entries::new(&v1::CEILINGS, v1::EntriesParts { entries: list }).expect("fixture entries");
    v1::Identity::new(
        &v1::CEILINGS,
        v1::IdentityParts {
            kind: v1::Kind::Github(entries),
            login: b"x-access-token".as_slice().into(),
            name: b"temper-test[bot]".as_slice().into(),
            email: b"4243+temper-test[bot]@users.noreply.github.com".as_slice().into(),
        },
    )
    .expect("fixture identity")
}
fn decoded(value: &v1::Identity) -> v1::Identity {
    let bytes = wire::identity(value);
    let mut reader = Reader::new(bytes.expose());
    let result = v1::Identity::decode(&v1::CEILINGS, &mut reader).expect("valid identity");
    assert!(reader.is_empty(), "the identity consumes exactly its encoding");
    result
}

#[test]
fn every_truncated_identity_is_refused() {
    let bytes = wire::identity(&identity(vec![entry(10, b"acme", FIRST), entry(20, b"umbra", SECOND)]));
    for length in 0..bytes.expose().len() {
        assert!(
            v1::Identity::decode(&v1::CEILINGS, &mut Reader::new(&bytes.expose()[..length])).is_err(),
            "accepted truncated identity at {length}"
        );
    }
}

#[test]
fn installation_count_is_bounded_at_both_entrances() {
    const ENTRIES: u32 = 2;
    let mut limits = v1::CEILINGS;
    limits.entries_entries = ENTRIES;
    for (count, accepted) in [(ENTRIES, true), (ENTRIES + 1, false)] {
        let value = identity((0..count).map(|_| entry(10, b"acme", FIRST)).collect());
        let v1::Kind::Github(entries) = value.kind() else { panic!("GitHub kind") };
        assert_eq!(v1::Entries::new(&limits, entries.clone().into_parts()).is_ok(), accepted);
        let bytes = wire::identity(&value);
        assert_eq!(v1::Identity::decode(&limits, &mut Reader::new(bytes.expose())).is_ok(), accepted);
    }
}

#[test]
fn installation_tokens_are_bounded_at_both_entrances() {
    const TOKEN_BYTES: u32 = 32;
    let mut limits = v1::CEILINGS;
    limits.entry_token = TOKEN_BYTES;
    for (length, accepted) in [(TOKEN_BYTES, true), (TOKEN_BYTES + 1, false)] {
        let token = vec![b't'; usize::try_from(length).expect("small fixture token")];
        let item = entry(10, b"acme", &token);
        assert_eq!(v1::Entry::new(&limits, item.clone().into_parts()).is_ok(), accepted);
        let bytes = wire::identity(&identity(vec![item]));
        assert_eq!(v1::Identity::decode(&limits, &mut Reader::new(bytes.expose())).is_ok(), accepted);
    }
}

#[test]
fn github_tokens_use_the_installation_bound_instead_of_forgejos_bound() {
    let mut limits = v1::CEILINGS;
    limits.token_token = 8;
    limits.entry_token = 64;
    let value = identity(vec![entry(10, b"acme", FIRST)]);
    let bytes = wire::identity(&value);
    assert!(v1::Identity::decode(&limits, &mut Reader::new(bytes.expose())).is_ok());
    assert!(v1::Token::new(&limits, v1::TokenParts { token: Secret::new(FIRST.into()) }).is_err());
}
