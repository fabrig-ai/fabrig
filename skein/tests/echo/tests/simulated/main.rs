#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[path = "../suite/scenarios.rs"]
mod scenarios;

#[path = "../suite/fuzzy"]
mod fuzzy {
    mod scenarios;
}
