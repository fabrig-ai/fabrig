//! B0's document loaders, also used by the real-loop repair and the box twin.
use crate::{forge, landing, llm};

#[must_use]
pub fn fixture() -> forge::Fixture {
    forge::Fixture {
        setup: || {
            let document = temper_fake::forge::Document::decode(include_bytes!("../tests/fixtures/b0/forge.json"))
                .expect("B0 forge document");
            let repository = &document.repositories()[0];
            let mut setup = (landing::fixture().setup)();
            setup.name =
                format!("{}/{}", String::from_utf8_lossy(&repository.owner), String::from_utf8_lossy(&repository.name))
                    .into_bytes()
                    .into();
            setup.tree = repository.tree.clone().expect("walk's seeded tree");
            // The walk's starter CI echoes success; it does not compile widget.
            setup.checks = temper_fake::forge::checks(repository.ci.as_ref());
            setup
        },
        configure: |value, agent, forge| {
            (landing::fixture().configure)(value, agent, forge);
            crate::fixtures::own_policy(value);
            // The codec's result-field envelope grows with the live review's failure text.
            value.limits.root.transcript_memory = 1_200_000_000_000_000_000_u64;
            // Two conversations can load together; a continuation needs room
            // while the completed page waits for reclaim.
            value.temper_over("root.loads.loads", 3);
            crate::fixtures::requester(value);
            value["world_settings"]["maximum_run_spend"] = 32.into();
            value["world_settings"]["cap"] = 1000.into();
            for project in value["world_settings"]["policies"].as_array_mut().expect("projects") {
                let policy = &mut project["policy"];
                policy["period_spend"] = 1000.into();
                policy["roles"][0]["period_spend"] = 1000.into();
                let families = jig_domain_core::ToolFamilies::JIG;
                policy["ceiling"]["tools"] = (families.delegate.0 | families.message.0 | families.read.0).into();
                policy["ceiling"]["delegation"]["depth"] = 5.into();
                policy["ceiling"]["budget"]["spend"] = 1000.into();
                for grant in policy["ceiling"]["grants"].as_array_mut().expect("grants") {
                    grant["pattern"]["value"] = "playground".into();
                }
                policy["roles"][0]["authority"] = policy["ceiling"].clone();
                for grant in policy["projections"].as_array_mut().expect("projection grants") {
                    grant["pattern"]["value"] = "playground".into();
                }
            }
            for rule in
                value["world_settings"]["landing"]["projects"][0]["rules"].as_array_mut().expect("landing rules")
            {
                rule["pattern"]["segments"][2] = "playground".into();
                rule["pattern"]["segments"][3] = "widget".into();
            }
            for (name, size) in [
                ("core.policy_bytes", 8192),
                ("tasks.inbox_bytes", 16384),
                ("tasks.spec_bytes", 4096),
                ("tasks.result_bytes", 512),
                ("documents.text_bytes", 2048),
                ("documents.lines", 32),
                ("brief.sections", 10),
            ] {
                value.jig_over(name, size);
            }
            // Keep the project's seeded policy in typed world defaults, as the reference stories do.
            for member in ["charters", "templates", "policies", "rules"] {
                value["engine"].as_object_mut().expect("engine").remove(member);
            }
            value["world_settings"]["case"] = true.into();
            value["world_settings"]["report_words"] = 32.into();
            landing::admit(value);
        },
        ..forge::fixture()
    }
}

#[must_use]
pub fn from_document(bytes: &[u8]) -> llm::Recipe {
    let document = temper_fake::llm::Document::decode(bytes).expect("B0 LLM document");
    let mut recipe = llm::scripted(Box::new([]));
    recipe.domain = document.domain(17);
    recipe.domain_limits = temper_fake::llm::LIMITS;
    recipe
}
