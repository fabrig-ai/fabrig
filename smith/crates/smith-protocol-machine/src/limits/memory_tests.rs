//! Scenario heap bounds for machine limits (programming-model.md, 6.3).

use super::Limits;
use crate::{Below, BelowEvent, Component, Environment, FromDomain, ToDomain};
use skein_io::{file, kernel};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
use skein_world::domain::heap::{Counting, Meter};
use smith_domain::{run, tools};

#[global_allocator]
static HEAP: Counting = Counting;

fn caps() -> Limits {
    Limits {
        operations: 0,
        path_bytes: 1,
        file_bytes: 65_536,
        tool_payload: 65_536,
        entries: 0,
        entry_bytes: 0,
        processes: 8,
        output_bytes: 0,
        search_hits: 0,
        search_bytes: 0,
        search_line_bytes: 1,
        env_bytes: 0,
        group_stop: Duration::from_secs(1),
        ..crate::fixture::SMALL
    }
}

#[test]
fn invalid_limits_are_refused() {
    let limits = caps();
    assert!(super::valid(&limits));
    assert!(!super::valid(&Limits { processes: u32::MAX, ..limits }));
    assert!(!super::valid(&Limits { search_line_bytes: u32::MAX, ..limits }));
}

fn workspace() -> run::Workspace {
    run::Workspace {
        directories: Box::new([run::Directory {
            host: b"/host".as_slice().into(),
            name: b"r".as_slice().into(),
            root: Token::new(1),
            writable: false,
            git: false,
            conflicts: Box::new([]),
        }]),
    }
}

#[test]
fn every_pending_listing_keeps_its_largest_filter_within_the_bound() {
    // Peak with eight pending listings holding their largest filters.
    // Measured 598_681 B on 2026-10-29; a quarter of headroom, rounded up.
    const FILTERS_CEILING: u64 = 750_000;

    let limits = caps();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let workspace = workspace();
    let mut terminals = Queue::<ToDomain>::with_capacity(1);
    let mut below = Queue::<Below>::with_capacity(1);
    let meter = Meter::new();
    meter.start();
    let mut component = Component::new(&limits, Environment::encode(&[], &limits).expect("empty environment"));
    component.workspace(&workspace);
    meter.check(meter.end(), FILTERS_CEILING, &"component and root");
    for index in 0..limits.processes {
        meter.start();
        component.from_domain(&env, listing(index), &mut terminals, &mut below);
        let measured = meter.end();
        assert!(terminals.is_empty());
        assert_eq!(below.pop(), Some(directory(index)));
        meter.check(measured, FILTERS_CEILING, &index);
    }
    assert_eq!(component.next_deadline(), Some(Time::from_nanos(100)));
    for index in 0..limits.processes {
        finish_listing(&mut component, &env, index, &mut terminals, &mut below);
    }
    assert_eq!(component.next_deadline(), None);
    component.close(&mut below);
    assert!(component.closed());
}

fn directory(index: u32) -> Below {
    Below::OpenDirectory {
        owner: Token::new(u64::from(index) + 10),
        root: Token::new(1),
        path: b".".as_slice().into(),
        no_follow: false,
        deadline: Time::from_nanos(100),
    }
}

fn finish_listing(
    component: &mut Component,
    env: &Env<Limits>,
    index: u32,
    terminals: &mut Queue<ToDomain>,
    below: &mut Queue<Below>,
) {
    let owner = Token::new(u64::from(index) + 10);
    component.from_below(
        env,
        BelowEvent::File(file::Event::Failed {
            owner,
            error: kernel::Error::NotFound,
            committed: false,
            residue: None,
        }),
        terminals,
        below,
    );
    assert_eq!(terminals.pop(), Some(ToDomain::Done { owner, done: tools::Done::Missing }));
}

fn listing(index: u32) -> FromDomain {
    let mut filter = skein_lib::bytes::zeroed(65_536);
    filter.fill(b'x');
    FromDomain::Op {
        owner: Token::new(u64::from(index) + 10),
        op: tools::Op::List {
            at: tools::Place { root: Token::new(1), path: Box::new([]) },
            depth: tools::Depth::Whole,
            glob: Some(filter),
            max: 0,
            max_bytes: 0,
        },
        deadline: Time::from_nanos(100),
    }
}

#[test]
fn saturated_listings_keep_their_pathname_copies_within_the_bound() {
    // Peak with eight listings retaining two largest pathnames apiece.
    // Measured 425_984 B on 2026-10-29; a quarter of headroom, rounded up.
    const LISTINGS_PEAK_CEILING: u64 = 540_000;
    let meter = Meter::new();
    meter.start();
    let mut listings: [Option<crate::Listing>; 8] = [const { None }; 8];
    for listing in &mut listings {
        *listing = Some(crate::Listing::new(0, 0, tools::Depth::Whole, 16_384));
    }
    let mut peak = meter.check(meter.end(), LISTINGS_PEAK_CEILING, &"eight record buffers");
    for listing in &mut listings {
        let listing = listing.as_mut().expect("eight saturated listings");
        for byte in [b'a', b'b'] {
            meter.start();
            for _ in 0_u32..16_384 {
                listing.push(byte);
            }
            listing.push(0);
            peak = peak.max(meter.check(meter.end(), LISTINGS_PEAK_CEILING, &byte));
        }
    }
    assert!(peak <= LISTINGS_PEAK_CEILING, "listing path peak {peak} exceeds {LISTINGS_PEAK_CEILING}");
}

#[test]
fn saturated_listings_allocate_terminal_cells_only_in_the_finishing_step() {
    // Peak while eight full 512-entry listings become terminal lists.
    // Measured 124_960 B on 2026-10-29; a quarter of headroom, rounded up.
    const LISTING_TERMINALS_PEAK_CEILING: u64 = 160_000;
    let meter = Meter::new();
    meter.start();
    let mut listings: [Option<crate::Listing>; 8] = [const { None }; 8];
    for slot in &mut listings {
        let mut listing = crate::Listing::new(512, 13_312, tools::Depth::Whole, 2);
        for index in 0_u16..512 {
            listing.push(b'a' + u8::try_from(index / 26).expect("first letter"));
            listing.push(b'a' + u8::try_from(index % 26).expect("second letter"));
            listing.push(0);
        }
        *slot = Some(listing);
    }
    let mut peak = meter.check(meter.end(), LISTING_TERMINALS_PEAK_CEILING, &"eight full entry lists");
    let mut terminals: [Option<tools::Done>; 8] = [const { None }; 8];
    for (listing, terminal) in listings.iter_mut().zip(&mut terminals) {
        meter.start();
        *terminal = Some(listing.take().expect("full listing").finish(Some(kernel::Exit::Code(0)), false));
        peak = peak.max(meter.check(meter.end(), LISTING_TERMINALS_PEAK_CEILING, &"one terminal list"));
    }
    assert!(
        peak <= LISTING_TERMINALS_PEAK_CEILING,
        "listing terminal peak {peak} exceeds {LISTING_TERMINALS_PEAK_CEILING}"
    );
}
