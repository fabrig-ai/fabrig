#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod access;
mod check;
mod create;
mod github;
mod github_convert;
mod github_install;
mod github_paste;
mod github_reconcile;
mod heap;
mod hooks;
mod host;
mod reachable;
mod reconcile;
mod take_over;

mod fuzzy {
    mod github_setup;
    mod reconcile;
}
