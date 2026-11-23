extern crate alloc;

#[path = "../suite/diff.rs"]
mod diff;
#[path = "../suite/largest.rs"]
mod largest;
#[path = "../suite/parse.rs"]
mod parse;
#[path = "../suite/values.rs"]
mod values;

#[cfg(test)]
mod maps;
mod patch_bridges;
mod patch_check;
mod patch_generated;
mod patch_registered;
mod patch_run;
mod patch_tree;
mod patch_wires;
