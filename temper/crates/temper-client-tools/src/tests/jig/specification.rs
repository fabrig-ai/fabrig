//! Task show's supplemental reads stay bounded and keep a usable continuation.
use crate::tests::fixtures as f;
use jig_client::contract as c;
use skein_lib::{Writer, bytes::find};

fn brief(prefix: usize, suffix: usize) -> Box<[u8]> {
    let link = b" forge run 91 http://forge/org/repo/actions/runs/91";
    let length = prefix.checked_add(suffix).unwrap().checked_add("é".len()).unwrap().checked_add(link.len()).unwrap();
    let mut words = Writer::new(length);
    for _ in 0..prefix {
        words.put(b"x").unwrap();
    }
    words.put("é".as_bytes()).unwrap();
    for _ in 0..suffix {
        words.put(b"y").unwrap();
    }
    words.put(link).unwrap();
    words.finish()
}

#[test]
fn task_show_joins_multiple_utf8_chunks_without_changing_its_digest() {
    let words = brief(127, 200);
    let (printed, offsets) = f::source_show(&words, &crate::limits::tools(), false);
    assert_eq!(printed.status, 0);
    assert_eq!(offsets.as_slice(), [127, 255]);
    assert!(find(&printed.stdout, &words).is_some());
    assert!(find(&printed.stdout, b"asked: ").is_some());
    assert!(find(&printed.stdout, b"read on with:").is_none());
    let (json, offsets) = f::source_show(&words, &crate::limits::tools(), true);
    assert!(offsets.is_empty(), "JSON retains its projected document and digest");
    let start = find(&printed.stdout, b"digest: task ").unwrap().checked_add(b"digest: task ".len()).unwrap();
    let digest = &printed.stdout[start..start.checked_add(16).unwrap()];
    assert!(find(&json.stdout, digest).is_some());
}

#[test]
fn task_show_stops_at_its_source_bound_with_an_exact_utf8_continuation() {
    let words = brief(259, 200);
    let mut limits = crate::limits::tools();
    limits.part = 260;
    let (printed, offsets) = f::source_show(&words, &limits, false);
    assert_eq!(printed.status, 0);
    assert_eq!(offsets.as_slice(), [128, 256]);
    assert!(find(&printed.stdout, &[b'x'; 259]).is_some());
    assert!(find(&printed.stdout, &continuation(259)).is_some());
    assert!(skein_lib::bytes::is_utf8(&printed.stdout));
    assert!(printed.stdout.len() <= usize::try_from(limits.text).unwrap());
}

#[test]
fn task_show_stops_at_its_result_byte_bound_with_a_continuation() {
    let words = [b'x'; 2000];
    for retained in [512, 700, 1024] {
        let mut limits = crate::limits::tools();
        limits.result_bytes = 1024 + retained * 6;
        let (printed, offsets) = f::source_show(&words, &limits, false);
        assert_eq!(printed.status, 0);
        assert!(skein_lib::len_u64(printed.stdout.len()) <= limits.result_bytes);
        assert!(find(&printed.stdout, &continuation(retained)).is_some());
        for from in offsets.as_slice() {
            assert!(*from < retained);
        }
    }
}

#[test]
fn task_show_continues_from_the_bytes_that_fit_its_printed_output() {
    let words = [b'x'; 500];
    let mut limits = crate::limits::tools();
    let (whole, _) = f::source_show(&words, &limits, false);
    limits.text = u32::try_from(whole.stdout.len().checked_sub(128).unwrap()).unwrap();
    let (printed, _) = f::source_show(&words, &limits, false);
    assert_eq!(printed.status, 0);
    assert!(printed.stdout.len() <= usize::try_from(limits.text).unwrap());
    let start = find(&printed.stdout, b"asked: ").unwrap().checked_add(b"asked: ".len()).unwrap();
    let shown = find(&printed.stdout[start..], b"\n").unwrap();
    assert!(shown < words.len(), "exercise the renderer's text ceiling");
    assert!(find(&printed.stdout, &continuation(u64::try_from(shown).unwrap())).is_some());
}

#[test]
fn task_show_keeps_its_overview_when_a_supplemental_read_cannot_advance() {
    let (core, limits, row) = f::source_task(&[b'x'; 300]);
    for result in [
        c::ReadResult::None(c::ReadNone::Unknown),
        c::ReadResult::Value(c::ReadValue::TaskText(Box::new(c::TaskText {
            task: 1,
            field: c::TaskTextField::Specification,
            from: 128,
            total: 300,
            next: Some(128),
            text: c::Text::new(Box::new([]), 128).unwrap(),
        }))),
    ] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["task", "show", "1"]);
        root.serve(call, f::task_result(f::source_summary(&core, &limits, &row)));
        let Some((_, c::Read::TaskText { from: 128, .. })) = root.reads.get(&call) else {
            panic!("supplemental read follows the overview");
        };
        root.serve(call, result);
        assert!(!root.reads.contains_key(&call));
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 0);
        assert!(find(&printed.stdout, &continuation(128)).is_some());
    }
}

fn continuation(from: u64) -> Box<[u8]> {
    let mut measure = crate::jig::render::Text::measure();
    continuation_line(from, &mut measure);
    let mut out = crate::jig::render::Text::writing_exact(measure);
    continuation_line(from, &mut out);
    out.finish()
}

fn continuation_line(from: u64, out: &mut crate::jig::render::Text) {
    out.put(b"read on with: task text 1 --field specification --from ");
    out.number(from);
}

#[expect(clippy::disallowed_methods, reason = "test argv converts decimal bytes into command arguments")]
fn text_read(words: &[u8], from: u64, limits: &crate::jig::Limits, json: bool) -> crate::jig::Printed {
    let (core, bounds, row) = f::source_task(words);
    let mut root = f::signed_in(f::PARTY);
    root.limits.tools = *limits;
    let offset = skein_lib::Decimal::of(from);
    let offset = core::str::from_utf8(offset.as_bytes()).unwrap();
    let argv = ["task", "text", "1", "--field", "instructions", "--from", offset, "--json"];
    let call = root.submit(argv.get(..if json { 8 } else { 7 }).unwrap());
    for _ in 0..words.len().max(1) {
        let Some((_, c::Read::TaskText { from, field, .. })) = root.reads.get(&call) else { break };
        assert_eq!(*field, c::TaskTextField::Instructions);
        let page = f::source_chunk(&core, &bounds, &row, jig_domain_core::reads::TaskTextField::Instructions, *from);
        root.serve(call, c::ReadResult::Value(c::ReadValue::TaskText(Box::new(page))));
    }
    root.answer(call).expect("bounded text retrieval completes")
}

#[test]
fn task_text_joins_3554_instruction_bytes_in_one_call() {
    const INSTRUCTION_BYTES: usize = 3554; // The charter size in finding 264.
    let words = "é".repeat(INSTRUCTION_BYTES / "é".len()).into_bytes();
    for json in [false, true] {
        for from in [0, 126] {
            let printed = text_read(&words, from, &crate::limits::tools(), json);
            assert_eq!(printed.status, 0);
            assert!(find(&printed.stdout, &words[usize::try_from(from).unwrap()..]).is_some());
            assert!(find(&printed.stdout, if json { b"\"next\":null" } else { b"no more" }).is_some());
        }
    }
}

#[test]
fn task_text_joins_to_its_bounds_with_exact_utf8_offsets() {
    let words = "é".repeat(1000).into_bytes();
    for json in [false, true] {
        let preset = crate::limits::tools();
        // Each ceiling admits 259 source bytes; the final é leaves 258 UTF-8 bytes.
        for limits in [
            crate::jig::Limits { part: 259, ..preset },
            crate::jig::Limits { text: 771, ..preset },
            crate::jig::Limits { result_bytes: 2066, ..preset },
        ] {
            let retained = 258;
            let printed = text_read(&words, 126, &limits, json);
            assert_eq!(printed.status, 0, "{printed:?}");
            assert!(find(&printed.stdout, &words[..retained]).is_some());
            let continuation = if json { "\"next\":384" } else { "--from 384" };
            assert!(find(&printed.stdout, continuation.as_bytes()).is_some(), "{printed:?}");
            assert!(printed.stdout.len() <= usize::try_from(limits.text).unwrap());
            assert!(skein_lib::len_u64(printed.stdout.len()) <= limits.result_bytes);
        }
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the step table inspects rendered UTF-8 lines")]
fn task_text_cut_pages_say_so_first() {
    // tools.md, 6.4: 600 source bytes span three 259-byte pages; 200 fit whole.
    let limits = crate::jig::Limits { part: 259, ..crate::limits::tools() };
    let mut failures = skein_lib::List::with_capacity(5); // Five page/control rows (tools.md, 11.1).
    let words = [b'x'; 600]; // Three 259-byte pages, with the last short.
    for (size, from, json, first, last, absent, present) in [
        (
            600,
            0,
            false,
            Some(
                "instructions page 1 of 3 holds 259 bytes; 341 bytes left; read on with: task text 1 --field instructions --from 259",
            ),
            Some("341 bytes left; read on with: task text 1 --field instructions --from 259"),
            None,
            None,
        ),
        (
            600,
            259,
            false,
            Some(
                "instructions page 2 of 3 holds 259 bytes; 82 bytes left; read on with: task text 1 --field instructions --from 518",
            ),
            None,
            None,
            None,
        ),
        (600, 518, false, Some("instructions page 3 of 3 holds 82 bytes; no more"), Some("no more"), None, None),
        (200, 0, false, Some("instructions from 0 of 200 bytes:"), None, Some(" page "), None),
        (600, 0, true, None, None, Some("\"page\""), Some("\"next\":259")),
    ] {
        let printed = text_read(words.get(..size).unwrap(), from, &limits, json);
        let stdout = core::str::from_utf8(&printed.stdout).unwrap();
        let first_matches = match first {
            Some(line) => stdout.lines().next() == Some(line),
            None => true,
        };
        let last_matches = match last {
            Some(line) => stdout.lines().last() == Some(line),
            None => true,
        };
        let absent_matches = match absent {
            Some(words) => !stdout.contains(words),
            None => true,
        };
        let present_matches = match present {
            Some(words) => stdout.contains(words),
            None => true,
        };
        let correct = printed.status == 0 && first_matches && last_matches && absent_matches && present_matches;
        if !correct {
            failures.push((size, from, json, Box::<str>::from(stdout.lines().next().unwrap_or("")))).unwrap();
        }
    }
    assert!(failures.is_empty(), "cut page positions: {failures:?}");
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the step table inspects rendered UTF-8 lines")]
fn task_text_keeps_a_continuation_when_retrieval_stalls_or_is_refused() {
    let (core, limits, row) = f::source_task(&[b'x'; 300]);
    let mut failures = skein_lib::List::with_capacity(2); // Stalled and refused retrievals.
    for refused in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["task", "text", "1", "--field", "instructions"]);
        let page = f::source_chunk(&core, &limits, &row, jig_domain_core::reads::TaskTextField::Instructions, 0);
        root.serve(call, c::ReadResult::Value(c::ReadValue::TaskText(Box::new(page))));
        assert_eq!(
            root.reads.get(&call).expect("join next page").1,
            c::Read::TaskText { task: 1, field: c::TaskTextField::Instructions, from: 128 }
        );
        let result = if refused {
            c::ReadResult::None(c::ReadNone::Unknown)
        } else {
            c::ReadResult::Value(c::ReadValue::TaskText(Box::new(c::TaskText {
                task: 1,
                field: c::TaskTextField::Instructions,
                from: 128,
                total: 300,
                next: Some(128),
                text: c::Text::new(Box::new([]), 128).unwrap(),
            })))
        };
        root.serve(call, result);
        let printed = root.answer(call).expect("stopped retrieval");
        assert_eq!(printed.status, 0);
        assert!(find(&printed.stdout, b"--from 128").is_some());
        let stdout = core::str::from_utf8(&printed.stdout).unwrap();
        if stdout.lines().next()
            != Some(
                "instructions page 1 of 2 holds 128 bytes; 172 bytes left; read on with: task text 1 --field instructions --from 128",
            )
        {
            failures.push(refused).unwrap();
        }
    }
    assert!(failures.is_empty(), "stalled/refused retrievals keep their continuation first; refused: {failures:?}");
}
