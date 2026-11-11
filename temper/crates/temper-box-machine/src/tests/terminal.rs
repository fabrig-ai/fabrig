use crate::terminal::{Fact, Formatter, Item, Target};
use skein_lib::{Wall, bytes};
fn wall(seconds: u64) -> Wall {
    Wall::from_nanos(seconds.checked_mul(1_000_000_000).unwrap())
}
fn item(target: Target, words: &[u8]) -> Item {
    Item { target, bytes: bytes::copy_of(words) }
}
#[test]
fn the_first_block_has_exact_words_and_three_targets() {
    let origin = b"http://[::1]:8367";
    let overhead = crate::terminal::FOOTER.len() + b"In a terminal: ".len() + 1 + origin.len() + 1;
    let fits = usize::try_from(crate::terminal::LINK_LINE_BYTES).unwrap() - overhead;
    let largest = Box::new([b'w'; 8207]);
    for (word, url, expected_word, expected_origin) in [
        (
            bytes::copy_of(b"temper"),
            b"http://link/path?secret=fixture".as_slice(),
            bytes::copy_of(b"temper"),
            b"http://link".as_slice(),
        ),
        (
            bytes::copy_of(b"~/'my temper'"),
            b"http://[::1]:8367/prefix/jig/link",
            bytes::copy_of(b"~/'my temper'"),
            origin.as_slice(),
        ),
        (
            bytes::copy_of(&largest[..fits]),
            b"http://[::1]:8367/path",
            bytes::copy_of(&largest[..fits]),
            origin.as_slice(),
        ),
        (bytes::copy_of(&largest[..=fits]), b"http://[::1]:8367/path", bytes::copy_of(b"temper"), origin.as_slice()),
    ] {
        let formatter = Formatter::new(crate::terminal::LinkForm::Bare, word);
        let line = crate::lines::joined(&[url, b"\n"]);
        let expected = crate::lines::joined(&[
            crate::terminal::FOOTER,
            b"In a terminal: ",
            &expected_word,
            b" ",
            expected_origin,
            b"\n",
        ]);
        let block = formatter.block(line.clone());
        assert!(
            block.as_slice()
                == [
                    item(Target::Stderr, b"Open this link in a browser on this machine.\n\n"),
                    item(Target::Stdout, &line),
                    item(Target::Stderr, &expected),
                ],
            "the first block keeps the given word within its bound and uses only the origin"
        );
        assert!(block.get(2).unwrap().bytes.len() <= usize::try_from(crate::terminal::LINK_LINE_BYTES).unwrap());
    }
}
#[test]
fn fresh_blocks_keep_state_across_restart_and_have_no_time() {
    let mut formatter = Formatter::new(crate::terminal::LinkForm::Bare, bytes::copy_of(b"temper"));
    formatter.admitted_block();
    assert_eq!(
        formatter.block(bytes::copy_of(b"http://fresh\n")).as_slice(),
        &[item(Target::Stderr, b"\n"), item(Target::Stdout, b"http://fresh\n")]
    );
}
#[test]
fn notices_time_every_line_before_and_after_a_block() {
    let mut formatter = Formatter::new(crate::terminal::LinkForm::Bare, bytes::copy_of(b"temper"));
    for _ in 0_u32..2 {
        assert_eq!(
            formatter.notice(b"first\nsecond\n", wall(3723)),
            item(Target::Stderr, b"01:02:03Z first\n01:02:03Z second\n")
        );
        formatter.admitted_block();
    }
}

fn labelled(label: &[u8], name: &[u8]) -> Box<[crate::layout::Placed]> {
    use temper_forge_setup::{Group, Kind, Stage, Steps};
    crate::layout::flatten(Steps {
        always: Box::new([]),
        asked: Box::new([Group {
            kind: None,
            remover: None,
            forge: 37,
            before: 0,
            recorded: false,
            label: bytes::copy_of(label),
            steps: Box::new([(Stage::Run, super::fixtures::step(name, Kind::Long))]),
        }]),
    })
}
#[test]
fn group_and_failure_facts_use_supplied_labels_and_exact_words() {
    use crate::machine::Why;
    use skein_io::kernel::Exit;
    use skein_lib::Duration;
    let steps = labelled(b"\xffother\0label", b"a step");
    let mut formatter = Formatter::new(crate::terminal::LinkForm::Bare, bytes::copy_of(b"temper"));
    formatter.admitted_block();
    for (fact, expected) in [
        (Fact::GroupStarting { forge: 37 }, b"\xffother\0label: starting\n".as_slice()),
        (Fact::GroupStarted { forge: 37 }, b"\xffother\0label: started\n"),
        (Fact::GroupStopped { forge: 37 }, b"\xffother\0label: stopped\n"),
        (
            Fact::GroupFailed { forge: 37, why: Why::Exited { step: 0, exit: Exit::Code(7) } },
            b"\xffother\0label: failed, a step exited 7\n",
        ),
        (
            Fact::GroupFailed { forge: 37, why: Why::NotStarted { step: 0 } },
            b"\xffother\0label: failed, a step did not start\n",
        ),
        (
            Fact::GroupFailed { forge: 37, why: Why::NotReady { step: 0 } },
            b"\xffother\0label: failed, a step version unavailable\n",
        ),
        (Fact::GroupFailed { forge: 37, why: Why::NotWritten }, b"\xffother\0label: failed, record not written\n"),
        (
            Fact::Restarting {
                step: 0,
                why: Why::Exited { step: 0, exit: Exit::Code(0) },
                delay: Duration::from_millis(500),
            },
            b"a step exited 0; restarting in 0.5 s\n",
        ),
        (
            Fact::Restarting { step: 0, why: Why::NotStarted { step: 0 }, delay: Duration::from_millis(1000) },
            b"a step did not start; restarting in 1 s\n",
        ),
        (
            Fact::Restarting { step: 0, why: Why::Busy, delay: Duration::from_nanos(1_000_000_001) },
            b"busy; restarting in 1.000000001 s\n",
        ),
        (
            Fact::GivingUp { step: 0, why: Why::Exited { step: 0, exit: Exit::Signal(15) } },
            b"a step killed by signal 15; giving up\n",
        ),
    ] {
        let expected = crate::lines::joined(&[b"23:59:59Z ", expected]);
        assert_eq!(formatter.fact(fact, &steps, wall(86_399)), item(Target::Stderr, &expected));
    }
    for (why, expected) in
        [(Why::Busy, b"busy".as_slice()), (Why::NotAsked, b"not asked"), (Why::Stopping, b"stopping")]
    {
        assert_eq!(crate::terminal::why(why, &steps).as_ref(), expected);
    }
}

#[test]
fn group_started_keeps_its_label_when_the_step_carries_hands() {
    let mut steps = labelled(b"Forge 37 (Forgejo)", b"forge");
    steps[0]
        .step
        .set_started_notice(b"Forgejo: api http://127.0.0.1:3000/api/v1\nForgejo: person ada\n".as_slice().into());
    let mut formatter = Formatter::new(crate::terminal::LinkForm::Bare, bytes::copy_of(b"temper"));
    formatter.admitted_block();
    assert_eq!(
        formatter.fact(Fact::GroupStarted { forge: 37 }, &steps, wall(3723)),
        item(Target::Stderr, b"01:02:03Z Forge 37 (Forgejo): started\n")
    );
}

#[test]
fn every_group_fact_binds_the_owned_number_and_keeps_the_playground_label() {
    use crate::{layout, machine::Why, terminal::max_bytes};
    for (template, expected) in
        [(b"Forge $forge (Forgejo)".as_slice(), b"Forge 65535 (Forgejo)".as_slice()), (b"Playground", b"Playground")]
    {
        let mut steps = labelled(template, b"forgejo-65535");
        steps[0].label = Some(layout::bound_label(template, u16::MAX));
        steps[0].asked.as_mut().unwrap().forge = u16::MAX;
        let mut formatter = Formatter::new(crate::terminal::LinkForm::Bare, bytes::copy_of(b"temper"));
        formatter.admitted_block();
        for (fact, suffix) in [
            (Fact::GroupStarting { forge: u16::MAX }, b": starting\n".as_slice()),
            (Fact::GroupStarted { forge: u16::MAX }, b": started\n"),
            (Fact::GroupFailed { forge: u16::MAX, why: Why::NotWritten }, b": failed, record not written\n"),
            (Fact::GroupStopped { forge: u16::MAX }, b": stopped\n"),
        ] {
            let output = formatter.fact(fact, &steps, wall(3723));
            assert_eq!(output.bytes, crate::lines::joined(&[b"01:02:03Z ", expected, suffix]));
            assert!(output.bytes.len() <= usize::try_from(max_bytes(&steps)).unwrap());
        }
    }
    assert_eq!(layout::bound_label(b"$forge/$forge", 17).as_ref(), b"17/17");
}
#[test]
fn lifecycle_output_bound_covers_long_labels_and_signal_numbers() {
    use crate::{machine::Why, terminal::max_bytes};
    use skein_io::kernel::Exit;
    use skein_lib::Duration;
    let label: Box<[u8]> = Box::new([b'L'; 8192]);
    let name: Box<[u8]> = Box::new([b'N'; 4096]);
    let steps = labelled(&label, &name);
    let why = Why::Exited { step: 0, exit: Exit::Signal(u32::MAX) };
    let mut formatter = Formatter::new(crate::terminal::LinkForm::Bare, bytes::copy_of(b"temper"));
    formatter.admitted_block();
    for fact in [
        Fact::GroupFailed { forge: 37, why },
        Fact::Restarting { step: 0, why, delay: Duration::from_nanos(u64::MAX) },
        Fact::GivingUp { step: 0, why },
    ] {
        let item = formatter.fact(fact, &steps, wall(86_399));
        assert!(item.bytes.len() <= usize::try_from(max_bytes(&steps)).unwrap());
        assert!(item.bytes.ends_with(b"\n"));
        assert!(super::fixtures::Harness::contains(&item.bytes, b"killed by signal 4294967295"));
    }
    assert!(max_bytes(&[]) >= crate::terminal::FIXED_BYTES);
    for words in [crate::terminal::PRELUDE, crate::terminal::FOOTER] {
        assert!(words.len() <= usize::try_from(crate::terminal::FIXED_BYTES).unwrap());
    }
}

#[test]
fn the_largest_tail_fits_one_output_item_and_reset_releases_every_line() {
    use crate::limits::MAX_TAIL_LINES;
    use crate::terminal::{Tail, failure_bytes};
    let steps = labelled(b"Forge with a long label", b"last step");
    let name = steps.first().unwrap().step.name();
    let piece = super::fixtures::line_cap();
    let line = crate::lines::joined(&[name, b": ", &piece, b"\n"]);
    let mut tail = Tail::new(MAX_TAIL_LINES);
    for _ in 0..MAX_TAIL_LINES {
        tail.push(line.clone());
    }
    let fact = Fact::GroupFailed {
        forge: 37,
        why: crate::machine::Why::Exited { step: 0, exit: skein_io::kernel::Exit::Code(1) },
    };
    let formatter = Formatter::new(crate::terminal::LinkForm::Bare, bytes::copy_of(b"temper"));
    let mut block = formatter.fact(fact, &steps, Wall::EPOCH);
    tail.append(&mut block, b"/data/box/logs/last-step.log");
    assert!(block.bytes.len() > 409_600);
    assert!(block.bytes.len() <= usize::try_from(failure_bytes(&steps, b"/data")).unwrap());
    tail.reset();
    let mut empty = formatter.fact(fact, &steps, Wall::EPOCH);
    tail.append(&mut empty, b"/data/box/logs/last-step.log");
    assert_eq!(empty, formatter.fact(fact, &steps, Wall::EPOCH));
}

#[test]
fn a_link_takes_osc_8_on_a_terminal_unless_term_is_dumb() {
    use crate::terminal::LinkForm::{self, Bare, Osc8};
    for (terminal, term, expected) in [
        (true, None, Osc8),
        (true, Some(b"xterm-256color".as_slice()), Osc8),
        (true, Some(b"eterm-color".as_slice()), Osc8),
        (true, Some(b"".as_slice()), Osc8),
        (true, Some(b"dumber".as_slice()), Osc8),
        (true, Some(b"dumb".as_slice()), Bare),
        (false, None, Bare),
        (false, Some(b"xterm-256color".as_slice()), Bare),
        (false, Some(b"dumb".as_slice()), Bare),
    ] {
        assert_eq!(LinkForm::of(terminal, term), expected, "terminal={terminal}, TERM={term:?}");
    }
}
#[test]
fn the_first_block_on_a_terminal_wraps_its_link() {
    let formatter = Formatter::new(crate::terminal::LinkForm::Osc8, bytes::copy_of(b"temper"));
    assert_eq!(formatter.block(bytes::copy_of(b"http://link\n")).as_slice(), &[
        item(Target::Stderr, b"Open this link in a browser on this machine.\n\n"),
        item(Target::Stdout, b"\x1b]8;;http://link\x1b\\http://link\x1b]8;;\x1b\\\n"),
        item(Target::Stderr, b"\nIt signs you in as this temper's owner, once, within 2 hours.\nOpened after that, it prints a fresh one here.\nIn a terminal: temper http://link\n"),
    ]);
}
#[test]
fn a_fresh_block_on_a_terminal_wraps_its_link() {
    let mut formatter = Formatter::new(crate::terminal::LinkForm::Osc8, bytes::copy_of(b"temper"));
    formatter.admitted_block();
    assert_eq!(
        formatter.block(bytes::copy_of(b"http://link\n")).as_slice(),
        &[item(Target::Stderr, b"\n"), item(Target::Stdout, b"\x1b]8;;http://link\x1b\\http://link\x1b]8;;\x1b\\\n"),]
    );
}
#[test]
fn the_widest_link_fills_its_line() {
    use crate::terminal::{LINK_LINE_BYTES, LinkForm};
    let url = super::fixtures::line_cap();
    let line = LinkForm::Osc8.line(crate::lines::joined(&[&url, b"\n"]));
    assert_eq!(LINK_LINE_BYTES, 8207);
    assert_eq!(line.len(), usize::try_from(LINK_LINE_BYTES).unwrap());
    assert_eq!(line, crate::lines::joined(&[b"\x1b]8;;", &url, b"\x1b\\", &url, b"\x1b]8;;\x1b\\\n"]));
    for form in [LinkForm::Bare, LinkForm::Osc8] {
        assert_eq!(form.line(bytes::copy_of(b"\n")).as_ref(), b"\n");
    }
}
