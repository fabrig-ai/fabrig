//! Domain bounds and identity field rules (protocol/channel.md, sections 6 and 8).
mod fixtures;
use crate::v1;
use alloc::boxed::Box;
use skein_lib::List;

/// Another family's version is distinct from a malformed Items body (protocol/wires.md, 5).
#[test]
fn items_refuse_another_version_before_reading_the_body() {
    use skein_codec::Reason;
    use skein_lib::Reader;

    let limits = fixtures::limits();
    let value = v1::Items::new(&limits, fixtures::items(1)).expect("items within bounds");
    let bytes = crate::items(&value);
    assert_eq!(skein_codec::version_of(&bytes), Ok(crate::wire::VERSION));
    assert_eq!(v1::Items::decode(&limits, &mut Reader::new(&bytes)), Ok(value));

    let other = crate::wire::VERSION.checked_sub(1).expect("a preceding version").to_be_bytes();
    let mut changed = bytes;
    for (byte, value) in changed.iter_mut().zip(other) {
        *byte = value;
    }
    for bytes in [changed.as_ref(), other.as_slice()] {
        assert_eq!(
            v1::Items::decode(&limits, &mut Reader::new(bytes)).expect_err("another version"),
            v1::Problem { path: v1::Path::ItemsVersion, reason: Reason::Version }
        );
    }
    assert_eq!(
        v1::Items::decode(&limits, &mut Reader::new(&crate::wire::VERSION.to_be_bytes()))
            .expect_err("missing repository count"),
        v1::Problem { path: v1::Path::ItemsRepositories, reason: Reason::Short }
    );
}

#[test]
fn limits_take_each_bound_from_its_source_field() {
    assert_eq!(
        fixtures::limits(),
        v1::Limits {
            merge_branch: 16,
            merged_branch: 16,
            merged_heads: 64,
            branch_branch: 16,
            writable_push: 16,
            writable_landing_some: 16,
            repository_name: 16,
            repository_remote: 48,
            repository_git_some: 4096,
            items_repositories: 2,
            job_spec_command: 4096,
            job_spec_request: 1_048_576,
            repository_save_some: 16,
            left_landed: 2,
            files_files: 2,
            files_files_item: 8,
            diagnostic_output: 512,
            saved_repositories: 2,
            token_token: 40,
            entries_entries: 3,
            entry_token: 24,
            entry_account_login: 16,
            identity_login: 16,
            identity_name: 16,
            identity_email: 16,
        }
    );
    let bounds = crate::Bounds { repositories: 65, ..fixtures::BOUNDS };
    assert_eq!(crate::limits(&bounds), None, "past the repository ceiling");
}

/// temper/docs/design/protocol/tooling.md, 2 and 6: command and request bytes are bounded.
#[test]
fn a_job_specs_fields_stop_at_their_bounds_when_built_and_decoded() {
    let limits = v1::Limits { job_spec_command: 8, job_spec_request: 16, ..fixtures::limits() };
    for (command, bound, path) in
        [(true, 8_usize, v1::Path::JobSpecCommand), (false, 16_usize, v1::Path::JobSpecRequest)]
    {
        for past in [false, true] {
            let length = bound.checked_add(usize::from(past)).expect("test length");
            let parts = v1::JobSpecParts {
                command: if command { skein_lib::bytes::zeroed(length) } else { b"tool".as_slice().into() },
                request: if command { b"{}".as_slice().into() } else { skein_lib::bytes::zeroed(length) },
            };
            let value = v1::JobSpec::new(&v1::CEILINGS, parts).expect("within schema ceilings");
            let built = v1::JobSpec::new(&limits, value.clone().into_parts());
            let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).expect("bounded length"));
            value.encode(&mut writer).expect("measured room");
            let bytes = writer.finish();
            let mut reader = skein_lib::Reader::new(&bytes);
            let decoded = v1::JobSpec::decode(&limits, &mut reader);
            if past {
                let problem = v1::Problem { path, reason: skein_codec::Reason::Bound };
                assert_eq!(built.expect_err("past bound"), problem);
                assert_eq!(decoded.expect_err("past bound"), problem);
            } else {
                assert_eq!(built.expect("at bound"), value);
                assert_eq!(decoded.expect("at bound"), value);
                assert!(reader.is_empty());
            }
        }
    }
}

#[test]
fn items_one_repository_past_their_bound_are_refused() {
    let limits = fixtures::limits();
    v1::Items::new(&limits, fixtures::items(2)).expect("at the bound");
    let problem = v1::Items::new(&limits, fixtures::items(3)).expect_err("past the count");
    assert_eq!(problem, v1::Problem { path: v1::Path::ItemsRepositories, reason: skein_codec::Reason::Bound });
}

#[test]
fn repository_byte_fields_stop_at_their_bounds() {
    let limits = fixtures::limits();
    for (program, bound, path) in
        [(false, 48_usize, v1::Path::RepositoryRemote), (true, 4096_usize, v1::Path::RepositoryGit)]
    {
        for past in [false, true] {
            let length = if past { bound.checked_add(1).expect("test length") } else { bound };
            let mut parts = fixtures::repository_parts(1, Box::from(&b"remote"[..]));
            let mut bytes = skein_lib::bytes::zeroed(length);
            bytes.fill(b'x');
            if program {
                parts.git = Some(bytes);
            } else {
                parts.remote = bytes;
            }
            let made = v1::Repository::new(&limits, parts);
            if past {
                assert_eq!(made.expect_err("past bound"), v1::Problem { path, reason: skein_codec::Reason::Bound });
            } else {
                made.expect("at bound");
            }
        }
    }
}

#[test]
fn saved_conflicts_past_their_count_are_refused() {
    let mut files = List::with_capacity(3);
    for _ in 0_u32..3 {
        files.push(Box::from(&b"file"[..])).expect("counted room");
    }
    let conflicts = v1::Files::new(&v1::CEILINGS, v1::FilesParts { files }).expect("within ceilings");
    let repository = v1::SavedRepository::new(
        &v1::CEILINGS,
        v1::SavedRepositoryParts { tag: 1, from_saved: false, landing: v1::Landing::Conflicted(conflicts) },
    )
    .expect("within ceilings");
    let mut repositories = List::with_capacity(1);
    repositories.push(repository).expect("counted room");
    let problem =
        v1::Saved::new(&fixtures::limits(), v1::SavedParts { repositories }).expect_err("past the conflicts count");
    assert_eq!(problem, v1::Problem { path: v1::Path::FilesFiles, reason: skein_codec::Reason::Bound });
}

#[test]
fn largest_parts_are_the_generated_encoded_bounds() {
    let limits = fixtures::limits();
    let sizes = crate::largest(&limits).expect("test limits fit");
    assert_eq!(u64::from(sizes.items), v1::Items::encoded_bound(&limits).expect("items"));
    assert_eq!(u64::from(sizes.left), v1::Left::encoded_bound(&limits).expect("left"));
    assert_eq!(u64::from(sizes.saved), v1::Saved::encoded_bound(&limits).expect("saved"));
    assert_eq!(u64::from(sizes.resource), v1::Preparation::encoded_bound(&limits).expect("preparation"));
    assert_eq!(u64::from(sizes.grant), v1::Identity::encoded_bound(&limits).expect("identity"));
    // Merged adds a list length and 64 full ids in place of Merge's base id.
    assert_eq!(v1::Items::encoded_bound(&v1::CEILINGS), Some(792_260));
    // At ceilings: 64 * 12,379-byte repositories + the four-byte list length.
    assert_eq!(sizes, crate::Largest { items: 12730, left: 84, saved: 1084, resource: 41, grant: 233 });
}

#[test]
fn identity_rules_refuse_each_forbidden_byte() {
    let original = fixtures::identity(b"forge-token");
    assert!(crate::valid(&original), "ordinary identity");
    let rows: [(&[u8], &[u8], &[u8]); 12] = [
        (b"a:b", b"name", b"email"),
        (b"a\x7f", b"name", b"email"),
        (b"login", b"", b"email"),
        (b"login", b"name", b""),
        (b"login", b"a\0b", b"email"),
        (b"login", b"a\nb", b"email"),
        (b"login", b"a<b", b"email"),
        (b"login", b"a>b", b"email"),
        (b"login", b"name", b"a\0b"),
        (b"login", b"name", b"a\nb"),
        (b"login", b"name", b"a<b"),
        (b"login", b"name", b"a>b"),
    ];
    for kind in [
        original.kind().clone(),
        v1::Kind::Github(
            v1::Entries::new(&fixtures::limits(), v1::EntriesParts { entries: List::with_capacity(0) })
                .expect("empty entries"),
        ),
    ] {
        for (row, (login, name, email)) in rows.iter().enumerate() {
            let parts = v1::IdentityParts {
                kind: kind.clone(),
                login: Box::from(*login),
                name: Box::from(*name),
                email: Box::from(*email),
            };
            let identity = v1::Identity::new(&fixtures::limits(), parts).expect("bounded valid text");
            assert!(!crate::valid(&identity), "forbidden row {row}");
        }
    }
}

#[test]
fn a_github_values_entries_stop_at_their_bound() {
    let limits = fixtures::limits();
    for (count, accepted) in
        [(limits.entries_entries, true), (limits.entries_entries.checked_add(1).expect("test count"), false)]
    {
        let mut entries = List::with_capacity(count);
        for account in 0..count {
            entries
                .push(
                    v1::Entry::new(
                        &limits,
                        v1::EntryParts {
                            account: u64::from(account),
                            account_login: Box::from(&b"owner"[..]),
                            token: skein_lib::Secret::new(Box::from(&b"token"[..])),
                        },
                    )
                    .expect("entry"),
                )
                .expect("counted room");
        }
        let kind = v1::Kind::Github(v1::Entries::new(&v1::CEILINGS, v1::EntriesParts { entries }).expect("ceiling"));
        let parts = v1::IdentityParts { kind, ..fixtures::identity(b"token").into_parts() };
        let made = v1::Identity::new(&limits, parts);
        if accepted {
            made.expect("at bound");
        } else {
            assert_eq!(
                made.expect_err("past bound"),
                v1::Problem { path: v1::Path::EntriesEntries, reason: skein_codec::Reason::Bound }
            );
        }
    }
}

#[test]
fn an_entrys_token_and_login_stop_at_their_bounds() {
    let limits = fixtures::limits();
    for (field, bound, path) in [
        ("token", limits.entry_token, v1::Path::EntryToken),
        ("login", limits.entry_account_login, v1::Path::EntryAccountLogin),
    ] {
        for past in [false, true] {
            let length = usize::try_from(bound.checked_add(u32::from(past)).expect("test bound")).expect("test length");
            let mut bytes = skein_lib::bytes::zeroed(length);
            bytes.fill(b'x');
            let mut parts = v1::EntryParts {
                account: 7,
                account_login: Box::from(&b"owner"[..]),
                token: skein_lib::Secret::new(Box::from(&b"token"[..])),
            };
            if field == "token" {
                parts.token = skein_lib::Secret::new(bytes);
            } else {
                parts.account_login = bytes;
            }
            let made = v1::Entry::new(&limits, parts);
            if past {
                assert_eq!(
                    made.expect_err("past bound"),
                    v1::Problem { path, reason: skein_codec::Reason::Bound },
                    "{field}"
                );
            } else {
                made.expect("at bound");
            }
        }
    }
}

#[test]
fn landing_branch_uses_the_name_bound_and_largest_items_count_it() {
    let limits = fixtures::limits();
    for length in [16, 17] {
        let parts = v1::WritableParts {
            push: Box::from(&b"topic"[..]),
            expected: None,
            landing: Some(skein_lib::bytes::zeroed(length)),
        };
        let value = v1::Writable::new(&limits, parts);
        if length == 16 {
            let value = value.expect("at bound");
            let mut bytes = skein_lib::Writer::new(usize::try_from(value.measure()).unwrap());
            value.encode(&mut bytes).unwrap();
            assert_eq!(v1::Writable::decode(&limits, &mut skein_lib::Reader::new(&bytes.finish())).unwrap(), value);
        } else {
            assert_eq!(value.expect_err("past bound").path, v1::Path::WritableLanding);
        }
    }
    let without = v1::Limits { writable_landing_some: 0, ..limits };
    assert_eq!(
        crate::largest(&limits).unwrap().items - crate::largest(&without).unwrap().items,
        limits.items_repositories * limits.writable_landing_some
    );
}

fn repository_with_save(length: u32) -> Result<v1::Repository, v1::Problem> {
    let mut parts = fixtures::repository_parts(1, b"remote".as_slice().into());
    parts.save = Some(skein_lib::bytes::zeroed(usize::try_from(length).expect("bounded length")));
    v1::Repository::new(&fixtures::limits(), parts)
}

#[test]
fn a_repositorys_save_branch_at_its_name_bound_is_admitted() {
    let limits = fixtures::limits();
    let repository = repository_with_save(limits.repository_save_some).expect("at the bound");
    let mut writer = skein_lib::Writer::new(usize::try_from(repository.measure()).expect("bounded length"));
    repository.encode(&mut writer).expect("measured room");
    let bytes = writer.finish();
    let mut reader = skein_lib::Reader::new(&bytes);
    assert_eq!(v1::Repository::decode(&limits, &mut reader).expect("roundtrip"), repository);
    assert!(reader.is_empty());
}

#[test]
fn a_repositorys_save_branch_past_its_name_bound_is_refused() {
    assert_eq!(
        repository_with_save(fixtures::limits().repository_save_some + 1).expect_err("past the bound").path,
        v1::Path::RepositorySave
    );
}

#[test]
fn merged_heads_stop_at_sixty_four_when_built_and_decoded() {
    const HEADS: u32 = 64; // channel.md, 4: at most 64 heads.
    let limits = fixtures::limits();
    let mut heads = List::with_capacity(HEADS);
    for _ in 0..HEADS {
        heads.push([3; 32]).expect("counted heads");
    }
    let value = v1::Merged::new(&limits, v1::MergedParts { branch: Box::from(&b"main"[..]), heads }).expect("at bound");
    let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).expect("bounded length"));
    value.encode(&mut writer).expect("measured room");
    let bytes = writer.finish();
    assert_eq!(v1::Merged::decode(&limits, &mut skein_lib::Reader::new(&bytes)).expect("roundtrip"), value);
    let smaller = v1::Limits { merged_heads: HEADS - 1, ..limits };
    assert_eq!(
        v1::Merged::decode(&smaller, &mut skein_lib::Reader::new(&bytes)).expect_err("past decode bound").path,
        v1::Path::MergedHeads
    );
    let mut parts = value.into_parts();
    // List capacity is fixed, so supply the past-bound count in a fresh list.
    let mut heads = List::with_capacity(HEADS + 1);
    for _ in 0..=HEADS {
        heads.push([3; 32]).expect("counted heads");
    }
    parts.heads = heads;
    assert_eq!(v1::Merged::new(&limits, parts).expect_err("past bound").path, v1::Path::MergedHeads);
}
