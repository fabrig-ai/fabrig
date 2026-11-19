//! Pure client and referee controls (jig/docs/design/domain/testing.md, 9).
#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod authorize;
mod owner_authority;
mod owner_budget;
mod referee;
