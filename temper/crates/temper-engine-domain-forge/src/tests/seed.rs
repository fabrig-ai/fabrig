use super::*;

fn record(number: u16) -> Forge {
    Forge {
        number,
        identity: record::Identity { number: u32::from(number) + 10, ..records::forge().identity },
        ..records::forge()
    }
}
fn seeded(seed: &[Forge]) -> Domain {
    Domain::new(
        &LIMITS,
        4,
        skein_forge_client::Config { namespace: b"seed".as_slice().into(), writers: Box::new([]) },
        seed,
        false,
    )
    .unwrap()
}
fn restored(d: &mut Domain) -> Box<[Request]> {
    outputs(d, Event::Restored { clock: skein_forge_client::RecoveryClock::Wall })
}
fn kept(out: &[Request]) -> Box<[Forge]> {
    let mut rows = skein_lib::List::with_capacity(LIMITS.forges);
    for request in out {
        if let Request::ForgeKept { forge } = request {
            rows.push(forge.as_ref().clone()).unwrap();
        }
    }
    rows.into_boxed()
}
#[test]
fn a_first_start_saves_the_seed_and_the_next_number() {
    let seeds: [Box<[Forge]>; 2] = [Box::new([]), Box::new([record(3), record(1)])];
    for seed in seeds {
        let mut d = seeded(&seed);
        for row in &seed {
            assert_eq!(d.configured(row.number), Some(&Configured::from_record(row)));
            assert_eq!(d.identities.get(&row.number), Some(&row.identity.number));
        }
        let out = restored(&mut d);
        let next = if seed.is_empty() { 1 } else { 4 };
        assert_eq!(d.next_forge(), next);
        let mut saves = skein_lib::List::with_capacity(LIMITS.forges + 1);
        for request in &out {
            if let Request::Save { record, .. } = request {
                saves.push(record.clone()).unwrap();
            }
        }
        let mut expected = skein_lib::List::with_capacity(LIMITS.forges + 1);
        for row in &seed {
            expected.push(Stored::Forge(row.clone())).unwrap();
        }
        expected.push(Stored::Forges { next }).unwrap();
        assert_eq!(saves.as_slice(), expected.as_slice());
    }
}
#[test]
fn a_later_start_keeps_the_records_not_the_seed() {
    let mut d = seeded(&[record(1), record(2), record(3)]);
    let mut changed = record(1);
    changed.identity.number = 90;
    for row in [Stored::Forge(changed.clone()), Stored::Forge(record(4)), Stored::Forges { next: 8 }] {
        assert!(outputs(&mut d, Event::Restore { record: row }).is_empty());
    }
    assert_eq!(d.configured(1), Some(&Configured::from_record(&changed)));
    assert_eq!(d.identities.get(&1), Some(&90));
    assert!(d.configured(2).is_some());
    let out = restored(&mut d);
    assert_eq!(kept(&out).as_ref(), [changed.clone(), record(4)]);
    assert_eq!(d.configured.len(), 2);
    assert_eq!(d.identities.len(), 2);
    assert_eq!(d.identities.get(&1), Some(&90));
    assert_eq!(d.configured(1), Some(&Configured::from_record(&changed)));
    assert_eq!(d.next_forge(), 8);
    for request in &out {
        if let Request::Save { record: Stored::Forge(_) | Stored::Forges { .. }, .. } = request {
            panic!("kept records are never seeded");
        }
    }
}
#[test]
fn a_removed_seeded_forge_is_not_seeded_again() {
    let mut d = seeded(&[record(2)]);
    outputs(&mut d, Event::Restore { record: Stored::Forges { next: 3 } });
    let out = restored(&mut d);
    assert!(kept(&out).is_empty());
    assert!(d.configured(2).is_none());
    assert!(d.identities.is_empty());
    assert!(d.forge_records.is_empty());
}
#[test]
fn every_kept_forge_is_told_to_the_service_in_order() {
    let mut d = seeded(&[record(3), record(1), record(2)]);
    assert_eq!(kept(&restored(&mut d)).as_ref(), [record(1), record(2), record(3)]);
}
#[test]
fn a_removing_record_leaves_the_rows() {
    let mut row = record(1);
    row.removing = true;
    let mut d = seeded(&[row.clone()]);
    assert!(d.forges().rows.len() == 1);
    let out = restored(&mut d);
    assert!(kept(&out).is_empty());
    assert!(d.forge_record(row.number).is_none());
    assert!(d.forges().rows.is_empty());
}
#[test]
fn the_playground_record_says_playground() {
    let mut row = record(1);
    row.playground = true;
    let mut d = seeded(&[row]);
    restored(&mut d);
    assert!(d.forges().rows[0].playground);
    assert!(d.demo.is_none());
}
