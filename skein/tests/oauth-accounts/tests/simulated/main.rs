#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[path = "../suite/access_only.rs"]
mod access_only;
#[path = "../suite/accounts.rs"]
mod accounts;
#[path = "../suite/debug.rs"]
mod debug;
#[path = "../suite/memory.rs"]
mod memory;
#[path = "../suite/private.rs"]
mod private;
#[path = "../suite/referee.rs"]
mod referee;
#[path = "../suite/shared.rs"]
mod shared;
#[path = "../suite/sign_in.rs"]
mod sign_in;
#[path = "../suite/sign_out.rs"]
mod sign_out;
#[path = "../suite/simulated.rs"]
mod simulated;
#[path = "../suite/world.rs"]
mod world;

#[path = "../suite/fuzzy"]
mod fuzzy {
    mod private;
    mod shared;
    mod sign_in;
    mod world;
}
