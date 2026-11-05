extern crate alloc;

use skein_fake_forge_forgejo::{
    Config, Event, Faults, Hosted, Limits, Repository, Server, Service, closed, delivery, down, resume,
    resume_with_domain, start, up, up_with_domain,
};

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod github_memory;
mod memory;
