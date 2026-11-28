//! Fixed version refusals and family admission envelopes (protocol/client/versions.md, 1, and protocol/client/README.md, 8).
use crate::{Settings, boundary::Served, translate};
use alloc::boxed::Box;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::documents;
use jig_domain_people::ApplicationScope;
use skein_lib::List;
pub(crate) fn serves(settings: &Settings, family: Option<u16>, version: u32) -> bool {
    for owner in &settings.served {
        if owner.family == family && owner.version == version {
            return true;
        }
    }
    false
}
pub(crate) fn not_served(settings: &Settings, owner: w::Owner) -> w::NotServed {
    w::NotServed { owner, served: wire_ranges(settings) }
}
pub(crate) fn wire_ranges(settings: &Settings) -> List<w::Served> {
    let mut served = List::with_capacity(settings.served.len());
    for &Served { family, version, .. } in &settings.served {
        let owner = match family {
            None => w::Owner::Jig,
            Some(number) => w::Owner::Family(w::Family { number }),
        };
        served.push(wire_range(owner, version)).expect("served ranges fit");
    }
    served
}
pub(crate) fn wire_range(owner: w::Owner, version: u32) -> w::Served {
    w::Served { owner, first: version, last: version }
}
pub(crate) fn scope(scope: &w::Scope) -> ApplicationScope {
    match scope {
        w::Scope::Project(project) => ApplicationScope::Project(project.value),
        w::Scope::Deployment => ApplicationScope::Deployment,
        w::Scope::Own => ApplicationScope::Own,
    }
}
pub(crate) fn window(window: &w::Window) -> Option<documents::Window> {
    let after = match &window.after {
        Some(cursor) => Some(translate::cursor::decode_window(cursor.value.as_ref())?),
        None => None,
    };
    Some(documents::Window { rows: window.rows, after })
}

pub(crate) struct Opening {
    pub request: crate::Request,
    pub rows: u32,
    pub parts: List<translate::parts::Slot>,
}
pub(crate) fn watch(
    subject: w::Owned,
    watcher: skein_lib::Token,
    sign_in: u64,
    rows: u32,
    limits: &w::Limits,
    settings: &Settings,
) -> Option<Opening> {
    match subject.owner {
        w::Owner::Jig => {
            let value = translate::watches::decode(&subject.record, limits)?;
            let mut event = translate::watches::watch(watcher, sign_in, value, rows)?;
            let mut parts = List::with_capacity(0);
            if let jig_domain_core_vocabulary::Event::Watch { subject: documents::Watched::Query { query }, .. } =
                &mut event
            {
                parts = translate::parts::plan(&query.parts, settings);
                query.parts = translate::parts::served(&parts);
            }
            let mut rows = rows;
            if let jig_domain_core_vocabulary::Event::Watch { window, .. } = &event {
                rows = window.rows;
            }
            Some(Opening { request: crate::Request::Core(Box::new(event)), rows, parts })
        }
        w::Owner::Family(value) => {
            let record = w::FamilyRecord::decode(limits, &mut skein_lib::Reader::new(&subject.record)).ok()?;
            let window = window(&record.window)?;
            let request = crate::Request::Application(crate::ToApplication::Watch {
                watcher,
                sign_in,
                family: value.number,
                version: subject.version,
                scope: scope(&record.scope),
                window,
                record: record.record,
            });
            Some(Opening { request, rows: window.rows, parts: List::with_capacity(0) })
        }
    }
}
