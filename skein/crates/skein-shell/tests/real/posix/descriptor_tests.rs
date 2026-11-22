use skein_io::kernel::Fd;
use skein_shell::descriptor_writes;
use std::os::fd::AsRawFd;

#[test]
fn descriptor_write_access_keeps_the_open_mode() {
    for (read, write, expected) in [(true, false, false), (false, true, true), (true, true, true)] {
        let file = std::fs::OpenOptions::new().read(read).write(write).open("/dev/null").expect("null");
        assert_eq!(descriptor_writes(Fd::new(file.as_raw_fd())), Ok(expected));
        assert_eq!(descriptor_writes(Fd::new(file.as_raw_fd())), Ok(expected), "flags unchanged");
    }
    assert!(descriptor_writes(Fd::new(-1)).is_err());
}
