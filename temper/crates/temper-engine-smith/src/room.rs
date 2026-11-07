//! Measure a charter's non-brief host part from the declarations and smith's counter
//! (temper/docs/design/domain/agent.md, 4.3).

use alloc::boxed::Box;
use skein_lib::Duration;
use smith_domain_run as run;
use temper_engine_domain::room::HostPart;

/// Build the table from the full host offer and smith's receiving schema costs.
#[must_use]
pub fn host_part(limits: &run::Limits) -> HostPart {
    let mut declared = 0_u64;
    let mut settings = 0_u64;
    let mut tools = crate::calls::tools(Duration::from_secs(1));
    for tool in &mut tools {
        let mut bytes = 0_u64;
        for text in [&tool.name, &tool.description, &tool.schema] {
            bytes = bytes.checked_add(skein_lib::len_u64(text.len())).expect("bounded tool declaration");
        }
        if tool.name.as_ref() == b"settings" || tool.name.as_ref() == b"propose_settings" {
            settings = settings.checked_add(bytes).expect("bounded settings declarations");
        } else {
            declared = declared.checked_add(bytes).expect("bounded declarations");
        }
        // Charter 1's finite runs carry the largest reserve-tool offer.
        tool.reserve = tool.name.as_ref() == b"message";
    }
    let mut charter = measurement_charter();
    charter.grants.host_tools = tools;
    // Count independently of the receiving bound, including a profile too small for this offer.
    let measuring = run::Limits { host_part: u64::from(u32::MAX), ..*limits };
    let room = run::brief_room(&charter, None, &measuring).expect("bounded largest host offer");
    let fixed = measuring
        .host_part
        .checked_sub(u64::from(room))
        .expect("measured host room")
        .checked_sub(declared)
        .expect("declarations counted by smith")
        .checked_sub(settings)
        .expect("settings counted by smith");
    // Smith's built-in tools and mechanics are outside its host part. Alternative-model
    // text is policy-dependent and is counted by HostPart::non_brief.
    HostPart { bound: limits.host_part, fixed, declared, settings, inspect: 0, modify: 0, shell: 0, agents: 0 }
}

/// Measure the largest translated Change contract, including mid-run delivery.
fn measurement_charter() -> run::Charter {
    use jig_protocol_charter as charter;
    let max = charter::CONTRACT_TEXT_MAX;
    let charter = charter::Charter {
        mode: run::Mode::Finite,
        instructions: Box::new([]),
        brief: Box::new([]),
        tools: Box::new([]),
        wait: charter::WAIT_GRANTED,
        waiting: Duration::ZERO,
        resumes: false,
        budget: charter::Budget {
            turns: 1,
            spend: 1,
            time: Duration::from_secs(1),
            reserve: charter::Reserve { turns: 1, time: Duration::from_secs(1) },
        },
        model: charter::Llm {
            grant: None,
            model: charter::Model {
                prices: charter::Prices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1 },
                dialect: 1,
                endpoint: 1,
                name: Box::new([]),
                window: 1,
                output: 1,
                effort: None,
            },
        },
        models: Box::new([]),
        workspace: charter::WorkspaceTools { inspect: false, modify: false, shell: false },
        delivers: false,
        agents: false,
        conventions: None,
        contract: charter::Contract {
            report: None,
            failure: Some(charter::TextRule { max, fields: Box::new([]) }),
            verdicts: Box::new([charter::VerdictRule {
                name: Box::from(b"no_change".as_slice()),
                text_max: max,
                fields: Box::new([]),
                items: charter::Items { min: 0, max: 0, kinds: Box::new([]) },
            }]),
            change: Some(charter::ChangeRule {
                checks_must_pass: charter::CONTRACT_CHECKS_MUST_PASS,
                fields: Box::new([
                    charter::FieldRule { name: Box::from(b"title".as_slice()), max },
                    charter::FieldRule { name: Box::from(b"body".as_slice()), max: charter::CHANGE_BODY_MAX },
                ]),
            }),
        },
    };
    jig_protocol_charter::charter(charter).expect("measurement charter's bounded contract")
}
