#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

use skein_ui::{phrases::worst_case as phrase_heap_bound, worst_case as heap_bound};

mod limits;
mod memory;
