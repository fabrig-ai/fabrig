//! Typed changes and their forms (hearth/docs/design/domain/render/README.md, 4.1 and 4.3).
use crate::form::{
    json::{digest, field},
    text::Lines,
};
use hearth_domain_render_vocabulary::{
    diff::{Change, Hunks, LineKind, Needs, PartChange, Rebuild, Side, Tally, TargetChange},
    parts::config::{Change as ConfigChange, EntryChange, Node},
};
use skein_json::writer::Encoder;

pub(crate) fn text(this: &Change, lines: &mut Lines) {
    if this.change == TargetChange::Same {
        return;
    }
    lines.put(&this.target);
    lines.pad_to(52.max(this.target.len().saturating_add(2)));
    match &this.change {
        TargetChange::Same => {}
        TargetChange::Removed => lines.put(b"removed"),
        TargetChange::Added { spec_digest } => {
            lines.put(b"added spec ");
            lines.hex12(&spec_digest.0);
        }
        TargetChange::Failed { side } => {
            lines.put(b"failed (");
            lines.put(self::side(*side));
            lines.put(b")");
        }
        TargetChange::Changed { rebuild, .. } => lines.put(rebuild_text(*rebuild)),
    }
    lines.put(b"\n");
    match &this.change {
        TargetChange::Changed { parts, .. } => {
            for part in &**parts {
                match part {
                    PartChange::Config(config) => config_text(config, lines),
                    PartChange::Meta(meta) => crate::form::meta::text(meta, lines),
                    PartChange::Packages(packages) => crate::parts::packages::change::text(packages, lines),
                    PartChange::Boot(boot) => crate::parts::boot::change_text(boot, lines),
                    PartChange::Exports(exports) => crate::parts::exports::change_text(exports, lines),
                    PartChange::Secrets(value) => crate::parts::secrets::change_text(value, lines),
                    PartChange::Software(value) => crate::parts::software::change_text(value, lines),
                    PartChange::Live(value) => crate::parts::live::change_text(value, lines),
                    PartChange::Data(data) => data::text(data, lines),
                }
            }
        }
        TargetChange::Same | TargetChange::Removed | TargetChange::Added { .. } | TargetChange::Failed { .. } => {}
    }
}

pub(crate) fn json(this: &Change, json: &mut Encoder) {
    field(json, b"record", b"change");
    field(json, b"target", &this.target);
    match &this.change {
        TargetChange::Added { spec_digest } => {
            field(json, b"kind", b"added");
            json.key(b"added");
            json.object_start();
            json.key(b"spec_digest");
            digest(json, &spec_digest.0);
            json.object_end();
        }
        TargetChange::Removed => field(json, b"kind", b"removed"),
        TargetChange::Same => field(json, b"kind", b"same"),
        TargetChange::Failed { side } => {
            field(json, b"kind", b"failed");
            json.key(b"failed");
            json.object_start();
            field(json, b"side", self::side(*side));
            json.object_end();
        }
        TargetChange::Changed { parts, rebuild } => {
            field(json, b"kind", b"changed");
            json.key(b"changed");
            json.object_start();
            json.key(b"parts");
            json.array_start();
            for part in &**parts {
                match part {
                    PartChange::Config(config) => config_json(config, json),
                    PartChange::Meta(meta) => crate::form::meta::json(meta, json),
                    PartChange::Packages(packages) => crate::parts::packages::change::json(packages, json),
                    PartChange::Boot(boot) => crate::parts::boot::change_json(boot, json),
                    PartChange::Exports(exports) => crate::parts::exports::change_json(exports, json),
                    PartChange::Secrets(value) => crate::parts::secrets::change_json(value, json),
                    PartChange::Software(value) => crate::parts::software::change_json(value, json),
                    PartChange::Live(value) => crate::parts::live::change_json(value, json),
                    PartChange::Data(data) => data::json(data, json),
                }
            }
            json.array_end();
            json.key(b"rebuild");
            rebuild_json(*rebuild, json);
            json.object_end();
        }
    }
}
pub(crate) fn rebuild_text(this: Rebuild) -> &'static [u8] {
    match this {
        Rebuild::FromPackages { resolution: Needs::Known } => b"rebuild from packages",
        Rebuild::FromPackages { resolution: Needs::Needed } => b"rebuild from packages, resolution needed",
        Rebuild::FromSoftware => b"rebuild from software",
        Rebuild::FromConfig => b"rebuild from config",
        Rebuild::SlotOnly => b"new slot only",
        Rebuild::Nothing => b"nothing to build",
    }
}

pub(crate) fn rebuild_json(this: Rebuild, json: &mut Encoder) {
    json.object_start();
    field(
        json,
        b"kind",
        match this {
            Rebuild::FromPackages { .. } => b"from_packages",
            Rebuild::FromSoftware => b"from_software",
            Rebuild::FromConfig => b"from_config",
            Rebuild::SlotOnly => b"slot_only",
            Rebuild::Nothing => b"nothing",
        },
    );
    match this {
        Rebuild::FromPackages { resolution } => field(
            json,
            b"resolution",
            match resolution {
                Needs::Known => b"known",
                Needs::Needed => b"needed",
            },
        ),
        Rebuild::FromSoftware | Rebuild::FromConfig | Rebuild::SlotOnly | Rebuild::Nothing => {}
    }
    json.object_end();
}
pub(crate) fn config_text(this: &ConfigChange, lines: &mut Lines) {
    accounts::accounts_text(this, lines);
    directives::directives_text(this, lines);
    let mut column = 52;
    for entry in &*this.tree {
        column = column.max(entry_path(entry).len().saturating_add(15));
    }
    for entry in &*this.tree {
        lines.put(b"  config   ");
        match entry {
            EntryChange::Added { .. } => lines.put(b"+ "),
            EntryChange::Removed { .. } => lines.put(b"- "),
            EntryChange::Changed { .. } => lines.put(b"~ "),
        }
        lines.put(entry_path(entry));
        lines.pad_to(column);
        let node = match entry {
            EntryChange::Added { node, .. } | EntryChange::Removed { node, .. } => node,
            EntryChange::Changed { after, .. } => after,
        };
        match node {
            Node::Link { target, owner, group } => {
                lines.put(b"link ");
                lines.put(owner);
                lines.put(b":");
                lines.put(group);
                lines.put(b" ");
                lines.put(target);
            }
            Node::Directory { mode, owner, group } => {
                lines.put(b"directory ");
                lines.mode(*mode);
                lines.put(b" ");
                lines.put(owner);
                lines.put(b":");
                lines.put(group);
            }
            Node::File { mode, owner, group, .. } => {
                lines.mode(*mode);
                lines.put(b" ");
                lines.put(owner);
                lines.put(b":");
                lines.put(group);
            }
        }
        lines.put(b"\n");
        match entry {
            EntryChange::Changed { hunks: Some(hunks), .. } => hunks_text(hunks, lines),
            EntryChange::Changed { hunks: None, .. } | EntryChange::Added { .. } | EntryChange::Removed { .. } => {}
        }
    }
}

pub(crate) fn config_json(this: &ConfigChange, json: &mut Encoder) {
    json.object_start();
    field(json, b"kind", b"config");
    accounts::accounts_json(this, json);
    directives::directives_json(this, json);
    json.key(b"tree");
    json.array_start();
    for entry in &*this.tree {
        entry_json(entry, json);
    }
    json.array_end();
    json.object_end();
}
pub(crate) fn entry_path(this: &EntryChange) -> &[u8] {
    match this {
        EntryChange::Added { path, .. } | EntryChange::Removed { path, .. } | EntryChange::Changed { path, .. } => path,
    }
}

pub(crate) fn entry_json(this: &EntryChange, json: &mut Encoder) {
    json.object_start();
    match this {
        EntryChange::Added { node, .. } | EntryChange::Removed { node, .. } => {
            field(
                json,
                b"kind",
                match this {
                    EntryChange::Added { .. } => b"added",
                    EntryChange::Removed { .. } => b"removed",
                    EntryChange::Changed { .. } => unreachable!("added or removed arm"),
                },
            );
            json.key(b"node");
            crate::parts::config::node_json(node, json);
            field(json, b"path", entry_path(this));
        }
        EntryChange::Changed { before, after, sizes, hunks, .. } => {
            json.key(b"after");
            crate::parts::config::node_json(after, json);
            json.key(b"before");
            crate::parts::config::node_json(before, json);
            if let Some(hunks) = hunks {
                json.key(b"hunks");
                hunks_json(hunks, json);
            }
            field(json, b"kind", b"changed");
            field(json, b"path", entry_path(this));
            if let Some(sizes) = sizes {
                json.key(b"sizes");
                json.object_start();
                json.key(b"after");
                json.unsigned(sizes.after);
                json.key(b"before");
                json.unsigned(sizes.before);
                json.object_end();
            }
        }
    }
    json.object_end();
}
pub(crate) fn hunks_text(this: &Hunks, lines: &mut Lines) {
    if this.binary {
        return;
    }
    for line in &*this.lines {
        lines.put(b"               ");
        lines.put(match line.kind {
            LineKind::Hunk => b"",
            LineKind::NoNewline => b"\\",
            LineKind::Context => b" ",
            LineKind::Removed => b"-",
            LineKind::Added => b"+",
        });
        lines.put(&line.text);
        lines.put(b"\n");
    }
    if this.left_out != 0 {
        lines.put(b"               ... ");
        lines.decimal(u64::from(this.left_out));
        lines.put(b" lines left out\n");
    }
}

pub(crate) fn hunks_json(this: &Hunks, json: &mut Encoder) {
    json.object_start();
    json.key(b"binary");
    json.boolean(this.binary);
    json.key(b"left_out");
    json.unsigned(u64::from(this.left_out));
    json.key(b"lines");
    json.array_start();
    for line in &*this.lines {
        json.object_start();
        field(
            json,
            b"kind",
            match line.kind {
                LineKind::Hunk => b"header",
                LineKind::NoNewline => b"no_newline",
                LineKind::Context => b"context",
                LineKind::Removed => b"removed",
                LineKind::Added => b"added",
            },
        );
        field(json, b"text", &line.text);
        json.object_end();
    }
    json.array_end();
    json.object_end();
}
pub(crate) fn tally_text(this: &Tally, lines: &mut Lines) {
    lines.put(b"\n");
    lines.decimal(u64::from(this.changed));
    lines.put(b" of ");
    lines.decimal(u64::from(this.targets));
    lines.put(if this.targets == 1 { b" target " } else { b" targets " });
    lines.put(if this.changed == 1 { b"changes; " } else { b"change; " });
    lines.decimal(u64::from(this.unchanged));
    lines.put(if this.unchanged == 1 { b" is unchanged.\n" } else { b" are unchanged.\n" });
}

pub(crate) fn tally_json(this: &Tally, json: &mut Encoder) {
    field(json, b"from", &this.from.0);
    field(json, b"to", &this.to.0);
    json.key(b"targets");
    json.unsigned(u64::from(this.targets));
    json.key(b"changed");
    json.unsigned(u64::from(this.changed));
    json.key(b"unchanged");
    json.unsigned(u64::from(this.unchanged));
}

mod accounts;

mod data;
pub(crate) mod directives;

pub(crate) mod placed;

/// The client's side names (hearth/docs/design/domain/render/contracts.md, 2).
#[must_use]
pub fn side(side: Side) -> &'static [u8] {
    match side {
        Side::From => b"from",
        Side::To => b"to",
        Side::Both => b"both",
    }
}
