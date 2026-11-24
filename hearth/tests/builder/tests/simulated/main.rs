//! Replaying builder worlds (hearth/docs/design/testing/README.md, 2).

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod bounds;
mod budgets;
mod domain;
mod files;
mod goldens;
mod memory;
mod orders;
mod properties;
mod protocol;
mod simulation;

mod diff {
    mod accounts;
    mod additions;
    mod data;
    mod exports;
    mod files;
    mod hunks;
    mod outcomes;
    mod packages;
    mod parts;
    mod resolution;
    mod scripts;
}

mod render {
    use ::hearth::Form;
    use hearth_builder_world::{fixtures, protocol};

    mod accounts;
    mod additions;
    mod boot;
    mod collisions;
    mod data;
    mod errors;
    mod exports;
    mod files;
    mod git;
    mod hearth;
    mod ids;
    mod outputs;
    mod reserved;
    mod resolution;
    mod resolve;
    mod scripts;
    mod software;

    fn render(overrides: &[fixtures::File]) -> protocol::Run {
        let files = fixtures::thin_with(overrides);
        render_files(&files, &[], Form::Text)
    }

    fn render_files(files: &[fixtures::File], targets: &[&[u8]], form: Form) -> protocol::Run {
        let scenario = fixtures::Scenario { form, ..fixtures::render_targets(files, targets, false) };
        protocol::run(&scenario, fixtures::SIMULATION_SEED)
    }
}

mod store {
    mod cache;
    mod records;
    mod scan;
    mod startup;
}

mod fuzzy {
    mod inputs;

    mod memory;
    mod protocol;
    mod simulation;
}
