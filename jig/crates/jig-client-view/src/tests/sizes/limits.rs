#[test]
fn valid_refuses_each_unrepresentable_limit() {
    use crate::tests::fixtures::Profile;
    use crate::valid;
    use skein_lib::Duration;
    let base = crate::tests::state::domain::spine::LIMITS;
    assert!(valid(&base, &(base).client));
    let cases = [
        Profile {
            view: crate::Config { ..(base).view },
            client: jig_client::DomainLimits { watches: 0, ..(base).client },
        },
        Profile {
            view: crate::Config { notice_life: Duration::ZERO, ..(base).view },
            client: jig_client::DomainLimits { ..(base).client },
        },
        Profile {
            view: crate::Config { linger: Duration::ZERO, ..(base).view },
            client: jig_client::DomainLimits { ..(base).client },
        },
        Profile {
            view: crate::Config { save_delay: Duration::ZERO, ..(base).view },
            client: jig_client::DomainLimits { ..(base).client },
        },
        Profile {
            view: crate::Config { sources: 0, ..(base).view },
            client: jig_client::DomainLimits { ..(base).client },
        },
        Profile {
            view: crate::Config { conversation_words: base.intents - 1, ..(base).view },
            client: jig_client::DomainLimits { ..(base).client },
        },
        Profile {
            view: crate::Config { ..(base).view },
            client: jig_client::DomainLimits { records: u32::MAX, ..(base).client },
        },
        Profile {
            view: crate::Config { objects: u32::MAX, ..(base).view },
            client: jig_client::DomainLimits { ..(base).client },
        },
        Profile {
            view: crate::Config { notices: u32::MAX, ..(base).view },
            client: jig_client::DomainLimits { ..(base).client },
        },
        Profile {
            view: crate::Config { ..(base).view },
            client: jig_client::DomainLimits { rows: u32::MAX, ..(base).client },
        },
        // The timer's tick and the tree's final name must fit.
        Profile {
            view: crate::Config { notices: u32::MAX - base.objects - base.client.records, ..(base).view },
            client: jig_client::DomainLimits { ..(base).client },
        },
        Profile {
            view: crate::Config { ..(base).view },
            client: jig_client::DomainLimits { rows: 65_536, ..(base).client },
        },
    ];
    for (case, limits) in cases.into_iter().enumerate() {
        assert!(!valid(&limits, &(limits).client), "case {case}");
    }
}

#[test]
fn validity_does_not_price_retained_documents() {
    let limits = crate::tests::fixtures::Profile {
        view: crate::Config { ..(crate::tests::state::domain::spine::LIMITS).view },
        client: jig_client::DomainLimits {
            document_bytes: u64::MAX,
            ..(crate::tests::state::domain::spine::LIMITS).client
        },
    };
    assert!(crate::valid(&limits, &(limits).client));
}

mod windows {
    #[test]
    fn valid_refuses_each_unrepresentable_limit() {
        let base = crate::tests::state::domain::spine::LIMITS;
        assert!(crate::state::windows::valid(&base.client));
        for (limits, expected) in [
            (
                crate::tests::fixtures::Profile {
                    view: crate::Config { ..(base).view },
                    client: jig_client::DomainLimits { rows: u32::MAX / 2, ..(base).client },
                },
                true,
            ),
            (
                crate::tests::fixtures::Profile {
                    view: crate::Config { ..(base).view },
                    client: jig_client::DomainLimits { rows: u32::MAX, ..(base).client },
                },
                false,
            ),
        ] {
            assert_eq!(crate::state::windows::valid(&limits.client), expected);
        }
    }
}

mod objects {
    #[test]
    fn valid_refuses_each_unrepresentable_limit() {
        let base = crate::tests::state::domain::spine::LIMITS;
        assert!(crate::state::objects::valid(&base));
        for (limits, expected) in [
            (
                crate::tests::fixtures::Profile {
                    view: crate::Config { objects: u32::MAX / 2, ..(base).view },
                    client: jig_client::DomainLimits { ..(base).client },
                },
                true,
            ),
            (
                crate::tests::fixtures::Profile {
                    view: crate::Config { objects: u32::MAX, ..(base).view },
                    client: jig_client::DomainLimits { ..(base).client },
                },
                false,
            ),
        ] {
            assert_eq!(crate::state::objects::valid(&limits), expected);
        }
    }
}

mod notices_kept {
    #[test]
    fn valid_refuses_each_unrepresentable_limit() {
        let base = crate::tests::state::domain::spine::LIMITS;
        assert!(crate::tree::notices::kept::valid(&base, &(base).client));
        for (limits, expected) in [
            (
                crate::tests::fixtures::Profile {
                    view: crate::Config { notices: u32::MAX / 2, ..(base).view },
                    client: jig_client::DomainLimits { ..(base).client },
                },
                true,
            ),
            (
                crate::tests::fixtures::Profile {
                    view: crate::Config { notices: u32::MAX, ..(base).view },
                    client: jig_client::DomainLimits { ..(base).client },
                },
                false,
            ),
        ] {
            assert_eq!(crate::tree::notices::kept::valid(&limits, &(limits).client), expected);
        }
    }
}

mod page_task {
    #[test]
    fn valid_refuses_each_unrepresentable_limit() {
        let base = crate::tests::state::domain::spine::LIMITS;
        assert!(crate::state::page::task::valid(&base.client));
        for (limits, expected) in [
            (
                crate::tests::fixtures::Profile {
                    view: crate::Config { ..(base).view },
                    client: jig_client::DomainLimits { rows: u32::MAX - 1, ..(base).client },
                },
                true,
            ),
            (
                crate::tests::fixtures::Profile {
                    view: crate::Config { ..(base).view },
                    client: jig_client::DomainLimits { rows: u32::MAX, ..(base).client },
                },
                false,
            ),
        ] {
            assert_eq!(crate::state::page::task::valid(&limits.client), expected);
        }
    }
}

mod page_inbox {
    #[test]
    fn valid_refuses_each_unrepresentable_limit() {
        let base = crate::tests::state::domain::spine::LIMITS;
        assert!(crate::state::page::inbox::valid(&base.client));
        for (limits, expected) in [
            (
                crate::tests::fixtures::Profile {
                    view: crate::Config { ..(base).view },
                    client: jig_client::DomainLimits { rows: u32::MAX / 4, ..(base).client },
                },
                true,
            ),
            (
                crate::tests::fixtures::Profile {
                    view: crate::Config { ..(base).view },
                    client: jig_client::DomainLimits { rows: u32::MAX, ..(base).client },
                },
                false,
            ),
        ] {
            assert_eq!(crate::state::page::inbox::valid(&limits.client), expected);
        }
    }
}

mod page_conversations {
    #[test]
    fn valid_refuses_each_unrepresentable_limit() {
        let base = crate::tests::state::domain::spine::LIMITS;
        assert!(crate::state::page::conversations::valid(&base.client));
        for (limits, expected) in [
            (
                crate::tests::fixtures::Profile {
                    view: crate::Config { ..(base).view },
                    client: jig_client::DomainLimits { rows: u32::MAX / 3, ..(base).client },
                },
                true,
            ),
            (
                crate::tests::fixtures::Profile {
                    view: crate::Config { ..(base).view },
                    client: jig_client::DomainLimits { rows: u32::MAX, ..(base).client },
                },
                false,
            ),
        ] {
            assert_eq!(crate::state::page::conversations::valid(&limits.client), expected);
        }
    }
}

mod page_project {
    #[test]
    fn valid_refuses_each_unrepresentable_limit() {
        let base = crate::tests::state::domain::spine::LIMITS;
        assert!(crate::state::page::project::valid(&base.client));
        for (limits, expected) in [
            (
                crate::tests::fixtures::Profile {
                    view: crate::Config { ..(base).view },
                    client: jig_client::DomainLimits { rows: u32::MAX / 2, ..(base).client },
                },
                true,
            ),
            (
                crate::tests::fixtures::Profile {
                    view: crate::Config { ..(base).view },
                    client: jig_client::DomainLimits { rows: u32::MAX, ..(base).client },
                },
                false,
            ),
        ] {
            assert_eq!(crate::state::page::project::valid(&limits.client), expected);
        }
    }
}

#[test]
fn the_view_allocates_from_the_clients_receiving_and_retention_bounds() {
    use crate::state::{address::Address, fields::Purpose, page::Arrangement};
    let config = crate::tests::state::domain::spine::LIMITS.view;
    let client = jig_client::DomainLimits {
        watches: 11,
        reads: 13,
        records: 7,
        kept: 9,
        rows: 3,
        fields: 1,
        text: 3,
        ..crate::tests::fixtures::CLIENT
    };
    let mut ui = crate::Ui::new(
        &config,
        &client,
        crate::page::navigation::Navigation::new(config.navigation_entries),
        Address::Deployment,
        Arrangement::none(),
    )
    .unwrap();
    assert_eq!(ui.handles.watches.capacity(), client.watches);
    assert_eq!(ui.handles.reads.capacity(), client.reads);
    assert_eq!(ui.kept.capacity(), client.records);
    assert_eq!(ui.kept_requests.capacity(), client.kept);
    assert_eq!(ui.object_rows.rows.capacity(), client.rows);
    let text = jig_client::contract::Text::new(Box::from(*b"abc"), client.text).unwrap();
    assert_eq!(ui.write_field(Purpose::NewGoal, text.clone()), Ok(()));
    assert_eq!(ui.write_field(Purpose::Reason, text), Err(jig_client::contract::Limit::Fields));
    let wide = jig_client::contract::Text::new(Box::from(*b"abcd"), 4).unwrap();
    assert_eq!(ui.write_field(Purpose::NewGoal, wide), Err(jig_client::contract::Limit::Words { limit: client.text }));
}
