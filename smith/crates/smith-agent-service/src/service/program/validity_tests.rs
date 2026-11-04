use super::*;

pub(super) fn configuration() -> Config {
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
fn configured_and_native_programs_fit_their_limits() {
    assert!(program_valid(&crate::service::tests::configuration()));
    assert!(program_valid(&configuration()));
}

#[test]
fn invalid_limits_are_refused() {
    let base = configuration().limits;
    for (name, limits) in [
        ("contract bytes", Limits { llm: llm::ComponentLimits { contract_bytes: u64::MAX, ..base.llm }, ..base }),
        ("socket overflow", Limits { io: io::Limits { sockets: u32::MAX, ..base.io }, ..base }),
        ("file overflow", Limits { file_slots: u32::MAX, ..base }),
        (
            "host path",
            Limits {
                domain: domain::Limits {
                    session: domain::session::Limits {
                        tools: domain::tools::Limits { host_path_bytes: u32::MAX, ..base.domain.session.tools },
                        ..base.domain.session
                    },
                    ..base.domain
                },
                ..base
            },
        ),
        (
            "conversations",
            Limits {
                domain: domain::Limits {
                    run: domain::run::Limits { conversations: u32::MAX, ..base.domain.run },
                    ..base.domain
                },
                ..base
            },
        ),
        (
            "receive mismatch",
            Limits { io: io::Limits { receive: base.io.receive.checked_add(1).unwrap(), ..base.io }, ..base },
        ),
        ("socket reserve", Limits { io: io::Limits { sockets: u32::MAX / 8 - 10, ..base.io }, ..base }),
    ] {
        let mut config = configuration();
        config.limits = limits;
        assert!(!program_valid(&config), "{name}");
    }
}

#[test]
fn each_conversation_reserves_a_programs_io_files_and_token_range() {
    let config = configuration();
    let program = config.program.as_ref().unwrap();
    let settings = settings(program, &config.limits, &config.environment).unwrap();
    for conversations in [1, 2, 3, 128] {
        let mut limits = config.limits;
        limits.domain.run.conversations = conversations;
        let reserved = reserved(&limits, &settings).expect("token ranges fit");
        assert_eq!(reserved.io.sockets, limits.io.sockets + IO_SLOTS * conversations);
        assert_eq!(reserved.file_slots, limits.file_slots + FILE_SLOTS * conversations);
    }
    let mut config = configuration();
    config.limits.domain.run.conversations = 129;
    assert!(reserved(&config.limits, &settings).is_none(), "the next child's range crosses the root-close token");
    let Err(ConfigError::Limits) = Service::new(config, 1) else {
        panic!("private token range is a configuration limit")
    };
}
