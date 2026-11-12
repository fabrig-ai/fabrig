//! Brief-room admission and its diagnostic (temper/docs/design/domain/agent/tools.md).

use crate::room::{BRIEF_ROOM, HostPart, Short};

const TABLE: HostPart = HostPart {
    bound: 51_717,
    fixed: 100,
    declared: 200,
    settings: 300,
    asking: 0,
    asking_settings: 0,
    inspect: 10,
    modify: 20,
    shell: 30,
    agents: 40,
};

#[test]
fn check_requires_at_least_2048_bytes() {
    let mut charter = crate::tests::resume_fixture::charter();
    charter.number = 7;
    let cost = TABLE.non_brief(&charter);
    let short = HostPart { bound: cost + BRIEF_ROOM - 1, ..TABLE };
    assert_eq!(short.check(&charter), Err(Short { charter: 7, room: 2_047, bound: short.bound }));
    let enough = HostPart { bound: cost + BRIEF_ROOM, ..TABLE };
    assert_eq!(enough.check(&charter), Ok(()));
    for bound in [0, cost - 1, cost] {
        let table = HostPart { bound, ..TABLE };
        assert_eq!(table.room(&charter), 0);
        assert_eq!(table.check(&charter), Err(Short { charter: 7, room: 0, bound }));
    }
}

#[test]
fn parts_include_their_join_and_the_joined_text_ending() {
    let mut charter = crate::tests::resume_fixture::charter();
    charter.policy.instructions = Box::new([]);
    let empty = TABLE.non_brief(&charter);
    charter.parts = Box::new([
        jig_domain_core::CharterPart { on: jig_domain_tasks::Reasons::FIRST, instructions: Box::new([b'x'; 3_000]) },
        jig_domain_core::CharterPart { on: jig_domain_tasks::Reasons::WORDS, instructions: Box::new([]) },
    ]);
    // Both joining blank lines count; the final empty part leaves a newline for smith.
    assert_eq!(TABLE.non_brief(&charter) - empty, 3_005);
    charter.parts = Box::new([jig_domain_core::CharterPart {
        on: jig_domain_tasks::Reasons::FIRST,
        instructions: b"x\n".as_slice().into(),
    }]);
    assert_eq!(TABLE.non_brief(&charter) - empty, 5);
    charter.parts.first_mut().unwrap().instructions = b"x".as_slice().into();
    assert_eq!(TABLE.non_brief(&charter) - empty, 5);
}

#[test]
fn short_names_the_charter_room_and_bound() {
    assert_eq!(
        Short { charter: 12_345, room: 2_047, bound: 51_717 }.words().as_ref(),
        b"charter 12,345 leaves its brief 2,047 bytes of its host part's 51,717; it needs at least 2,048"
    );
    assert_eq!(
        Short { charter: 0, room: 0, bound: u64::MAX }.words().as_ref(),
        b"charter 0 leaves its brief 0 bytes of its host part's 18,446,744,073,709,551,615; it needs at least 2,048"
    );
}

#[test]
fn the_table_counts_only_enabled_families_and_saturates() {
    let mut charter = crate::tests::resume_fixture::charter();
    charter.policy.instructions = Box::new([]);
    charter.policy.inspect = false;
    charter.policy.modify = false;
    charter.policy.shell = false;
    charter.policy.agents = false;
    charter.policy.tools = jig_domain_tasks::Tools(512);
    assert_eq!(TABLE.non_brief(&charter), 300);
    charter.policy.tools = jig_domain_core::RunPolicy::EVERY_FAMILY;
    assert_eq!(TABLE.non_brief(&charter), 600);
    charter.policy.inspect = true;
    charter.policy.modify = true;
    charter.policy.shell = true;
    charter.policy.agents = true;
    assert_eq!(TABLE.non_brief(&charter), 700);
    let huge = HostPart { fixed: u64::MAX, ..TABLE };
    assert_eq!(huge.non_brief(&charter), u64::MAX);
    assert_eq!(huge.room(&charter), 0);
}
