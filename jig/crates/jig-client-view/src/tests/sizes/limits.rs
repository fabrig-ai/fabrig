#[test]
fn valid_refuses_each_unrepresentable_limit() {
    use crate::{Limits, valid};
    use skein_lib::Duration;
    let base = crate::tests::state::domain::spine::LIMITS;
    assert!(valid(&base));
    let cases = [
        Limits { watches: 0, ..base },
        Limits { notice_life: Duration::ZERO, ..base },
        Limits { linger: Duration::ZERO, ..base },
        Limits { save_delay: Duration::ZERO, ..base },
        Limits { sources: 0, ..base },
        Limits { conversation_words: base.intents - 1, ..base },
        Limits { records: u32::MAX, ..base },
        Limits { objects: u32::MAX, ..base },
        Limits { notices: u32::MAX, ..base },
        Limits { rows: u32::MAX, ..base },
        // The timer's tick and the tree's final name must fit.
        Limits { notices: u32::MAX - base.objects - base.records, ..base },
        Limits { rows: 65_536, ..base },
    ];
    for (case, limits) in cases.into_iter().enumerate() {
        assert!(!valid(&limits), "case {case}");
    }
}

#[test]
fn validity_does_not_price_retained_documents() {
    let limits = crate::Limits { document_bytes: u64::MAX, ..crate::tests::state::domain::spine::LIMITS };
    assert!(crate::valid(&limits));
}

mod windows {
    #[test]
    fn valid_refuses_each_unrepresentable_limit() {
        let base = crate::tests::state::domain::spine::LIMITS;
        assert!(crate::state::windows::valid(&base));
        for (limits, expected) in
            [(crate::Limits { rows: u32::MAX / 2, ..base }, true), (crate::Limits { rows: u32::MAX, ..base }, false)]
        {
            assert_eq!(crate::state::windows::valid(&limits), expected);
        }
    }
}

mod objects {
    #[test]
    fn valid_refuses_each_unrepresentable_limit() {
        let base = crate::tests::state::domain::spine::LIMITS;
        assert!(crate::state::objects::valid(&base));
        for (limits, expected) in [
            (crate::Limits { objects: u32::MAX / 2, ..base }, true),
            (crate::Limits { objects: u32::MAX, ..base }, false),
        ] {
            assert_eq!(crate::state::objects::valid(&limits), expected);
        }
    }
}

mod notices_kept {
    #[test]
    fn valid_refuses_each_unrepresentable_limit() {
        let base = crate::tests::state::domain::spine::LIMITS;
        assert!(crate::tree::notices::kept::valid(&base));
        for (limits, expected) in [
            (crate::Limits { notices: u32::MAX / 2, ..base }, true),
            (crate::Limits { notices: u32::MAX, ..base }, false),
        ] {
            assert_eq!(crate::tree::notices::kept::valid(&limits), expected);
        }
    }
}

mod page_task {
    #[test]
    fn valid_refuses_each_unrepresentable_limit() {
        let base = crate::tests::state::domain::spine::LIMITS;
        assert!(crate::state::page::task::valid(&base));
        for (limits, expected) in
            [(crate::Limits { rows: u32::MAX - 1, ..base }, true), (crate::Limits { rows: u32::MAX, ..base }, false)]
        {
            assert_eq!(crate::state::page::task::valid(&limits), expected);
        }
    }
}

mod page_inbox {
    #[test]
    fn valid_refuses_each_unrepresentable_limit() {
        let base = crate::tests::state::domain::spine::LIMITS;
        assert!(crate::state::page::inbox::valid(&base));
        for (limits, expected) in
            [(crate::Limits { rows: u32::MAX / 4, ..base }, true), (crate::Limits { rows: u32::MAX, ..base }, false)]
        {
            assert_eq!(crate::state::page::inbox::valid(&limits), expected);
        }
    }
}

mod page_conversations {
    #[test]
    fn valid_refuses_each_unrepresentable_limit() {
        let base = crate::tests::state::domain::spine::LIMITS;
        assert!(crate::state::page::conversations::valid(&base));
        for (limits, expected) in
            [(crate::Limits { rows: u32::MAX / 3, ..base }, true), (crate::Limits { rows: u32::MAX, ..base }, false)]
        {
            assert_eq!(crate::state::page::conversations::valid(&limits), expected);
        }
    }
}

mod page_project {
    #[test]
    fn valid_refuses_each_unrepresentable_limit() {
        let base = crate::tests::state::domain::spine::LIMITS;
        assert!(crate::state::page::project::valid(&base));
        for (limits, expected) in
            [(crate::Limits { rows: u32::MAX / 2, ..base }, true), (crate::Limits { rows: u32::MAX, ..base }, false)]
        {
            assert_eq!(crate::state::page::project::valid(&limits), expected);
        }
    }
}
