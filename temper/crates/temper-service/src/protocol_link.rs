//! Engine and worker channel stacks joined through bytes
//! (skein/docs/foundation/testing-strategy.md, 2.5; temper/docs/design/protocol/channel.md, 7 and 8).
#![forbid(unsafe_code)]
extern crate alloc;

use temper_engine_service::{values, workers};
use temper_service::{limits, settings, worker_settings};
use temper_worker_service::{Worker, worker_route};

mod translate {
    pub use temper_engine_service::translate::limits;
}

#[cfg(test)]
mod tests {
    #[global_allocator]
    static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

    #[path = "link_fixtures.rs"]
    pub(crate) mod fixtures;
    #[path = "link_start.rs"]
    mod link_room;
}
