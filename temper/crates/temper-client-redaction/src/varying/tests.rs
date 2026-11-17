use super::*;

fn run(build: &[u8], codes: &[&[u8]], fields: &[(&[u8], Framing)]) -> Normalised {
    let mut varied = Varied::new();
    varied.build(build);
    for code in codes {
        varied.code(code);
    }
    for (field, framing) in fields {
        varied.seen(field, *framing);
    }
    let mut recording = Vec::new();
    for (field, _) in fields {
        let mut field = Box::from(*field);
        varied.replace(&mut field);
        recording.extend_from_slice(&field);
    }
    varied.counts(recording.into())
}
fn counts(result: &Normalised) -> (u32, u32, u32, u32, u32) {
    (result.build, result.port, result.code, result.places, result.stood)
}

#[test]
fn every_kind_uses_the_placeholders_matching_form() {
    let build = b"v<>\"&\\\n";
    let input = b"v<>\"&\\\n|v%3C%3E%22%26%5C%0A|v%3c%3e%22%26%5c%0a|v&lt;&gt;&quot;&amp;\\\n|v<>\\\"&\\\\\\n|WDJBMJHT WDJB-MJHT FKLMNPQR FKLM-NPQR|127.0.0.1:34073 127.0.0.1%3A34073 127.0.0.1%3a34073 %5B%3A%3A1%5D%3A34073 %5b%3a%3a1%5d%3a34073";
    let expected = b"0.0.0 (000000000000)|0.0.0%20%28000000000000%29|0.0.0%20%28000000000000%29|0.0.0 (000000000000)|0.0.0 (000000000000)|BBBBBBBB BBBB-BBBB CCCCCCCC CCCC-CCCC|127.0.0.1:10001 127.0.0.1%3A10001 127.0.0.1%3a10001 %5B%3A%3A1%5D%3A10001 %5b%3a%3a1%5d%3a10001";
    let result = run(build, &[b"WDJB-MJHT", b"FKLMNPQR"], &[(input, Framing::Text)]);
    assert_eq!(result.recording.as_ref(), expected);
    assert_eq!(counts(&result), (1, 1, 2, 14, 0));
}

#[test]
fn ports_are_address_scoped_and_numbered_in_byte_order() {
    // Encoded localhost appears before the raw IPv4 host, not after it in a host table.
    let input = b"http://localhost%3a41000/x [::1]:42000 127.0.0.1:41000 127.0.0.1:43000 10.0.0.1:43000 example.com:43000 :43000 43000 127.0.0.1:654321 127.0.0.1:70000 localhost:0 localhost:1 localhost:65535";
    let result = run(b"", &[], &[(input, Framing::Text)]);
    assert_eq!(result.recording.as_ref(), b"http://localhost%3a10001/x [::1]:10002 127.0.0.1:10001 127.0.0.1:10003 10.0.0.1:43000 example.com:43000 :43000 43000 127.0.0.1:654321 127.0.0.1:70000 localhost:0 localhost:10004 localhost:10005");
    assert_eq!(counts(&result), (0, 5, 0, 6, 0));
}

#[test]
fn clean_and_dirty_builds_have_one_fixed_placeholder() {
    for build in [b"0.0.0 (aa699a65adfb-dirty)".as_slice(), b"0.0.0 (aa699a65adfb)"] {
        let result = run(build, &[], &[(build, Framing::Text)]);
        assert_eq!(result.recording.as_ref(), b"0.0.0 (000000000000)");
        assert_eq!(counts(&result), (1, 0, 0, 1, 0));
    }
    for build in [b"".as_slice(), b"0.0.0 (000000000000)", b"xxxxxxxx"] {
        let result = run(build, &[b"xxxxxxxx"], &[(build, Framing::Text)]);
        assert_eq!(result.recording.as_ref(), build);
        assert_eq!(counts(&result), (0, 0, 0, 0, 0));
    }
}

#[test]
fn a_framed_width_stands_everywhere_but_text_width_is_free() {
    let fields = [
        (b"localhost:8080|127.0.0.1%3A8080|".as_slice(), Framing::Text),
        (b"[::1]:8080|localhost:41000".as_slice(), Framing::Framed),
    ];
    let result = run(b"", &[], &fields);
    assert_eq!(result.recording.as_ref(), b"localhost:8080|127.0.0.1%3A8080|[::1]:8080|localhost:10002");
    assert_eq!(counts(&result), (0, 1, 0, 1, 1));
    let free = run(b"", &[], &[(b"127.0.0.1:8080", Framing::Text)]);
    assert_eq!(free.recording.as_ref(), b"127.0.0.1:10001");
    assert_eq!(counts(&free), (0, 1, 0, 1, 0));
    // The encoded form, not merely the unescaped original, must fit its frame.
    let build = b"a&";
    let result = run(build, &[], &[(b"a&amp;", Framing::Framed), (build, Framing::Text)]);
    assert_eq!(result.recording.as_ref(), b"a&amp;a&");
    assert_eq!(counts(&result), (0, 0, 0, 0, 1));
}

#[test]
fn swapped_originals_do_not_chain_and_normalisation_is_idempotent() {
    let first = run(
        b"build",
        &[b"CCCCCCCC", b"BBBBBBBB"],
        &[(b"build localhost:10002 [::1]:10001 CCCC-CCCC BBBBBBBB", Framing::Text)],
    );
    assert_eq!(first.recording.as_ref(), b"0.0.0 (000000000000) localhost:10001 [::1]:10002 BBBB-BBBB CCCCCCCC");
    assert_eq!(counts(&first), (1, 2, 2, 5, 0));
    let second = run(BUILD, &[b"BBBBBBBB", b"CCCCCCCC"], &[(first.recording.as_ref(), Framing::Text)]);
    assert_eq!(second.recording, first.recording);
    assert_eq!(counts(&second), (0, 0, 0, 0, 0));
}

#[test]
fn originals_past_the_placeholder_ranges_stand() {
    let mut varied = Varied::new();
    let mut fields = Vec::new();
    for number in 40000..40100 {
        let field = format!("localhost:{number}").into_bytes().into_boxed_slice();
        varied.seen(&field, Framing::Text);
        fields.push(field);
    }
    for letter in CODE_ALPHABET {
        varied.code(&[b'B', b'C', b'D', b'F', b'G', b'H', b'J', letter]);
    }
    varied.code(b"WDJBMJHT");
    let mut codes = Box::from(b"BCDFGHJZ WDJBMJHT".as_slice());
    varied.seen(&codes, Framing::Text);
    varied.replace(&mut codes);
    for field in &mut fields {
        varied.replace(field);
    }
    assert_eq!(fields.last().expect("hundredth port").as_ref(), b"localhost:40099");
    assert_eq!(codes.as_ref(), b"ZZZZZZZZ WDJBMJHT");
    assert_eq!(counts(&varied.counts(Box::default())), (0, 99, 1, 100, 2));
}
