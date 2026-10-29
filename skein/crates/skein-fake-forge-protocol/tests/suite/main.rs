extern crate alloc;

use skein_fake_forge_protocol::{
    Config, Event, Faults, Hosted, Limits, Repository, Server, Service, closed, delivery, down, resume,
    resume_with_domain, start, up, up_with_domain,
};

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod memory;
