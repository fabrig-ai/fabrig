//! The shared checkout's git entry points and remote contract
//! (skein's testing/fakes/checkout.md, section 4). Local operations keep objects and merge
//! metadata; temper's world-owned `Remote` implementations retain forge policy.
//! These explicit exports preserve their trait identity and terminal outcomes.

pub use skein_fake_checkout::git::{
    CloneInfo, CommitFailure, Created, Fault, Merged, NotFetched, ObjectFormat, Pushed, Remote, Tree, Want, What,
    check_out, clone_bare, clone_repository, commit, commit_always, commit_merging, create, fetch, init,
    init_with_format, merge, object_format, object_present, push, push_expected, update_ref,
};
