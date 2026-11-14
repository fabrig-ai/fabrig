//! Answering kept host-tool contracts (protocol/wires.md, 7).

use alloc::boxed::Box;
use skein_lib::List;
use smith_domain_run as run;
use temper_engine_domain as engine;

use crate::{Decoded, Problem, answers::Text};

#[expect(clippy::disallowed_macros, reason = "protocol/wires.md, 7 embeds every kept contract for versioned answering")]
pub(crate) const KEPT: &[&str] =
    &[include_str!("../contract/host-tools-v1.txt"), include_str!("../contract/host-tools-v2.txt")];
const ADAPTERS: &[Adapter] = &[];
const CONTRACT: Contract =
    Contract { floor: crate::wire::FLOOR, current: crate::wire::VERSION, kept: KEPT, adapters: ADAPTERS };

/// Decode a call by the contract its run's charter named, then by current code.
pub fn call_for_contract(
    version: u16,
    name: run::CallName,
    tool: &[u8],
    input: &run::HostInput,
    models: &[engine::Model],
    catalogue: &[jig_domain_core::settings::CatalogueModel],
    repositories: &[engine::RepositoryIdentity],
) -> Result<Decoded, Problem> {
    CONTRACT.call(version, name, tool, input, models, catalogue, repositories)
}

struct Contract {
    floor: u16,
    current: u16,
    kept: &'static [&'static str],
    adapters: &'static [Adapter],
}

// Section 7's adapter table names pure mapping functions, with no captured state.
struct Adapter {
    version: u16,
    tool: &'static [u8],
    map: fn(&run::HostInput) -> Result<Mapped, Problem>,
}

struct Mapped {
    tool: Box<[u8]>,
    input: run::HostInput,
}

impl Contract {
    fn text(&self, version: u16) -> &str {
        let position = version.checked_sub(1).expect("contract versions start at 1");
        self.kept.get(usize::from(position)).expect("each contract version has its kept text")
    }

    #[expect(clippy::manual_find, reason = "step code searches the finite adapter table without closures")]
    fn adapter(&self, version: u16, tool: &[u8]) -> Option<&Adapter> {
        for adapter in self.adapters {
            if adapter.version == version && adapter.tool == tool {
                return Some(adapter);
            }
        }
        None
    }

    fn unchanged(&self, version: u16, tool: &[u8]) -> bool {
        let old = tool_lines(self.text(version), tool);
        !old.is_empty() && old == tool_lines(self.text(self.current), tool)
    }

    #[expect(clippy::too_many_arguments, reason = "dispatch preserves the configured current decoder's context")]
    fn call(
        &self,
        version: u16,
        name: run::CallName,
        tool: &[u8],
        input: &run::HostInput,
        models: &[engine::Model],
        catalogue: &[jig_domain_core::settings::CatalogueModel],
        repositories: &[engine::RepositoryIdentity],
    ) -> Result<Decoded, Problem> {
        if version < self.floor || version > self.current {
            let mut text = Text::new();
            text.add(b"this run's host tools are version ")?;
            text.number(u64::from(version))?;
            text.add(b"; this engine answers ")?;
            text.number(u64::from(self.floor))?;
            text.add(b"..")?;
            text.number(u64::from(self.current))?;
            return Ok(Decoded::Answered(text.finish(true)?));
        }
        if version == self.current || self.unchanged(version, tool) {
            return crate::call_in(name, tool, input, models, catalogue, repositories);
        }
        let adapter = self.adapter(version, tool).ok_or(Problem::UnknownTool)?;
        let mapped = (adapter.map)(input)?;
        crate::call_in(name, &mapped.tool, &mapped.input, models, catalogue, repositories)
    }

    #[cfg(test)]
    fn check(&self) -> Result<(), Box<[u8]>> {
        for version in self.floor..self.current {
            for line in self.text(version).lines() {
                let tool = declaration_name(line);
                if !self.unchanged(version, &tool) && self.adapter(version, &tool).is_none() {
                    let mut text = Text::new();
                    text.add(b"host tool ").expect("fixed diagnostic");
                    text.add(&tool).expect("kept tool name");
                    text.add(b" in version ").expect("fixed diagnostic");
                    text.number(u64::from(version)).expect("version digits");
                    text.add(b" needs an adapter, or the floor raised by a breaking move").expect("fixed diagnostic");
                    return Err(text.finish(true).expect("bounded diagnostic").into_parts().0);
                }
            }
        }
        Ok(())
    }
}

fn declaration_name(line: &str) -> Box<[u8]> {
    crate::json::parse(line.as_bytes())
        .expect("kept contract declaration is JSON")
        .required(b"name")
        .expect("declaration has a name")
        .text(b"name")
        .expect("declaration name is text")
}

fn tool_lines(text: &str, tool: &[u8]) -> Box<[Box<[u8]>]> {
    let mut lines = List::with_capacity(u32::try_from(text.lines().count()).expect("kept declarations fit u32"));
    for line in text.lines() {
        if declaration_name(line).as_ref() == tool {
            lines.push(line.as_bytes().into()).expect("one slot per kept declaration");
        }
    }
    lines.into_boxed()
}

#[cfg(test)]
#[path = "tests/contract.rs"]
mod tests;
