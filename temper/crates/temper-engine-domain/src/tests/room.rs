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
    let policy = crate::tests::resume_fixture::charter().policy;
    let cost = TABLE.non_brief(&policy);
    let short = HostPart { bound: cost + BRIEF_ROOM - 1, ..TABLE };
    assert_eq!(short.check(7, &policy), Err(Short { charter: 7, room: 2_047, bound: short.bound }));
    let enough = HostPart { bound: cost + BRIEF_ROOM, ..TABLE };
    assert_eq!(enough.check(7, &policy), Ok(()));
    for bound in [0, cost - 1, cost] {
        let table = HostPart { bound, ..TABLE };
        assert_eq!(table.room(&policy), 0);
        assert_eq!(table.check(7, &policy), Err(Short { charter: 7, room: 0, bound }));
    }
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
    let mut policy = crate::tests::resume_fixture::charter().policy;
    policy.instructions = Box::new([]);
    policy.inspect = false;
    policy.modify = false;
    policy.shell = false;
    policy.agents = false;
    policy.tools = jig_domain_tasks::Tools(512);
    assert_eq!(TABLE.non_brief(&policy), 300);
    policy.tools = jig_domain_core::RunPolicy::EVERY_FAMILY;
    assert_eq!(TABLE.non_brief(&policy), 600);
    policy.inspect = true;
    policy.modify = true;
    policy.shell = true;
    policy.agents = true;
    assert_eq!(TABLE.non_brief(&policy), 700);
    let huge = HostPart { fixed: u64::MAX, ..TABLE };
    assert_eq!(huge.non_brief(&policy), u64::MAX);
    assert_eq!(huge.room(&policy), 0);
}
