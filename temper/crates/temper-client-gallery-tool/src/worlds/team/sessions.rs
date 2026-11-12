//! Committed sessions for the role card (client/web/gallery.md, 2).
use super::{List, c};
pub(super) fn page() -> c::ActivityPage {
    use skein_lib::Wall;
    // Session starts, UTC, 6–8 November 2026.
    let friday = 1_794_006_300_000_000_000;
    let saturday = 1_794_068_400_000_000_000;
    let sunday = 1_794_102_720_000_000_000;
    let mut rows = List::with_capacity(11);
    for (position, nanos, line) in [
        (9, sunday + 60_000_000_000, c::Activity::NowWritten {
            attempt: 4,
            line: c::Text::new(b"Reproduced finding 91; asked you about its fix.".as_slice().into(), 512).expect("Now line fits"),
        }),
        (8, sunday, c::Activity::RunBegan { attempt: 4, why: c::RunWhy::Ready, resumed: false }),
        (7, sunday - 120_000_000_000, c::Activity::SessionEnded { attempt: 2, end: c::SessionEnd::Quiet }),
        (6, saturday + 1_320_000_000_000, c::Activity::RunEnded { attempt: 3, end: c::RunEnd::Parked, spent: 4 }),
        (5, saturday + 900_000_000_000, c::Activity::RunBegan { attempt: 3, why: c::RunWhy::Ready, resumed: true }),
        (4, saturday + 600_000_000_000, c::Activity::RunEnded { attempt: 2, end: c::RunEnd::Parked, spent: 13 }),
        (
            3,
            saturday + 500_000_000_000,
            c::Activity::NowWritten {
                attempt: 2,
                line: c::Text::new(b"Triaged three findings;\nrouted two to you.".as_slice().into(), 512)
                    .expect("Now line fits"),
            },
        ),
        (2, saturday, c::Activity::RunBegan { attempt: 2, why: c::RunWhy::Ready, resumed: false }),
        (1, saturday - 1_200_000_000_000, c::Activity::SessionEnded { attempt: 1, end: c::SessionEnd::Quiet }),
    ] {
        rows.push(c::DecodedActivityLine {
            source: Box::new(c::ActivityLine { task: 52, position: position + 2, at: Wall::from_nanos(nanos), line }),
            decoded: None,
        })
        .expect("session lines fit");
    }
    rows.push(c::DecodedActivityLine {
        source: Box::new(c::ActivityLine {
            task: 52,
            position: 2,
            at: Wall::from_nanos(friday + 1_860_000_000_000),
            line: c::Activity::RunEnded { attempt: 1, end: c::RunEnd::Parked, spent: 4 },
        }),
        decoded: None,
    })
    .expect("Friday end");
    rows.push(c::DecodedActivityLine {
        source: Box::new(c::ActivityLine {
            task: 52,
            position: 1,
            at: Wall::from_nanos(friday),
            line: c::Activity::RunBegan { attempt: 1, why: c::RunWhy::Ready, resumed: false },
        }),
        decoded: None,
    })
    .expect("Friday beginning");
    c::ActivityPage { cut: false, rows, before: None, after: None }
}
