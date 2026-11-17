//! Fixed version refusals and family admission envelopes (protocol/client/README.md, 6 and 8).
use crate::{Settings, boundary::Served, translate};
use alloc::boxed::Box;
<<<<<<< HEAD
use jig_client_protocol_wire::v28 as w;
=======
use jig_client_protocol_wire as w;
>>>>>>> 1ec4d558d2251b97ed59056727b4b65e5fbbf56b
use jig_domain_core_vocabulary::documents;
use jig_domain_people::ApplicationScope;
use skein_lib::List;
pub(crate) fn serves(settings: &Settings, family: Option<u16>, version: u32) -> bool {
    for range in &settings.served {
        if range.family == family && range.first <= version && version <= range.last {
            return true;
        }
    }
    false
}
pub(crate) fn not_served(settings: &Settings, owner: w::Owner, limits: &w::Limits) -> w::NotServed {
    let mut served = List::with_capacity(settings.served.len());
    for Served { family, first, last } in &settings.served {
        let owner = match family {
            None => w::Owner::Jig,
            Some(number) => {
                w::Owner::Family(w::Family::new(limits, w::FamilyParts { number: *number }).expect("family"))
            }
        };
        served
            .push(w::Served::new(limits, w::ServedParts { owner, first: *first, last: *last }).expect("range"))
            .expect("served bound");
    }
    w::NotServed::new(limits, w::NotServedParts { owner, served }).expect("settings served bound")
}
pub(crate) fn scope(scope: &w::Scope) -> ApplicationScope {
    match scope {
        w::Scope::Project(project) => ApplicationScope::Project(project.value()),
        w::Scope::Deployment => ApplicationScope::Deployment,
        w::Scope::Own => ApplicationScope::Own,
    }
}
pub(crate) fn window(window: &w::Window) -> Option<documents::Window> {
    let after = match window.after() {
        Some(cursor) => Some(translate::cursor::decode_window(cursor.value())?),
        None => None,
    };
    Some(documents::Window { rows: window.rows(), after })
}

pub(crate) struct Opening {
    pub request: crate::Request,
    pub project: u32,
    pub rows: u32,
    pub parts: List<translate::parts::Slot>,
}
pub(crate) fn watch(
    subject: w::OwnedParts,
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
                parts = translate::parts::plan(&query.parts, settings, limits);
                query.parts = translate::parts::served(&parts);
            }
            let project = 0;
            let mut rows = rows;
            if let jig_domain_core_vocabulary::Event::Watch { window, .. } = &event {
                rows = window.rows;
            }
            Some(Opening { request: crate::Request::Core(Box::new(event)), project, rows, parts })
        }
        w::Owner::Family(value) => {
            let record = w::FamilyRecord::decode(limits, &mut skein_lib::Reader::new(&subject.record)).ok()?;
            let window = window(record.window())?;
            let request = crate::Request::Application(crate::ToApplication::Watch {
                watcher,
                sign_in,
                family: value.number(),
                version: subject.version,
                scope: scope(record.scope()),
                window,
                record: record.into_parts().record,
            });
            Some(Opening { request, project: 0, rows: window.rows, parts: List::with_capacity(0) })
        }
    }
}
