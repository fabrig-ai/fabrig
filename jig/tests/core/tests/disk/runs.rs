use crate::closed_proofs;
use jig_domain_tasks as tasks;

#[test]
fn a_closed_window_starts_fresh_with_its_tail() {
    use closed_proofs as f;
    let mut world = f::parked();
    let mut control = f::parked();
    f::close(&mut world);
    f::restart(&mut world);
    f::restart(&mut control);
    world.say(87);
    control.say(87);
    f::fresh(&world);
    assert_eq!(f::tail(&world), Some(f::TAIL));
    assert_eq!(f::tries(&world), f::tries(&control));
    let mut activity = f::activity(&world);
    let Some(tasks::Line::RunBegan { resumed, .. }) = activity.last_mut() else { panic!("fresh run activity") };
    *resumed = true;
    assert_eq!(activity, f::activity(&control));
    assert!(control.assigned.as_ref().expect("control resumes").run.policy.resume);
    assert_eq!(
        control.assigned.as_ref().expect("control resumes").transcript.as_ref(),
        [f::turn(b"closed window"), f::turn(b"window continuation"),]
    );
    assert_eq!(f::tail(&control), None);
}

#[test]
fn a_closed_window_keeps_its_tail_after_a_transient_try() {
    use closed_proofs as f;
    let mut world = f::parked();
    let mut control = f::parked();
    for run in [&mut world, &mut control] {
        // The failed attempt resumed the parked window but wrote no new turn.
        run.say(87);
        run.end(tasks::End::Failed(tasks::Class::Transient), 0);
    }
    f::close(&mut world);
    f::restart(&mut world);
    f::restart(&mut control);
    world.retry_due();
    control.retry_due();
    f::fresh(&world);
    assert_eq!(f::tail(&world), Some(f::TAIL));
    assert_eq!(f::tries(&world).transient, 1);
    assert_eq!(f::tries(&world), f::tries(&control));
    assert!(control.assigned.as_ref().expect("control resumes").run.policy.resume);
    let mut activity = f::activity(&world);
    let Some(tasks::Line::RunBegan { resumed, .. }) = activity.last_mut() else { panic!("fresh run activity") };
    *resumed = true;
    assert_eq!(activity, f::activity(&control));
}

#[test]
fn a_closed_windows_unread_tail_starts_fresh_without_counting() {
    use closed_proofs as f;
    let mut world = f::parked();
    let mut control = f::parked();
    f::close(&mut world);
    f::restart(&mut world);
    f::restart(&mut control);
    world.store.unread_transcript(f::proof(&world).task);
    world.say(87);
    control.say(87);
    f::fresh(&world);
    assert_eq!(f::tail(&world), None);
    assert_eq!(f::tries(&world), f::tries(&control));
    let mut activity = f::activity(&world);
    let Some(tasks::Line::RunBegan { resumed, .. }) = activity.last_mut() else { panic!("fresh run activity") };
    *resumed = true;
    assert_eq!(activity, f::activity(&control));
    // The store deliberately keeps the closed transcript unreadable.
    world.broken();
}
