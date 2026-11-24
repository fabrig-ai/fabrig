#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[path = "../steps/support.rs"]
mod support;

#[path = "../suite/digests.rs"]
mod digests;
#[path = "../suite/domain_referee.rs"]
mod domain_referee;
#[path = "../suite/domain_utilities.rs"]
mod domain_utilities;
#[path = "../suite/hosted.rs"]
mod hosted;
#[path = "../suite/roots.rs"]
mod roots;
#[path = "../suite/signals.rs"]
mod signals;
#[path = "../suite/steps.rs"]
mod steps;
#[path = "../suite/wire_trace.rs"]
mod wire_trace;
#[path = "../suite/world.rs"]
mod world;

#[path = "../suite/fuzzy"]
mod fuzzy {
    mod hosted;
    mod roots;
    mod steps;
}

mod drive;
mod harness;
mod nested;
mod writer_pipe;

mod end_to_end;
