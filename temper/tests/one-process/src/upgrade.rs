//! A registry and job upgrade at the build's levels (temper/docs/design/draft/store-floor.md, 4.4).
pub mod check;
mod maps;
pub mod rehearsal;
pub mod story;
use jig_protocol_store::{KeptMap, Owner, OwnerSets, codec, keys};
use std::{collections::BTreeSet, path::Path};

/// A pending production job upgrades the second build without moving any owner's level.
pub(super) const JOB: &str = "deleted-repository-forge-rows";
pub(super) const CURRENT_SETS: OwnerSets = temper_engine_service::store::OWNER_SETS;

thread_local! { static FIRST_SETS: OwnerSets = first_build(); }

pub(super) fn first_sets() -> OwnerSets {
    FIRST_SETS.with(|sets| *sets)
}

fn first_build() -> OwnerSets {
    OwnerSets {
        sets: Box::leak(
            CURRENT_SETS
                .sets
                .iter()
                .map(|set| {
                    if set.owner != Owner::Application {
                        return *set;
                    }
                    jig_protocol_store::OwnerSet {
                        jobs: Box::leak(
                            set.jobs
                                .iter()
                                .copied()
                                .filter(|job| job.name != JOB)
                                .collect::<Vec<_>>()
                                .into_boxed_slice(),
                        ),
                        families: Box::leak(
                            set.families
                                .iter()
                                .map(|family| {
                                    if family.name != "temper-store" {
                                        return *family;
                                    }
                                    // The stories write no receipts. Only their unused root has an
                                    // earlier layout; production supplies the field's default.
                                    let current = family.current.replacen(
                                        "record ReceiptRow { report: ReceiptReport = unset ",
                                        "record ReceiptRow { ",
                                        1,
                                    );
                                    assert_ne!(current, family.current, "receipt report field removed");
                                    jig_protocol_store::KeptFamily {
                                        current: Box::leak(current.into_boxed_str()),
                                        ..*family
                                    }
                                })
                                .collect::<Vec<_>>()
                                .into_boxed_slice(),
                        ),
                        ..*set
                    }
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ),
    }
}

/// Every non-reserved record belongs to exactly one family. Report all bad keys,
/// including both family names when ranges overlap.
pub fn golden_ranges(directory: &Path) {
    let mut disk = crate::golden::files::load(directory);
    let maps = maps::Maps::read(&mut disk);
    let reserved = [
        (keys::encode_own(&keys::Own::Format, 64).expect("format key"), "format"),
        (keys::encode_own(&keys::Own::Layouts, 64).expect("layouts key"), "layouts"),
        (keys::encode_own(&keys::Own::Wires, 64).expect("wires key"), "wires"),
        (keys::encode_own(&keys::Own::Floor, 64).expect("floor key"), "floor"),
        (keys::installation(64).expect("installation key"), "installation"),
    ];
    let mut seen = BTreeSet::new();
    let mut bad = Vec::new();
    for (map, rows) in [(KeptMap::Store, &maps.store), (KeptMap::Secrets, &maps.secrets)] {
        for (key, value) in rows {
            let mut families = Vec::new();
            for set in temper_engine_service::store::OWNER_SETS.sets {
                let relative = if set.owner == Owner::Application {
                    key.strip_prefix(match map {
                        KeptMap::Store => &[keys::APPLICATION][..],
                        KeptMap::Secrets => &[2, 0][..],
                    })
                } else {
                    Some(key.as_slice())
                };
                let Some(relative) = relative else { continue };
                for family in set.families.iter().filter(|family| family.map == map) {
                    for range in family.keys {
                        if range.from <= relative && relative < range.to {
                            families.push(family.name);
                        }
                    }
                }
            }
            let own = (map == KeptMap::Store)
                .then(|| reserved.iter().find(|(reserved, _)| reserved.as_ref() == key.as_slice()))
                .flatten();
            let component = (map == KeptMap::Store).then(|| keys::decode_own(key)).flatten();
            if map == KeptMap::Store && key.starts_with(&[0, 9]) {
                assert!(families.is_empty(), "reserved log extent {key:02x?} in {families:?}");
                assert_eq!(key.len(), 10, "extent's log number");
                let _: [u8; 8] = value.as_slice().try_into().expect("extent's length");
                seen.insert("log extent");
            } else if let Some(keys::Own::History { .. } | keys::Own::TurnLog { .. } | keys::Own::LogAttempt { .. }) =
                component
            {
                assert!(families.is_empty(), "reserved log place {key:02x?} in {families:?}");
                let name = log_place(component.as_ref().expect("log place"), value);
                seen.insert(name);
            } else if let Some(keys::Own::Layout { .. } | keys::Own::Text(_) | keys::Own::Job { .. }) = component {
                assert_eq!(families, ["jig-store-registry"], "component registry key {key:02x?}");
                let name = registry(component.as_ref().expect("registry key"), value);
                seen.insert(name);
            } else if let Some((_, name)) = own {
                seen.insert(*name);
                assert!(families.is_empty(), "reserved {name} key {key:02x?} in {families:?}");
                match *name {
                    "format" => {
                        codec::component::Format::decode(value).expect("format at its own key");
                    }
                    "layouts" => {
                        codec::component::Layouts::decode(value).expect("layouts at their own key");
                    }
                    "floor" => {
                        let floor = jig_protocol_store::component_v3::Floor::decode(
                            &jig_protocol_store::component_v3::CEILINGS,
                            &mut skein_lib::Reader::new(value),
                        )
                        .expect("floor at its own key");
                        assert_eq!(floor.number(), 1);
                    }
                    "wires" => {
                        jig_protocol_store::component_v3::Wires::decode(
                            &jig_protocol_store::component_v3::CEILINGS,
                            &mut skein_lib::Reader::new(value),
                        )
                        .expect("wires at their own key");
                    }
                    "installation" => {
                        codec::accounts::Installation::decode(value).expect("installation at its own key");
                    }
                    _ => unreachable!("listed reserved row"),
                }
            } else if families.len() != 1 {
                bad.push(format!("{map:?} {key:02x?}: {families:?}"));
            }
        }
    }
    assert_eq!(
        seen,
        reserved
            .iter()
            .map(|(_, name)| *name)
            .chain(["registry layout", "registry text", "done job", "log extent", "history", "turn log", "log attempt"])
            .collect(),
        "all component-owned rows present"
    );
    assert!(bad.is_empty(), "keys in none or multiple ranges:\n{}", bad.join("\n"));
    eprintln!("golden ranges: {} store keys, {} secrets keys", maps.store.len(), maps.secrets.len());
}

fn log_place(component: &keys::Own, value: &[u8]) -> &'static str {
    match component {
        keys::Own::History { .. } => {
            codec::component::HistoryLog::decode(value).expect("history place");
            "history"
        }
        keys::Own::TurnLog { .. } => {
            codec::component::TurnLog::decode(value).expect("turns place");
            "turn log"
        }
        keys::Own::LogAttempt { .. } => {
            codec::component::LogAttempt::decode(value).expect("attempt place");
            "log attempt"
        }
        _ => unreachable!("log place"),
    }
}

fn registry(component: &keys::Own, value: &[u8]) -> &'static str {
    let reader = &mut skein_lib::Reader::new(value);
    match component {
        keys::Own::Layout { .. } => {
            jig_protocol_store::registry_v1::Layout::decode(&jig_protocol_store::registry_v1::CEILINGS, reader)
                .expect("registry layout");
            "registry layout"
        }
        keys::Own::Text(_) => {
            jig_protocol_store::registry_v1::Text::decode(&jig_protocol_store::registry_v1::CEILINGS, reader)
                .expect("registry text");
            "registry text"
        }
        keys::Own::Job { .. } => {
            jig_protocol_store::registry_v1::Job::decode(&jig_protocol_store::registry_v1::CEILINGS, reader)
                .expect("done job");
            "done job"
        }
        _ => unreachable!("registry key"),
    }
}

/// Only the unused receipt layout is registered and retired; the pending job ends once.
fn registry_upgrade(before: &maps::Maps, after: &maps::Maps) -> BTreeSet<Vec<u8>> {
    use jig_protocol_store::registry_v1 as wire;
    use skein_lib::Reader;
    assert_eq!(before.format(), after.format(), "registry and job upgrades move no levels or versions");
    let family = temper_protocol_store::OWNER_SET
        .families
        .iter()
        .find(|family| family.name == "temper-store")
        .expect("temper family");
    let ty = family.types.iter().find(|ty| ty.variant == Some("receipt")).expect("receipt type");
    for maps in [before, after] {
        assert!(
            !maps.store.keys().any(|key| key
                .strip_prefix(&[keys::APPLICATION])
                .is_some_and(|key| ty.keys.iter().any(|range| range.from <= key && key < range.to))),
            "the stories write no receipt value"
        );
    }
    let layout = |maps: &maps::Maps| {
        let rows = maps.store.iter().filter(|(key, _)| matches!(keys::decode_own(key), Some(keys::Own::Layout {family: ref name, number, ..}) if name.as_ref() == family.name && number == ty.number)).collect::<Vec<_>>();
        let [(key, value)] = rows.as_slice() else { panic!("one receipt layout: {rows:?}") };
        ((*key).clone(), wire::Layout::decode(&wire::CEILINGS, &mut Reader::new(value)).expect("receipt layout"))
    };
    let (old_key, old) = layout(before);
    let (new_key, new) = layout(after);
    let schema = skein_codec::schema::parse(family.current.as_bytes()).expect("production text");
    let root = schema.root(ty.root.as_bytes(), ty.variant.map(str::as_bytes)).expect("receipt root");
    assert_eq!(*new.layout(), schema.layout(root, &[]).expect("receipt fingerprint").digest());
    assert_ne!(old.layout(), new.layout(), "second build registers a new layout");
    assert!(!after.store.contains_key(&old_key), "unused earlier layout retired");
    assert!(!before.store.contains_key(&new_key), "production layout not yet registered");
    let text_key = keys::encode_own(&keys::Own::Text(*new.text()), 1024).expect("text key");
    let text =
        wire::Text::decode(&wire::CEILINGS, &mut Reader::new(after.store.get(text_key.as_ref()).expect("new text")))
            .expect("registry text");
    assert_eq!(text.text(), family.current.as_bytes());
    assert!(!before.store.contains_key(text_key.as_ref()), "second build brings production text");
    let job_key =
        keys::encode_own(&keys::Own::Job { owner: Owner::Application, name: JOB.into() }, 1024).expect("job key");
    assert!(!before.store.contains_key(job_key.as_ref()), "first build leaves the job pending");
    let job =
        wire::Job::decode(&wire::CEILINGS, &mut Reader::new(after.store.get(job_key.as_ref()).expect("job done row")))
            .expect("done job");
    assert!(job.ended() > 0, "job end committed");
    BTreeSet::from([old_key, new_key, text_key.into_vec(), job_key.into_vec()])
}
