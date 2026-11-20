//! Turn wire boundaries (protocol/transcript.md, sections 2, 6 and 7).

use crate::support::goldens::Goldens;
use smith_transcript::Record;

use skein_codec::Reason;
use skein_lib::Reader;
use smith_transcript::v7::{CEILINGS, Turn, paths};

#[test]
fn turn_goldens_decode_at_their_measured_length() {
    let goldens = Goldens::new("transcript", 7);
    goldens.assert_measured("turn", |bytes| {
        Turn::decode(&CEILINGS, &mut Reader::new(bytes))
            .expect("turn decodes")
            .measure(&CEILINGS)
            .expect("bounded transcript")
    });
}

#[test]
fn turn_rejects_an_unsupported_version_before_fields() {
    let problem = Turn::decode(&CEILINGS, &mut Reader::new(&[0, 2])).expect_err("version two");
    assert_eq!((problem.path, problem.reason), (paths::TURN_VERSION, Reason::Version));
}

#[test]
fn nesting_problem_goldens_have_their_own_wire_tag() {
    use smith_transcript::v7::CallProblem;
    for bytes in [
        crate::support::goldens::bytes("transcript", 7, "enum_call_problem_nesting_full.bin").as_slice(),
        crate::support::goldens::bytes("transcript", 7, "enum_call_problem_nesting_smallest.bin").as_slice(),
    ] {
        let mut reader = Reader::new(bytes);
        assert_eq!(CallProblem::decode(&CEILINGS, &mut reader).expect("nesting problem"), CallProblem::Nesting);
        assert_eq!(bytes, [8], "the handwritten tag is distinct from every prior problem");
    }
}

#[test]
fn failure_goldens_keep_their_tags_and_now_require_a_resolved_path_payload() {
    use smith_transcript::{Outcome, Resolved, ResolvedFailure, WorkingPath};
    let failure = ResolvedFailure { path: Resolved::Working(WorkingPath { path: Box::new([]) }) };
    for (name, tag, expected) in [
        ("outside", 7, Outcome::Outside(failure.clone())),
        ("read_only", 8, Outcome::ReadOnly(failure.clone())),
        ("not_found", 10, Outcome::NotFound(failure.clone())),
        ("not_directory", 14, Outcome::NotDirectory(failure)),
    ] {
        let bytes = crate::support::goldens::bytes("transcript", 7, &format!("enum_outcome_{name}_smallest.bin"));
        assert_eq!(bytes, [tag, 0, 0, 0, 0, 0], "tag, Working descriptor, zero path length");
        let decoded = Outcome::decode(&CEILINGS, &mut Reader::new(&bytes)).expect("resolved golden");
        assert_eq!(decoded, expected);
        assert!(Outcome::decode(&CEILINGS, &mut Reader::new(&[tag])).is_err(), "bare failure lacks its path");
    }
}

#[test]
fn indexed_edit_failure_goldens_keep_the_first_failure_and_matching_reason() {
    use skein_lib::{List, Writer};
    use smith_transcript::{Ambiguous, EditFailed, LineNumber, Mismatch, Outcome};
    let mut lines = List::with_capacity(2);
    for number in [3, 7] {
        lines.push(LineNumber { number }).expect("room");
    }
    let value = Outcome::EditFailed(EditFailed { edit: 2, why: Mismatch::Ambiguous(Ambiguous { count: 5, lines }) });
    let mut writer =
        Writer::new(usize::try_from(value.measure(&CEILINGS).expect("bounded transcript")).expect("bound"));
    value.write(&CEILINGS, &mut writer).expect("sized encoding");
    let bytes = writer.finish();
    assert_eq!(bytes.as_ref(), &[18, 0, 0, 0, 2, 1, 0, 0, 0, 5, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 7]);
    assert_eq!(Outcome::decode(&CEILINGS, &mut Reader::new(&bytes)).expect("decode"), value);
}

#[test]
fn a_turn_round_trips_its_highest_job_number() {
    use skein_lib::Writer;
    let smallest = &crate::support::goldens::bytes("transcript", 7, "record_turn_smallest.bin");
    let mut parts = Turn::decode(&CEILINGS, &mut Reader::new(smallest)).expect("smallest turn");
    parts.jobs = 37;
    let turn = parts;
    let mut writer =
        Writer::new(usize::try_from(turn.measure(&CEILINGS).expect("bounded transcript")).expect("turn length"));
    turn.write(&CEILINGS, &mut writer).expect("sized encoding");
    let bytes = writer.finish();
    let decoded = Turn::decode(&CEILINGS, &mut Reader::new(&bytes)).expect("turn decodes");
    assert_eq!(decoded.jobs, 37);
    assert_eq!(decoded, turn);
}

#[test]
fn a_running_result_round_trips() {
    use smith_transcript::{BytesClamp, CommandRunning, Outcome, TimeoutClamp};
    let running = CommandRunning {
        job: 12,
        head: b"before yield".as_slice().into(),
        last: b"latest".as_slice().into(),
        dropped: 19,
        timeout: TimeoutClamp::Within,
        output: BytesClamp::Within,
        after: TimeoutClamp::Within,
    };
    round_trip_outcome(&Outcome::Running(running));
}

#[test]
fn a_job_result_round_trips_each_end() {
    use smith_transcript::{BytesClamp, Exit, JobEnd, JobExit, JobRead, Outcome, TimeoutClamp};
    let exited = JobExit { exit: Exit::TimedOut };
    for end in [JobEnd::Running, JobEnd::Ended(exited), JobEnd::Stopped, JobEnd::Cancelled] {
        let read = JobRead {
            job: 12,
            head: b"after yield".as_slice().into(),
            last: b"last".as_slice().into(),
            dropped: 7,
            output: BytesClamp::Within,
            wait: TimeoutClamp::Within,
            end,
        };
        round_trip_outcome(&Outcome::Job(read));
    }
}

fn round_trip_outcome(value: &smith_transcript::Outcome) {
    use skein_lib::Writer;
    let mut writer =
        Writer::new(usize::try_from(value.measure(&CEILINGS).expect("bounded transcript")).expect("result length"));
    value.write(&CEILINGS, &mut writer).expect("sized encoding");
    let bytes = writer.finish();
    let mut reader = Reader::new(&bytes);
    let decoded = smith_transcript::Outcome::decode(&CEILINGS, &mut reader).expect("result decodes");
    assert_eq!(&decoded, value, "result {value:?}");
    assert!(reader.is_empty(), "the result consumed all bytes");
}

#[test]
fn the_transcripts_effort_bound_equals_the_charters() {
    assert_eq!(smith_transcript::CEILINGS.turn_effort_some, smith_charter::CEILINGS.llm_effort_some);
}

#[test]
fn a_not_run_why_survives_a_round_trip() {
    use skein_lib::{List, Writer};
    use smith_transcript::{AfterFailedWrite, AllowedCalls, CallName, NotRun, NotRunWhy, OutsideChoice, Returned};
    const ALLOWED_CALLS: u32 = 2;
    let mut names = List::with_capacity(ALLOWED_CALLS);
    for name in [b"finish".as_slice(), b"deliver".as_slice()] {
        names.push(CallName { name: name.into() }).expect("room");
    }
    let allowed = AllowedCalls { names };
    for why in [
        NotRunWhy::Stopped,
        NotRunWhy::OutsideChoice(OutsideChoice { only: None }),
        NotRunWhy::OutsideChoice(OutsideChoice { only: Some(allowed) }),
        NotRunWhy::AfterFailedWrite(AfterFailedWrite { call: 12 }),
    ] {
        let value = Returned::NotRun(NotRun { why });
        let mut writer =
            Writer::new(usize::try_from(value.measure(&CEILINGS).expect("bounded transcript")).expect("size"));
        value.write(&CEILINGS, &mut writer).expect("encoding");
        assert_eq!(Returned::decode(&CEILINGS, &mut Reader::new(&writer.finish())).expect("decoding"), value);
    }
}

#[test]
fn a_version_four_transcript_is_refused() {
    let problem = Turn::decode(&CEILINGS, &mut Reader::new(&[0, 4])).expect_err("version four");
    assert_eq!((problem.path, problem.reason), (paths::TURN_VERSION, Reason::Version));
}

#[test]
fn a_version_five_transcript_is_refused() {
    let problem = Turn::decode(&CEILINGS, &mut Reader::new(&[0, 5])).expect_err("version five");
    assert_eq!((problem.path, problem.reason), (paths::TURN_VERSION, Reason::Version));
}

#[test]
fn a_turn_carries_its_dialects_number() {
    let smallest = &crate::support::goldens::bytes("transcript", 7, "record_turn_smallest.bin");
    for (name, dialect) in [("openai", 3), ("anthropic", 2)] {
        let mut parts = Turn::decode(&CEILINGS, &mut Reader::new(smallest)).expect("turn");
        parts.dialect = dialect;
        let turn = parts;
        let mut writer = skein_lib::Writer::new(
            usize::try_from(turn.measure(&CEILINGS).expect("bounded transcript")).expect("length"),
        );
        turn.write(&CEILINGS, &mut writer).expect("encoding");
        let bytes = writer.finish();
        let decoded = Turn::decode(&CEILINGS, &mut Reader::new(&bytes)).expect("decode number");
        assert_eq!(decoded.dialect, dialect, "{name}");
    }
}

#[test]
fn an_opening_round_trips_each_account_and_backend() {
    use smith_transcript::{Affinity, Backend, Opening, Program};
    for (account, backend) in [
        (None, Backend::Direct),
        (Some(7), Backend::Direct),
        (None, Backend::Program(Program { name: b"claude_code".as_slice().into() })),
        (Some(7), Backend::Program(Program { name: b"claude_code".as_slice().into() })),
    ] {
        let affinity = Affinity { key: [0x42; 16], thread: 7 };
        let opening = Opening { affinity, account, backend };
        let mut writer = skein_lib::Writer::new(
            usize::try_from(opening.measure(&CEILINGS).expect("bounded transcript")).expect("length"),
        );
        opening.write(&CEILINGS, &mut writer).expect("encoding");
        let bytes = writer.finish();
        assert_eq!(Opening::decode(&CEILINGS, &mut Reader::new(&bytes)).expect("opening"), opening, "{account:?}");
    }
}
