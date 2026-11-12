use super::*;
pub(super) use smith_agent_profile_fixtures::program_configuration as configuration;

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
