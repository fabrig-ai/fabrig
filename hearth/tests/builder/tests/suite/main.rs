#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod goldens;
mod properties;
mod resolution;
mod store;
mod tests {
    mod proxy;
}
