//! Schemas and descriptions for Smith's tools.
//! This module retains no state. The prompt translation offers these declarations
//! under the domain's grants; host declarations pass through unchanged.
//! Contract: protocol/llm.md, section 3; domain/tools.md, section 3.

use alloc::boxed::Box;

use skein_lib::List;
use smith_domain::run;
use smith_domain::{llm, tools};

use crate::{Error, ToolKind, ToolSchema};

/// The bounded Smith inventory derived from the prompt and admitted charter rules.
pub fn schemas_for_contract(
    prompt: &llm::Prompt,
    stated: &crate::Stated,
    outcome: &run::outcome::OutcomeSpec,
    deliver: Option<&run::outcome::ChangeSpec>,
    maximum: u32,
) -> Result<Box<[ToolSchema]>, Error> {
    let mut descriptors = List::with_capacity(12);
    for (granted, tool, name) in [
        (prompt.tools.inspect, tools::Tool::Read, b"read".as_slice()),
        (prompt.tools.inspect, tools::Tool::List, b"list"),
        (prompt.tools.inspect, tools::Tool::Search, b"search"),
        (prompt.tools.modify, tools::Tool::Write, b"write"),
        (prompt.tools.modify, tools::Tool::Edit, b"edit"),
        (prompt.tools.modify, tools::Tool::Remove, b"remove"),
        (prompt.tools.shell, tools::Tool::Shell, b"shell"),
        (prompt.tools.shell, tools::Tool::Job, b"job"),
    ] {
        if granted {
            add(&mut descriptors, owned(tool, name, stated, &prompt.namespace))?;
        }
    }
    for offered in &prompt.served {
        let descriptor = match offered {
            llm::Served::Host(_) => continue,
            llm::Served::Wait => waiting(),
            llm::Served::SubAgent => sub_agent(stated.host_part),
            llm::Served::Deliver => {
                delivering(crate::contract::deliver_schema(deliver.ok_or(Error::Invalid)?, maximum)?)
            }
            llm::Served::Finish => finishing(crate::contract::finish_schema(outcome, maximum)?),
        };
        add(&mut descriptors, descriptor)?;
    }
    Ok(descriptors.into_boxed())
}

/// Measure Smith's fixed workspace schemas and fixed wait/sub-agent declarations.
/// Contract-generated finish and deliver declarations belong to the host's part.
#[must_use]
pub fn fixed_schema_bytes(stated: &crate::Stated) -> Option<u64> {
    let several = tools::Namespace::Several {
        working: tools::Name::new(b"work".as_slice().into()).expect("fixed directory name"),
    };
    Some(fixed_namespace_bytes(stated, &tools::Namespace::One)?.max(fixed_namespace_bytes(stated, &several)?))
}

fn fixed_namespace_bytes(stated: &crate::Stated, namespace: &tools::Namespace) -> Option<u64> {
    let mut bytes = 0_u64;
    for schema in [
        owned(tools::Tool::Read, b"read", stated, namespace),
        owned(tools::Tool::List, b"list", stated, namespace),
        owned(tools::Tool::Search, b"search", stated, namespace),
        owned(tools::Tool::Write, b"write", stated, namespace),
        owned(tools::Tool::Edit, b"edit", stated, namespace),
        owned(tools::Tool::Remove, b"remove", stated, namespace),
        owned(tools::Tool::Shell, b"shell", stated, namespace),
        owned(tools::Tool::Job, b"job", stated, namespace),
        waiting(),
        sub_agent(stated.host_part),
    ] {
        for field in [&schema.name, &schema.description, &schema.schema] {
            bytes = bytes.checked_add(u64::try_from(field.len()).ok()?)?;
        }
    }
    Some(bytes)
}

fn waiting() -> ToolSchema {
    descriptor(
        ToolKind::Wait,
        b"wait",
        b"Wait for the next message or outstanding result.",
        br#"{"type":"object","properties":{}}"#,
    )
}

fn sub_agent(host_part: u64) -> ToolSchema {
    let prefix = br#"{"type":"object","properties":{"brief":{"type":"string","maxLength":"#;
    let suffix = br#"},"tools":{"type":"array","items":{"type":"string","enum":["inspect","modify","shell"]}},"llm":{"type":"string"},"max_turns":{"type":"integer","minimum":1},"max_seconds":{"type":"integer","minimum":1}},"required":["brief","tools"]}"#;
    let decimal = skein_lib::Decimal::of(host_part);
    let bytes = decimal.as_bytes();
    let length = prefix
        .len()
        .checked_add(bytes.len())
        .expect("fixed prefix and u64")
        .checked_add(suffix.len())
        .expect("fixed suffix");
    let mut writer = skein_lib::Writer::new(length);
    writer.put(prefix).expect("measured prefix");
    writer.put(bytes).expect("measured bound");
    writer.put(suffix).expect("measured suffix");
    ToolSchema { kind: ToolKind::SubAgent, name: Box::from(b"sub_agent".as_slice()),
        description: Box::from(b"Ask a sub-agent to work under the granted tool families and optional model. Optional max_turns and max_seconds ask for a share, clamped to what is left above the finishing reserve. The child is told its effective share. The brief and inherited context fit the stated host part.".as_slice()),
        schema: writer.finish() }
}

pub(crate) fn contract_text_bytes() -> Option<[u64; 2]> {
    let finish = finishing(Box::new([]));
    let deliver = delivering(Box::new([]));
    Some([
        u64::try_from(finish.name.len()).ok()?.checked_add(u64::try_from(finish.description.len()).ok()?)?,
        u64::try_from(deliver.name.len()).ok()?.checked_add(u64::try_from(deliver.description.len()).ok()?)?,
    ])
}

fn delivering(schema: Box<[u8]>) -> ToolSchema {
    ToolSchema {
        kind: ToolKind::Deliver,
        name: b"deliver".as_slice().into(),
        description: b"Offer the checked change to the host under the change contract.".as_slice().into(),
        schema,
    }
}

fn finishing(schema: Box<[u8]>) -> ToolSchema {
    ToolSchema {
        kind: ToolKind::Finish,
        name: b"finish".as_slice().into(),
        description: b"Declare the final result under the host's result contract.".as_slice().into(),
        schema,
    }
}

fn add(descriptors: &mut List<ToolSchema>, schema: ToolSchema) -> Result<(), Error> {
    descriptors.push(schema).or(Err(Error::Limit))
}

fn descriptor(kind: ToolKind, name: &[u8], description: &[u8], schema: &[u8]) -> ToolSchema {
    ToolSchema { kind, name: name.into(), description: description.into(), schema: schema.into() }
}

fn owned(tool: tools::Tool, name: &[u8], stated: &crate::Stated, namespace: &tools::Namespace) -> ToolSchema {
    ToolSchema {
        kind: ToolKind::Owned(tool),
        name: name.into(),
        description: crate::descriptions::description(tool, stated, namespace),
        schema: crate::descriptions::schema(tool, stated, namespace),
    }
}
