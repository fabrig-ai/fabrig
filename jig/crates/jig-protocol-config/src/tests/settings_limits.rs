//! Settings lists take the document reader's limits rules and name diagnostics from the root.
use super::fixtures;
use crate::{
    Defaults, Directories, DocumentLimits, Fixed, Limits, Path, Problem, Problems, engine_over, with_overrides,
};
use jig_domain_core_vocabulary::settings::Override;
use skein_lib::{Decimal, Duration, Writer, bytes};

fn given(name: &[u8], value: u64) -> Override {
    Override { name: bytes::copy_of(name), value, reason: bytes::copy_of(b"deployment capacity") }
}

fn engine(base: &Limits, text: &[u8]) -> Result<crate::Engine, Problems> {
    let document = crate::document(text, &DocumentLimits { bytes: 8192, tokens: 2048, depth: 16 }).unwrap();
    engine_over(&document, 0, &Directories::working(), Defaults::Document, &Fixed::NONE, base)
}

fn document_form(overrides: &[Override]) -> Box<[u8]> {
    let prefix = br#"{"overrides":{"#;
    let mut length = prefix.len().checked_add(2).unwrap();
    for (index, item) in overrides.iter().enumerate() {
        let member =
            item.name.len().checked_add(3).unwrap().checked_add(Decimal::of(item.value).as_bytes().len()).unwrap();
        length = length.checked_add(member).unwrap().checked_add(usize::from(index > 0)).unwrap();
    }
    let mut out = Writer::new(length);
    out.put(prefix).unwrap();
    for (index, item) in overrides.iter().enumerate() {
        if index > 0 {
            out.put(b",").unwrap();
        }
        out.put(b"\"").unwrap();
        out.put(&item.name).unwrap();
        out.put(b"\":").unwrap();
        out.put(Decimal::of(item.value).as_bytes()).unwrap();
    }
    out.put(b"}}").unwrap();
    fixtures::with(fixtures::ENGINE_SMALL, br#"{"profile": "standard"}"#, &out.finish())
}

#[test]
fn an_empty_settings_list_leaves_even_underived_base_figures_unchanged() {
    let mut base = crate::stock();
    base.clients.streams = 100;
    base.core.documents.segments = 1;
    assert_eq!(with_overrides(&base, &[]), Ok(base));
}

#[test]
fn settings_lists_and_engine_documents_resolve_the_same_figures() {
    let mut base = crate::stock();
    // This application's inbox admits a raise to 1,024 tasks.
    base.core.people.inbox_tasks = 2048;
    for list in [
        [given(b"tasks.tasks", 1024)],
        [given(b"link.ping_ms", 6000)],
        [given(b"tasks.spec_bytes", 4096)],
        [given(b"clients.streams", 100)],
        [given(b"tasks.authority_segments", 32)],
    ] {
        let from_list = with_overrides(&base, &list).unwrap();
        let from_document = engine(&base, &document_form(&list)).unwrap().limits;
        assert_eq!(from_list, from_document, "{list:?}");
        match list.first().unwrap().name.as_ref() {
            b"tasks.tasks" => assert_eq!(from_list.core.tasks.tasks, 1024),
            b"link.ping_ms" => assert_eq!(from_list.link.ping, Duration::from_millis(6000)),
            b"tasks.spec_bytes" => assert_eq!(from_list.core.people.spec_bytes, 4096),
            b"clients.streams" => {
                assert_eq!(from_list.clients_http.calls, 104);
                assert_eq!(from_list.clients.exchanges, 104);
            }
            b"tasks.authority_segments" => assert_eq!(from_list.core.documents.segments, 32),
            _ => unreachable!("the equivalence cases name their distinguishing figure"),
        }
    }
    let list = [given(b"clients.exchanges", 120), given(b"clients.streams", 100), given(b"clients_http.calls", 110)];
    let resolved = with_overrides(&base, &list).unwrap();
    assert_eq!(resolved, engine(&base, &document_form(&list)).unwrap().limits);
    assert_eq!((resolved.clients.exchanges, resolved.clients_http.calls), (120, 110));
}

#[test]
fn settings_refusals_keep_the_documents_what_and_one_dotted_member() {
    let base = crate::stock();
    for (name, value) in [
        (b"tasks.tasks".as_slice(), 0),
        (b"tasks.absent".as_slice(), 1),
        (b"link.ping_ms".as_slice(), u64::MAX),
        (b"tasks.batch".as_slice(), 1025),
        (b"clients.public.max_age".as_slice(), 61),
    ] {
        let list = [given(name, value)];
        let from_list = with_overrides(&base, &list).unwrap_err();
        let from_document = engine(&base, &document_form(&list)).unwrap_err();
        assert_eq!(from_list.problems.len(), 1, "{name:?}: {from_list:?}");
        assert_eq!(from_document.problems.len(), 1, "{name:?}: {from_document:?}");
        let expected = from_document.problems.get(0).unwrap();
        assert_eq!(expected.path, Path::root().member(b"limits").member(b"overrides").member(name));
        assert_eq!(from_list.problems.as_slice(), &[Problem { path: Path::root().member(name), what: expected.what }]);
    }
}

#[test]
fn an_engine_without_limits_reads_the_application_base() {
    let mut base = crate::stock();
    base.core.tasks.tasks = 1024;
    base.core.people.inbox_tasks = 2048;
    // Absent limits takes the base, not the empty document's carrier derivation.
    base.clients.streams = 100;
    let text = fixtures::with(fixtures::ENGINE_SMALL, br#""limits": {"profile": "standard"},"#, b"");
    assert_eq!(engine(&base, &text).unwrap().limits, base);
    let text = fixtures::with(&text, br#""memory_bytes": 34359738368,"#, b"");
    let parsed = engine(&base, &text).unwrap();
    assert_eq!(parsed.limits, base);
    assert_eq!(parsed.memory, None);
}

#[test]
fn a_worker_without_limits_reads_the_application_base() {
    let mut base = crate::stock();
    base.core.tasks.tasks = 1024;
    base.core.people.inbox_tasks = 2048;
    base.hub.slots = 2;
    base.hub.source_interval = Duration::from_millis(60000);
    let text = fixtures::with(fixtures::WORKER_SMALL, br#""limits": {"profile": "standard"}, "#, b"");
    let document = crate::document(&text, &DocumentLimits { bytes: 8192, tokens: 256, depth: 16 }).unwrap();
    let parsed = crate::worker_over(&document, 0, &Directories::working(), &crate::FixedWorker::NONE, &base).unwrap();
    assert_eq!(parsed.limits, base);
}
