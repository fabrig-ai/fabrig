//! Costs measured by smith, supplied as facts to the domain
//! (temper/docs/design/domain/agent/room.md, 4.4).

use alloc::boxed::Box;
use skein_lib::{Duration, List};
use smith_domain_run as run;
use smith_protocol_translate::{Endpoint, Endpoints};
use temper_engine_domain::room::{Cost, HostPart};

/// Measure the frame, models and declarations independently of either receiving bound.
#[must_use]
pub fn host_part(limits: Option<&run::Limits>, allowance: u64, endpoints: &Endpoints) -> HostPart {
    let mut longest = b"".as_slice();
    for endpoint in endpoints.entries() {
        if endpoint.name.len() > longest.len() {
            longest = &endpoint.name;
        }
    }
    let mut entries = List::with_capacity(1);
    entries.push(Endpoint { name: longest.into(), number: 1, dialect: 1 }).expect("one measurement endpoint");
    let endpoints = Endpoints::new(entries);
    let mut charter = measurement_charter();
    let fixed = measure(&charter, limits, &endpoints);
    charter.models = Box::new([charter.llm.clone()]);
    let model = difference(measure(&charter, limits, &endpoints), fixed);
    charter.models = Box::new([]);
    let mut table = HostPart { allowance, fixed, model, ..HostPart::EMPTY };
    if let Some(limits) = limits {
        table.bound = Some(limits.host_part);
    }
    for tool in
        crate::calls::tools_for(Duration::from_secs(1), jig_domain_tasks::Tools(3_072), jig_domain_tasks::Tools(3_072))
    {
        let settings = tool.name.as_ref() == b"propose_settings";
        charter.grants.host_tools = Box::new([tool]);
        let cost = difference(measure(&charter, limits, &endpoints), fixed);
        if settings {
            table.asking_settings = table.asking_settings.plus(cost);
        } else {
            table.asking = table.asking.plus(cost);
        }
    }
    for mut tool in crate::calls::tools_for(
        Duration::from_secs(1),
        temper_engine_domain::RunPolicy::EVERY_FAMILY,
        temper_engine_domain::RunPolicy::EVERY_FAMILY,
    ) {
        let family = match tool.name.as_ref() {
            b"delegate" | b"goal" | b"change" | b"finding" | b"red" | b"follow" => Some(0),
            b"settings" | b"propose_settings" => Some(10),
            _ => None,
        };
        // Charter 1's finite runs carry the reserved message and its wind-down words.
        tool.reserve = tool.name.as_ref() == b"message";
        charter.grants.host_tools = Box::new([tool]);
        let cost = difference(measure(&charter, limits, &endpoints), fixed);
        match family {
            Some(index) => {
                let slot = table.families.get_mut(index).expect("JIG family slot");
                *slot = slot.plus(cost);
            }
            None => table.always = table.always.plus(cost),
        }
    }
    table
}

fn difference(whole: Cost, base: Cost) -> Cost {
    Cost {
        rendered: whole.rendered.checked_sub(base.rendered).expect("additive rendered cost"),
        owned: whole.owned.checked_sub(base.owned).expect("additive ownership"),
        encoded: whole.encoded.checked_sub(base.encoded).expect("additive encoding"),
    }
}

fn measure(charter: &run::Charter, limits: Option<&run::Limits>, endpoints: &Endpoints) -> Cost {
    let mut rendered = 0;
    if let Some(limits) = limits {
        let measuring = run::Limits { host_part: u64::from(u32::MAX), ..*limits };
        rendered = measuring
            .host_part
            .checked_sub(u64::from(run::brief_room(charter, None, &measuring).expect("bounded measurement charter")))
            .expect("measured room");
    }
    Cost {
        rendered,
        owned: charter.owned_bytes().expect("bounded measurement ownership"),
        encoded: skein_lib::len_u64(
            smith_protocol_translate::encode_charter(charter, &smith_charter::CEILINGS, endpoints)
                .expect("bounded measurement encoding")
                .len(),
        ),
    }
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
            grant: Some(1),
            model: charter::Model {
                prices: charter::Prices { input: 1, cached: 1, cache_write: Some(1), output: 1, unit: 1 },
                dialect: 1,
                endpoint: 1,
                name: Box::new([]),
                window: 1,
                output: 1,
                effort: Some(Box::new([])),
            },
        },
        models: Box::new([]),
        workspace: charter::WorkspaceTools { inspect: false, modify: false, shell: false },
        delivers: false,
        agents: false,
        conventions: Some(charter::Conventions {
            guide: b"AGENTS.md".as_slice().into(),
            checks: b".temper/pre-pr".as_slice().into(),
        }),
        words: u32::MAX,
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
                    charter::FieldRule { name: Box::from(b"body".as_slice()), max: u32::MAX },
                ]),
            }),
        },
    };
    // Optional model fields are present, effort empty: absent fields in stock models
    // make this encoded sum conservative (room.md, 4.4), never an underestimate.
    let mut measured = jig_protocol_charter::charter(charter).expect("measurement charter's bounded contract");
    measured.grants.contract = crate::wire::VERSION;
    // The project may supply a delivery-check bound, whose optional payload also belongs to the frame.
    measured.check_timeout = Some(Duration::from_secs(86_400));
    measured
}
