#[cfg(target_os = "linux")]
#[path = "../support/mod.rs"]
mod support;

#[cfg(target_os = "linux")]
mod cli {
    use skein_throwaway_forgejo::testing::run_user;

    mod tests;
}

mod throwaway;
