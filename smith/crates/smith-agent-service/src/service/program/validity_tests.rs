use super::*;

fn configuration() -> Config {
    let mut config = crate::service::tests::configuration();
    config.program = Some(ProgramBackend {
        program: Programs { claude: Box::from(b"/claude".as_slice()), directory: Box::from(b"/programs".as_slice()) },
        endpoints: Box::new([0]),
        relay: Box::from(b"/agent".as_slice()),
        entropy: Secret::new(Box::new([7; 32])),
        effective_user: 1000,
    });
    config
}

#[test]
fn invalid_limits_are_refused() {
    assert!(program_valid(&crate::service::tests::configuration()));
    assert!(program_valid(&configuration()));
    for case in 0_u8..7 {
        let mut config = configuration();
        match case {
            0 => config.limits.llm.contract_bytes = u64::MAX,
            1 => config.limits.io.sockets = u32::MAX,
            2 => config.limits.file_slots = u32::MAX,
            3 => config.limits.domain.session.tools.host_path_bytes = u32::MAX,
            4 => config.limits.domain.run.conversations = u32::MAX,
            5 => config.limits.io.receive += 1,
            6 => config.limits.io.sockets = u32::MAX / 8 - 10,
            _ => unreachable!(),
        }
        assert!(!program_valid(&config), "invalid program case {case}");
    }
}
