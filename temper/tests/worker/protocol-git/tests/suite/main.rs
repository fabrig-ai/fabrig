//! Git protocol worlds (temper/docs/design/testing/tiers.md, 1).

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod protocol_git;
mod transcripts;

mod fuzzy {
    mod protocol_git;
}

mod git_identity;

mod tests {
    mod reds;
}
mod trash;
mod view;
