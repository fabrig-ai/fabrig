//! The format row and header checked before restart (protocol/store/records.md, 3 and protocol/store/opening.md).
use crate::upgrade::admission;
use crate::{Event, Family, Older, OpenRefusal, Phase, Store, codec, format, keys, restore};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{CoreKey, CoreRecord, Key, Record};
use skein_lib::{Queue, Token};

#[expect(clippy::too_many_lines, reason = "the ordered open checks include the rehearsal's pre-plan walk")]
pub(crate) fn open(store: &mut Store, owner: Token, above: &mut Queue<Event>, below: &mut crate::Below) {
    if store.phase != Phase::Closed {
        refuse(store, owner, OpenRefusal::Failed, above);
        return;
    }
    crate::rehearsal::announce(store, above);
    if let Err(why) = crate::jobs::admission::check(&store.owner_sets) {
        return refuse(store, owner, OpenRefusal::Set { owner: why }, above);
    }
    if let Err(why) = admission::checked(store) {
        return refuse(store, owner, OpenRefusal::Set { owner: why }, above);
    }
    let format_key = keys::encode_own(&keys::Own::Format, store.limits.store.key).expect("format key fits");
    let header_key = keys::encode(&Key::Core(CoreKey::Deployment), store.limits.store.key).expect("header key fits");
    let Ok(found) = get_open(store, owner, format_key) else {
        refuse(store, owner, OpenRefusal::Failed, above);
        return;
    };
    let Ok(header) = get_open(store, owner, header_key) else {
        refuse(store, owner, OpenRefusal::Failed, above);
        return;
    };
    let last = store.stores.store.last();
    let Some(found) = found else {
        if header.is_some() {
            refuse(
                store,
                owner,
                OpenRefusal::Format {
                    family: Family::Component,
                    found: 0,
                    wanted: u32::from(format::current(store.application.version, [0; 16]).component),
                },
                above,
            );
        } else if last == 0 && store.directory.purpose == crate::Purpose::Checkpoint {
            refuse(store, owner, OpenRefusal::Unready { why: crate::Unready::Upgrade }, above);
        } else if last == 0 {
            if let Err(why) = crate::registry::open(store, true) {
                return refuse(store, owner, why, above);
            }
            ready(store, owner, last, true, above, below);
        } else {
            refuse(store, owner, OpenRefusal::Numbers { store: last, header: 0 }, above);
        }
        return;
    };
    let binary = admission::binary(&store.owner_sets);
    let Some(found) = decode(store, owner, &found, binary.jig, above) else { return };
    if !crate::restoring::deployment(store, found.deployment) {
        return refuse(store, owner, OpenRefusal::Deployment, above);
    }
    let found_levels = found.levels;
    let compared = format::compare(found_levels, binary);
    if let Err(why) = &compared
        && *why != (OpenRefusal::OlderBuild { why: Older::Behind })
    {
        return refuse(store, owner, why.clone(), above);
    }
    if !floor(store, owner, above) {
        return;
    }
    if let Err(why) = compared {
        return refuse(store, owner, why, above);
    }
    // Compare the families' versions before reading the registry (protocol/store/records.md, 3).
    let wanted = format::current(store.application.version, found.deployment);
    if let Err(why) = format::check_versions(&found, &wanted) {
        return refuse(store, owner, why, above);
    }
    if !crate::rehearsal::records(store, owner, above) {
        return;
    }
    let jobs = match crate::jobs::kept::pending(store) {
        Ok(pending) => pending,
        Err(why) => return refuse(store, owner, why, above),
    };
    if jobs && store.directory.purpose == crate::Purpose::Checkpoint {
        return refuse(store, owner, OpenRefusal::Unready { why: crate::Unready::Upgrade }, above);
    }
    if jobs {
        store.directory.patch = Some(Box::new(found_levels));
    }
    let registry = match crate::registry::open(store, false) {
        Ok(writing) => writing,
        Err(why) => return refuse(store, owner, why, above),
    };
    if registry && store.directory.purpose == crate::Purpose::Checkpoint {
        return refuse(store, owner, OpenRefusal::Unready { why: crate::Unready::Upgrade }, above);
    }
    if registry && store.directory.patch.is_none() {
        store.directory.patch = Some(Box::new(found_levels));
    }
    if header.is_none() {
        refuse(store, owner, OpenRefusal::Numbers { store: last, header: 0 }, above);
        return;
    }
    let Some(header) = decode_header(store) else {
        corrupt_header(store, owner, restore::DecodeFailure::Value, above);
        return;
    };
    match header {
        Record::Core(CoreRecord::Deployment(header)) => {
            let wanted = codec::component::Format { deployment: header.id, ..wanted };
            if let Err(why) = format::check_versions(&found, &wanted) {
                refuse(store, owner, why, above);
            } else if header.commits != last {
                refuse(store, owner, OpenRefusal::Numbers { store: last, header: header.commits }, above);
            } else {
                restore_installation(store, owner, last, above, below);
            }
        }
        Record::Core(
            CoreRecord::Settings(_)
            | CoreRecord::Projection(_)
            | CoreRecord::Call(_)
            | CoreRecord::RunProof(_)
            | CoreRecord::Turn(_)
            | CoreRecord::Terminal(_)
            | CoreRecord::ProposalDecision(_)
            | CoreRecord::EscalationDecision(_),
        )
        | Record::Tasks(_)
        | Record::People(_)
        | Record::Notes(_)
        | Record::Accounts(_) => {
            corrupt_header(store, owner, restore::DecodeFailure::KeyValueDisagree, above);
        }
    }
}

/// The floor precedes planning and rehearsal decoding (protocol/store/records.md, 3).
fn floor(store: &mut Store, owner: Token, above: &mut Queue<Event>) -> bool {
    let key = keys::encode_own(&keys::Own::Floor, store.limits.store.key).expect("floor key fits");
    let bytes = match get_open(store, owner, key.clone()) {
        Ok(bytes) => bytes,
        Err(why) => {
            refuse(store, owner, why, above);
            return false;
        }
    };
    let number = if let Some(bytes) = bytes {
        let Ok(row) =
            crate::component_v3::Floor::decode(&crate::component_v3::CEILINGS, &mut skein_lib::Reader::new(&bytes))
        else {
            data(store, &key, restore::DecodeFailure::Value);
            corrupt(store, owner, above);
            return false;
        };
        Some(row.number())
    } else {
        None
    };
    if number < Some(crate::FLOOR) {
        refuse(store, owner, OpenRefusal::OlderBuild { why: Older::Floor }, above);
        return false;
    }
    true
}

/// The header's registered layout may add defaults before this open writes.
pub(crate) fn decode_header(store: &mut Store) -> Option<Record> {
    let key = Key::Core(CoreKey::Deployment);
    let encoded = keys::encode(&key, store.limits.store.key)?;
    let mut bytes = crate::view::get(store, encoded.clone()).ok()??;
    crate::registry::header(store, &encoded, &mut bytes);
    codec::decode(&key, &bytes, &store.codec_limits, &store.names)
}

/// The identifier kept by the first commit replaces the drawn one (4.4); a
/// checked store without it is corrupt.
fn restore_installation(
    store: &mut Store,
    owner: Token,
    last: u64,
    above: &mut Queue<Event>,
    below: &mut crate::Below,
) {
    let key = keys::installation(store.limits.store.key).expect("installation key fits");
    let Ok(found) = get_open(store, owner, key) else {
        refuse(store, owner, OpenRefusal::Failed, above);
        return;
    };
    let Some(bytes) = found else {
        store.open_failed(restore::OpenFailure::Missing(restore::site(
            store,
            &keys::installation(store.limits.store.key).expect("installation key fits"),
        )));
        corrupt(store, owner, above);
        return;
    };
    let Ok(kept) = codec::accounts::Installation::decode(&bytes) else {
        data(
            store,
            &keys::installation(store.limits.store.key).expect("installation key fits"),
            restore::DecodeFailure::Value,
        );
        corrupt(store, owner, above);
        return;
    };
    store.installation = kept.id;
    ready(store, owner, last, false, above, below);
}

pub(crate) fn get(store: &mut Store, owner: Token, key: Box<[u8]>) -> Result<Option<Box<[u8]>>, OpenRefusal> {
    get_row(store, owner, key, false)
}

pub(crate) fn get_open(store: &mut Store, owner: Token, key: Box<[u8]>) -> Result<Option<Box<[u8]>>, OpenRefusal> {
    get_row(store, owner, key, true)
}

fn get_row(store: &mut Store, _owner: Token, key: Box<[u8]>, opening: bool) -> Result<Option<Box<[u8]>>, OpenRefusal> {
    let site = if opening { Some(restore::site(store, &key)) } else { None };
    match crate::view::get(store, key.clone()) {
        Ok(mut value) => {
            if let Some(value) = &mut value {
                crate::registry::read(store, &key, value);
            }
            Ok(value)
        }
        Err(failure) => {
            if let Some(site) = site {
                store.open_failed(restore::OpenFailure::Data(restore::Failure::at(
                    restore::Cause::Join { target: site.clone(), failure },
                    site,
                )));
            }
            Err(OpenRefusal::Failed)
        }
    }
}

fn ready(
    store: &mut Store,
    owner: Token,
    last: u64,
    format_due: bool,
    above: &mut Queue<Event>,
    below: &mut crate::Below,
) {
    let Some(next) = last.checked_add(1) else {
        refuse(store, owner, OpenRefusal::Failed, above);
        return;
    };
    if let Err(why) = admission::checked(store) {
        refuse(store, owner, OpenRefusal::Set { owner: why }, above);
        return;
    }
    // The copy's serving machines are made after its check.
    if store.directory.patch.is_none() {
        store.transcripts.machines = match crate::transcripts::patches::make(&store.owner_sets, &store.limits) {
            Ok(machines) => machines,
            Err(why) => {
                refuse(store, owner, OpenRefusal::Set { owner: why }, above);
                return;
            }
        };
        // Serving codecs read the catalogue after the registry's commit and jobs (protocol/store/opening.md, 2).
        if let Err(failure) = crate::catalogue::catalogue(store) {
            store.open_failed(restore::OpenFailure::Data(failure));
            corrupt(store, owner, above);
            return;
        }
    }
    store.phase = Phase::Ready { next, format_due };
    store.directory.owner = owner;
    crate::directory::checked(store, above, below);
}

pub(crate) fn refuse(store: &mut Store, owner: Token, why: OpenRefusal, above: &mut Queue<Event>) {
    crate::rehearsal::announce(store, above);
    store.directory.checked = None;
    store.open_failed(restore::OpenFailure::Refused(why.clone()));
    store.phase = Phase::Stopped;
    above.push(Event::OpenRefused { owner, why });
}

fn corrupt_header(store: &mut Store, owner: Token, failure: restore::DecodeFailure, above: &mut Queue<Event>) {
    let key = keys::encode(&Key::Core(CoreKey::Deployment), store.limits.store.key).expect("header key fits");
    data(store, &key, failure);
    corrupt(store, owner, above);
}

pub(crate) fn data(store: &mut Store, key: &[u8], failure: restore::DecodeFailure) {
    if store.start_open_failure.is_none() {
        store.open_failed(restore::OpenFailure::Data(restore::Failure::at(
            restore::Cause::Decode(failure),
            restore::site(store, key),
        )));
    }
}

fn corrupt(store: &mut Store, owner: Token, above: &mut Queue<Event>) {
    store.phase = Phase::Stopped;
    above.push(Event::Corrupt { owner });
}

/// A later component's levels are unread; a same-version row that no longer
/// decodes is an older layout, including L's row before levels were added.
fn decode(
    store: &mut Store,
    owner: Token,
    bytes: &[u8],
    jig: Option<u32>,
    above: &mut Queue<Event>,
) -> Option<codec::component::Format> {
    let wanted_version = format::current(store.application.version, [0; 16]).component;
    if let Some(version) = codec::component::Format::version(bytes)
        && version > wanted_version
    {
        refuse(store, owner, OpenRefusal::Ahead { owner: crate::Owner::Jig, found: None, wanted: jig }, above);
        return None;
    }
    let Ok(found) = codec::component::Format::decode(bytes) else {
        // Another version of the component's own rows says which (5.3).
        let wanted = format::current(store.application.version, [0; 16]).component;
        match codec::component::Format::version(bytes) {
            Some(version) if version != wanted => refuse(
                store,
                owner,
                OpenRefusal::Format { family: Family::Component, found: u32::from(version), wanted: u32::from(wanted) },
                above,
            ),
            Some(_) => refuse(store, owner, OpenRefusal::OlderBuild { why: Older::Layout(Family::Component) }, above),
            None => {
                data(
                    store,
                    &keys::encode_own(&keys::Own::Format, store.limits.store.key).expect("format key fits"),
                    restore::DecodeFailure::Value,
                );
                corrupt(store, owner, above);
            }
        }
        return None;
    };
    Some(found)
}
