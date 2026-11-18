//! The setup owner checks its document before commands read any files.
use jig_protocol_config as config;
use temper_engine_service::startup::Named;
use temper_forge_setup::{check::Checked, problem};

#[derive(Debug)]
pub struct Deployment {
    pub named: Named,
    pub checked: Checked,
    pub limits: temper_box_machine::limits::Limits,
    pub demo: Option<temper_fake::demo::Demo>,
}

pub fn check_with(
    bytes: &[u8],
    directories: &config::Directories,
    given: &temper_engine_service::stock::Given,
) -> Result<Deployment, String> {
    let named = temper_engine_service::startup::named_with(bytes, directories, given)
        .map_err(|why| String::from_utf8_lossy(&temper_engine_service::startup::lines(&why)).into_owned())?;
    let setup_overrides: Vec<_> =
        named.overrides.iter().filter(|(name, _)| name.starts_with(b"setup.")).cloned().collect();
    let (setup_limits, fetch_bytes) = temper_forge_setup::check::limits(&setup_overrides)
        .map_err(|why| String::from_utf8_lossy(&problem::lines(&why)).into_owned())?;
    let checked =
        Checked { limits: setup_limits, fetch_bytes, setup: temper_forge_setup::Setup { entries: Box::new([]) } };
    let demo = None;
    let limits = box_limits(&named.overrides)?;
    Ok(Deployment { named, checked, limits, demo })
}
pub fn check(bytes: &[u8], directories: &config::Directories) -> Result<Deployment, String> {
    check_with(bytes, directories, &temper_engine_service::stock::Given::Stock)
}

fn box_limits(overrides: &[(Box<[u8]>, u64)]) -> Result<temper_box_machine::limits::Limits, String> {
    use temper_box_machine::limits::{self, Refused};
    let mut limits = limits::stock();
    let mut problems = Vec::new();
    let prefix = "application.limits.overrides.";
    for (name, value) in overrides.iter().filter(|(name, _)| name.starts_with(b"box.")) {
        if let Err(why) = limits::apply(&mut limits, name, *value) {
            let reason = match why {
                Refused::Unknown => "no limit of temper's has this name",
                Refused::Range => "outside its type",
            };
            problems.push(format!("{prefix}{}: {reason}", String::from_utf8_lossy(name)));
        }
    }
    if let Err(why) = limits::check(&limits) {
        let names = why.names();
        let name = String::from_utf8_lossy(names[0]);
        let reason = match names.get(1) {
            Some(other) => format!("must agree with {}", String::from_utf8_lossy(other)),
            None => "past its bound".into(),
        };
        problems.push(format!("{prefix}{name}: {reason}"));
    }
    if problems.is_empty() { Ok(limits) } else { Err(problems.join("\n")) }
}
