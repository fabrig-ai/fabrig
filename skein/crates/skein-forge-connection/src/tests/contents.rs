//! Contents plans through HTTP bytes, with literal answers and exchange counts.
use alloc::boxed::Box;
use skein_forge::api::{Answer, File};
const HEAD: [u8; 32] = {
    let mut bytes = [0; 32];
    bytes[0] = 1;
    bytes
};
const BASE: [u8; 32] = {
    let mut bytes = [0; 32];
    bytes[0] = 3;
    bytes
};
#[test]
fn contents_sizes_agree_with_the_clients_measure() {
    for (name, answer, expected) in measured_contents() {
        assert_eq!(
            skein_forge::forgejo::plans::reads::size::answer(&answer, 8),
            Ok(expected),
            "{name}: running measure"
        );
        assert_eq!(skein_forge::measure::answer(&answer), Some(expected), "{name}: client measure");
    }
}
fn measured_contents() -> [(&'static str, Answer, u64); 4] {
    [
        (
            "job with its step",
            Answer::Job {
                attempt: ATTEMPT,
                step: Some(Step { number: 1, name: Box::from(&b"build"[..]) }),
                log: Box::from(&b"abc"[..]),
                truncated: false,
            },
            8,
        ),
        (
            "file",
            Answer::File { head: HEAD, path: Box::from(&b"x"[..]), bytes: Box::from(&b"abc"[..]), truncated: false },
            4,
        ),
        (
            "pull files",
            Answer::PullFiles {
                head: HEAD,
                files: Box::new([File {
                    path: Box::from(&b"x"[..]),
                    before: Some(Box::from(&b"a"[..])),
                    after: Some(Box::from(&b"bc"[..])),
                }]),
                more: false,
                cut: false,
            },
            68,
        ),
        (
            "comparison",
            Answer::Compare {
                before: BASE,
                after: HEAD,
                files: Box::new([Box::from(&b"x"[..])]),
                ahead: 1,
                behind: 0,
                changed: 1,
            },
            17,
        ),
    ]
}
use skein_forge::api::{JobAttempt, Step};
const ATTEMPT: JobAttempt = JobAttempt { head: HEAD, run: 7, job: 10, attempt: 3 };
