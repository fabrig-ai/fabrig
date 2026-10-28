//! Login observations in the existing accounts page.
use super::accounts::run;
use crate::{accounts::words::Word as W, words::Words};
use jig_client::contract as c;
use skein_ui::{Builder, Role, Run};

fn whole(b: &mut Builder, w: &Words, bytes: &[u8], text_per_node: u32) {
    b.run(Run::Plain, b" ");
    if bytes.len() <= usize::try_from(text_per_node / 8).expect("text bound fits") {
        b.run(Run::Plain, bytes);
    } else {
        run(b, w, W::Cut);
    }
    b.run(Run::Plain, b" ");
}
pub(super) fn failure(b: &mut Builder, w: &Words, reason: c::OwnerProgramFailure, detail: &[u8], text_per_node: u32) {
    run(
        b,
        w,
        match reason {
            c::OwnerProgramFailure::Missing => W::Missing,
            c::OwnerProgramFailure::Deadline => W::Deadline,
            c::OwnerProgramFailure::Unreadable => W::Unreadable,
            c::OwnerProgramFailure::Exited => W::Exited,
            c::OwnerProgramFailure::Unconfigured => W::Unconfigured,
            c::OwnerProgramFailure::Unknown => W::Unknown,
            c::OwnerProgramFailure::Closed => W::Closing,
        },
    );
    whole(b, w, detail, text_per_node);
}
pub(super) fn failures(b: &mut Builder, w: &Words, hosts: &[c::AccountHostFailure], text_per_node: u32) {
    // The contract caps the retained answer at 128 configured workers.
    for host in hosts {
        b.open(Role::Paragraph);
        b.open(Role::Text);
        whole(b, w, &host.name, text_per_node);
        run(
            b,
            w,
            match host.why {
                c::AccountHostWhy::NotInContact => W::NotInContact,
                c::AccountHostWhy::Lost => W::Lost,
                c::AccountHostWhy::Failed => W::Failed,
            },
        );
        b.close();
        b.close();
    }
}
