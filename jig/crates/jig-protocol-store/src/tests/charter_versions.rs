//! The written job reads both roots of the kept settings text (protocol/store/jobs.md, 4).
#![expect(clippy::disallowed_methods, reason = "ordinary Rust codec contract tests")]
use super::{fixtures, settings};
use crate::{JobLayout, JobPage, JobRow, JobWrite, codec, jobs, settings_v3 as wire};
use jig_domain_core_vocabulary::{self as core, settings as s};
use skein_lib::{Reader, Writer};

#[expect(clippy::disallowed_macros, reason = "the contract pins the immutable kept v3 schema text")]
pub(crate) const KEPT: &str = include_str!("../../schema/kept/jig-store-settings-v3.schema");
pub(crate) fn layout() -> JobLayout {
    JobLayout { root: "CharterVersion".into(), variant: None, text: KEPT.as_bytes().into(), held: Box::new([]) }
}
pub(crate) fn record(overlay: bool, charter: bool) -> core::Record {
    core::Record::Core(core::CoreRecord::Settings(s::Record::Charter {
        charter: 1,
        revision: if overlay { 2 } else { 1 },
        value: s::CharterVersion {
            charter: charter.then(|| Box::new(settings::charter())),
            overlay: overlay.then(|| Box::new(settings::overlay())),
        },
    }))
}
/// Encode the two kept roots explicitly, using only their unchanged children.
pub(crate) fn old(record: &core::Record) -> Box<[u8]> {
    let limits = codec::limits(&fixtures::CORE_LIMITS).unwrap();
    let bytes = codec::encode(record, &limits, &[]).unwrap();
    let parts = wire::CharterVersion::decode(&limits.settings, &mut Reader::new(&bytes)).unwrap().into_parts();
    let mut writer = Writer::new(bytes.len().checked_sub(1).unwrap());
    writer.put(&3_u16.to_be_bytes()).unwrap();
    writer.put(&[u8::from(parts.charter.is_some())]).unwrap();
    if let Some(charter) = parts.charter {
        charter.encode(&mut writer).unwrap();
    }
    let root = if let Some(overlay) = parts.overlay {
        overlay.encode(&mut writer).unwrap();
        b"OverlaidCharterVersion".as_slice()
    } else {
        b"CharterVersion".as_slice()
    };
    let bytes = writer.finish();
    let schema = skein_codec::schema::parse(KEPT.as_bytes()).unwrap();
    skein_codec::value::Value::with_room(4096).decode(&schema, schema.find(root).unwrap(), bytes.clone()).unwrap();
    bytes
}
pub(crate) fn key(record: &core::Record) -> Box<[u8]> {
    crate::keys::encode(&record.key(), fixtures::KEY_LIMIT).unwrap()
}
#[test]
fn charter_versions_job_puts_both_kept_forms_as_one_root_on_every_page() {
    for charter in [true, false] {
        let records = [record(false, charter), record(true, charter)];
        let mut page = JobPage {
            rows: records.iter().map(|r| JobRow { key: key(r), value: old(r) }).collect(),
            end: false,
            layout: Some(layout()),
        };
        // Layout heads are not family versions.
        page.rows[0].value[..2].copy_from_slice(&99_u16.to_be_bytes());
        if !charter {
            page.layout = None;
        }
        let answer = jobs::page("charter-versions", &page).unwrap();
        assert_eq!(answer.through, None);
        assert_eq!(answer.writes.len(), records.len());
        let limits = codec::limits(&fixtures::CORE_LIMITS).unwrap();
        for (write, record) in answer.writes.iter().zip(records) {
            let JobWrite::Put { key: found, value } = write else { panic!("conversion must put") };
            assert_eq!(found, &key(&record));
            assert_eq!(codec::decode(&record.key(), value, &limits, &[]), Some(record));
        }
    }
}
#[test]
fn charter_versions_job_refuses_neither_form_naming_the_key() {
    for bytes in [b"".as_slice(), &[0, 3, 2], &[0, 3, 0, 255], &[0, 3, 0, 0, 0]] {
        let key = key(&record(false, false));
        let page = JobPage {
            rows: Box::new([JobRow { key: key.clone(), value: bytes.into() }]),
            end: false,
            layout: Some(layout()),
        };
        let refused = jobs::page("charter-versions", &page).unwrap_err();
        assert_eq!(refused.key, Some(key));
        assert!(refused.why.contains("neither"));
    }
}
#[test]
fn charter_versions_end_is_empty_and_unknown_jobs_are_refused() {
    let page = JobPage { rows: Box::new([]), end: true, layout: None };
    let answer = jobs::page("charter-versions", &page).unwrap();
    assert!(answer.writes.is_empty());
    assert_eq!(answer.through, None);
    let refused = jobs::page("unknown", &page).unwrap_err();
    assert_eq!(refused.key, None);
    assert!(refused.why.contains("no such jig job"));
}
#[test]
fn charter_versions_from_is_the_kept_v3_root_layout_and_the_job_is_admitted() {
    let set = jig_protocol_accounts::JIG_PATCH_SET;
    let job = set.jobs.iter().find(|job| job.name == "charter-versions").unwrap();
    let converts = job.converts.unwrap();
    let family = set.families.iter().find(|family| family.name == converts.family).unwrap();
    assert_eq!(family.name, "jig-store-settings");
    let ty = family.types.iter().find(|ty| ty.number == converts.value_type).unwrap();
    assert_eq!((ty.number, ty.root), (4, "CharterVersion"));
    let schema = skein_codec::schema::parse(KEPT.as_bytes()).unwrap();
    assert_eq!(schema.version(), 3);
    let expected = schema.layout(schema.root(ty.root.as_bytes(), None).unwrap(), &[]).unwrap().digest();
    assert_eq!(converts.from, expected);
    jobs::admission::check(&crate::PatchSets { sets: Box::leak(Box::new([jig_set()])) }).unwrap();
}

/// The dev dependency links the non-test store library. Copy metadata into the
/// test library's nominal types without maintaining a second catalogue.
pub(crate) fn jig_set() -> crate::PatchSet {
    let set = jig_protocol_accounts::JIG_PATCH_SET;
    let store_map = set.families.iter().find(|f| f.name == "jig-store-settings").unwrap().map;
    crate::PatchSet {
        owner: crate::Owner::Jig,
        name: set.name,
        patches: set.patches,
        fresh: set.fresh,
        jobs: Box::leak(
            set.jobs
                .iter()
                .map(|job| crate::Job {
                    name: job.name,
                    keys: Box::leak(job.keys.iter().map(|r| crate::KeyRange { from: r.from, to: r.to }).collect()),
                    converts: job.converts.map(|c| crate::Converts {
                        family: c.family,
                        value_type: c.value_type,
                        from: c.from,
                    }),
                })
                .collect(),
        ),
        families: Box::leak(
            set.families
                .iter()
                .map(|f| crate::KeptFamily {
                    name: f.name,
                    texts: f.texts,
                    current: f.current,
                    frames: f.frames,
                    map: if f.map == store_map { crate::KeptMap::Store } else { crate::KeptMap::Secrets },
                    keys: Box::leak(
                        f.keys.iter().map(|k| crate::Keys { from: k.from, to: k.to, root: k.root }).collect(),
                    ),
                    flights: Box::leak(
                        f.flights.iter().map(|k| crate::Keys { from: k.from, to: k.to, root: k.root }).collect(),
                    ),
                    types: Box::leak(
                        f.types
                            .iter()
                            .map(|t| crate::ValueType {
                                number: t.number,
                                root: t.root,
                                variant: t.variant,
                                keys: Box::leak(
                                    t.keys.iter().map(|r| crate::KeyRange { from: r.from, to: r.to }).collect(),
                                ),
                            })
                            .collect(),
                    ),
                })
                .collect(),
        ),
    }
}
