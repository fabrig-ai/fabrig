//! A sweep holds its prefix once (`skein/docs/design/testing/README.md`, 6).
mod process;

#[cfg(target_os = "linux")]
pub use process::signal_parent;
pub use process::{inherit, passed, split};

/// How a forked child ended.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Exit {
    Code(i32),
    Signal(i32),
}

/// Each side of one fork.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Split {
    /// Continue here, ending at `passed` or the first panic.
    Child,
    /// The child has ended and the parent can continue.
    Parent(Exit),
}

#[cfg(test)]
mod tests;
