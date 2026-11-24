use super::prepare_in_read;
use std::io::ErrorKind;
use temper_service_shell::configuration;

#[test]
fn serve_refuses_one_byte_past_the_configuration_bound_before_preparing() {
    // The standalone configuration read admits at most 1 MiB.
    const OVER_BOUND: usize = (1 << 20) + 1;
    let text = vec![b' '; OVER_BOUND];
    let read = temper_config_shell::read_with(
        temper_config_shell::Shape::Standalone,
        || Err(ErrorKind::NotFound.into()),
        |max| {
            if text.len() > usize::try_from(max).expect("read bound fits usize") {
                Err(skein_shell::Unread::TooLarge { max })
            } else {
                Ok(text.into_boxed_slice())
            }
        },
        || panic!("machine facts are not read after an oversized file"),
    );
    let read = configuration::read_with(read, || panic!("data and cache must not be opened"));
    let prepared = prepare_in_read::<()>("temper", read, |_| panic!("serve must not prepare after a refused read"));
    assert_eq!(prepared, Err(format!("temper serve: {}: past 1048576 bytes\n", temper_config_shell::FILE)));
}
