pub use hearth_domain_render_vocabulary::parts::config::{Action, Line, LineKind, Preset, Script, Tmpfile};

pub(super) fn encode(config: &super::Config, record: &mut hearth_lib::encode::Encoder) {
    record.key(b"presets");
    record.map(u64::try_from(config.presets.len()).expect("entry bound"));
    for preset in &config.presets {
        record.key(&preset.unit);
        record.bytes(preset.action.name());
    }
    encode_scripts(&config.scripts, record);
    encode_tmpfiles(&config.tmpfiles, record);
}
pub(crate) fn encode_scripts(scripts: &[Script], record: &mut hearth_lib::encode::Encoder) {
    record.key(b"scripts");
    record.list(u64::try_from(scripts.len()).expect("task bound"));
    for script in scripts {
        record.map(3);
        record.key(b"content_digest");
        record.digest(&script.content_digest);
        record.key(b"order");
        record.uint(u64::from(script.order));
        record.key(b"task");
        record.bytes(&script.task);
    }
}
fn encode_tmpfiles(tmpfiles: &[Tmpfile], record: &mut hearth_lib::encode::Encoder) {
    record.key(b"tmpfiles");
    record.map(u64::try_from(tmpfiles.len()).expect("entry bound"));
    for tmpfile in tmpfiles {
        record.key(&tmpfile.path);
        let line = &tmpfile.line;
        let mut fields = 1_u64;
        for present in [
            line.mode.is_some(),
            line.owner.is_some(),
            line.group.is_some(),
            line.age.is_some(),
            line.argument.is_some(),
        ] {
            fields = fields.saturating_add(u64::from(present));
        }
        record.map(fields);
        optional(record, b"age", line.age.as_deref());
        optional(record, b"argument", line.argument.as_deref());
        optional(record, b"group", line.group.as_deref());
        if let Some(mode) = line.mode {
            record.key(b"mode");
            record.uint(u64::from(mode));
        }
        optional(record, b"owner", line.owner.as_deref());
        record.key(b"type");
        record.bytes(line.kind.name());
    }
}
fn optional(record: &mut hearth_lib::encode::Encoder, key: &[u8], value: Option<&[u8]>) {
    if let Some(value) = value {
        record.key(key);
        record.bytes(value);
    }
}
