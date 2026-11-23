#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod deadline;
mod end_to_end;
mod interrupt;
mod nested_real;
mod real;
mod real_conformance;
#[cfg(target_os = "linux")]
mod real_failure;
mod real_memory;
mod real_roots;
mod stop_continue;
#[cfg(target_os = "linux")]
mod tree {
    mod tests;
}
mod writer_pipe;
