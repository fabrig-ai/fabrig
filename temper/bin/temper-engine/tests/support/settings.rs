//! Stored settings fixtures for upgrade checks.
/// Stored pre-range changes bypass a set's admission check; the moved pair leaves removals behind.
pub fn seed_numbered_settings(directories: &temper_shell::directories::Directories, moved: bool) {
    use jig_domain_core::{CoreRecord, Record as Core, settings as s};
    use jig_protocol_store::Write;
    use std::{os::unix::ffi::OsStrExt, path::Path};
    let data = Path::new(std::ffi::OsStr::from_bytes(&directories.data));
    if !moved {
        // The floor needs upgrade jobs; seed a current-format stopped store instead.
        let state = data.join("state");
        std::fs::remove_dir_all(&state).expect("discard copied floor");
        std::fs::create_dir(&state).expect("new stopped deployment");
        std::fs::set_permissions(&state, std::os::unix::fs::PermissionsExt::from_mode(0o700))
            .expect("private deployment");
    }
    let bytes = temper_config_shell::read(directories, temper_config_shell::Shape::EngineAlone)
        .expect("engine configuration")
        .document;
    let stock =
        temper_engine_shell::serve::settings(&bytes, directories, &temper_engine_service_stock::stock::Given::Stock);
    assert!(!stock.templates.iter().any(|item| item.number == 9));
    assert!(!stock.charters.iter().any(|item| item.number == 17));
    let mut template = stock.templates.first().expect("stock template").clone();
    let mut charter = stock.charters.first().expect("stock charter").clone();
    template.number = if moved { 33 } else { 9 };
    charter.number = if moved { 34 } else { 17 };
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
        let core = jig_protocol_config::core(&stock, [1; 16], 0);
        let current = s::Current::new(core.defaults, &stock.limits.core, &[]);
        for (setting, default) in &current.defaults {
            writes.push(Write::Save(Core::Core(CoreRecord::Settings(s::Record::Default {
                setting: *setting,
                default: default.clone(),
            }))));
        }
    }
    for (setting, value) in items {
        writes.push(Write::Save(Core::Core(CoreRecord::Settings(s::Record::Changed {
            setting,
            changed: s::Changed { value, revision },
        }))));
    }
    // A persisted head makes this a restored settings set, not a first start.
    writes.push(Write::Save(Core::Core(CoreRecord::Settings(s::Record::Revision {
        history: false,
        revision: s::Revision {
            head: s::Head {
                number: revision,
                against: revision - 1,
                at: skein_lib::Wall::EPOCH,
                by: if moved { s::Decider::Release } else { s::Decider::FirstStart },
                proposal: None,
                moved: false,
            },
            items: Box::new([]),
            applications: Box::new([]),
        },
    }))));
    let mut own = temper_engine_service_vocabulary::limits::stock();
    temper_engine_service_vocabulary::limits::derive(&mut own, temper_service_core::derived::STOCK)
        .expect("stock limits");
    temper_github_app::seed_configured(&data.join("state"), writes, &stock.limits, &own)
        .expect("durable settings changes");
}
