//! Deployment fixtures shared by the test binaries (tools/crates/design.md, 3.2).
use super::{Options, Plan};
use std::ffi::OsStr;
use temper_config_shell::{File, Shape};

pub(crate) const SESSION_PID: u32 = 42;

pub(super) fn plan() -> Plan {
    Plan::new(
        Options {
            bin: "/build/bin".into(),
            bundle: Some("/build/bundle".into()),
            deployment: None,
            scratch: "/scratch".into(),
            scripts: vec!["/build/one.script".into(), "/build/two.script".into()],
            strict: true,
            ctl: false,
            journey: false,
            widths: vec![],
            ports: None,
            files: None,
        },
        "/browser/chromium".into(),
        OsStr::new("/bin"),
        SESSION_PID,
    )
}

#[must_use]
pub fn deployment_plan() -> Plan {
    let mut plan = plan();
    plan.options.deployment = Some("/recipe/config".into());
    plan.options.bundle = None;
    plan.options.ctl = true;
    plan.options.ports = Some("4101,4102,4103,4104,4105,4106,4107,4108,4109".into());
    plan.options.files = Some("/recipe/files".into());
    plan
}

pub fn configuration(name: &str, bytes: &[u8], shape: Shape) -> Result<File, String> {
    super::deployment::configuration(name, bytes, shape)
}

pub mod io {
    pub mod deployment {
        use crate::{
            drive::{Plan, deployment::Values, io::deployment as files},
            line::Document,
        };
        use std::path::{Path, PathBuf};

        pub fn prepare_with(plan: &Plan, random: Option<[u8; 32]>) -> Result<Values, String> {
            files::prepare_with(plan, random)
        }

        pub fn patch(plan: &Plan, document: Document, name: &[u8]) -> Result<(PathBuf, Box<[u8]>), String> {
            files::patch(plan, document, name)
        }

        pub fn replace(path: &Path, bytes: &[u8]) -> Result<(), String> {
            files::replace(path, bytes)
        }

        pub fn directory(path: &Path) -> Result<(), String> {
            files::directory(path)
        }

        pub fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
            files::write(path, bytes)
        }

        pub fn remove_link(scratch: &Path) -> Result<(), String> {
            files::remove_link(scratch)
        }
    }
}
