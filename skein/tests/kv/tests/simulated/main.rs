#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[path = "../suite/census.rs"]
mod census;
#[path = "../suite/disk.rs"]
mod disk;
mod golden;
#[path = "../suite/holds.rs"]
mod holds;
#[path = "../suite/logs.rs"]
mod logs;
#[path = "../suite/memory.rs"]
mod memory;
#[path = "../suite/open.rs"]
mod open;
#[path = "../suite/recovery.rs"]
mod recovery;
#[path = "../suite/store.rs"]
mod store;
#[path = "../suite/target.rs"]
mod target;

#[path = "../suite/fuzzy"]
mod fuzzy {
    mod census;
    mod holds;
    mod kept_reads;
    mod logs;
    mod map;
}
