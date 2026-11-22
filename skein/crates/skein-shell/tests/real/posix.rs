mod descriptor_tests;
mod group;
mod links;
#[cfg(target_os = "linux")]
mod parent_death;

#[cfg(target_os = "linux")]
pub(crate) use skein_shell::testing::nonblocking;

#[cfg(target_os = "linux")]
pub(crate) mod process {
    pub(crate) use skein_shell::testing::signal_number;
}

pub(crate) mod terminal {
    pub(crate) use skein_shell::testing::signal_mask;
}
