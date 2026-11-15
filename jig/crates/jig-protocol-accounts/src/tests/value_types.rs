//! Stable value-type numbers (jig protocol/store/layouts.md, 2).

type Row = (u16, &'static str, Option<&'static str>, &'static [u8]);

#[test]
#[expect(clippy::too_many_lines, reason = "each stable number is pinned beside its family's table")]
fn registered_value_type_numbers_are_pinned() {
    let expected: &[(&str, &[Row])] = &[
        (
            "jig-store-core",
            &[
                (0, "Row", Some("deployment"), &[1, 0, 0]),
                (1, "Row", Some("projection"), &[1, 0, 1]),
                (2, "Row", Some("run_proof"), &[1, 0, 2]),
                (3, "Row", Some("call"), &[1, 0, 3]),
                (4, "Row", Some("turn"), &[1, 0, 4]),
                (5, "Row", Some("terminal"), &[1, 0, 5]),
                (6, "Row", Some("proposal_decision"), &[1, 0, 6]),
                (7, "Row", Some("escalation_decision"), &[1, 0, 7]),
            ],
        ),
        (
            "jig-store-tasks",
            &[
                (0, "Row", None, &[1, 1]),
                (1, "Row", Some("stub"), &[1, 1, 1]),
                (2, "Row", Some("ledger"), &[1, 1, 2]),
                (3, "Row", Some("writer"), &[1, 1, 3]),
                (4, "Row", Some("pool"), &[1, 1, 4]),
                (5, "Row", Some("person_proposal"), &[1, 1, 5]),
                (6, "Row", None, &[1, 1, 6]),
                (7, "Row", Some("closed_ledger"), &[1, 1, 7]),
                (8, "Row", Some("decided_proposal"), &[1, 1, 8]),
                (9, "Row", Some("history"), &[1, 1, 9]),
                (10, "Row", Some("milestone"), &[1, 1, 10]),
                (11, "Row", Some("activity"), &[1, 1, 11]),
                (12, "Row", Some("update"), &[1, 1, 12]),
                (13, "Row", Some("result"), &[1, 1, 13]),
                (14, "Row", Some("ended_delegate"), &[1, 1, 17]),
                (15, "Row", Some("delegate"), &[1, 1, 18]),
                (16, "Row", Some("ended_by_party"), &[1, 1, 19]),
                (17, "Row", Some("ended_by_project"), &[1, 1, 20]),
                (18, "Row", Some("placed_hold"), &[1, 1, 21]),
                (19, "Row", Some("attempt"), &[1, 1, 22]),
                (20, "Row", Some("hold"), &[1, 1, 23]),
                (21, "Row", Some("live_cold"), &[1, 1, 24]),
                (22, "Row", Some("summary"), &[1, 1, 30]),
                (23, "Row", Some("retention"), &[1, 1, 31]),
            ],
        ),
        (
            "jig-store-people",
            &[
                (0, "Row", Some("person"), &[1, 2]),
                (1, "Row", Some("read_results"), &[1, 2, 1]),
                (2, "Row", Some("read_conversation"), &[1, 2, 2]),
                (3, "Row", Some("sign_in"), &[1, 2, 3]),
                (4, "Row", Some("roles"), &[1, 2, 4]),
                (5, "Row", Some("policy"), &[1, 2, 5]),
                (6, "Row", Some("answer"), &[1, 2, 6]),
                (7, "Row", Some("ended_read"), &[1, 2, 7]),
                (8, "Row", Some("acted"), &[1, 2, 8]),
                (9, "Row", Some("deletion"), &[1, 2, 9]),
                (10, "Row", Some("public"), &[1, 2, 10]),
                (11, "Row", Some("deleted"), &[1, 2, 11]),
            ],
        ),
        ("jig-store-notes", &[(0, "Row", Some("entry"), &[1, 3]), (1, "Row", Some("line"), &[1, 3, 1])]),
        (
            "jig-store-accounts",
            &[
                (0, "Row", Some("next"), &[1, 4, 0]),
                (1, "Row", Some("account"), &[1, 4, 1]),
                (2, "Row", Some("source"), &[1, 4, 2]),
                (3, "Row", Some("issued"), &[1, 4, 4]),
                (4, "Row", Some("facts"), &[1, 4, 5]),
            ],
        ),
        (
            "jig-store-settings",
            &[
                (0, "Default", None, &[1, 0, 8, 0]),
                (1, "Changed", None, &[1, 0, 8, 1, 0]),
                (2, "ApplicationState", None, &[1, 0, 8, 1, 4]),
                (3, "Revision", None, &[1, 0, 8, 2]),
                (4, "CharterVersion", None, &[1, 0, 8, 3]),
                (5, "Proposal", None, &[1, 0, 8, 4]),
                (6, "Extent", None, &[1, 0, 8, 5]),
            ],
        ),
    ];
    let mut expected = expected.iter();
    for family in crate::JIG_PATCH_SET.families {
        if family.types.is_empty() {
            continue;
        }
        let (name, rows) = expected.next().expect("registered family");
        assert_eq!(family.name, *name);
        assert_eq!(family.types.len(), rows.len(), "{name}");
        for row in *rows {
            let mut found = false;
            for ty in family.types {
                if ty.number == row.0 {
                    assert_eq!(
                        (ty.number, ty.root, ty.variant, ty.keys.first().expect("type range").from),
                        *row,
                        "{name}"
                    );
                    found = true;
                }
            }
            assert!(found, "{name}: missing type {}", row.0);
        }
    }
    assert!(expected.next().is_none());
}
