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
    mod exports;
    mod files;
    mod hunks;
    mod outcomes;
    mod packages;
    mod parts;
    mod resolution;
}

mod render {
    mod errors;
    mod git;
    mod outputs;
    mod resolution;
    mod resolve;
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
