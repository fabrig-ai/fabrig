//! Brief-room admission and its diagnostic (temper/docs/design/domain/agent/tools.md).
#![cfg(test)]

use jig_domain_tasks::Reasons;
use temper_engine_domain::room::{BRIEF_ROOM, Cost, HostPart, Measure, Short, Unfit};
const ZERO: Cost = Cost { rendered: 0, owned: 0, encoded: 0 };

const TABLE: HostPart = HostPart {
    bound: Some(51_717),
    allowance: u64::MAX,
    fixed: Cost { rendered: 100, ..ZERO },
    model: ZERO,
    always: Cost { rendered: 200, ..ZERO },
    families: [ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, Cost { rendered: 300, ..ZERO }, ZERO],
    asking: ZERO,
    asking_settings: ZERO,
    inspect: 10,
    modify: 20,
    shell: 30,
    agents: 40,
    floor: ZERO,
};

#[test]
fn check_requires_at_least_2048_bytes() {
    let mut charter = jig_test_profile::start_values::charter();
    charter.number = 7;
    let cost = TABLE.non_brief(&charter);
    let short = HostPart { bound: Some(cost + BRIEF_ROOM - 1), ..TABLE };
    assert_eq!(short.check_rendered(&charter), Err(Short { charter: 7, room: 2_047, bound: short.bound.unwrap() }));
    let enough = HostPart { bound: Some(cost + BRIEF_ROOM), ..TABLE };
    assert_eq!(enough.check_rendered(&charter), Ok(()));
    for bound in [0, cost - 1, cost] {
        let table = HostPart { bound: Some(bound), ..TABLE };
        assert_eq!(table.room(&charter), 0);
        assert_eq!(table.check_rendered(&charter), Err(Short { charter: 7, room: 0, bound }));
    }
}

#[test]
fn parts_include_their_join_and_the_joined_text_ending() {
    let mut charter = jig_test_profile::start_values::charter();
    charter.policy.instructions = Box::new([]);
    let empty = TABLE.non_brief(&charter);
    charter.parts = Box::new([
        jig_domain_core::CharterPart { on: Reasons::FIRST, instructions: Box::new([b'x'; 3_000]) },
        jig_domain_core::CharterPart { on: Reasons::WORDS, instructions: Box::new([]) },
    ]);
    // Both joining blank lines count; the final empty part leaves a newline for smith.
    assert_eq!(TABLE.non_brief(&charter) - empty, 3_005);
    charter.parts =
        Box::new([jig_domain_core::CharterPart { on: Reasons::FIRST, instructions: b"x\n".as_slice().into() }]);
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
    let mut charter = jig_test_profile::start_values::charter();
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
    let huge = HostPart { fixed: Cost { rendered: u64::MAX, ..ZERO }, ..TABLE };
    assert_eq!(huge.non_brief(&charter), u64::MAX);
    assert_eq!(huge.room(&charter), 0);
}

fn fixed_charter() -> temper_engine_domain::Charter {
    let mut charter = jig_test_profile::start_values::charter();
    charter.number = 5;
    charter.policy.instructions = b"base".as_slice().into();
    charter.policy.model.name = b"main".as_slice().into();
    charter.policy.model.effort = Some(b"high".as_slice().into());
    charter.policy.alternatives = Box::new([]);
    charter.policy.tools = jig_domain_tasks::Tools(0);
    charter.parts = Box::new([]);
    charter
}

#[test]
fn fixed_parts_select_each_wake_and_refuse_the_largest_first_tie() {
    let mut charter = fixed_charter();
    // Resume is counted with its parts too, as a standing session reads them.
    charter.policy.resume = true;
    charter.parts = Box::new([
        jig_domain_core::CharterPart { on: Reasons::FIRST, instructions: b"one".as_slice().into() },
        jig_domain_core::CharterPart {
            on: Reasons::NEWS.union(Reasons::TIMER),
            instructions: b"longer".as_slice().into(),
        },
    ]);
    let host = HostPart { bound: None, allowance: 19, ..TABLE };
    assert_eq!(host.on_wake(&charter, Reasons::FIRST).owned, 17);
    assert_eq!(host.on_wake(&charter, Reasons::NEWS).owned, 20);
    assert_eq!(host.on_wake(&charter, Reasons::TIMER).owned, 20);
    assert_eq!(
        host.check_fixed(&charter),
        Err(Unfit { charter: 5, wake: Reasons::NEWS, measure: Measure::Owned, bytes: 20, allowance: 19, floor: 0 })
    );
    assert_eq!(HostPart { allowance: 20, ..host }.check_fixed(&charter), Ok(()));
    let floor = HostPart { allowance: 21, floor: Cost { owned: 2, encoded: 2, ..ZERO }, ..host };
    assert_eq!(
        floor.check_fixed(&charter),
        Err(Unfit { charter: 5, wake: Reasons::NEWS, measure: Measure::Owned, bytes: 20, allowance: 21, floor: 2 })
    );
    assert_eq!(HostPart { allowance: 22, ..floor }.check_fixed(&charter), Ok(()));
}

#[test]
fn fixed_ownership_precedes_encoding_and_fixed_precedes_rendered() {
    let charter = fixed_charter();
    let host = HostPart { allowance: 13, bound: Some(0), fixed: Cost { owned: 2, encoded: 50, ..ZERO }, ..TABLE };
    let owned = Unfit { charter: 5, wake: Reasons::FIRST, measure: Measure::Owned, bytes: 14, allowance: 13, floor: 0 };
    assert_eq!(host.check(&charter), Err(temper_engine_domain::room::Refusal::Fixed(owned)));
    let encoded = HostPart { allowance: 14, ..host }.check_fixed(&charter).unwrap_err();
    assert_eq!(encoded, Unfit { bytes: 62, allowance: 14, measure: Measure::Encoded, ..owned });
}

#[test]
fn fixed_declarations_pay_named_families_or_the_asking_offer() {
    let mut charter = fixed_charter();
    let mut host = HostPart {
        fixed: ZERO,
        always: Cost { rendered: 1, owned: 2, encoded: 3 },
        inspect: 0,
        modify: 0,
        shell: 0,
        agents: 0,
        ..TABLE
    };
    host.families[0] = Cost { rendered: 10, owned: 20, encoded: 30 };
    host.families[10] = Cost { rendered: 100, owned: 200, encoded: 300 };
    host.asking = Cost { rendered: 4, owned: 5, encoded: 6 };
    host.asking_settings = Cost { rendered: 7, owned: 8, encoded: 9 };
    for (tools, rendered, owned, encoded) in
        [(0, 1, 14, 15), (1, 11, 34, 45), (1_025, 111, 234, 345), (2_048, 4, 17, 18), (3_072, 11, 25, 27)]
    {
        charter.policy.tools = jig_domain_tasks::Tools(tools);
        assert_eq!(host.on_wake(&charter, Reasons::FIRST), Cost { rendered: rendered + 6, owned, encoded });
    }
}

#[test]
fn fixed_models_pay_names_efforts_and_each_alternatives_frame() {
    let mut charter = fixed_charter();
    let host = HostPart { fixed: ZERO, always: ZERO, model: Cost { owned: 100, encoded: 200, ..ZERO }, ..TABLE };
    assert_eq!(host.on_wake(&charter, Reasons::FIRST).owned, 12);
    let mut alternative = charter.policy.model.clone();
    alternative.name = b"alt".as_slice().into();
    alternative.effort = None;
    charter.policy.alternatives = Box::new([alternative, charter.policy.model.clone()]);
    let cost = host.on_wake(&charter, Reasons::FIRST);
    assert_eq!((cost.owned, cost.encoded), (223, 423));
}

#[test]
fn unfit_words_name_the_wake_measure_and_allowance() {
    let unfit =
        Unfit { charter: 5, wake: Reasons::FIRST, measure: Measure::Owned, bytes: 66_012, allowance: 65_536, floor: 0 };
    assert_eq!(unfit.words().as_ref(), b"charter 5 on a first wake is 66,012 bytes owned before its brief, past its allowance of 65,536; trim its text, parts or tool families");
    assert_eq!(Unfit { wake: Reasons::QUESTION_ANSWERED, measure: Measure::Encoded, ..unfit }.words().as_ref(), b"charter 5 on a question_answered wake is 66,012 bytes encoded before its brief, past its allowance of 65,536; trim its text, parts or tool families");
}

#[test]
fn fixed_parts_reserve_the_floor_after_both_allowance_checks() {
    let charter = fixed_charter();
    let host = HostPart {
        bound: None,
        allowance: 65_536,
        fixed: Cost { owned: 54_308, encoded: 54_308, ..ZERO },
        floor: Cost { owned: 13_880, encoded: 13_500, ..ZERO },
        ..TABLE
    };
    let unfit = Unfit {
        charter: 5,
        wake: Reasons::FIRST,
        measure: Measure::Owned,
        bytes: 54_320,
        allowance: 65_536,
        floor: 13_880,
    };
    assert_eq!(host.check_fixed(&charter), Err(unfit));
    assert_eq!(unfit.words().as_ref(), b"charter 5 on a first wake is 54,320 bytes owned before its brief, leaving its brief 11,216 of the 13,880 its floor needs; trim its text, parts or tool families");
    let fits = HostPart { fixed: Cost { owned: 51_644, encoded: 52_024, ..ZERO }, ..host };
    assert_eq!(fits.check_fixed(&charter), Ok(()));
    assert_eq!(
        HostPart { fixed: Cost { owned: 51_645, ..fits.fixed }, ..fits }.check_fixed(&charter),
        Err(Unfit { bytes: 51_657, ..unfit })
    );
    let encoded = HostPart { fixed: Cost { encoded: 52_025, ..fits.fixed }, ..fits }.check_fixed(&charter).unwrap_err();
    assert_eq!(encoded, Unfit { measure: Measure::Encoded, bytes: 52_037, floor: 13_500, ..unfit });
    assert_eq!(encoded.words().as_ref(), b"charter 5 on a first wake is 52,037 bytes encoded before its brief, leaving its brief 13,499 of the 13,500 its floor needs; trim its text, parts or tool families");
    // An encoded allowance failure precedes even an ownership floor failure.
    let over = HostPart { fixed: Cost { encoded: 66_000, ..host.fixed }, ..host };
    let past = over.check_fixed(&charter).unwrap_err();
    assert_eq!(past, Unfit { measure: Measure::Encoded, bytes: 66_012, floor: 0, ..unfit });
    assert_eq!(past.words().as_ref(), b"charter 5 on a first wake is 66,012 bytes encoded before its brief, past its allowance of 65,536; trim its text, parts or tool families");
    let over = HostPart { fixed: Cost { owned: 66_000, ..over.fixed }, ..over };
    assert_eq!(over.check_fixed(&charter), Err(Unfit { measure: Measure::Owned, ..past }));
}
