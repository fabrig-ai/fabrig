//! Named setting references (client/ctl/tools.md, section 1).
use crate::jig::{
    call::{Call, Value, Values},
    render::{Parts, Text},
};
use alloc::boxed::Box;
use jig_client::contract as c;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Charter,
    Template,
    Policy,
}

impl Kind {
    pub(crate) fn number(self, settings: &c::SettingsDocument, name: &[u8]) -> Option<u32> {
        for line in &settings.settings {
            let ((Self::Charter, c::Setting::Charter(number))
            | (Self::Template, c::Setting::Template(number))
            | (Self::Policy, c::Setting::Policy(number))) = (self, line.setting)
            else {
                continue;
            };
            if line.name.as_deref() == Some(name) {
                return Some(number.number);
            }
        }
        None
    }

    fn word(self) -> &'static [u8] {
        match self {
            Self::Charter => b"charter",
            Self::Template => b"template",
            Self::Policy => b"policy",
        }
    }
}

pub(crate) fn template(me: &c::Me, project: u32, name: &[u8]) -> Option<u32> {
    for membership in &me.projects {
        if membership.project != project {
            continue;
        }
        for template in &membership.templates {
            if template.name.as_deref() == Some(name) {
                return Some(template.number);
            }
        }
        break;
    }
    None
}

pub(crate) fn charter(me: &c::Me, name: &[u8]) -> Option<u32> {
    for charter in &me.charters {
        if charter.name.as_ref() == name {
            return Some(charter.number);
        }
    }
    None
}

pub(crate) fn resolved(call: &mut Call, argument: usize, number: u32) {
    *call.values.get_mut(argument).expect("setting reference argument") =
        Some(Values::One(Value::Number(u64::from(number))));
}

#[derive(Debug)]
pub struct Missing {
    kind: Kind,
    name: Box<[u8]>,
    cut: bool,
}
impl Missing {
    pub(crate) fn new(kind: Kind, name: &[u8]) -> Self {
        // Names served by the engine are short ASCII; an unsuccessful argument can be arbitrary bytes.
        const SHOWN: usize = 128;
        let shown = name.get(..SHOWN).unwrap_or(name);
        Self { kind, name: Box::from(shown), cut: shown.len() != name.len() }
    }

    pub(crate) fn words(&self, parts: &dyn Parts) -> Box<[u8]> {
        let prefix = parts.diagnostic_prefix();
        let mut measure = Text::measure();
        self.write(prefix, &mut measure);
        let mut out = Text::writing_exact(measure);
        self.write(prefix, &mut out);
        out.finish()
    }

    fn write(&self, prefix: &[u8], out: &mut Text) {
        out.put(prefix);
        out.put(b": no ");
        out.put(self.kind.word());
        out.put(b" is named ");
        for byte in &self.name {
            out.put(&[if byte.is_ascii_graphic() || *byte == b' ' { *byte } else { b'?' }]);
        }
        if self.cut {
            out.put(b"...");
        }
        out.put(b"; ");
        out.put(prefix);
        out.put(b" settings show lists them");
    }
}
