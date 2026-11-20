//! Stored settings fixtures for upgrade checks.
use jig_domain_core::{CoreRecord, Record as Core, settings as s};
use jig_protocol_store::Write;
use std::{os::unix::ffi::OsStrExt, path::Path};
use temper_shell::directories::Directories;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Numbers {
    Kept,
    Moved,
    Past,
    Taken,
}

/// Stored changes bypass admission; a taken case omits the changed charter's stored default.
pub fn seed_numbered_settings(directories: &Directories, numbers: Numbers) {
    let moved = numbers == Numbers::Moved;
    let data = Path::new(std::ffi::OsStr::from_bytes(&directories.data));
    if !moved {
        fresh_state(data);
    }
    let stock = stock(directories);
    assert!(!stock.templates.iter().any(|item| item.number == 9));
    assert!(!stock.charters.iter().any(|item| item.number == 17));
    let mut template = stock.templates.first().expect("stock template").clone();
    let mut charter = stock.charters.first().expect("stock charter").clone();
    template.number = if moved { 33 } else { 9 };
    charter.number = match numbers {
        Numbers::Kept => 17,
        Numbers::Moved => 34,
        Numbers::Past => 65,
        Numbers::Taken => charter.number,
    };
    let taken = (numbers == Numbers::Taken).then_some(s::Setting::Charter(charter.number));
    let revision = if moved { 2 } else { 1 };
    let mut items = vec![
        (s::Setting::Template(template.number), Some(s::Value::Template(Box::new(template)))),
        (s::Setting::Charter(charter.number), Some(s::Value::Charter(Box::new(charter)))),
    ];
    if moved {
        items.extend([(s::Setting::Template(9), None), (s::Setting::Charter(17), None)]);
    }
    let mut writes = Vec::new();
    if !moved {
        writes = defaults(&stock, taken);
    }
    for (setting, value) in items {
        writes.push(Write::Save(Core::Core(CoreRecord::Settings(s::Record::Changed {
            setting,
            changed: s::Changed { value, revision },
        }))));
    }
    writes.push(head(revision, if moved { s::Decider::Release } else { s::Decider::FirstStart }));
    seed(data, writes, &stock);
}

/// A copy whose own `limits` setting raises `core.settings.versions` to `raised`, past the stock's
/// figure, holding one charter version more than the stock allows. Returns the versions held.
pub fn seed_raised_versions(directories: &Directories, raised: u64) -> u32 {
    let data = Path::new(std::ffi::OsStr::from_bytes(&directories.data));
    fresh_state(data);
    let stock = stock(directories);
    let held = stock.limits.core.settings.versions.checked_add(1).expect("one version past the stock");
    assert!(u64::from(held) <= raised, "the raised figure admits what the copy holds");
    let charter = stock.charters.first().expect("stock charter").number;
    let revision = held.checked_add(1).expect("the newest revision follows the held versions");
    let mut writes = defaults(&stock, None);
    let raise = s::Override {
        name: b"core.settings.versions".as_slice().into(),
        value: raised,
        reason: b"the copy holds more versions than the stock".as_slice().into(),
    };
    writes.push(Write::Save(Core::Core(CoreRecord::Settings(s::Record::Changed {
        setting: s::Setting::Limits,
        changed: s::Changed { value: Some(s::Value::Limits(Box::new([raise]))), revision },
    }))));
    for version in 1..=held {
        writes.push(Write::Save(Core::Core(CoreRecord::Settings(s::Record::Charter {
            charter,
            revision: version,
            value: s::CharterVersion { overlay: None, charter: None },
        }))));
    }
    writes.push(head(revision, s::Decider::FirstStart));
    seed(data, writes, &stock);
    held
}

/// The floor needs upgrade jobs; seed a current-format stopped store instead.
fn fresh_state(data: &Path) {
    let state = data.join("state");
    std::fs::remove_dir_all(&state).expect("discard copied floor");
    std::fs::create_dir(&state).expect("new stopped deployment");
    std::fs::set_permissions(&state, std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("private deployment");
}

fn stock(directories: &Directories) -> jig_protocol_config::Engine {
    let bytes = temper_config_shell::read(directories, temper_config_shell::Shape::EngineAlone)
        .expect("engine configuration")
        .document;
    temper_engine_shell::serve::settings(&bytes, directories, &temper_engine_service_stock::stock::Given::Stock)
}

/// The stock's stored defaults, but for `taken`'s.
fn defaults(stock: &jig_protocol_config::Engine, taken: Option<s::Setting>) -> Vec<Write> {
    let core = jig_protocol_config::core(stock, [1; 16], 0);
    let current = s::Current::new(core.defaults, &stock.limits.core, &[]);
    let mut writes = Vec::new();
    for (setting, default) in &current.defaults {
        if Some(*setting) == taken {
            continue;
        }
        writes.push(Write::Save(Core::Core(CoreRecord::Settings(s::Record::Default {
            setting: *setting,
            default: default.clone(),
        }))));
    }
    writes
}

/// A persisted head makes this a restored settings set, not a first start.
fn head(number: u32, by: s::Decider) -> Write {
    Write::Save(Core::Core(CoreRecord::Settings(s::Record::Revision {
        history: false,
        revision: s::Revision {
            head: s::Head { number, against: number - 1, at: skein_lib::Wall::EPOCH, by, proposal: None, moved: false },
            items: Box::new([]),
            applications: Box::new([]),
        },
    })))
}

/// Stored changes bypass admission.
fn seed(data: &Path, writes: Vec<Write>, stock: &jig_protocol_config::Engine) {
    let mut own = temper_engine_service_vocabulary::limits::stock();
    temper_engine_service_vocabulary::limits::derive(&mut own, temper_service_core::derived::STOCK)
        .expect("stock limits");
    temper_github_app::seed_configured(&data.join("state"), writes, &stock.limits, &own)
        .expect("durable settings changes");
}
