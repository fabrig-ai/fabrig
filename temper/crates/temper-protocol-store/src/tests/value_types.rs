//! Stable value-type numbers (jig protocol/store/layouts.md, 2).

type Row = (u16, &'static str, Option<&'static str>, &'static [u8]);

#[test]
fn registered_value_type_numbers_are_pinned() {
    let expected: &[Row] = &[
        (0, "Row", Some("call"), &[0]),
        (1, "Row", Some("proposed_effect"), &[1]),
        (2, "Row", Some("repository"), &[2]),
        (3, "Row", Some("hold"), &[3]),
        (4, "Row", Some("names"), &[4]),
        (5, "Row", Some("subscription"), &[5]),
        (6, "Row", Some("branch_head"), &[6]),
        (7, "Row", Some("pull_state"), &[7]),
        (8, "Row", Some("ci"), &[8]),
        (9, "Row", Some("landed"), &[9]),
        (10, "Row", Some("entry"), &[10]),
        (11, "Row", Some("client"), &[11]),
        (13, "Row", Some("issue"), &[13]),
        (14, "Row", Some("release"), &[14]),
        (15, "Row", Some("application"), &[15]),
        (16, "Row", Some("writer"), &[16]),
        (17, "Row", Some("binding"), &[18]),
        (18, "Row", Some("setting"), &[20]),
        (19, "Row", Some("flight"), &[21]),
        (20, "Row", Some("landing"), &[23]),
        (21, "Row", Some("receipt"), &[24]),
        (22, "Row", Some("forge"), &[25]),
        (23, "Row", Some("forge_number"), &[26]),
        (24, "Row", Some("playground"), &[27]),
        (25, "Row", Some("tier"), &[28]),
        (26, "Row", Some("verification"), &[29]),
        (27, "Row", Some("landing_log"), &[30]),
        (28, "Row", Some("finding"), &[31]),
        (29, "Row", Some("dismissed"), &[34]),
        (30, "Row", Some("making"), &[35]),
        (31, "Row", Some("project_facts"), &[36]),
        (32, "Row", Some("looked"), &[37]),
        (33, "Row", Some("away"), &[38]),
        (34, "Row", Some("decided"), &[39]),
        (35, "Row", Some("closer"), &[40]),
        (36, "Row", Some("landed_draft"), &[41]),
        (37, "Row", Some("follow"), &[42]),
        (38, "Row", Some("device"), &[43]),
        (39, "Row", Some("red"), &[44]),
        (40, "Row", Some("red_number"), &[45]),
        (46, "Row", Some("red_pass"), &[46]),
        (47, "Row", Some("change_cold"), &[47]),
        (48, "Row", Some("change_hot"), &[48]),
        (41, "Row", Some("history_project"), &[127]),
        (42, "Row", Some("history_event"), &[128]),
        (43, "Row", Some("history_change"), &[129]),
        (44, "Row", Some("history_landing"), &[130]),
        (45, "Row", Some("history_last"), &[131]),
    ];
    let mut registered = false;
    for family in crate::PATCH_SET.families {
        if family.types.is_empty() {
            continue;
        }
        assert!(!registered);
        registered = true;
        assert_eq!(family.name, "temper-store");
        let [retired] = family.retired else { panic!("one retired value type") };
        assert_eq!(retired.number, 12);
        let [range] = retired.keys else { panic!("one retired range") };
        assert_eq!((range.from, range.to), (&[12][..], &[13][..]));
        assert_eq!(family.types.len(), expected.len());
        for row in expected {
            let mut found = false;
            for ty in family.types {
                if ty.number == row.0 {
                    assert_eq!((ty.number, ty.root, ty.variant, ty.keys.first().expect("type range").from), *row);
                    found = true;
                }
            }
            assert!(found, "missing type {}", row.0);
        }
    }
    assert!(registered);
}
