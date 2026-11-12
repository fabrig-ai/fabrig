//! External settings files and their directory sets (client/ctl/overlays.md, 2).
use super::{OneShot, Parsed};
use temper_client_tools::jig::{self, call::{Call, SettingsFile, Value, Values}};
use settings_form::overlay;

pub(super) struct SettingsInput {
    position: usize,
    file: SettingsFile,
}
pub(super) fn settings(call: &Call) -> bool {
    matches!(call.tool, jig::rows::CHANGE_SETTINGS_AT | jig::rows::PROPOSE_SETTINGS_AT)
}
fn values(call: &Call) -> &[Value] {
    match call.values.first().and_then(Option::as_ref) {
        Some(Values::One(value)) => core::slice::from_ref(value),
        Some(Values::Many(values)) => values,
        None => &[],
    }
}
fn pending(call: &Call) -> Option<(usize, &Value)> {
    values(call).iter().enumerate().find(|(_, value)| matches!(value, Value::File(_) | Value::Stdin))
}
impl OneShot {
    pub(super) fn start_settings_input(&mut self) {
        let Some(Parsed::Call { call, .. }) = &self.call else { unreachable!("settings call") };
        match pending(call) {
            Some((position, Value::File(path))) => self.directory.load_text(u8::try_from(position).expect("input index"), path, true),
            Some((_, Value::Stdin)) => { self.input = Some(Vec::new()); self.read_input(); }
            Some(_) => unreachable!("external input"),
            None => { self.directory.start(self.class); self.preparing = false; }
        }
    }
    pub(super) fn settings_file_input(&self, call: &Call) -> bool {
        self.settings_input.is_some() || matches!(pending(call), Some((_, Value::File(_))))
    }
    pub(super) fn settings_input_bytes(&mut self, bytes: Box<[u8]>) {
        let Some(Parsed::Call { call, .. }) = &self.call else { unreachable!("settings call") };
        let (position, source) = pending(call).expect("pending file");
        let stdin = *source == Value::Stdin;
        let name = if let Value::File(name) = source { name.clone() } else { b"-".as_slice().into() };
        let limits = &self.root_env.limits.tools;
        let directories = match jig::settings_file::directories(&bytes, limits) {
            Ok(items) => items,
            Err(why) => { self.settings_fail(&name, &why); return; }
        };
        if stdin {
            for directory in &directories {
                if !directory.path.starts_with(b"/") {
                    let why = overlay::refusal(directory, "is relative, and standard input has no directory; name it absolutely");
                    self.settings_fail(&name, &why); return;
                }
            }
        }
        let remaining = u32::try_from(limits.ask_bytes).expect("input bound") - u32::try_from(bytes.len()).expect("file bounded");
        self.settings_input = Some(SettingsInput { position, file: SettingsFile { name, bytes, directories: skein_lib::List::with_capacity(0) } });
        self.directory.load_sets(directories, remaining, stdin);
    }
    pub(super) fn finish_settings_input(&mut self) -> bool {
        let Some(directories) = self.directory.loaded_sets.take() else { return false };
        let mut input = self.settings_input.take().expect("loaded settings input");
        input.file.directories = directories;
        let Some(Parsed::Call { call, .. }) = &mut self.call else { unreachable!("settings call") };
        let value = Value::SettingsFile(Box::new(input.file));
        match call.values.first_mut().and_then(Option::as_mut) {
            Some(Values::One(slot)) => *slot = value,
            Some(Values::Many(slots)) => *slots.get_mut(input.position).expect("file position") = value,
            None => unreachable!("file slot"),
        }
        self.prepare_start();
        true
    }
    fn settings_fail(&mut self, name: &[u8], why: &[u8]) {
        let Some(Parsed::Call { call, .. }) = &self.call else { unreachable!("settings call") };
        if values(call).len() > 1 { self.fail(&jig::settings_file::named_refusal(name, why)); } else { self.fail(why); }
    }
}
