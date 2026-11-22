//! Fixtures and observations for the world harness's tests (tools/crates/design.md, 3.2).

pub mod deadline {
    pub use crate::deadline::testing::at;
    #[cfg(target_os = "linux")]
    pub use crate::deadline::testing::linux::{
        ThreadBaseline, Watch, Watches, figure, has_boot_origin, has_cgroup, thread_baseline, watches,
    };
}

#[cfg(target_os = "linux")]
pub use crate::tree::testing::open_cgroup_member;
