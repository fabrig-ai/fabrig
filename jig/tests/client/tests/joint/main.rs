//! Joint client and engine stories (jig/docs/design/client/domain/testing.md, 5).
#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod approval;
mod check_replay;
mod unlinked;

mod stories {
    mod projects;
}

mod fuzzy {
    mod joint;
}
