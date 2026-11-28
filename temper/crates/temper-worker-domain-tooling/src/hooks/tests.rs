use crate::{CONFIG_BYTES, Times, cut_text, times};

const SILENT: Times = Times { deliver: None, finish: None, merge: None, close: None };
const PAST_CONFIG_BYTES: usize = CONFIG_BYTES + 1;

#[test]
fn reads_times_without_filling_in_silent_moments() {
    let cases: &[(&[u8], Times)] = &[
        (b"# how long\n[close]\ntimeout_seconds = 1800\n", Times { close: Some(1800), ..SILENT }),
        (
            b"\t[ deliver ] # first\r\n\ttimeout_seconds\t=\t1 # minimum\r\n \t\r\n\
              [finish]\r\ntimeout_seconds = 250\r\n\
              [merge]\t# third\r\ntimeout_seconds = 600\r\n\
              [\tclose\t]\r\ntimeout_seconds = 4294967295\t# maximum\r\n",
            Times { deliver: Some(1), finish: Some(250), merge: Some(600), close: Some(u32::MAX) },
        ),
        (b"", SILENT),
        (b"# d\xc3\xa9lais\n[finish]\n\n[close]", SILENT),
    ];
    for &(file, expected) in cases {
        assert_eq!(times(file), Ok(expected), "file {file:?}");
    }
}

#[test]
fn accepts_a_file_at_the_byte_bound() {
    assert_eq!(times(&[b' '; CONFIG_BYTES]), Ok(SILENT));
}

#[test]
fn reports_the_reason_on_the_first_faulty_line() {
    let cases: &[(&[u8], u32, &[u8])] = &[
        (b"[close]\ntimeout_seconds = 0", 2, b"a value of 0"),
        (b"[close]\ntimeout = 5", 2, b"unknown key `timeout`"),
        (b"[prepare]", 1, b"unknown table `prepare`"),
        (b"timeout_seconds = 1", 1, b"key before any header"),
        (b"[close]\n[merge]\n[close]", 3, b"duplicate table"),
        (b"[close]\ntimeout_seconds = 2\ntimeout_seconds = 3", 3, b"duplicate key"),
        (b"[close]\ntimeout_seconds = 1_800", 2, b"does not parse"),
        (b"[close]\nclose.timeout_seconds = 1800", 2, b"does not parse"),
        (b"[close] timeout_seconds = 1", 1, b"does not parse"),
        (b"[close]\ntimeout_seconds = \"2\"", 2, b"does not parse"),
        (b"[close]\ntimeout_seconds = 4294967296", 2, b"does not parse"),
        (b"[close]\ntimeout_seconds = +2", 2, b"does not parse"),
        (b"[close]\ntimeout_seconds =", 2, b"does not parse"),
        (b"[[close]]", 1, b"does not parse"),
        (b"[close]\x0c", 1, b"does not parse"),
        (b"[close]\r\ntimeout_seconds = 1\r\n# \xff\r\n", 3, b"not UTF-8"),
        (&[b' '; PAST_CONFIG_BYTES], 0, b"file too large"),
    ];
    for &(file, line, why) in cases {
        let fault = times(file).expect_err("faulty time file");
        assert_eq!((fault.line, fault.why.as_ref()), (line, why), "file {file:?}");
    }
}

#[test]
fn fault_text_names_the_file_line_and_reason() {
    let fault = times(b"[close]\n# why\ntimeout = 5").expect_err("unknown key");
    assert_eq!(fault.text().as_ref(), b".temper/hooks/config.toml:3: unknown key `timeout`");
}

#[test]
fn cut_text_names_the_hook_and_both_times() {
    assert_eq!(
        cut_text(b"close", 7200, 3600).as_ref(),
        b"hook `.temper/hooks/close`: asks 7200 s, past the project's ceiling: runs at 3600 s"
    );
}
