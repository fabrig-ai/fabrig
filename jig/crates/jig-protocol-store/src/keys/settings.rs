//! The core's live settings (tag 8) and revision-ordered history (tag 9).
use jig_domain_core_vocabulary::settings::{Key, Setting};
use jig_domain_people::Proposer;
use skein_kv::{KeyReader, KeyWriter};
pub(super) fn write(writer: &mut KeyWriter, key: &Key) {
    match key {
        Key::History(revision) => {
            writer.tag(9);
            writer.u64(u64::from(*revision));
        }
        Key::Default(setting) => {
            writer.tag(8);
            writer.tag(0);
            setting_write(writer, *setting);
        }
        Key::Changed(setting) => {
            writer.tag(8);
            writer.tag(1);
            setting_write(writer, *setting);
        }
        Key::Held(revision) => {
            writer.tag(8);
            writer.tag(2);
            writer.u64(u64::from(*revision));
        }
        Key::Charter { charter, revision } => {
            writer.tag(8);
            writer.tag(3);
            writer.u64(u64::from(*charter));
            writer.u64(u64::from(*revision));
        }
        Key::Proposal { proposer, proposal } => {
            writer.tag(8);
            writer.tag(4);
            match proposer {
                Proposer::Task(number) => {
                    writer.tag(0);
                    writer.u64(*number);
                }
                Proposer::Party(number) => {
                    writer.tag(1);
                    writer.u64(*number);
                }
            }
            writer.u64(u64::from(*proposal));
        }
        Key::Extent => {
            writer.tag(8);
            writer.tag(5);
        }
    }
}
fn setting_write(writer: &mut KeyWriter, setting: Setting) {
    let (kind, number) = match setting {
        Setting::ProjectPolicy { project } => {
            writer.tag(5);
            writer.u64(u64::from(project));
            return;
        }
        Setting::Charter(n) => (0, n),
        Setting::Template(n) => (1, n),
        Setting::Policy(n) => (2, n),
        Setting::Rules => (3, 0),
        Setting::Model(n) => (6, n),
        Setting::Application { kind, scope, number } => {
            writer.tag(4);
            writer.u64(u64::from(kind));
            writer.u64(u64::from(scope));
            writer.u64(u64::from(number));
            return;
        }
    };
    writer.tag(kind);
    writer.u64(0);
    writer.u64(u64::from(number));
}
pub(super) fn read(reader: &mut KeyReader<'_>, history: bool) -> Option<Key> {
    if history {
        return Some(Key::History(u32::try_from(reader.u64()?).ok()?));
    }
    Some(match reader.tag()? {
        0 => Key::Default(setting_read(reader)?),
        1 => Key::Changed(setting_read(reader)?),
        2 => Key::Held(u32::try_from(reader.u64()?).ok()?),
        3 => Key::Charter { charter: u32::try_from(reader.u64()?).ok()?, revision: u32::try_from(reader.u64()?).ok()? },
        4 => {
            let proposer = match reader.tag()? {
                0 => Proposer::Task(reader.u64()?),
                1 => Proposer::Party(reader.u64()?),
                _ => return None,
            };
            Key::Proposal { proposer, proposal: u32::try_from(reader.u64()?).ok()? }
        }
        5 => Key::Extent,
        _ => return None,
    })
}
fn setting_read(reader: &mut KeyReader<'_>) -> Option<Setting> {
    let kind = reader.tag()?;
    if kind == 5 {
        return Some(Setting::ProjectPolicy { project: u32::try_from(reader.u64()?).ok()? });
    }
    if kind == 4 {
        return Some(Setting::Application {
            kind: u32::try_from(reader.u64()?).ok()?,
            scope: u32::try_from(reader.u64()?).ok()?,
            number: u32::try_from(reader.u64()?).ok()?,
        });
    }
    if reader.u64()? != 0 {
        return None;
    }
    let number = u32::try_from(reader.u64()?).ok()?;
    Some(match kind {
        0 => Setting::Charter(number),
        1 => Setting::Template(number),
        2 => Setting::Policy(number),
        6 => Setting::Model(number),
        3 if number == 0 => Setting::Rules,
        _ => return None,
    })
}

/// Only model changes enter the open's catalogue overlay (protocol/store/records.md).
pub(crate) fn model_changes() -> skein_kv::Range {
    skein_kv::Range::prefix(&[super::JIG, 0, 8, 1, 6])
}

pub(crate) fn changed_model(bytes: &[u8]) -> Option<u32> {
    let suffix = bytes.strip_prefix(model_changes().start.as_ref())?;
    let mut reader = KeyReader::new(suffix);
    if reader.u64()? != 0 {
        return None;
    }
    let number = u32::try_from(reader.u64()?).ok()?;
    if !reader.is_empty() || number == 0 || number > jig_domain_tasks::MODELS {
        return None;
    }
    Some(number)
}
