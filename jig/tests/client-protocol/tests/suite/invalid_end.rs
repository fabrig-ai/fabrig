//! Run failure causes through server bytes and the client link (protocol/client/wire.md, 1).
use jig_client::contract as c;
use jig_client_protocol_wire::v22 as w;
use jig_domain_tasks as t;
use jig_tasks_kit::{LIMITS, World, task};
use skein_lib::{Reader, Writer};

fn ended(end: t::End) -> c::RunEnd {
    let mut world = World::new(450, LIMITS);
    world.make(t::Party::Person(1), vec![task(1, &[])]);
    world.claim_task(1);
    world.terminal(1, end);
    if world.record(1).invalid_result.is_some() {
        world.elapse(jig_tasks_kit::RETRY.max.saturating_add(skein_lib::Duration::from_nanos(1)));
        world.claim(1, 2);
        assert_eq!(world.record(1).invalid_result, None, "the next attempt clears only the current reason");
    }
    let line = world
        .records
        .values()
        .find_map(|record| match record {
            t::Stored::Activity(row) if matches!(row.line, t::Line::RunEnded { .. }) => Some(row),
            _ => None,
        })
        .expect("committed run end");
    through_link(line)
}

fn through_link(line: &t::ActivityLine) -> c::RunEnd {
    let page = wire_page(line);
    let limits = jig_client_protocol_link::Limits::new(jig_client_protocol_world::fixtures::domain_limits(), 1)
        .expect("client limits");
    let page = jig_client_protocol_link::translate::pages::activity_page(page, &limits).expect("client page");
    let c::Activity::RunEnded { end, .. } = page.rows.get(0).expect("end row").source.line else { panic!("run ended") };
    end
}

fn wire_page(line: &t::ActivityLine) -> w::ActivityPage {
    let row = jig_protocol_engine_clients::translate::activity::page_turn(
        1,
        line,
        None,
        &jig_protocol_engine_clients::Private::none(),
        &w::CEILINGS,
    )
    .expect("server translates end");
    let mut bytes = Writer::new(usize::try_from(row.measure()).expect("row size") + 4);
    let mut row_bytes = Writer::new(usize::try_from(row.measure()).expect("row size"));
    row.encode(&mut row_bytes).expect("row encodes");
    let row_bytes = row_bytes.finish();
    bytes.put(&u32::try_from(row_bytes.len()).expect("small row").to_be_bytes()).expect("piece length");
    bytes.put(&row_bytes).expect("piece");
    let page = w::ActivityPage::new(
        &w::CEILINGS,
        w::ActivityPageParts { cut: false, rows: bytes.finish(), before: None, after: None },
    )
    .expect("page");
    let mut bytes = Writer::new(usize::try_from(page.measure()).expect("page size"));
    page.encode(&mut bytes).expect("page encodes");
    let bytes = bytes.finish();
    w::ActivityPage::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("wire page")
}

#[test]
fn a_contract_invalid_run_end_keeps_its_reason_through_the_link() {
    let end = ended(t::End::Finished {
        result: t::TaskResult::Verdict { fields: Box::new([]), code: 0, words: Box::new([]) },
        cancel_delegates: false,
    });
    assert_eq!(end, c::RunEnd::Failed { class: c::HoldFailure::Invalid, invalid_reason: Some(c::InvalidReason::Form) });
    for (kind, reason) in [
        (t::InvalidResult::Form, c::InvalidReason::Form),
        (t::InvalidResult::Verdict, c::InvalidReason::Verdict),
        (t::InvalidResult::Words, c::InvalidReason::Words),
        (t::InvalidResult::Change, c::InvalidReason::Change),
        (t::InvalidResult::Followups, c::InvalidReason::Followups),
    ] {
        let line = t::ActivityLine {
            task: 1,
            position: 1,
            at: skein_lib::Wall::EPOCH,
            line: t::Line::RunEnded {
                attempt: 1,
                spent: 0,
                end: t::EndKind::Failed { class: t::Class::Invalid, invalid_result: Some(kind) },
            },
        };
        assert_eq!(
            through_link(&line),
            c::RunEnd::Failed { class: c::HoldFailure::Invalid, invalid_reason: Some(reason) }
        );
    }
}

#[test]
fn an_unreadable_run_end_says_why_through_the_link() {
    let end = ended(t::End::Failed(t::Class::Invalid));
    assert_eq!(
        end,
        c::RunEnd::Failed { class: c::HoldFailure::Invalid, invalid_reason: Some(c::InvalidReason::UnreadableResult) }
    );
    for (class, shown) in [
        (t::Class::Transient, c::HoldFailure::Transient),
        (t::Class::Permanent, c::HoldFailure::Permanent),
        (t::Class::Run, c::HoldFailure::Run),
        (t::Class::Agent, c::HoldFailure::Agent),
        (t::Class::Lost, c::HoldFailure::Lost),
    ] {
        assert_eq!(ended(t::End::Failed(class)), c::RunEnd::Failed { class: shown, invalid_reason: None });
    }
}
