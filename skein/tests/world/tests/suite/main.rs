#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[path = "../steps/support.rs"]
mod support;

mod deadline;
mod digests;
mod domain_referee;
mod domain_utilities;
mod drive;
mod end_to_end;
mod hosted;
mod interrupt;
mod real;
mod real_conformance;
mod real_memory;
mod real_roots;
mod roots;
mod signals;
mod steps;
mod stop_continue;
mod world;
mod writer_pipe;

mod fuzzy {
    mod hosted;
    mod roots;
    mod steps;
}
