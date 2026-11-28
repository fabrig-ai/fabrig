use crate::{CONFIG_BYTES, Fault, Times, cut_text, times};

const SILENT: Times = Times { deliver: None, finish: None, merge: None, close: None };
const PAST_CONFIG_BYTES: usize = CONFIG_BYTES + 1;
const WORKER: &[u8] = b"temper-worker";

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
fn format_one_keeps_the_unmarked_hook_times() {
    // The repository's unmarked .temper/hooks/config.toml.
    const OWN_FILE: &[u8] =
        b"# Hook timeouts (temper/docs/design/protocol/hooks.md, 2.3).\n[close]\ntimeout_seconds = 1800\n";
    let cases: &[&[u8]] = &[
        b"format = 1\n[close]\ntimeout_seconds = 1800\n",
        b"[close]\ntimeout_seconds = 1800\n",
        b"# timeouts\r\n\tformat\t=\t01\t# format 1\r\n[close]\r\ntimeout_seconds = 1800\r\n",
        OWN_FILE,
    ];
    for file in cases {
        assert_eq!(times(file), Ok(Times { close: Some(1800), ..SILENT }), "file {file:?}");
    }
}

#[test]
fn a_newer_format_names_the_written_format_and_reader() {
    let fault = times(b"format = 2\n[close]\ntimeout_seconds = 1800\n").expect_err("newer format");
    assert_eq!(fault, Fault::NewerFormat { line: 1, written: b"2".as_slice().into(), newest: 1 });
    assert_eq!(
        fault.text(WORKER).as_ref(),
        b".temper/hooks/config.toml:1: format 2; this temper-worker reads format 1: upgrade it, or write the file at format 1"
    );
    assert_eq!(
        fault.text(b"temper-engine").as_ref(),
        b".temper/hooks/config.toml:1: format 2; this temper-engine reads format 1: upgrade it, or write the file at format 1"
    );
}

#[test]
fn format_digits_are_not_bounded_by_machine_integers() {
    // One past u64::MAX, with a leading zero in the file.
    assert_eq!(
        times(b"\nformat = 018446744073709551616"),
        Err(Fault::NewerFormat { line: 2, written: b"18446744073709551616".as_slice().into(), newest: 1 })
    );
}

#[test]
fn a_malformed_format_names_its_value_rule_and_line() {
    let cases: &[&[u8]] = &[b"format = 0", b"format = x", b"format = 1_0", b"format = +1", b"format ="];
    for file in cases {
        let fault = times(file).expect_err("malformed format");
        assert_eq!(fault, Fault::BadFormat { line: 1 }, "file {file:?}");
        assert_eq!(fault.text(WORKER).as_ref(), b".temper/hooks/config.toml:1: format: a whole number from 1");
    }
}

#[test]
fn format_must_be_unique_and_before_any_header() {
    let cases: &[&[u8]] = &[b"[close]\nformat = 1\n", b"format = 1\nformat = 1\n", b"format = 2\nformat = 1\n"];
    for file in cases {
        let fault = times(file).expect_err("format placement");
        assert_eq!(fault.text(WORKER).as_ref(), b".temper/hooks/config.toml:2: does not parse", "file {file:?}");
    }
}

#[test]
fn syntax_faults_precede_format_judgement() {
    let cases: &[(&[u8], u32, &[u8])] = &[
        (b"format = 2\n[close]\nunknown = 1\n[merge", 4, b"does not parse"),
        (b"format = 0\n[close]\n# \xff", 3, b"not UTF-8"),
        (b"format = 2\n[prepare]\n[prepare]", 3, b"duplicate table"),
        (b"format = 2\n[prepare]\nunknown = 1\nunknown = 2", 4, b"duplicate key"),
        (b"format = 2\nunknown = 1", 2, b"key before any header"),
    ];
    for &(file, line, why) in cases {
        assert_eq!(times(file), Err(Fault::Line { line, why: why.into() }), "file {file:?}");
    }
}

#[test]
fn a_format_fault_precedes_table_judgement() {
    let cases: &[(&[u8], Fault)] = &[
        (
            b"format = 2\n[close]\nunknown = 1\n",
            Fault::NewerFormat { line: 1, written: b"2".as_slice().into(), newest: 1 },
        ),
        (b"format = 0\n[prepare]\ntimeout_seconds = 0\n", Fault::BadFormat { line: 1 }),
    ];
    for (file, fault) in cases {
        assert_eq!(times(file).as_ref(), Err(fault), "file {file:?}");
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
        assert_eq!(times(file), Err(Fault::Line { line, why: why.into() }), "file {file:?}");
    }
}

#[test]
fn fault_text_names_the_file_line_and_reason() {
    let fault = times(b"[close]\n# why\ntimeout = 5").expect_err("unknown key");
    assert_eq!(fault.text(WORKER).as_ref(), b".temper/hooks/config.toml:3: unknown key `timeout`");
}

#[test]
fn cut_text_names_the_hook_and_both_times() {
    assert_eq!(
        cut_text(b"close", 7200, 3600).as_ref(),
        b"hook `.temper/hooks/close`: asks 7200 s, past the project's ceiling: runs at 3600 s"
    );
}
