mod tests {
    use skein_io::kernel::{Error, Fd};
    use skein_shell::testing::Kept;
    use skein_shell::{Input, Keys, raw_input};
    use std::os::fd::AsRawFd;

    #[test]
    fn kept_mode_changes_refuse_a_pipe() {
        let (reader, _writer) = std::io::pipe().expect("a pipe is not a terminal");
        let fd = Fd::new(reader.as_raw_fd());
        assert!(matches!(raw_input(fd, Keys::Pass), Ok(Input::NotATerminal)));
        // tcsetattr must refuse the descriptor, whatever modes were kept.
        let kept = Kept {
            fd,
            modes: libc::termios {
                c_iflag: 0,
                c_oflag: 0,
                c_cflag: 0,
                c_lflag: 0,
                #[cfg(target_os = "linux")]
                c_line: 0,
                c_cc: [0; libc::NCCS],
                c_ispeed: 0,
                c_ospeed: 0,
            },
            keys: Keys::Pass,
        };
        assert_eq!(kept.set_kept(), Err(Error::Other(libc::ENOTTY)));
        assert_eq!(kept.set_raw(), Err(Error::Other(libc::ENOTTY)));
    }
}
