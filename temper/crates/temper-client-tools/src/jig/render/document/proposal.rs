//! Proposal reads and the settings review supplied by wire 19 (client/tools.md, 5.2).
use super::{Cut, Limits, Text, settings};
use crate::jig::runner::{proposal::slot, reads::Reading};
use alloc::boxed::Box;
use jig_client::{Readout, contract as c};
use skein_json::writer::Encoder;

fn selected(value: &c::ReadValue) -> &c::Proposal {
    match value {
        c::ReadValue::Proposal(p) => p,

        c::ReadValue::Settings(_)
        | c::ReadValue::Setting(_)
        | c::ReadValue::Revision(_)
        | c::ReadValue::Revisions(_)
        | c::ReadValue::TaskText(_)
        | c::ReadValue::Tasks(_)
        | c::ReadValue::Project(_)
        | c::ReadValue::Ended(_)
        | c::ReadValue::Me(_)
        | c::ReadValue::Turn(_)
        | c::ReadValue::Task(_)
        | c::ReadValue::Escalation(_)
        | c::ReadValue::Question(_)
        | c::ReadValue::PersonTask(_)
        | c::ReadValue::Result(_)
        | c::ReadValue::EndedPage(_)
        | c::ReadValue::History(_)
        | c::ReadValue::Activity(_)
        | c::ReadValue::Inbox(_)
        | c::ReadValue::Spend(_)
        | c::ReadValue::Notes(_)
        | c::ReadValue::Deployment(_) => unreachable!("selected proposal read"),
    }
}

pub(super) fn text(value: &c::ReadValue, reading: &Reading, limits: &Limits, out: &mut Text) {
    let (key, reason, action) = {
        let p = selected(value);
        (p.key, p.reason.bytes.bytes(), owner_action(&p.action))
    };
    out.put(b"proposal ");
    settings::proposal_text(key, out);
    out.put(b" \xc2\xb7 ");
    out.put(action);
    release_text(value, out);
    if let c::ReadValue::Proposal(p) = value
        && let c::ProposalAction::Settings(s) = &p.action
    {
        out.put(b" \xc2\xb7 against revision ");
        out.number(u64::from(s.against));
        if let Some(review) = &s.review {
            out.put(b", newest ");
            out.number(u64::from(review.newest));
        }
    }
    out.put(b"\n");
    out.words(reason);
    out.put(b"\n");
    if let c::ReadValue::Proposal(p) = value
        && let c::ProposalAction::Settings(s) = &p.action
        && let Some(review) = &s.review
    {
        for (index, item) in review.items.iter().enumerate() {
            if let c::Setting::Application { .. } = item.setting {
                continue;
            }
            settings::setting_text(item.setting, reading.parts, out);
            out.put(b" \xc2\xb7 ");
            out.put(review_change(item));
            out.put(b" \xc2\xb7 now ");
            review_value_text(
                &item.now,
                item.setting,
                reading.pages.get(&slot(index, false)),
                limits,
                reading.parts,
                out,
            );
            match item.setting {
                c::Setting::ProjectPolicy { .. } => {}
                c::Setting::Charter(_)
                | c::Setting::Template(_)
                | c::Setting::Policy(_)
                | c::Setting::Rules
                | c::Setting::Application { .. } => {
                    out.put(b" (");
                    origin_text(&item.origin, out);
                    out.put(b")");
                }
            }
            out.put(b" \xc2\xb7 would be ");
            review_value_text(
                &item.would_be,
                item.setting,
                reading.pages.get(&slot(index, true)),
                limits,
                reading.parts,
                out,
            );
            out.put(b"\n");
        }
        application_items_text(reading, limits, out);
        if review.cut {
            out.put(b"cut: more not shown\n");
        }
    }
}
fn review_change(item: &c::ReviewItem) -> &'static [u8] {
    match item.edit {
        c::RevisionItemEdit::Set => b"set",
        c::RevisionItemEdit::Removed => b"removed",
        c::RevisionItemEdit::Reset => {
            if item.now == item.would_be {
                b"kept"
            } else {
                b"changed"
            }
        }
    }
}
fn resolved<'a>(
    value: &'a c::ReviewValue,
    setting: c::Setting,
    page: Option<&'a Readout>,
) -> Result<Option<&'a c::SettingValue>, Box<[u8]>> {
    match value {
        c::ReviewValue::Value(set) => Ok(Some(&set.value)),
        c::ReviewValue::None => Ok(None),
        c::ReviewValue::TooLarge(_) => read_value(setting, page),
    }
}
fn read_value(setting: c::Setting, page: Option<&Readout>) -> Result<Option<&c::SettingValue>, Box<[u8]>> {
    match page {
        Some(Readout::Jig(c::ReadResult::Value(c::ReadValue::Setting(document)))) if document.setting == setting => {
            Ok(document.value.as_ref())
        }
        Some(Readout::Jig(c::ReadResult::None(why))) => {
            let reason: &[u8] = match why {
                c::ReadNone::Refused => b"refused",
                c::ReadNone::Unknown => b"unknown",
                c::ReadNone::Gone => b"gone",
                c::ReadNone::Busy => b"busy",
                c::ReadNone::SignedOut => b"signed out",
                c::ReadNone::Unreachable => b"unreachable",
                c::ReadNone::Malformed => b"malformed document",
                c::ReadNone::TooLarge { .. } => b"document limit",
                c::ReadNone::TooLinked { joins } => return Err(super::super::join_limit(*joins)),
            };
            Err(Box::from(reason))
        }
        Some(Readout::Limit) => Err(Box::from(b"read table limit".as_slice())),
        Some(Readout::NotServed { .. }) => Err(Box::from(b"version not served".as_slice())),
        Some(Readout::Jig(c::ReadResult::Value(_)) | Readout::Family { .. }) | None => {
            Err(Box::from(b"tools' bug; unexpected setting document; report it as a finding".as_slice()))
        }
    }
}

fn review_value_text(
    value: &c::ReviewValue,
    setting: c::Setting,
    page: Option<&Readout>,
    limits: &Limits,
    vocabulary: &dyn crate::jig::render::Parts,
    out: &mut Text,
) {
    if let c::Setting::ProjectPolicy { .. } = setting
        && let c::ReviewValue::TooLarge(size) = value
        && page.is_none()
    {
        out.put(b"too large, ");
        out.number(size.bytes);
        out.put(b" bytes");
        return;
    }
    value_text(resolved(value, setting, page), setting, limits, vocabulary, out);
}
fn value_text(
    value: Result<Option<&c::SettingValue>, Box<[u8]>>,
    setting: c::Setting,
    limits: &Limits,
    vocabulary: &dyn crate::jig::render::Parts,
    out: &mut Text,
) {
    match value {
        Ok(Some(value)) => settings::named_value_inline(setting, value, limits, vocabulary, out),
        Ok(None) => out.put(b"none"),
        Err(why) => out.put(&why),
    }
}
fn task_number(task: &c::TaskLink) -> u64 {
    match task {
        c::TaskLink::Live(chip) | c::TaskLink::Retained { chip, .. } => chip.task,
        c::TaskLink::Stored { number, .. } => *number,
    }
}
fn release_text(value: &c::ReadValue, out: &mut Text) {
    if let c::ReadValue::Proposal(p) = value
        && let c::ProposalAction::Release { task, hold } = &p.action
    {
        out.put(b" task ");
        out.number(task_number(task));
        if let Some(hold) = hold
            && let Some(placed) = &hold.placed
        {
            out.put(b" from hold ");
            out.number(placed.number);
            out.put(b" (revision ");
            out.number(placed.revision.value);
            out.put(b")");
        }
    }
}
fn owner_action(action: &c::ProposalAction) -> &'static [u8] {
    match action {
        c::ProposalAction::Settings(_) => b"settings",
        c::ProposalAction::Batch(_) => b"batch",
        c::ProposalAction::Effect { .. } => b"effect",
        c::ProposalAction::ProcedureEffect { .. } => b"procedure_effect",
        c::ProposalAction::Amend { .. } => b"amend",
        c::ProposalAction::Widen { .. } => b"widen",
        c::ProposalAction::Release { .. } => b"release",
    }
}

pub(super) fn encode(value: &c::ReadValue, reading: &Reading, out: &mut Encoder, budget: &mut Cut) {
    out.object_start();
    let proposal = selected(value);
    let (key, project, acts) = (proposal.key, proposal.project, &proposal.acts);
    out.key(b"key");
    super::encode_proposal_key(&key, out, budget);
    out.key(b"project");
    out.unsigned(u64::from(project));
    let p = proposal;

    out.key(b"reason");
    super::encode_document_text(&p.reason, out, budget);
    out.key(b"action");
    match &p.action {
        c::ProposalAction::Settings(s) => encode_settings(s, reading, out),
        c::ProposalAction::Release { task, hold } => {
            out.object_start();
            out.key(b"release");
            out.object_start();
            out.key(b"task");
            out.unsigned(task_number(task));
            out.key(b"hold");
            match hold {
                Some(hold) => super::encode_owner_chip_hold(hold, out, budget),
                None => out.null(),
            }
            out.object_end();
            out.object_end();
        }
        action @ (c::ProposalAction::Batch(_)
        | c::ProposalAction::Effect { .. }
        | c::ProposalAction::ProcedureEffect { .. }
        | c::ProposalAction::Amend { .. }
        | c::ProposalAction::Widen { .. }) => out.string(owner_action(action)),
    }
    out.key(b"as_holder");
    out.boolean(p.as_holder);
    out.key(b"waiting");
    out.object_start();
    out.key(b"holder");
    holder(p.waiting.holder, out, budget);
    out.key(b"since");
    out.unsigned(p.waiting.since.as_nanos());
    out.object_end();
    out.key(b"funding");
    out.array_start();
    for offer in &p.funding {
        out.object_start();
        out.key(b"source");
        super::encode_funder(&offer.source, out, budget);
        out.key(b"needed");
        out.unsigned(offer.needed);
        for (key, value) in [(b"left".as_slice(), offer.left), (b"after".as_slice(), offer.after)] {
            out.key(key);
            match value {
                Some(n) => out.unsigned(n),
                None => out.null(),
            }
        }
        out.object_end();
    }
    out.array_end();

    out.key(b"acts");
    out.array_start();
    for act in acts {
        super::encode_offered(act, out, budget);
    }
    out.array_end();
    out.object_end();
}
fn holder(holder: c::HistoryHolder, out: &mut Encoder, budget: &mut Cut) {
    out.object_start();
    match holder {
        c::HistoryHolder::Task(n) => holder_number(b"task", n, out),
        c::HistoryHolder::Person(n) => holder_number(b"person", n, out),
        c::HistoryHolder::Policy { project, kind } => {
            out.key(b"policy");
            out.object_start();
            out.key(b"project");
            out.unsigned(u64::from(project));
            out.key(b"kind");
            super::read_owner_words::encode_owner_message_proposal_kind(kind, out, budget);
            out.object_end();
        }
    }
    out.object_end();
}
fn holder_number(kind: &[u8], n: u64, out: &mut Encoder) {
    out.key(kind);
    out.object_start();
    out.key(b"value");
    out.unsigned(n);
    out.object_end();
}

fn encode_settings(s: &c::ProposalActionSettings, reading: &Reading, out: &mut Encoder) {
    encode_settings_fields(s.against, &s.items, s.review.as_ref(), s.made, Some(reading), out);
}
fn encode_settings_fields(
    against: u32,
    items: &skein_lib::List<c::RevisionItemHead>,
    review: Option<&c::SettingsReview>,
    made: Option<u32>,
    reading: Option<&Reading>,
    out: &mut Encoder,
) {
    out.object_start();
    out.key(b"settings");
    out.object_start();
    out.key(b"against");
    out.unsigned(u64::from(against));
    out.key(b"items");
    out.array_start();
    for item in items {
        out.object_start();
        out.key(b"setting");
        settings::encode_setting(item.setting, out);
        out.key(b"edit");
        out.string(settings::edit_word(&item.edit));
        out.object_end();
    }
    out.array_end();
    out.key(b"review");
    match review {
        Some(review) => encode_review(review, reading.expect("pending review reads"), out),
        None => out.null(),
    }
    if let Some(reading) = reading
        && reading.application.items.is_some()
    {
        out.key(b"application_items");
        encode_application_items(reading, out);
    }
    out.key(b"made");
    match made {
        Some(n) => out.unsigned(u64::from(n)),
        None => out.null(),
    }
    out.object_end();
    out.object_end();
}
fn encode_review(review: &c::SettingsReview, reading: &Reading, out: &mut Encoder) {
    out.object_start();
    out.key(b"newest");
    out.unsigned(u64::from(review.newest));
    out.key(b"items");
    out.array_start();
    for (index, item) in review.items.iter().enumerate() {
        out.object_start();
        out.key(b"setting");
        settings::encode_setting(item.setting, out);
        out.key(b"edit");
        out.string(settings::edit_word(&item.edit));
        out.key(b"now");
        encode_review_value(&item.now, item.setting, reading.pages.get(&slot(index, false)), reading.parts, out);
        out.key(b"origin");
        match &item.origin {
            c::SettingOrigin::Default(r) => origin_revision(b"default", r.revision, out),
            c::SettingOrigin::Change(r) => origin_revision(b"change", r.revision, out),
            c::SettingOrigin::NotOffered => out.string(b"not_offered"),
        }
        out.key(b"would_be");
        encode_review_value(&item.would_be, item.setting, reading.pages.get(&slot(index, true)), reading.parts, out);
        out.object_end();
    }
    out.array_end();
    out.key(b"cut");
    out.boolean(review.cut);
    out.object_end();
}
fn origin_revision(kind: &[u8], revision: u32, out: &mut Encoder) {
    out.object_start();
    out.key(kind);
    out.object_start();
    out.key(b"revision");
    out.unsigned(u64::from(revision));
    out.object_end();
    out.object_end();
}

fn encode_review_value(
    value: &c::ReviewValue,
    setting: c::Setting,
    page: Option<&Readout>,
    vocabulary: &dyn crate::jig::render::Parts,
    out: &mut Encoder,
) {
    if let c::Setting::ProjectPolicy { .. } = setting
        && let c::ReviewValue::TooLarge(size) = value
        && page.is_none()
    {
        out.object_start();
        out.key(b"too_large");
        out.object_start();
        out.key(b"bytes");
        out.unsigned(size.bytes);
        out.object_end();
        out.object_end();
        return;
    }
    encode_value_result(resolved(value, setting, page), setting, vocabulary, out);
}
fn encode_value_result(
    value: Result<Option<&c::SettingValue>, Box<[u8]>>,
    setting: c::Setting,
    vocabulary: &dyn crate::jig::render::Parts,
    out: &mut Encoder,
) {
    match value {
        Ok(Some(value)) => {
            out.object_start();
            out.key(b"value");
            out.object_start();
            out.key(b"value");
            out.object_start();
            out.key(match value {
                c::SettingValue::Charter(_) => b"charter",
                c::SettingValue::Template(_) => b"template",
                c::SettingValue::Policy(_) => b"policy",
                c::SettingValue::Application(_) => b"application",
                c::SettingValue::Rules(_) => b"rules",
            });
            settings::encode_named_value(setting, value, vocabulary, out);
            out.object_end();
            out.object_end();
            out.object_end();
        }
        Ok(None) => out.string(b"none"),
        Err(why) => {
            out.object_start();
            out.key(b"refused");
            out.string(&why);
            out.object_end();
        }
    }
}

fn history_state(state: &c::ProposalHistoryState) -> &'static [u8] {
    match state {
        c::ProposalHistoryState::Pending { .. } => b"pending",
        c::ProposalHistoryState::Accepted { .. } => b"accepted",
        c::ProposalHistoryState::Rejected { .. } => b"rejected",
        c::ProposalHistoryState::Withdrawn => b"withdrawn",
    }
}
fn retained_kind(action: &c::RetainedAction) -> &'static [u8] {
    match action {
        c::RetainedAction::Settings(_) => b"settings",
        c::RetainedAction::Batch(_) => b"batch",
        c::RetainedAction::Effect { .. } => b"effect",
        c::RetainedAction::ProcedureEffect { .. } => b"procedure_effect",
        c::RetainedAction::Amend { .. } => b"amend",
        c::RetainedAction::Widen { .. } => b"widen",
        c::RetainedAction::Release { .. } => b"release",
    }
}
fn person_state(state: &c::PersonProposalState) -> &'static [u8] {
    match state {
        c::PersonProposalState::Pending { .. } => b"pending",
        c::PersonProposalState::Accepted { .. } => b"accepted",
        c::PersonProposalState::Rejected { .. } => b"rejected",
    }
}
pub(super) fn history_text(value: &c::Ended, out: &mut Text) {
    let key = super::super::ended_proposal(value).expect("validated proposal history");
    out.put(b"proposal ");
    settings::proposal_text(key, out);
    out.put(b" \xc2\xb7 ");
    match &value.value {
        c::EndedValue::Proposal(history) => {
            out.put(retained_kind(&history.action));
            out.put(b" \xc2\xb7 ");
            out.put(history_state(&history.state));
            out.put(b"\n");
            if let c::RetainedAction::Settings(settings) = &history.action {
                out.put(b"against revision ");
                out.number(u64::from(settings.against));
                if let Some(made) = settings.made {
                    out.put(b", made revision ");
                    out.number(u64::from(made));
                }
                out.put(b"\n");
            }
        }
        c::EndedValue::PersonProposal { proposal, .. } => {
            out.put(b"batch \xc2\xb7 ");
            out.put(person_state(&proposal.state));
            out.put(b"\n");
        }
        c::EndedValue::Task { .. }
        | c::EndedValue::Escalation { .. }
        | c::EndedValue::Question { .. }
        | c::EndedValue::PersonTask { .. } => unreachable!("validated proposal history"),
    }
}
pub(super) fn encode_history(value: &c::Ended, out: &mut Encoder, budget: &mut Cut) {
    let key = super::super::ended_proposal(value).expect("validated proposal history");
    out.object_start();
    out.key(b"key");
    super::encode_proposal_key(&key, out, budget);
    out.key(b"action");
    match &value.value {
        c::EndedValue::Proposal(history) => {
            match &history.action {
                c::RetainedAction::Settings(s) => {
                    encode_settings_fields(s.against, &s.items, None, s.made, None, out);
                }
                other @ (c::RetainedAction::Batch(_)
                | c::RetainedAction::Effect { .. }
                | c::RetainedAction::ProcedureEffect { .. }
                | c::RetainedAction::Amend { .. }
                | c::RetainedAction::Widen { .. }
                | c::RetainedAction::Release { .. }) => out.string(retained_kind(other)),
            }
            out.key(b"state");
            encode_history_state(&history.state, out, budget);
        }
        c::EndedValue::PersonProposal { proposal, .. } => {
            out.string(b"batch");
            out.key(b"state");
            encode_person_state(&proposal.state, out, budget);
        }
        c::EndedValue::Task { .. }
        | c::EndedValue::Escalation { .. }
        | c::EndedValue::Question { .. }
        | c::EndedValue::PersonTask { .. } => unreachable!("validated proposal history"),
    }
    out.object_end();
}
fn encode_person_state(state: &c::PersonProposalState, out: &mut Encoder, budget: &mut Cut) {
    out.object_start();
    out.key(person_state(state));
    out.object_start();
    match state {
        c::PersonProposalState::Pending { since } => {
            out.key(b"since");
            out.unsigned(since.as_nanos());
        }
        c::PersonProposalState::Accepted { by } => {
            out.key(b"by");
            super::encode_owner_requester(by, out, budget);
        }
        c::PersonProposalState::Rejected { by, reason, .. } => {
            out.key(b"by");
            super::encode_owner_requester(by, out, budget);
            out.key(b"reason");
            budget.words(reason.bytes(), out);
        }
    }
    out.object_end();
    out.object_end();
}
fn encode_history_state(state: &c::ProposalHistoryState, out: &mut Encoder, budget: &mut Cut) {
    out.object_start();
    out.key(history_state(state));
    out.object_start();
    match state {
        c::ProposalHistoryState::Pending { holder: held, since } => {
            out.key(b"holder");
            holder(*held, out, budget);
            out.key(b"since");
            out.unsigned(since.as_nanos());
        }
        c::ProposalHistoryState::Accepted { by } => {
            out.key(b"by");
            super::encode_owner_requester(by, out, budget);
        }
        c::ProposalHistoryState::Rejected { by, reason } => {
            out.key(b"by");
            super::encode_owner_requester(by, out, budget);
            out.key(b"reason");
            super::encode_document_text(reason, out, budget);
        }
        c::ProposalHistoryState::Withdrawn => {}
    }
    out.object_end();
    out.object_end();
}

fn current_item<'a>(reading: &'a Reading, item: &c::SettingItem) -> Option<&'a Readout> {
    reading.application.current.get(&crate::jig::runner::application::key(item.setting))
}
fn application_items_text(reading: &Reading, limits: &Limits, out: &mut Text) {
    use c::SettingEdit;
    let Some(items) = &reading.application.items else { return };
    for item in items {
        let setting = item.setting;
        settings::setting_text(setting, reading.parts, out);
        out.put(b" \xc2\xb7 now ");
        value_text(read_value(setting, current_item(reading, item)), setting, limits, reading.parts, out);
        if let Some(origin) = application_origin(reading, setting) {
            out.put(b" (");
            origin_text(origin, out);
            out.put(b")");
        }
        out.put(b" \xc2\xb7 ");
        match &item.edit {
            SettingEdit::Set(value) => {
                out.put(b"set ");
                settings::named_value_inline(setting, &value.value, limits, reading.parts, out);
            }
            SettingEdit::Removed => out.put(b"removed"),
            SettingEdit::Reset => out.put(b"reset to default"),
        }
        out.put(b"\n");
    }
}
fn encode_application_items(reading: &Reading, out: &mut Encoder) {
    use c::SettingEdit;
    out.array_start();
    if let Some(items) = &reading.application.items {
        for item in items {
            let setting = item.setting;
            out.object_start();
            out.key(b"setting");
            settings::encode_setting(setting, out);
            out.key(b"edit");
            out.string(match &item.edit {
                SettingEdit::Set(_) => b"set",
                SettingEdit::Removed => b"removed",
                SettingEdit::Reset => b"reset",
            });
            if let SettingEdit::Set(value) = &item.edit {
                out.key(b"value");
                settings::encode_named_value(setting, &value.value, reading.parts, out);
            }
            if let Some(origin) = application_origin(reading, setting) {
                out.key(b"origin");
                match origin {
                    c::SettingOrigin::Default(r) => origin_revision(b"default", r.revision, out),
                    c::SettingOrigin::Change(r) => origin_revision(b"change", r.revision, out),
                    c::SettingOrigin::NotOffered => out.string(b"not_offered"),
                }
            }
            out.key(b"now");
            encode_value_result(read_value(setting, current_item(reading, item)), setting, reading.parts, out);
            out.object_end();
        }
    }
    out.array_end();
}

fn origin_text(origin: &c::SettingOrigin, out: &mut Text) {
    match origin {
        c::SettingOrigin::Change(r) => {
            out.put(b"revision ");
            out.number(u64::from(r.revision));
        }
        c::SettingOrigin::Default(_) => out.put(b"default"),
        c::SettingOrigin::NotOffered => out.put(b"not offered"),
    }
}
fn application_origin(reading: &Reading, setting: c::Setting) -> Option<&c::SettingOrigin> {
    let (_, review) = crate::jig::runner::proposal::review(reading.pages.get(&1)?)?;
    for item in &review.items {
        if item.setting == setting {
            return Some(&item.origin);
        }
    }
    None
}
