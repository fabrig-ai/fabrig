//! Temper's setting names, family reads and values (temper/docs/design/client/ctl/README.md, 4, and temper/docs/design/client/ctl/tools.md, 2).
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
use temper_protocol_reaches::{Classes, Way, classes_from, classes_record};
pub(crate) fn resolve_project(call: &mut Call, me: Option<&c::Me>) -> bool {
    if call.tool != rows::SETTING_AT || call.form != 2 && call.form != 4 && call.form != 7 {
        return true;
    }
    let at = if call.form == 4 { 1 } else { 2 };
    let project = match value(call, at) {
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
    *call.values.get_mut(at).expect("setting project argument") =
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
        Query::RevisionSetting { setting: c::Setting::Application { kind, scope, number }, revision } => {
            w::Read::RevisionSetting(
                w::RevisionSettingRead::new(
                    &w::CEILINGS,
                    w::RevisionSettingReadParts { kind, scope, number, revision },
                )
                .expect("fixed read"),
            )
        }
        Query::Setting(_) | Query::RevisionSetting { .. } => unreachable!("application setting"),
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
        12 => w::Follow::decode(&w::CEILINGS, &mut reader).is_ok(),
        11 => w::TemplateGoal::decode(&w::CEILINGS, &mut reader).is_ok(),
        6 => w::ProjectQuestions::decode(&w::CEILINGS, &mut reader).is_ok(),
        7 => w::ProjectWays::decode(&w::CEILINGS, &mut reader).is_ok(),
        8 => match w::ProjectClasses::decode(&w::CEILINGS, &mut reader) {
            Ok(record) => classes_from(&record).check().is_ok(),
            Err(_) => false,
        },
        9 => w::CharterWords::decode(&w::CEILINGS, &mut reader).is_ok(),
        10 => w::ProjectWords::decode(&w::CEILINGS, &mut reader).is_ok(),
        _ => false,
    };
    decoded && reader.is_empty()
}
fn setting(page: &Readout, kind: u32) -> Result<Option<c::SettingValue>, ()> {
    let record = wire::decode_setting_answer(bytes(page).ok_or(())?, &w::CEILINGS).ok_or(())?;
    match record.into_parts().value {
        Some(bytes) if valid(kind, &bytes) => Ok(Some(c::SettingValue::Application(bytes))),
        Some(_) => Err(()),
        None => Ok(stock(kind)),
    }
}
fn stock(kind: u32) -> Option<c::SettingValue> {
    let bytes = match kind {
        6 => {
            let value =
                w::ProjectQuestions::new(&w::CEILINGS, w::ProjectQuestionsParts { route: w::QuestionsRoute::None })
                    .expect("stock questions");
            let mut out = skein_lib::Writer::new(usize::try_from(value.measure()).expect("fixed stock size"));
            value.encode(&mut out).expect("fixed stock record");
            out.finish()
        }
        7 => {
            let value = w::ProjectWays::new(
                &w::CEILINGS,
                w::ProjectWaysParts { drafts: true, goals: false, changes: false, way: 0 },
            )
            .expect("stock ways");
            let mut out = skein_lib::Writer::new(usize::try_from(value.measure()).expect("fixed stock size"));
            value.encode(&mut out).expect("fixed stock record");
            out.finish()
        }
        8 => {
            let value = classes_record(&Classes::stock(Way::AgentsReview)).expect("stock classes fit");
            let mut out = skein_lib::Writer::new(usize::try_from(value.measure()).expect("fixed stock size"));
            value.encode(&mut out).expect("fixed stock record");
            out.finish()
        }
        _ => return None,
    };
    Some(c::SettingValue::Application(bytes))
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
        Query::Setting(c::Setting::Application { kind, .. })
        | Query::RevisionSetting { setting: c::Setting::Application { kind, .. }, .. } => match setting(page, kind) {
            Ok(value) => Some(Answer::Setting(value)),
            Err(()) => None,
        },
        Query::Setting(_) | Query::RevisionSetting { .. } => unreachable!("application setting"),
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
    if call.form == 4 {
        let Some(Value::Project(Project::Number(project))) = value(call, 1) else { unreachable!("resolved project") };
        let Some(Value::Word(kind)) = value(call, 0) else { unreachable!("setting kind") };
        return Some(c::Setting::Application {
            kind: match *kind {
                0 => 7,
                1 => 6,
                2 => 8,
                3 => 10,
                _ => unreachable!("project setting kind"),
            },
            scope: *project,
            number: 1,
        });
    }
    if call.form == 2 || call.form == 3 || call.form == 5 || call.form == 6 || call.form == 7 {
        let scope = if call.form == 3 || call.form == 5 || call.form == 6 {
            0
        } else {
            match value(call, 2) {
                Some(Value::Project(Project::Number(project))) => *project,
                Some(_) => unreachable!("resolved project"),
                None => 0,
            }
        };
        return Some(c::Setting::Application {
            kind: if call.form == 7 {
                12
            } else if call.form == 6 {
                11
            } else if call.form == 5 {
                9
            } else if call.form == 3 {
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
        b"`KIND N, revision R`, or `rules, revision R`, then the whole value; project classes in words; temper's settings also show their state",
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
            revision_flag(),
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
            revision_flag(),
        ],
        &[
            positional(
                b"kind",
                b"project-ways|project-questions|project-classes|project-words",
                Kind::Word { set: &[b"project-ways", b"project-questions", b"project-classes", b"project-words"] },
                Required,
                b"A project's ways, questions, classes or words",
            ),
            positional(
                b"project",
                b"PROJECT",
                Kind::Project { empty: false },
                Required,
                b"The project by number or name",
            ),
            revision_flag(),
        ],
        &[
            positional(
                b"kind",
                b"charter-words",
                Kind::Word { set: &[b"charter-words"] },
                Required,
                b"A charter's words",
            ),
            positional(
                b"number",
                b"NUMBER",
                Kind::NumberRange { min: 1, max: 4_294_967_295 },
                Required,
                b"The charter's number",
            ),
            revision_flag(),
        ],
        &[
            positional(
                b"kind",
                b"template-goal",
                Kind::Word { set: &[b"template-goal"] },
                Required,
                b"A goal template's work",
            ),
            positional(
                b"number",
                b"NUMBER",
                Kind::NumberRange { min: 1, max: 4_294_967_295 },
                Required,
                b"The template's number",
            ),
            revision_flag(),
        ],
        &[
            positional(b"kind", b"follow", Kind::Word { set: &[b"follow"] }, Required, b"A project's follow"),
            positional(b"number", b"NUMBER", Kind::NumberRange { min: 1, max: 4_294_967_295 }, Required, b"The follow's number"),
            flag(b"project", b"PROJECT", Kind::Project { empty: false }, Required, b"The follow's project"),
            revision_flag(),
        ],
    ],
    ..rows::SETTING
};

pub(crate) fn name(setting: c::Setting, out: &mut Text) -> bool {
    let c::Setting::Application { kind, scope, number } = setting else { return false };
    if !wire::settings_kind(kind) {
        return false;
    }
    out.put(match kind {
        3 => b"forge-charter ",
        11 => b"template-goal ",
        12 => b"follow ",
        6 => b"project-questions ",
        7 => b"project-ways ",
        8 => b"project-classes ",
        9 => b"charter-words ",
        10 => b"project-words ",
        _ => b"landing-rule ",
    });
    out.number(u64::from(if kind == 6 || kind == 7 || kind == 8 || kind == 10 { scope } else { number }));
    if kind == 2 || kind == 12 {
        out.put(b" --project ");
        out.number(u64::from(scope));
    }
    true
}

mod value;
pub(crate) fn render(kind: u32, bytes: &[u8], out: &mut skein_json::writer::Encoder) -> bool {
    let mut reader = Reader::new(bytes);
    match kind {
        12 => return settings_form::render::application_value(kind, bytes, out),
        9 => {
            let Ok(value) = w::CharterWords::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            out.string(value.line());
        }
        10 => {
            let Ok(value) = w::ProjectWords::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            out.object_start();
            out.key(b"headline");
            out.string(value.headline());
            out.key(b"lede");
            out.string(value.lede());
            out.object_end();
        }
        _ => return value::render(kind, bytes, out),
    }
    true
}

const fn revision_flag() -> crate::jig::table::Argument {
    flag(
        b"revision",
        b"REVISION",
        Kind::NumberRange { min: 1, max: 4_294_967_295 },
        Optional,
        b"The value set by this retained revision; the current value when absent",
    )
}
