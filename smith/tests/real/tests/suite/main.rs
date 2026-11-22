#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod config {
    pub use smith_agent_document::config::find_rg;

    mod tests {
        mod native;
    }
}
mod dump;
mod program {
    mod tests;
}
mod program_configuration;
mod stories;
mod tests;
