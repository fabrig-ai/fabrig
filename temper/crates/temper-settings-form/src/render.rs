//! Deployment settings and retained revision documents (client/tools.md, 5.2).
use crate::Limits;
use crate::view as c;
use crate::{Text, values};
use skein_json::writer::Encoder;

pub fn setting_text(setting: c::Setting, out: &mut Text) {
    let (kind, number) = match setting {
        c::Setting::Charter(n) => (b"charter".as_slice(), Some(n.number)),
        c::Setting::Template(n) => (b"template".as_slice(), Some(n.number)),
        c::Setting::Policy(n) => (b"policy".as_slice(), Some(n.number)),
        c::Setting::Application { kind: 1 | 2, scope, number } => {
            out.put(b"landing-rule ");
            out.number(u64::from(number));
            if let c::Setting::Application { kind: 2, .. } = setting {
                out.put(b" --project ");
                out.number(u64::from(scope));
            }
            return;
        }
        c::Setting::Application { kind: 3, number, .. } => {
            out.put(b"forge-charter ");
            out.number(u64::from(number));
            return;
        }
        c::Setting::Application { kind, scope, number } => {
            out.put(b"application kind ");
            out.number(u64::from(kind));
            out.put(b", ");
            out.number(u64::from(scope));
            out.put(b", ");
            out.number(u64::from(number));
            return;
        }
        c::Setting::ProjectPolicy { project } => (b"project-policy".as_slice(), Some(project)),
        c::Setting::Rules => (b"rules".as_slice(), None),
    };
    out.put(kind);
    if let Some(number) = number {
        out.put(b" ");
        out.number(u64::from(number));
    }
}
pub fn encode_setting(setting: c::Setting, out: &mut Encoder) {
    let (kind, number) = match setting {
        c::Setting::Charter(n) => (b"charter".as_slice(), Some(n.number)),
        c::Setting::Template(n) => (b"template".as_slice(), Some(n.number)),
        c::Setting::Policy(n) => (b"policy".as_slice(), Some(n.number)),
        c::Setting::Application { kind, scope, number } => {
            out.object_start();
            out.key(b"kind");
            match kind {
                1 | 2 => out.string(b"landing-rule"),
                3 => out.string(b"forge-charter"),
                _ => out.unsigned(u64::from(kind)),
            }
            if kind == 2 {
                out.key(b"project");
                out.unsigned(u64::from(scope));
            } else if kind != 1 && kind != 3 {
                out.key(b"scope");
                out.unsigned(u64::from(scope));
            }
            out.key(b"number");
            out.unsigned(u64::from(number));
            out.object_end();
            return;
        }
        c::Setting::ProjectPolicy { project } => (b"project-policy".as_slice(), Some(project)),
        c::Setting::Rules => (b"rules".as_slice(), None),
    };
    out.object_start();
    out.key(b"kind");
    out.string(kind);
    if let Some(n) = number {
        if let c::Setting::ProjectPolicy { .. } = setting {
            out.key(b"project");
        } else {
            out.key(b"number");
        }
        out.unsigned(u64::from(n));
    }
    out.object_end();
}
fn encode_named_value(setting: c::Setting, value: &c::SettingValue, out: &mut Encoder) {
    let c::Setting::Application { kind, .. } = setting else {
        encode_value(value, out);
        return;
    };
    let c::SettingValue::Application(bytes) = value else {
        encode_value(value, out);
        return;
    };
    if crate::kinds::render(kind, bytes, out) {
        return;
    }
    encode_value(value, out);
}
fn named_value_text(setting: c::Setting, value: &c::SettingValue, limits: &Limits, out: &mut Text) {
    let bounds =
        skein_json::writer::Limits { depth: 32, length: u32::try_from(limits.result_bytes).expect("result bound") };
    let mut measured = Encoder::measure(&bounds);
    encode_named_value(setting, value, &mut measured);
    let mut write = Encoder::write(measured.measured().expect("received value fits"), &bounds);
    encode_named_value(setting, value, &mut write);
    out.put(&write.finish());
    out.put(b"\n");
    template_resources_text(value, out);
}
pub fn encode_value(value: &c::SettingValue, out: &mut Encoder) {
    match value {
        c::SettingValue::Charter(v) => values::encode_charter(v, out),
        c::SettingValue::Template(v) => values::encode_template(v, out),
        c::SettingValue::Policy(v) => values::encode_setting_policy(v, out),
        c::SettingValue::Application(v) => {
            out.array_start();
            for &byte in v {
                out.unsigned(u64::from(byte));
            }
            out.array_end();
        }
        c::SettingValue::Rules(v) => values::encode_rules(v, out),
    }
}
pub fn value_inline(value: &c::SettingValue, limits: &Limits, out: &mut Text) {
    let bounds =
        skein_json::writer::Limits { depth: 32, length: u32::try_from(limits.result_bytes).expect("result bound") };
    let mut measured = Encoder::measure(&bounds);
    encode_value(value, &mut measured);
    // The document entrance has already measured the same bounded value.
    let length = measured.measured().expect("received settings value fits result envelope");
    let mut write = Encoder::write(length, &bounds);
    encode_value(value, &mut write);
    out.put(&write.finish());
}
fn value_text(value: &c::SettingValue, limits: &Limits, out: &mut Text) {
    value_inline(value, limits, out);
    out.put(b"\n");
    template_resources_text(value, out);
}
fn template_resources_text(value: &c::SettingValue, out: &mut Text) {
    if let c::SettingValue::Template(template) = value {
        for resource in &template.resources {
            out.put(b"connector ");
            out.number(u64::from(resource.connector));
            out.put(b" symbol ");
            out.number(u64::from(resource.symbol));
            out.put(b"\n");
        }
    }
}
pub fn proposal_text(proposal: c::ProposalKey, out: &mut Text) {
    let (tag, owner) = match proposal.proposer {
        c::Proposer::Task(n) => (b"t".as_slice(), n),
        c::Proposer::Party(n) => (b"p".as_slice(), n),
    };
    out.put(tag);
    out.number(owner);
    out.put(b"/");
    out.number(proposal.number);
}
fn head_text(head: &c::RevisionHead, out: &mut Text) {
    out.put(b"revision ");
    out.number(u64::from(head.number));
    out.put(b", against revision ");
    out.number(u64::from(head.against));
    out.put(b", at ");
    crate::utc(head.at, out);
    out.put(b" UTC");
    out.put(b", by ");
    match &head.by {
        c::RevisionDecider::FirstStart => out.put(b"the first start"),
        c::RevisionDecider::Release => out.put(b"the release"),
        c::RevisionDecider::Party(p) => {
            out.put(b"party ");
            out.number(p.number);
        }
    }
    if let Some(p) = head.proposal {
        out.put(b", from ");
        proposal_text(p, out);
    }
    out.put(if head.moved { b", running work moves\n" } else { b", running work stays\n" });
}
fn encode_head(head: &c::RevisionHead, out: &mut Encoder) {
    out.object_start();
    out.key(b"number");
    out.unsigned(u64::from(head.number));
    out.key(b"against");
    out.unsigned(u64::from(head.against));
    out.key(b"at");
    out.unsigned(head.at.as_nanos());
    out.key(b"by");
    match &head.by {
        c::RevisionDecider::FirstStart => out.string(b"first_start"),
        c::RevisionDecider::Release => out.string(b"release"),
        c::RevisionDecider::Party(p) => {
            out.object_start();
            out.key(b"party");
            out.unsigned(p.number);
            out.object_end();
        }
    }
    out.key(b"proposal");
    match head.proposal {
        Some(p) => {
            let mut measured = Text::measure();
            proposal_text(p, &mut measured);
            let mut text = Text::writing_exact(measured);
            proposal_text(p, &mut text);
            out.string(&text.finish());
        }
        None => out.null(),
    }
    out.key(b"moved");
    out.boolean(head.moved);
    out.object_end();
}
pub fn text(value: &c::ReadValue, requested: Option<u64>, limits: &Limits, out: &mut Text) {
    match value {
        c::ReadValue::Settings(document) => inventory_text(document, out),
        c::ReadValue::Setting(document) => {
            setting_text(document.setting, out);
            out.put(b", revision ");
            out.number(u64::from(document.revision));
            out.put(b"\n");
            if let Some(value) = &document.value {
                named_value_text(document.setting, value, limits, out);
            }
            if let Some(state) = &document.application {
                application_text(state, out);
            }
            if let Some(edit) = &document.proposed {
                sized_edit_text(edit, out);
            }
        }
        c::ReadValue::Revision(answer) => revision_text(answer, requested.expect("revision call"), limits, out),
        c::ReadValue::Revisions(page) => {
            for row in &page.rows {
                head_text(&row.head, out);
                for item in &row.items {
                    out.put(b"  ");
                    setting_text(item.setting, out);
                    out.put(b": ");
                    out.put(edit_word(&item.edit));
                    out.put(b"\n");
                }
                if row.more != 0 {
                    out.put(b"  ");
                    out.number(u64::from(row.more));
                    out.put(b" more items\n");
                }
            }
            match page.next {
                Some(next) => {
                    out.put(b"read on: revision list --before ");
                    out.number(u64::from(next));
                    out.put(b"\n");
                }
                None => {
                    out.put(if page.forgotten { b"older revisions were forgotten\n" } else { b"no older revisions\n" });
                }
            }
        }
    }
}
#[must_use]
pub fn edit_word(edit: &c::RevisionItemEdit) -> &'static [u8] {
    match edit {
        c::RevisionItemEdit::Set => b"set",
        c::RevisionItemEdit::Removed => b"removed",
        c::RevisionItemEdit::Reset => b"reset",
    }
}
pub fn encode(value: &c::ReadValue, out: &mut Encoder) {
    out.object_start();
    match value {
        c::ReadValue::Settings(document) => encode_inventory(document, out),
        c::ReadValue::Setting(document) => {
            out.key(b"setting");
            encode_setting(document.setting, out);
            out.key(b"revision");
            out.unsigned(u64::from(document.revision));
            out.key(b"value");
            match &document.value {
                Some(value) => encode_named_value(document.setting, value, out),
                None => out.null(),
            }
            if let Some(state) = &document.application {
                out.key(b"application");
                encode_application(state, out);
            }
            if let Some(edit) = &document.proposed {
                out.key(b"proposed");
                encode_sized_edit(edit, out);
            }
        }
        c::ReadValue::Revision(answer) => encode_revision(answer, out),
        c::ReadValue::Revisions(page) => {
            out.key(b"rows");
            out.array_start();
            for row in &page.rows {
                out.object_start();
                out.key(b"head");
                encode_head(&row.head, out);
                out.key(b"items");
                out.array_start();
                for item in &row.items {
                    out.object_start();
                    out.key(b"setting");
                    encode_setting(item.setting, out);
                    out.key(b"edit");
                    out.string(edit_word(&item.edit));
                    out.object_end();
                }
                out.array_end();
                out.key(b"more");
                out.unsigned(u64::from(row.more));
                out.object_end();
            }
            out.array_end();
            out.key(b"next");
            match page.next {
                Some(n) => out.unsigned(u64::from(n)),
                None => out.null(),
            }
            out.key(b"forgotten");
            out.boolean(page.forgotten);
        }
    }
    out.object_end();
}
#[must_use]
pub const fn fault_word(fault: c::SettingFault) -> &'static [u8] {
    match fault {
        c::SettingFault::Shape => b"shape",
        c::SettingFault::Count => b"count",
        c::SettingFault::Reference => b"reference",
        c::SettingFault::Rules => b"rules",
        c::SettingFault::Budget => b"budget",
        c::SettingFault::Grace => b"grace",
        c::SettingFault::Judged => b"refused by its owner",
        c::SettingFault::Owner => b"owner",
    }
}

fn inventory_text(document: &c::SettingsDocument, out: &mut Text) {
    out.put(b"newest revision ");
    out.number(u64::from(document.newest));
    out.put(b"\nsettings:\n");
    for line in &document.settings {
        out.put(b"  ");
        setting_text(line.setting, out);
        if let Some(name) = &line.name {
            out.put(b" \"");
            out.words(name.bytes());
            out.put(b"\"");
        }
        out.put(b": ");
        match line.follows {
            c::SettingFollows::Default => {
                out.put(b"default");
                if let Some(last) = line.last
                    && last != 0
                {
                    out.put(b", last changed in revision ");
                    out.number(u64::from(last));
                }
            }
            c::SettingFollows::Change => {
                out.put(b"changed in revision ");
                if let Some(last) = line.last {
                    out.number(u64::from(last));
                }
            }
        }
        out.put(b"\n");
    }
    out.put(b"held revisions:\n");
    for held in &document.held {
        out.put(b"  ");
        head_text(&held.head, out);
        out.put(b"    live tasks: ");
        out.number(u64::from(held.holders));
        out.put(b"\n");
    }
    out.put(b"pending proposals:\n");
    for p in &document.pending {
        out.put(b"  ");
        proposal_text(c::ProposalKey { proposer: p.proposer, number: u64::from(p.number) }, out);
        out.put(b"\n");
    }
}

fn revision_text(answer: &c::RevisionAnswer, requested: u64, limits: &Limits, out: &mut Text) {
    match answer {
        c::RevisionAnswer::Revision(document) => {
            head_text(&document.head, out);
            for item in &document.items {
                out.put(b"  ");
                setting_text(item.setting, out);
                out.put(b": ");
                match &item.edit {
                    c::RevisionEdit::Set(set) => {
                        out.put(b"set\n");
                        value_text(&set.value, limits, out);
                    }
                    c::RevisionEdit::TooLarge(size) => {
                        revision_size_text(item.setting, document.head.number, size, out);
                    }
                    c::RevisionEdit::Application(edit) => sized_edit_text(edit, out),
                    c::RevisionEdit::Removed => out.put(b"removed\n"),
                    c::RevisionEdit::Reset => out.put(b"reset\n"),
                }
                replaced_text(&item.replaced, limits, out);
            }
        }
        c::RevisionAnswer::Forgotten(f) => {
            out.put(b"revision ");
            out.number(requested);
            out.put(b" is forgotten; the oldest kept is revision ");
            out.number(u64::from(f.oldest));
            out.put(b"\n");
        }
        c::RevisionAnswer::Unknown => out.put(b"unknown revision\n"),
    }
}

fn revision_size_text(setting: c::Setting, revision: u32, size: &c::Size, out: &mut Text) {
    out.put(b"too large, ");
    out.number(size.bytes);
    if let c::Setting::ProjectPolicy { .. } = setting {
        out.put(b" bytes\n");
        return;
    }
    if let c::Setting::Charter(_) = setting {
        out.put(b" bytes; read with settings show ");
    } else {
        out.put(b" bytes; read current value with settings show ");
    }
    setting_text(setting, out);
    if let c::Setting::Charter(_) = setting {
        out.put(b" --revision ");
        out.number(u64::from(revision));
    }
    out.put(b"\n");
}

fn replaced_text(replaced: &c::RevisionReplaced, limits: &Limits, out: &mut Text) {
    out.put(b"    was: ");
    match replaced {
        c::RevisionReplaced::Value(set) => value_text(&set.value, limits, out),
        c::RevisionReplaced::TooLarge(size) => {
            out.put(b"too large, ");
            out.number(size.bytes);
            out.put(b" bytes\n");
        }
        c::RevisionReplaced::Application { size } => {
            out.number(u64::from(*size));
            out.put(b" bytes\n");
        }
        c::RevisionReplaced::Default => out.put(b"default\n"),
        c::RevisionReplaced::Removed => out.put(b"removed\n"),
        c::RevisionReplaced::NotOffered => out.put(b"not offered\n"),
    }
}

fn encode_inventory(document: &c::SettingsDocument, out: &mut Encoder) {
    out.key(b"newest");
    out.unsigned(u64::from(document.newest));
    out.key(b"settings");
    out.array_start();
    for line in &document.settings {
        out.object_start();
        out.key(b"setting");
        encode_setting(line.setting, out);
        out.key(b"name");
        match &line.name {
            Some(name) => values::encode_document_text(name, out),
            None => out.null(),
        }
        out.key(b"follows");
        out.string(match line.follows {
            c::SettingFollows::Default => b"default",
            c::SettingFollows::Change => b"change",
        });
        out.key(b"last");
        match line.last {
            Some(last) => out.unsigned(u64::from(last)),
            None => out.null(),
        }
        out.object_end();
    }
    out.array_end();
    out.key(b"held");
    out.array_start();
    for held in &document.held {
        out.object_start();
        out.key(b"head");
        encode_head(&held.head, out);
        out.key(b"holders");
        out.unsigned(u64::from(held.holders));
        out.object_end();
    }
    out.array_end();
    out.key(b"pending");
    out.array_start();
    for p in &document.pending {
        out.object_start();
        out.key(b"proposal");
        let mut measured = Text::measure();
        proposal_text(c::ProposalKey { proposer: p.proposer, number: u64::from(p.number) }, &mut measured);
        let mut write = Text::writing_exact(measured);
        proposal_text(c::ProposalKey { proposer: p.proposer, number: u64::from(p.number) }, &mut write);
        out.string(&write.finish());
        out.object_end();
    }
    out.array_end();
}

fn encode_revision(answer: &c::RevisionAnswer, out: &mut Encoder) {
    match answer {
        c::RevisionAnswer::Revision(document) => {
            out.key(b"head");
            encode_head(&document.head, out);
            out.key(b"items");
            out.array_start();
            for item in &document.items {
                out.object_start();
                out.key(b"setting");
                encode_setting(item.setting, out);
                match &item.edit {
                    c::RevisionEdit::Set(set) => {
                        out.key(b"set");
                        encode_value(&set.value, out);
                    }
                    c::RevisionEdit::TooLarge(size) => {
                        out.key(b"too_large");
                        out.unsigned(size.bytes);
                    }
                    c::RevisionEdit::Application(edit) => {
                        out.key(b"application");
                        encode_sized_edit(edit, out);
                    }
                    c::RevisionEdit::Removed => {
                        out.key(b"edit");
                        out.string(b"removed");
                    }
                    c::RevisionEdit::Reset => {
                        out.key(b"edit");
                        out.string(b"reset");
                    }
                }
                out.key(b"replaced");
                encode_replaced(&item.replaced, out);
                out.object_end();
            }
            out.array_end();
        }
        c::RevisionAnswer::Forgotten(f) => {
            out.key(b"forgotten");
            out.object_start();
            out.key(b"oldest");
            out.unsigned(u64::from(f.oldest));
            out.object_end();
        }
        c::RevisionAnswer::Unknown => {
            out.key(b"unknown");
            out.boolean(true);
        }
    }
}

fn encode_replaced(replaced: &c::RevisionReplaced, out: &mut Encoder) {
    match replaced {
        c::RevisionReplaced::Value(set) => {
            out.object_start();
            out.key(b"set");
            encode_value(&set.value, out);
            out.object_end();
        }
        c::RevisionReplaced::TooLarge(size) => {
            out.object_start();
            out.key(b"too_large");
            out.unsigned(size.bytes);
            out.object_end();
        }
        c::RevisionReplaced::Application { size } => {
            out.object_start();
            out.key(b"application");
            out.unsigned(u64::from(*size));
            out.object_end();
        }
        c::RevisionReplaced::Default => out.string(b"default"),
        c::RevisionReplaced::Removed => out.string(b"removed"),
        c::RevisionReplaced::NotOffered => out.string(b"not_offered"),
    }
}

fn sized_edit_text(edit: &c::SizedEdit, out: &mut Text) {
    match edit {
        c::SizedEdit::Set { size } => {
            out.put(b"set, ");
            out.number(u64::from(*size));
            out.put(b" bytes\n");
        }
        c::SizedEdit::Remove => out.put(b"removed\n"),
        c::SizedEdit::Reset => out.put(b"reset to default\n"),
    }
}
fn application_text(state: &c::ApplicationState, out: &mut Text) {
    match state.follows {
        c::ApplicationFollows::Default => out.put(b"default"),
        c::ApplicationFollows::Changed { revision } => {
            out.put(b"changed at revision ");
            out.number(u64::from(revision));
        }
        c::ApplicationFollows::Removed { revision } => {
            out.put(b"removed at revision ");
            out.number(u64::from(revision));
        }
    }
    out.put(b", ");
    out.number(u64::from(state.size));
    out.put(b" bytes\n");
}
fn encode_sized_edit(edit: &c::SizedEdit, out: &mut Encoder) {
    match edit {
        c::SizedEdit::Set { size } => {
            out.object_start();
            out.key(b"set");
            out.unsigned(u64::from(*size));
            out.object_end();
        }
        c::SizedEdit::Remove => out.string(b"removed"),
        c::SizedEdit::Reset => out.string(b"reset"),
    }
}
fn encode_application(state: &c::ApplicationState, out: &mut Encoder) {
    out.object_start();
    out.key(b"follows");
    match state.follows {
        c::ApplicationFollows::Default => out.string(b"default"),
        c::ApplicationFollows::Changed { revision } => {
            out.string(b"change");
            out.key(b"revision");
            out.unsigned(u64::from(revision));
        }
        c::ApplicationFollows::Removed { revision } => {
            out.string(b"removed");
            out.key(b"revision");
            out.unsigned(u64::from(revision));
        }
    }
    out.key(b"size");
    out.unsigned(u64::from(state.size));
    out.key(b"charters");
    out.array_start();
    for &charter in &state.charters {
        out.unsigned(u64::from(charter));
    }
    out.array_end();
    out.object_end();
}
