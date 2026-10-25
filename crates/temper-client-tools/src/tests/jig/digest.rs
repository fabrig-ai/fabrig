use crate::{
    jig::{
        digest::{self, Digest},
        render::Part,
    },
    tests::fixtures,
};
#[test]
fn the_checked_digest_prints_sixteen_lower_case_hex_digits() {
    assert_eq!(Digest::from_bytes(fixtures::DIGEST_BYTES).hex(), *b"0123456789abcdef");
}
#[test]
fn the_digest_is_the_sha256_prefix() {
    assert_eq!(Digest::of(b"").hex(), *b"e3b0c44298fc1c14");
}
#[test]
fn the_printed_part_without_digest_is_the_bytes_digested() {
    let task = fixtures::goal(fixtures::TASK, fixtures::waiting_on_me());
    let printed = fixtures::read_answer(fixtures::TASK_JSON_CALL, fixtures::task_result(task.clone()));
    assert_eq!(printed.status, 0);
    assert_eq!(
        Digest::of(&fixtures::json_without_digest(&printed.stdout)),
        digest::part(&Part::Task(&task), &fixtures::limits())
    );
    let before = fixtures::printed_digest(&printed.stdout);
    assert_eq!(before, Digest::of(&fixtures::json_without_digest(&printed.stdout)).hex());

    let mut changed = task.clone();
    changed.activity = changed.activity.checked_add(1).expect("next task activity");
    let printed = fixtures::read_answer(fixtures::TASK_JSON_CALL, fixtures::task_result(changed.clone()));
    assert_eq!(printed.status, 0);
    assert_eq!(
        Digest::of(&fixtures::json_without_digest(&printed.stdout)),
        digest::part(&Part::Task(&changed), &fixtures::limits())
    );
    let after = fixtures::printed_digest(&printed.stdout);
    assert_eq!(after, Digest::of(&fixtures::json_without_digest(&printed.stdout)).hex());
    assert_ne!(before, after);
}
#[test]
fn a_tasks_page_has_the_watch_windows_subject_bytes() {
    let (page, watched) = fixtures::tasks_first_window(fixtures::PAGE_ROWS, true);
    assert_eq!(
        crate::jig::render::document::part_bytes(&Part::TaskPage(&page), &fixtures::limits()),
        crate::jig::render::document::part_bytes(&Part::Tasks(&watched), &fixtures::limits()),
    );
}
#[test]
fn a_digest_moves_when_a_task_fact_changes() {
    let task = fixtures::goal(fixtures::TASK, fixtures::waiting_on_me());
    let mut changed = task.clone();
    changed.chip.task = fixtures::OTHER_TASK;
    assert_ne!(
        digest::part(&Part::Task(&task), &fixtures::limits()),
        digest::part(&Part::Task(&changed), &fixtures::limits())
    );
}
#[test]
fn a_direct_historical_task_prints_its_task_subject_digest() {
    let task = fixtures::ended_task(jig_client::contract::Phase::Done);
    let printed = fixtures::read_answer(
        fixtures::TASK_JSON_CALL,
        fixtures::historical_task_result(task.clone(), jig_client::contract::Ending::Done),
    );
    assert_eq!(printed.status, 0);
    assert_eq!(fixtures::printed_digest(&printed.stdout), digest::part(&Part::Task(&task), &fixtures::limits()).hex());
}

#[test]
fn a_read_matches_a_watch_windows_digest() {
    let (read, watched) = fixtures::inbox_first_window(fixtures::OVER_WINDOW);
    assert_eq!(
        digest::part(&Part::InboxPage(&read), &fixtures::limits()),
        digest::part(&Part::Inbox(&watched), &fixtures::limits())
    );
    assert_eq!(read.rows.len(), fixtures::FULL_WINDOW);
    assert!(Part::InboxPage(&read).more());
    let last_need = read.rows.get(fixtures::SHORT_WINDOW - fixtures::PAGE_ROWS).expect("last need");
    let first_update = read.rows.get(fixtures::SHORT_WINDOW).expect("first update");
    assert_eq!(
        last_need.object,
        jig_client::contract::ObjectKey::Question {
            task: fixtures::TASK + u64::from(fixtures::SHORT_WINDOW - fixtures::PAGE_ROWS),
            message: 2
        }
    );
    assert_eq!(first_update.object, jig_client::contract::ObjectKey::Result(fixtures::TASK + 1));
}
#[test]
fn an_inbox_read_cut_before_updates_digests_the_watchs_first_window() {
    let printed = fixtures::inbox_json_answer(fixtures::SHORT_WINDOW, fixtures::WINDOW_UPDATES, false);
    let (_, watched) = fixtures::inbox_first_window(fixtures::OVER_WINDOW);
    let expected = digest::part(&Part::Inbox(&watched), &fixtures::limits()).hex();
    assert!(skein_lib::bytes::find(&printed.stdout, &expected).is_some());
    assert_eq!(
        fixtures::occurrences(&printed.stdout, b"\"group\":\"questions\""),
        usize::try_from(fixtures::SHORT_WINDOW).expect("row count fits")
    );
    assert!(skein_lib::bytes::find(&printed.stdout, b"\"group\":\"results\"").is_none());
    assert!(skein_lib::bytes::find(&printed.stdout, b"inbox --after 01").is_some());
}
#[test]
fn a_task_page_cursor_changes_without_moving_its_digest() {
    let (mut page, _) = fixtures::tasks_first_window(fixtures::FULL_WINDOW, true);
    let before = digest::part(&Part::TaskPage(&page), &fixtures::limits());
    page.live_after = Some(
        jig_client::contract::Cursor::new(Box::from(fixtures::OTHER_CURSOR), fixtures::FULL_WINDOW)
            .expect("other cursor"),
    );
    assert_eq!(before, digest::part(&Part::TaskPage(&page), &fixtures::limits()));
}
#[test]
fn changing_more_moves_the_digest_of_a_full_window() {
    let (mut page, _) = fixtures::tasks_first_window(fixtures::FULL_WINDOW, false);
    let before = digest::part(&Part::TaskPage(&page), &fixtures::limits());
    page.live_after = Some(fixtures::cursor());
    assert_ne!(before, digest::part(&Part::TaskPage(&page), &fixtures::limits()));
}

#[test]
fn a_delivery_digest_ignores_the_clock_or_live_me() {
    let first = fixtures::task_under(fixtures::PARTY, fixtures::OTHER_CLOCK);
    let other = fixtures::task_under(fixtures::DIGEST_OTHER_PARTY, fixtures::OTHER_CLOCK + fixtures::OTHER_CLOCK);
    assert_eq!(first.stdout, other.stdout);
}
#[test]
fn non_watch_reads_print_no_digest() {
    for (argv, page) in fixtures::non_subject_reads() {
        let json = fixtures::extended(argv.as_slice(), &["--json"]);
        let printed = fixtures::read_answer(json.as_slice(), page);
        assert!(skein_lib::bytes::find(&printed.stdout, b"\"digest\":").is_none(), "{argv:?}");
    }
}
#[test]
fn chat_prints_no_digest() {
    let argv = fixtures::extended(fixtures::CHAT_CALL, &["--json"]);
    let (mut root, call) = fixtures::read_call(argv.as_slice());
    root.serve(call, fixtures::activity_too_large());
    root.deliver_run(fixtures::no_provisional());
    let printed = root.answer(call).expect("chat answered");
    assert!(skein_lib::bytes::find(&printed.stdout, b"\"digest\":").is_none());
}
#[test]
fn supported_task_query_filters_print_a_digest() {
    for filter in fixtures::WATCH_QUERY_FILTERS {
        let argv = fixtures::extended(fixtures::TASKS_CALL, filter);
        let json = fixtures::extended(argv.as_slice(), &["--json"]);
        let printed = fixtures::read_answer(json.as_slice(), fixtures::task_page_with_cursor());
        assert!(skein_lib::bytes::find(&printed.stdout, b"\"digest\":").is_some(), "{filter:?}");
    }
}
#[test]
fn a_task_composite_prints_each_subjects_digest() {
    let argv = fixtures::extended(fixtures::COMPOSITE_CALL, &["--json"]);
    let (mut root, call) = fixtures::composite_call(argv.as_slice());
    let printed = root.answer(call).expect("composite answered");
    let task = fixtures::goal(fixtures::TASK, fixtures::waiting_on_me());
    let tree = fixtures::composite_tree();
    let run = fixtures::no_provisional();
    assert_eq!(fixtures::occurrences(&printed.stdout, b"\"digest\":"), fixtures::SUBJECT_PARTS);
    for part in [Part::Task(&task), Part::Tree(&tree), Part::delivery(&run).expect("run form")] {
        assert!(
            skein_lib::bytes::find(&printed.stdout, &digest::part(&part, &fixtures::limits()).hex()).is_some(),
            "{part:?}"
        );
    }
}
#[test]
fn owner_read_cursors_do_not_change_row_identity() {
    for rows in [fixtures::SHORT_WINDOW, fixtures::FULL_WINDOW, fixtures::OVER_WINDOW] {
        let (mut page, mut watched) = fixtures::owner_first_window(rows, true);
        let before = digest::part(&Part::OwnerInbox(&watched), &fixtures::limits());
        page.after = Some(
            jig_client::contract::Cursor::new(Box::from(fixtures::OTHER_CURSOR), fixtures::FULL_WINDOW)
                .expect("cursor"),
        );
        for index in 0..page.rows.len() {
            page.rows.get_mut(index).expect("supplied row").cursor =
                jig_client::contract::OwnerInboxCursor::Number { number: fixtures::OTHER_TASK };
        }
        watched.cached_needs = fixtures::FULL_WINDOW;
        watched.cached_updates.value = fixtures::FULL_WINDOW;
        watched.results_seen = fixtures::OTHER_TASK;
        assert_eq!(before, digest::part(&Part::OwnerInboxPage(&page), &fixtures::limits()), "rows={rows}");
        assert_eq!(before, digest::part(&Part::OwnerInbox(&watched), &fixtures::limits()), "rows={rows}");
    }
}
#[test]
fn supplied_row_order_moves_the_digest() {
    let (mut page, _) = fixtures::tasks_first_window(fixtures::WINDOW_UPDATES, false);
    let before = digest::part(&Part::TaskPage(&page), &fixtures::limits());
    let mut reversed = skein_lib::List::with_capacity(fixtures::WINDOW_UPDATES);
    for row in page.live.as_slice().iter().rev() {
        reversed.push(row.clone()).expect("same rows");
    }
    page.live = reversed;
    assert_ne!(before, digest::part(&Part::TaskPage(&page), &fixtures::limits()));
}

#[test]
fn a_delivery_hashes_exactly_its_printed_subject_fields() {
    let me = fixtures::me(fixtures::PARTY);
    let tree = fixtures::composite_tree();
    let run = fixtures::no_provisional();
    let (_, inbox) = fixtures::inbox_first_window(fixtures::WINDOW_UPDATES + fixtures::PAGE_ROWS);
    let (_, owner) = fixtures::owner_first_window(fixtures::PAGE_ROWS, false);
    let (_, tasks) = fixtures::tasks_first_window(fixtures::PAGE_ROWS, false);
    for part in [
        Part::Me(&me),
        Part::Tree(&tree),
        Part::delivery(&run).expect("run form"),
        Part::Inbox(&inbox),
        Part::OwnerInbox(&owner),
        Part::Tasks(&tasks),
    ] {
        let printed = fixtures::subject_json(&part);
        assert_eq!(
            Digest::of(&fixtures::json_without_digest(&printed)),
            digest::part(&part, &fixtures::limits()),
            "{part:?}"
        );
    }
}
#[test]
fn an_owner_rows_raw_identity_moves_its_digest() {
    let (mut page, _) = fixtures::owner_first_window(fixtures::PAGE_ROWS, false);
    let row = page.rows.get_mut(0).expect("one row");
    row.object = jig_client::contract::OwnerInboxObject::Escalation { revision: fixtures::ROW_REVISION };
    let card = row.card_key();
    let before = digest::part(&Part::OwnerInboxPage(&page), &fixtures::limits());
    let row = page.rows.get_mut(0).expect("one row");
    row.object = jig_client::contract::OwnerInboxObject::Escalation { revision: fixtures::OTHER_ROW_REVISION };
    assert_eq!(row.card_key(), card);
    assert_ne!(before, digest::part(&Part::OwnerInboxPage(&page), &fixtures::limits()));
}

#[test]
fn inbox_watch_counts_stay_outside_the_digest() {
    let (mut page, mut watched) = fixtures::inbox_first_window(fixtures::OVER_WINDOW);
    let before = digest::part(&Part::Inbox(&watched), &fixtures::limits());
    watched.needs.proposals = fixtures::FULL_WINDOW;
    watched.updates.value = fixtures::FULL_WINDOW;
    watched.updates.limited = true;
    watched.results_seen = fixtures::OTHER_TASK;
    page.before = Some(fixtures::cursor());
    page.after = Some(
        jig_client::contract::Cursor::new(Box::from(fixtures::OTHER_CURSOR), fixtures::FULL_WINDOW)
            .expect("opaque cursor"),
    );
    page.cut = true;
    assert_eq!(before, digest::part(&Part::InboxPage(&page), &fixtures::limits()));
    assert_eq!(before, digest::part(&Part::Inbox(&watched), &fixtures::limits()));
}

#[test]
fn a_tasks_window_is_capped_at_the_rows_limit() {
    for (supplied, more, printed, marker) in fixtures::WINDOW_CASES {
        let (_, watched) = fixtures::tasks_first_window(*supplied, *more);
        let bytes = crate::jig::render::document::part_bytes(&Part::Tasks(&watched), &fixtures::limits());
        assert_eq!(
            fixtures::occurrences(&bytes, b"\"chip\":"),
            usize::try_from(*printed).expect("row count fits"),
            "rows={supplied}, more={more}"
        );
        assert!(skein_lib::bytes::find(&bytes, marker).is_some(), "rows={supplied}, more={more}");
    }
}
