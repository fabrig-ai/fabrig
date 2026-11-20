use super::{Answer, Ids, Question, Report, Rule, arguments, decode, encode, questions};
use skein_io::kernel::{Error, Rights, Unenforced};

#[test]
fn arguments_round_trip_nested_questions_and_byte_paths() {
    let asked: Box<[Question]> = Box::new([
        Question::Read(Box::from(b"a\n \xff".as_slice())),
        Question::Write(Box::new([])),
        Question::List(Box::from(b".".as_slice())),
        Question::Execute(Box::from(b"run".as_slice())),
        Question::SetId(Box::from(b"set-id".as_slice())),
        Question::NoNewPrivs,
        Question::SignalParent,
        Question::Descriptors,
        Question::Nest {
            rules: Box::new([Rule { path: Box::from(b"inner".as_slice()), rights: Rights::READ }]),
            questions: Box::new([Question::Read(Box::from(b"inner/file".as_slice()))]),
        },
    ]);
    assert_eq!(questions(&arguments(asked.as_ref())), Ok(asked));
}

#[test]
fn reports_round_trip_nested_answers_and_kernel_refusals() {
    let report = Report {
        answers: Box::new([
            Answer::Granted,
            Answer::Refused(Error::Permission),
            Answer::Refused(Error::NotFound),
            Answer::Refused(Error::Other(5)),
            Answer::NoNewPrivs(true),
            Answer::Ids(Ids::Kept),
            Answer::Ids(Ids::Gained),
            Answer::Descriptors(Box::new([0, 1, 2, 9])),
            Answer::Nested(Report {
                answers: Box::new([
                    Answer::Refused(Error::Reach(Unenforced::Rule { index: 1, errno: 2_i32 })),
                    Answer::NoNewPrivs(false),
                ]),
            }),
        ]),
    };
    assert_eq!(decode(&encode(&report)), Ok(report));
}

#[test]
fn malformed_probe_text_is_refused_without_partial_answers() {
    skein_rows::check(
        [
            ("truncated nested question", b"nest 1 61".as_slice()),
            ("invalid path", b"read gg".as_slice()),
            ("trailing tokens", b"descriptors extra".as_slice()),
        ],
        |text| assert_eq!(questions(&[Box::from(text)]), Err(Error::InvalidArgument)),
    );
    skein_rows::check(
        [
            ("truncated line", b"granted".as_slice()),
            ("missing nested answer", b"nested 1\n".as_slice()),
            ("bad boolean", b"no-new-privs 2\n".as_slice()),
        ],
        |text| assert_eq!(decode(text), Err(Error::InvalidArgument)),
    );
}
