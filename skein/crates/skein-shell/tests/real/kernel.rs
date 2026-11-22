mod tests {
    #[cfg(target_os = "macos")]
    use skein_shell::OpenError;
    use skein_shell::{Backend, Chosen, Config, Kernel};

    // This opens Linux's completion ring.
    #[cfg(target_os = "linux")]
    #[test]
    fn the_host_chooses_the_ring() {
        let kernel = Kernel::open(Config { operations: 2 }).expect("the host's ring is usable");
        assert_eq!(kernel.chosen(), Chosen { backend: Backend::Ring, ring_refused: None });
        assert_eq!(kernel.chosen().to_string(), "io_uring");
    }

    // This opens Linux's completion ring.
    #[cfg(target_os = "linux")]
    #[test]
    fn forcing_a_backend_reports_it() {
        for backend in [Backend::Ring, Backend::Readiness] {
            let kernel = Kernel::open_on(Config { operations: 2 }, backend).expect("the forced backend is usable");
            assert_eq!(kernel.chosen(), Chosen { backend, ring_refused: None });
            assert_eq!(kernel.in_flight(), 0);
            assert_eq!(kernel.room(), 2);
            assert_eq!(kernel.resubmitted(), 0);
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn darwin_chooses_readiness_and_refuses_the_ring() {
        let kernel = Kernel::open(Config { operations: 2 }).expect("kqueue is usable");
        assert_eq!(kernel.chosen(), Chosen { backend: Backend::Readiness, ring_refused: None });
        assert_eq!(kernel.chosen().to_string(), "readiness (kqueue)");
        assert_eq!(kernel.room(), 2);
        assert_eq!(
            Kernel::open(Config { operations: 0 }).expect_err("zero capacity is refused"),
            OpenError::NoOperations
        );
        for operations in [0, 2] {
            assert_eq!(
                Kernel::open_on(Config { operations }, Backend::Ring).expect_err("Darwin has no ring"),
                OpenError::Unsupported
            );
        }
        assert_eq!(OpenError::Unsupported.to_string(), "the ring is not available on this system");
        assert_eq!(OpenError::BelowFloor.to_string(), "the kernel is below the floor, 24.0");
    }
}
