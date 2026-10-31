use super::*;
use jig_client_test_domain::Address;
use jig_client_view::address::{Address as Page, TaskSection};
use stories::closing::{CANCELLED, CHAT, MOVED, PARTY};
const SEED: u64 = 893;
#[test]
fn closing_a_chat_cancels_one_delegate_and_moves_the_other_to_the_person() {
    let mut world = stories::closing::world(SEED);
    let client = world.client(PARTY, stories::chat());
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::Button, b"Close"), within: WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Close") },
            Step::Press { find: Find::named(Role::Button, b"Cancel delegate 10 and its subtree") },
            Step::Press { find: Find::named(Role::Button, b"Take over delegate 11, reserving 1") },
            Step::Type { find: Find::named(Role::TextArea, b"Reason"), words: b"finished".as_slice().into() },
            Step::Press { find: Find::named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Waiting for delegates"), within: WITHIN },
            Step::Go { address: Address::Jig(Page::Task { task: CHAT, section: TaskSection::Overview }) },
            Step::See { find: Find::text(b"Live delegates: 0"), within: WITHIN },
            Step::Go { address: Address::Jig(Page::Task { task: CANCELLED, section: TaskSection::Overview }) },
            Step::See { find: Find::text(b"Cancelled delegate"), within: WITHIN },
            Step::See { find: Find::text(b"Cancelled with reason"), within: WITHIN },
            Step::See { find: Find::text(b"finished"), within: WITHIN },
            Step::See { find: Find::text(b"Cancellation requester person: 7"), within: WITHIN },
            Step::See { find: Find::text(b"Requested by task 9"), within: WITHIN },
            Step::Go { address: Address::Jig(Page::Task { task: MOVED, section: TaskSection::Overview }) },
            Step::See { find: Find::text(b"Moved delegate"), within: WITHIN },
            Step::See { find: Find::text(b"Requested by person 7"), within: WITHIN },
            Step::See { find: Find::text(b"Active attempt 1"), within: WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, skein_world::domain::Verdict::Passed);
}
