//! The project's frame (temper/docs/design/client/view.md, 1 and 3).
use super::fixtures as f;
use temper_client_domain::{self as d, Address, ChatsFilter};

// The rail's fixtures use one goal template and one row in its fact watch.

// Deliver the rail's project read, preserving the address the row checks.

#[test]
fn drafts_offered_reads_drafts() {
    const OTHER_PROJECT: u32 = 17;
    use d::forge::ways::Ways;
    for (ways, offered) in [
        (None, false),
        (Some(Ways::STOCK), true),
        (Some(Ways { drafts: false, ..Ways::STOCK }), false),
        (Some(Ways { drafts: false, goals: true, ..Ways::STOCK }), false),
    ] {
        let root = f::root_with_ways(Address::Chats { filter: ChatsFilter::All }, ways);
        assert_eq!(crate::pages::chats::drafts_offered(&root, f::PROJECT), offered);
        assert!(!crate::pages::chats::drafts_offered(&root, OTHER_PROJECT));
    }
}
