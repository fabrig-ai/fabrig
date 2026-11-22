use std::{os::unix::ffi::OsStrExt, path::Path};
use temper_config_shell as machine;

fn directories(path: &Path) -> temper_shell::directories::Directories {
    temper_shell::directories::Directories {
        config: path.as_os_str().as_bytes().into(),
        data: path.as_os_str().as_bytes().into(),
        cache: path.as_os_str().as_bytes().into(),
        origins: [skein_shell::Origin::Flag; 3],
    }
}
#[test]
fn machine_facts_take_the_host_name_from_the_shell() {
    let scratch = skein_scratch::Scratch::new("config-machine-facts");
    let found = machine::facts(&directories(scratch.path())).expect("machine facts");
    let hostname = skein_shell::host_name().expect("kernel host name");
    assert_eq!(found.hostname.as_bytes(), hostname.as_ref());
    #[cfg(target_os = "linux")]
    {
        let proc = std::fs::read("/proc/sys/kernel/hostname").expect("Linux host name");
        assert_eq!(found.hostname.as_bytes(), proc.strip_suffix(b"\n").expect("proc's final line feed"));
    }
}
