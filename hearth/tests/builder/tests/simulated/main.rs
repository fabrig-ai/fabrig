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
    mod errors;
    mod exports;
    mod git;
    mod hearth;
    mod outputs;
    mod reserved;
    mod resolution;
    mod resolve;

    use ::hearth::{Call, Form};
    use hearth_builder_world::{fixtures, protocol};

    fn render(files: &[fixtures::File], targets: &[&[u8]], form: Form) -> protocol::Run {
        let scenario = fixtures::Scenario {
            files,
            call: Call::Render {
                resolve: false,
                repository: fixtures::GIT_DIR.into(),
                revision: b"HEAD".as_slice().into(),
                targets: targets.iter().map(|target| Box::<[u8]>::from(*target)).collect(),
            },
            form,
            ..fixtures::thin()
        };
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
