//! Application roots on the worker channel (jig/docs/design/protocol/channel/layouts.md, 7).
use crate::Spoken;
use alloc::boxed::Box;
use skein_codec::{
    layout::{Mode, Refusal, Side},
    schema,
};
use skein_lib::{Buffer, List};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Carried {
    Items,
    Left,
    Saved,
    Preparation,
    JobSpec,
    JobResult,
    Grant,
    Ask,
    Answer,
}

#[derive(Clone, Copy, Debug)]
pub struct Root {
    pub carried: Carried,
    pub wire: &'static str,
    pub declaration: &'static str,
}

/// A value type and the worker layout for which its owner supplies a converter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Written {
    pub wire: &'static str,
    pub declaration: &'static str,
    pub layout: [u8; 32],
}

#[derive(Debug)]
pub struct Application {
    pub spoken: List<Spoken>,
    pub roots: List<Root>,
    pub written: List<Written>,
}
impl Application {
    /// An application whose opaque parts cross unchanged.
    #[must_use]
    pub fn opaque(spoken: List<Spoken>) -> Self {
        Self { spoken, roots: List::uncapped(), written: List::uncapped() }
    }
}

/// Jig's own written entries, including those for the families it carries from smith.
#[must_use]
pub fn written() -> List<Written> {
    List::uncapped()
}

/// Jig's written converters (jig/docs/design/protocol/channel/layouts.md, 6).
#[expect(clippy::disallowed_methods, reason = "the protocol validates parsed schema identifiers")]
pub fn convert(_mode: Mode, from: &Side<'_>, _to: &Side<'_>, _value: &[u8], out: &mut Buffer) -> Result<(), Refusal> {
    out.clear();
    let name = from.text.spelled(from.text.declaration(from.root.decl).name);
    Err(Refusal {
        declaration: Box::from(core::str::from_utf8(name).expect("schema declaration names are ASCII")),
        part: Box::from(""),
        why: Box::from("jig has no written converter for this layout pair"),
    })
}

/// Checks entries on a link, which declares its wires but never converts.
#[must_use]
pub fn written_present(spoken: &List<Spoken>, entries: &List<Written>) -> bool {
    for (index, entry) in entries.iter().enumerate() {
        for earlier in entries.iter().take(index) {
            if earlier == entry {
                return false;
            }
        }
        let mut found = false;
        for wire in spoken {
            if wire.name == entry.wire {
                let Ok(text) = schema::parse(wire.text.as_bytes()) else { return false };
                found = text.find(entry.declaration.as_bytes()).is_some();
                break;
            }
        }
        if !found {
            return false;
        }
    }
    true
}
