use std::fs;
use temper_box_shell::testing::box_command;

#[test]
fn a_held_descriptor_is_refused_as_held() {
    use std::os::fd::AsRawFd;
    let file = fs::File::open("/dev/null").expect("open a non-socket");
    let fd = skein_io::kernel::Fd::new(file.as_raw_fd());
    let held = vec![(fd, std::num::NonZeroU16::new(4).expect("nonzero port"))];
    assert_eq!(box_command::held_listeners(held), Err(format!("held listener: fd {} is not a socket", fd.raw())));
}
