//! Startup tests for the shell over the document readers.

mod config;

fn resources() -> crate::Resources {
    crate::Resources {
        input: skein_io::kernel::Fd::new(0),
        output: skein_io::kernel::Fd::new(1),
        signals: skein_io::kernel::Fd::new(2),
        seed: 1,
        pid: 1,
        trace: None,
        error: None,
        roots: None,
        programs: None,
    }
}
