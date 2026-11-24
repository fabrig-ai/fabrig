#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod binary;

mod support {
    pub use smith_fake_claude::invocation as print;
}
