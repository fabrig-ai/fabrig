//! Store pages through the shared written fixture (jig/docs/design/protocol/store/jobs.md, 6).
use super::*;
use crate::{Converts, Job, JobAnswer, JobPage, JobRefusal, JobWrite, Layouts};
use jig_test_layouts::layouts::{WRITTEN_NUMBER, WRITTEN_TEXT};
use skein_codec::{
    layout::{Mode, Side, Why},
    schema::parse,
};

const RANGES: &[KeyRange] = &[KeyRange { from: b"r", to: b"s" }];
const NUMBER: KeptFamily = KeptFamily {
    name: "fixture-written",
    current: WRITTEN_NUMBER,
    keys: &[Keys { from: b"r", to: b"s", root: "Retyped" }],
    types: &[ValueType { number: 0, root: "Retyped", variant: None, keys: RANGES }],
    ..FAMILY
};
const TEXT: KeptFamily = KeptFamily { current: WRITTEN_TEXT, ..NUMBER };
const NUMBER_SET: OwnerSet = OwnerSet { name: NUMBER.name, families: &[NUMBER], ..SET };
const TEXT_SET: OwnerSet = OwnerSet { families: &[TEXT], ..NUMBER_SET };
const JOB: &str = "retyped";

fn with_job(to: OwnerSet, from: &KeptFamily) -> OwnerSet {
    let schema = parse(from.current.as_bytes()).unwrap();
    let fingerprint = schema.layout(schema.root(b"Retyped", None).unwrap(), &[]).unwrap().digest();
    let job = Job {
        name: JOB,
        keys: RANGES,
        converts: Some(Converts { family: from.name, value_type: 0, from: Layouts::One(fingerprint) }),
        counts: &[],
    };
    OwnerSet { jobs: Box::leak(Box::new([job])), ..to }
}

fn page(job: &str, page: &JobPage, binary: &KeptFamily) -> Result<JobAnswer, JobRefusal> {
    assert_eq!(job, JOB);
    let layout = page.layout.as_ref().expect("the written job has a layout");
    let old = parse(&layout.text).unwrap();
    let new = parse(binary.current.as_bytes()).unwrap();
    let held: Vec<_> = layout.held.iter().map(|text| parse(text).unwrap()).collect();
    let held: Vec<_> = held.iter().collect();
    let from = Side {
        text: &old,
        root: old.root(layout.root.as_bytes(), layout.variant.as_deref().map(str::as_bytes)).unwrap(),
        held: &held,
    };
    let to = Side { text: &new, root: new.root(b"Retyped", None).unwrap(), held: &[] };
    let mut out = Buffer::with_capacity(64);
    let mut writes = Vec::new();
    for row in &page.rows {
        let value = row.at_version(old.version())?;
        jig_test_layouts::written::convert(layout.mode, &from, &to, &value, &mut out)
            .map_err(|refusal| JobRefusal { key: Some(row.key.clone()), why: refusal.why })?;
        writes.push(JobWrite::Put { key: row.key.clone(), value: out.to_boxed() });
    }
    assert!(u32::try_from(writes.len()).unwrap() <= page.room.writes);
    assert!(writes.iter().map(JobWrite::bytes).sum::<u64>() <= page.room.bytes);
    Ok(JobAnswer { writes: writes.into_boxed_slice(), through: None, counts: Box::new([]) })
}

#[test]
fn a_written_job_converts_stored_numbers_to_decimal_text() {
    let mut disk = Disk::new(&[]);
    let mut previous = fresh(&mut disk, NUMBER_SET);
    back::write(
        &mut disk,
        &mut previous,
        &[(b"r/0", b"\0\x01\0\0\0\x07\0\0\0\x01a"), (b"r/1", b"\0\x01\0\0\0*\0\0\0\x01b")],
    );
    let items = disk.items();
    let mut disk = Disk::new(&items);
    let refused = disk.open(&mut store(&disk, TEXT_SET));
    let [
        Event::OpenRefused {
            why: OpenRefusal::Layout { family, value_type: 0, id: 1, values: Some(2), why: Unplanned::Underived(why) },
            ..
        },
    ] = refused.as_ref()
    else {
        panic!("{refused:?}")
    };
    assert_eq!((family.as_ref(), why.why), (NUMBER.name, Why::Retyped));

    let mut disk = Disk::new(&items);
    let set = with_job(TEXT_SET, &NUMBER);
    let mut current = store(&disk, set);
    let mut events = disk.open(&mut current);
    for _ in 0_u32..3 {
        let Some(Event::Job { owner, of: Owner::Application, job, page: batch }) = events.first() else { break };
        assert_eq!(batch.layout.as_ref().unwrap().mode, Mode::Read);
        events = disk.request(&mut current, Request::Job { owner: *owner, answer: page(job, batch, &TEXT) });
    }
    let [Event::Ran { rows: 2, .. }, Event::Opened { pending: true, last, .. }, Event::Census { .. }] = events.as_ref()
    else {
        panic!("{events:?}")
    };
    let last = *last;
    switch(&mut disk, &mut current);
    let mut current = store(&disk, set);
    assert_opened(&disk.open(&mut current), last);
    for (key, decimal, label) in [(b"r/0", b"7".as_slice(), b"a".as_slice()), (b"r/1", b"42", b"b")] {
        let bytes = back::get(&mut disk, &mut current, key);
        let mut read = Reader::new(&bytes);
        assert_eq!(read.u16(), Some(1));
        let n = read.u32().unwrap();
        assert_eq!(read.bytes(n), Some(decimal));
        let n = read.u32().unwrap();
        assert_eq!(read.bytes(n), Some(label));
        assert!(read.is_empty());
    }
}

#[test]
fn a_written_job_names_the_refused_decimal_row_and_keeps_the_active_generation() {
    let mut disk = Disk::new(&[]);
    fresh(&mut disk, NUMBER_SET);
    let mut disk = Disk::new(&disk.items());
    let mut later = store(&disk, TEXT_SET);
    let events = disk.open(&mut later);
    let [Event::Opened { pending: true, .. }, Event::Census { .. }] = events.as_ref() else { panic!("{events:?}") };
    switch(&mut disk, &mut later);
    let bad = b"\0\x01\0\0\0\x02x1\0\0\0\x01a";
    let last = back::write(&mut disk, &mut later, &[(b"r/bad", bad)]);
    let mut disk = Disk::new(&disk.items());
    let active = disk.bytes(b"active");

    let mut current = store(&disk, with_job(NUMBER_SET, &TEXT));
    let events = disk.open(&mut current);
    let [Event::Job { owner, job, page: batch, .. }] = events.as_ref() else { panic!("{events:?}") };
    // The text layout was introduced second. Reintroducing the retired number
    // layout allocates an id past it, but does not change either's order.
    assert_eq!(batch.layout.as_ref().unwrap().mode, Mode::Keep);
    assert_eq!(crate::registry::head(&batch.rows[0].value), Some(2));
    let events = disk.request(&mut current, Request::Job { owner: *owner, answer: page(job, batch, &NUMBER) });
    let [Event::OpenRefused { why: OpenRefusal::Job { owner: Owner::Application, job, key: Some(key), why }, .. }] =
        events.as_ref()
    else {
        panic!("{events:?}")
    };
    assert_eq!((job.as_ref(), key.as_ref(), why.as_ref()), (JOB, b"r/bad".as_slice(), "expected decimal digits"));
    assert_eq!(disk.bytes(b"active"), active);
    let mut later = store(&disk, TEXT_SET);
    assert_opened(&disk.open(&mut later), last);
    assert_eq!(back::get(&mut disk, &mut later, b"r/bad").as_ref(), bad);
}
