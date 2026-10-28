//! Adversarial identity wire boundaries: protocol/channel.md, 4 and 6;
//! protocol/credentials.md, 7. Generated-code driving follows this world's
//! `worker_wire` harness; cases come from the design.
use skein_lib::{List, Reader, Secret};
use temper_protocol_worker_wire::{self as wire, v1};

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
fn an_empty_installation_list_remains_github() {
    let value = identity(vec![]);
    assert_eq!(decoded(&value), value);
    assert!(matches!(decoded(&value).kind(), v1::Kind::Github(entries) if entries.entries().is_empty()));
}

#[test]
fn installation_order_is_preserved_independently_of_account_id_order() {
    // Installation order is not account-id order. These account ids descend.
    let value = identity(vec![entry(90, b"AcMe", FIRST), entry(10, b"umbra", SECOND)]);
    let output = decoded(&value);
    let v1::Kind::Github(entries) = output.kind() else { panic!("GitHub kind") };
    let accounts: Vec<_> = entries.entries().iter().map(v1::Entry::account).collect();
    assert_eq!(accounts, [90, 10]);
    assert_eq!(output, value, "logins' case and each account's own token survive");
}

#[test]
fn account_ids_keep_all_sixty_four_bits() {
    let value = identity(vec![entry(u64::MAX, b"acme", FIRST), entry(1_u64 << 53, b"umbra", SECOND)]);
    assert_eq!(decoded(&value), value, "no JSON-number or u32 narrowing");
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
fn nested_identity_debug_never_prints_either_installation_token() {
    let value = identity(vec![entry(10, b"acme", FIRST), entry(20, b"umbra", SECOND)]);
    let v1::Kind::Github(entries) = value.kind() else { panic!("GitHub kind") };
    for printed in [format!("{value:?}"), format!("{entries:?}"), format!("{:?}", entries.entries())] {
        for token in [FIRST, SECOND] {
            assert!(!printed.contains(std::str::from_utf8(token).expect("ASCII token")));
            assert!(!printed.contains(&format!("{token:?}")), "no numeric secret bytes");
        }
    }
}

#[test]
fn entry_count_bound_is_enforced_at_construction() {
    let mut limits = v1::CEILINGS;
    limits.entries_entries = 2;
    for count in [2, 3] {
        let mut entries = List::with_capacity(count);
        for _ in 0..count {
            entries.push(entry(10, b"acme", FIRST)).expect("reserved list");
        }
        assert_eq!(v1::Entries::new(&limits, v1::EntriesParts { entries }).is_ok(), count == 2);
    }
}

#[test]
fn entry_count_bound_is_enforced_on_received_bytes() {
    let mut limits = v1::CEILINGS;
    limits.entries_entries = 2;
    for count in [2, 3] {
        let value = identity((0..count).map(|_| entry(10, b"acme", FIRST)).collect());
        let bytes = wire::identity(&value);
        assert_eq!(v1::Identity::decode(&limits, &mut Reader::new(bytes.expose())).is_ok(), count == 2);
    }
}

#[test]
fn installation_token_bound_is_enforced_on_received_bytes() {
    let mut limits = v1::CEILINGS;
    limits.entry_token = 32;
    for length in [32, 33] {
        let value = identity(vec![entry(10, b"acme", &vec![b't'; length])]);
        let bytes = wire::identity(&value);
        assert_eq!(v1::Identity::decode(&limits, &mut Reader::new(bytes.expose())).is_ok(), length == 32);
    }
}

#[test]
fn installation_token_bound_is_enforced_at_construction() {
    let mut limits = v1::CEILINGS;
    limits.entry_token = 32;
    for length in [32, 33] {
        let parts = v1::EntryParts {
            account: 10,
            account_login: b"acme".as_slice().into(),
            token: Secret::new(vec![b't'; length].into_boxed_slice()),
        };
        assert_eq!(v1::Entry::new(&limits, parts).is_ok(), length == 32);
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

#[test]
fn repository_account_absence_is_distinct_from_a_full_width_account() {
    use skein_lib::Writer;
    // A generated schema fixture supplies unrelated fields; only account varies.
    let bytes = include_bytes!("../../../crates/temper-protocol-worker-wire/golden/record_repository_full.bin");
    let original = v1::Repository::decode(&v1::CEILINGS, &mut Reader::new(bytes)).expect("schema fixture");
    for account in [None, Some(1_u64 << 53), Some(u64::MAX)] {
        let value =
            v1::Repository::new(&v1::CEILINGS, v1::RepositoryParts { account, ..original.clone().into_parts() })
                .expect("repository account");
        let mut writer = Writer::new(usize::try_from(value.measure()).expect("small repository"));
        value.encode(&mut writer).expect("measured capacity");
        let bytes = writer.finish();
        let output = v1::Repository::decode(&v1::CEILINGS, &mut Reader::new(&bytes)).expect("repository decoded");
        assert_eq!(output.account(), &account);
        assert_eq!(output, value);
    }
}
