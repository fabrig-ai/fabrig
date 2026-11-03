//! Temper's setting names, family reads and values (temper/docs/design/client/tools.md, 4 and 5.2).
use crate::jig::{
    call::{Call, Project, Value, number, value},
    render::Text,
    rows,
    runner::application::{Answer, Query},
};
use alloc::collections::BTreeSet;
use jig_client::{Family, Readable, Readout, contract as c};
use skein_lib::{List, Reader};
use temper_client_protocol_wire::{self as wire, v1 as w};
pub(crate) fn resolve_project(call: &mut Call, me: Option<&c::Me>) -> bool {
    if call.tool != rows::SETTING_AT || call.form != 2 {
        return true;
    }
    let project = match value(call, 2) {
        Some(Value::Project(Project::Number(number))) => Some(*number),
        Some(Value::Project(Project::Name(name))) => {
            let mut found = None;
            if let Some(me) = me {
                for project in &me.projects {
                    if project.name.bytes() == name.as_ref() {
                        found = Some(project.project);
                        break;
                    }
                }
            }
            found
        }
        Some(_) => unreachable!("project form"),
        None => return true,
    };
    let Some(project) = project else { return false };
    *call.values.get_mut(2).expect("setting project argument") =
        Some(crate::jig::call::Values::One(Value::Project(Project::Number(project))));
    true
}
pub(crate) fn setting_query(kind: u32, scope: u32, number: u32) -> w::Read {
    w::Read::Setting(
        w::SettingRead::new(&w::CEILINGS, w::SettingReadParts { kind, scope, number }).expect("fixed read"),
    )
}
pub(crate) fn read(query: Query) -> Readable {
    let query = match query {
        Query::Setting(c::Setting::Application { kind, scope, number }) => setting_query(kind, scope, number),
        Query::Setting(_) => unreachable!("application setting"),
        Query::ProposalItems { key } => w::Read::ProposalItems(
            w::ProposalItemsRead::new(&w::CEILINGS, w::ProposalItemsReadParts { key }).expect("fixed read"),
        ),
    };
    let record = w::Reading::new(&w::CEILINGS, w::ReadingParts { read: query }).expect("fixed read");
    Readable::Family {
        scope: c::Scope::Deployment,
        window: c::Window { rows: 1, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::reading(&record) },
    }
}
fn bytes(page: &Readout) -> Option<&[u8]> {
    match page {
        Readout::Family { family: wire::FORGE, record: Some(bytes) } => Some(bytes),
        Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. } => None,
    }
}
fn valid(kind: u32, bytes: &[u8]) -> bool {
    let mut reader = Reader::new(bytes);
    let decoded = match kind {
        1 | 2 => w::LandingRule::decode(&w::CEILINGS, &mut reader).is_ok(),
        3 => w::ForgeCharter::decode(&w::CEILINGS, &mut reader).is_ok(),
        _ => false,
    };
    decoded && reader.is_empty()
}
fn setting(page: &Readout, kind: u32) -> Result<Option<c::SettingValue>, ()> {
    let record = wire::decode_setting_answer(bytes(page).ok_or(())?, &w::CEILINGS).ok_or(())?;
    match record.into_parts().value {
        Some(bytes) if valid(kind, &bytes) => Ok(Some(c::SettingValue::Application(bytes))),
        Some(_) => Err(()),
        None => Ok(None),
    }
}
fn proposal_items(page: &Readout) -> Option<w::ProposalItemsAnswer> {
    let record = wire::decode_proposal_items_answer(bytes(page)?, &w::CEILINGS)?;
    let mut keys = BTreeSet::new();
    for item in record.items() {
        if !keys.insert((item.kind(), item.scope(), item.number())) {
            return None;
        }
        if let w::SettingEdit::Set(set) = item.edit()
            && !valid(item.kind(), set.value())
        {
            return None;
        }
    }
    Some(record)
}
pub(crate) fn decode(query: Query, page: &Readout) -> Option<Answer> {
    match query {
        Query::Setting(c::Setting::Application { kind, .. }) => match setting(page, kind) {
            Ok(value) => Some(Answer::Setting(value)),
            Err(()) => None,
        },
        Query::Setting(_) => unreachable!("application setting"),
        Query::ProposalItems { .. } => {
            let record = proposal_items(page)?;
            let mut items = List::with_capacity(record.items().len());
            for item in record.into_parts().items.into_boxed() {
                let item = item.into_parts();
                let edit = match item.edit {
                    w::SettingEdit::Set(value) => c::SettingEdit::Set(c::SettingSet {
                        value: c::SettingValue::Application(value.into_parts().value),
                    }),
                    w::SettingEdit::Removed => c::SettingEdit::Removed,
                    w::SettingEdit::Reset => c::SettingEdit::Reset,
                };
                items
                    .push(c::SettingItem {
                        setting: c::Setting::Application { kind: item.kind, scope: item.scope, number: item.number },
                        edit,
                    })
                    .expect("one decoded item");
            }
            Some(Answer::ProposalItems(items))
        }
    }
}
pub(crate) fn selector(call: &Call) -> Option<c::Setting> {
    if call.form == 2 || call.form == 3 {
        let scope = match value(call, 2) {
            Some(Value::Project(Project::Number(project))) => *project,
            Some(_) => unreachable!("resolved project"),
            None => 0,
        };
        return Some(c::Setting::Application {
            kind: if call.form == 3 {
                3
            } else if scope == 0 {
                1
            } else {
                2
            },
            scope,
            number: u32::try_from(number(call, 1).expect("setting number")).expect("parsed u32"),
        });
    }
    None
}

use crate::jig::table::{
    Kind,
    Occurs::{Optional, Required},
    Tool, flag, positional,
};
#[expect(clippy::indexing_slicing, reason = "the host extends the kit's two fixed setting forms in order")]
pub const SETTING: Tool = Tool {
    prints:
        b"`KIND N, revision R`, or `rules, revision R`, then the value as JSON; temper's settings also show their state",
    forms: &[
        rows::SETTING.forms[0],
        rows::SETTING.forms[1],
        &[
            positional(b"kind", b"landing-rule", Kind::Word { set: &[b"landing-rule"] }, Required, b"A landing rule"),
            positional(
                b"number",
                b"NUMBER",
                Kind::NumberRange { min: 1, max: 4_294_967_295 },
                Required,
                b"The rule's number",
            ),
            flag(
                b"project",
                b"PROJECT",
                Kind::Project { empty: false },
                Optional,
                b"The project's rule; the deployment's when absent",
            ),
        ],
        &[
            positional(
                b"kind",
                b"forge-charter",
                Kind::Word { set: &[b"forge-charter"] },
                Required,
                b"One of the forge's charters",
            ),
            positional(
                b"number",
                b"NUMBER",
                Kind::NumberRange { min: 1, max: 2 },
                Required,
                b"1 the change's charter, 2 its reviews'",
            ),
        ],
    ],
    ..rows::SETTING
};

pub(crate) fn name(setting: c::Setting, out: &mut Text) -> bool {
    if let c::Setting::Application { kind: 1..=3, scope, number } = setting {
        out.put(if let c::Setting::Application { kind: 3, .. } = setting {
            b"forge-charter "
        } else {
            b"landing-rule "
        });
        out.number(u64::from(number));
        if let c::Setting::Application { kind: 2, .. } = setting {
            out.put(b" --project ");
            out.number(u64::from(scope));
        }
        true
    } else {
        false
    }
}
mod value;
pub(crate) use value::render;
