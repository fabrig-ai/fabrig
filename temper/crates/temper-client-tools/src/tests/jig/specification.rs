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
