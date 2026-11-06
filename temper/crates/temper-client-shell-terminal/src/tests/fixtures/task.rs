use super::*;
use skein_ui::{Role, Run};
fn text(b: &mut skein_ui::Builder, role: Role, words: &[u8]) {
    b.open(role);
    b.text(Run::Plain, words);
    b.close();
}
fn region(b: &mut skein_ui::Builder, name: &[u8]) {
    b.open(Role::Region);
    b.name(name);
    text(b, Role::Heading { level: 2 }, name);
}
/// A goal's header, needs, report, side and composer (client/README.md, 5.8).
pub fn page() -> skein_ui::Tree {
    use temper_client_view::look;
    let mut b = skein_ui::Builder::new(&tree_limits());
    synthetic_frame(&mut b, false);
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::GOAL);
    b.open(Role::Banner);
    b.style(look::GOAL_HEAD);
    text(
        &mut b,
        Role::Paragraph,
        "Refresh token handling · Needs you · 1 of 3 tasks · 7 of 500 spent · since 00:00".as_bytes(),
    );
    text(&mut b, Role::Heading { level: 1 }, b"Tokens rotate safely; the review needs you.");
    text(&mut b, Role::Paragraph, b"Written by its agent at 00:00.");
    b.close();
    b.open(Role::Region);
    b.name(b"Needs you");
    b.style(look::GOAL_BAND);
    text(&mut b, Role::Paragraph, "Needs you · held · The review needs attention.".as_bytes());
    b.open(Role::Link);
    b.name(b"Open");
    b.bind(binding());
    b.close();
    b.close();
    b.open(Role::Slot);
    b.style(look::GOAL_TWO);
    region(&mut b, b"Report");
    b.style(look::GOAL_REPORT);
    text(&mut b, Role::Paragraph, b"Token rotation is implemented. The review is held.");
    text(&mut b, Role::Paragraph, b"Left: Review the change and write the release notes.");
    text(&mut b, Role::Paragraph, b"Risk: The review has not finished.");
    text(&mut b, Role::Paragraph, b"By its agent at 00:00.");
    b.close();
    b.open(Role::Complementary);
    b.style(look::GOAL_SIDE);
    goal_tasks(&mut b);
    for (name, facts) in [
        (b"Landing".as_slice(), b"Each task lands on its own once checked.".as_slice()),
        (b"How it runs", "Task by task · always asks you".as_bytes()),
        (b"Decided by the goal", b"Rotate credentials in place"),
    ] {
        region(&mut b, name);
        b.style(look::GOAL_SECTION);
        text(&mut b, Role::Paragraph, facts);
        b.close();
    }
    b.close();
    b.close();
    b.open(Role::Slot);
    b.style(look::GOAL_COMPOSER_BOX);
    b.open(Role::Slot);
    b.style(look::GOAL_COMPOSER);
    b.open(Role::TextArea);
    b.name(b"Tell Refresh token handling something");
    b.value(b"", 0);
    b.bind(binding());
    b.close();
    b.open(Role::Button);
    b.name(b"Send");
    b.state(skein_ui::State::Disabled);
    b.bind(binding());
    b.close();
    b.close();
    text(
        &mut b,
        Role::Paragraph,
        b"Its agent reads it at once. A change of plan becomes an amendment it shows you first.",
    );
    b.close();
    b.close();
    b.close();
    b.finish()
}

fn goal_tasks(b: &mut skein_ui::Builder) {
    use temper_client_view::look;
    region(b, b"Tasks");
    b.style(look::GOAL_SECTION);
    b.open(Role::Image);
    b.style(look::GOAL_BAR);
    b.name(b"1 landed of 3");
    for landed in [true, false, false] {
        b.open(Role::Text);
        if landed {
            b.style(look::GOAL_BAR_LANDED);
        }
        b.close();
    }
    b.close();
    for (title, state) in [
        (b"Review the change".as_slice(), b"waits on you".as_slice()),
        (b"Write the release notes", b"waits on Review the change"),
    ] {
        b.open(Role::Paragraph);
        b.style(look::GOAL_LINE);
        task_link(b, title);
        b.text(Run::Plain, " · ".as_bytes());
        text(b, Role::Text, state);
        b.close();
    }
    b.open(Role::Disclosure);
    b.style(look::GOAL_LANDED_FOLD);
    b.name(b"1 landed");
    b.open(Role::Paragraph);
    b.style(look::GOAL_LINE);
    task_link(b, b"Advise on the change");
    b.text(Run::Plain, " · ".as_bytes());
    text(b, Role::Text, b"00:00");
    b.close();
    b.close();
    b.close();
}

/// A live delegate's brief and conversation (client/README.md, 5.9).
#[expect(clippy::too_many_lines, reason = "one synthetic page keeps the terminal agent layout together")]
pub fn agent_task_page() -> skein_ui::Tree {
    use temper_client_view::look;
    let mut b = skein_ui::Builder::new(&tree_limits());
    synthetic_frame(&mut b, false);
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::AGENT_PAGE);
    b.open(Role::Slot);
    b.style(look::AGENT_LINEAGE);
    b.open(Role::Slot);
    task_link(&mut b, b"Task 12");
    text(&mut b, Role::Paragraph, b"Rotate the credentials");
    b.close();
    b.open(Role::Slot);
    text(&mut b, Role::Paragraph, b"Task 34");
    text(&mut b, Role::Paragraph, b"Refresh token handling");
    b.close();
    b.close();
    b.open(Role::Slot);
    b.style(look::TASKBAR);
    b.open(Role::Slot);
    text(&mut b, Role::Heading { level: 1 }, "T34 · Refresh token handling".as_bytes());
    b.open(Role::Status);
    b.style(look::STATUS_ACTIVE);
    b.text(Run::Plain, "running · writing".as_bytes());
    b.close();
    b.close();
    b.open(Role::Slot);
    text(&mut b, Role::Paragraph, b"Agent");
    text(&mut b, Role::Paragraph, b"7 spent");
    b.open(Role::Button);
    b.name(b"Amend");
    b.bind(binding());
    b.close();
    b.close();
    b.close();
    b.open(Role::Slot);
    b.style(look::THREAD);
    region(&mut b, b"Task brief");
    b.style(look::AGENT_BRIEF);
    skein_ui::markdown::read(b"Rotate tokens **safely**.", 3, &mut b);
    text(&mut b, Role::Paragraph, b"Asked by task 12");
    b.open(Role::List);
    b.name(b"Dependencies of task 34");
    b.style(look::DEPENDENCIES);
    b.open(Role::Item);
    b.key(skein_ui::Key { owner: temper_client_view::frame::owner(), kind: 0, a: 17, b: 0 });
    task_link(&mut b, b"Task 17");
    b.open(Role::Status);
    b.style(look::STATUS_DONE);
    b.text(Run::Plain, b"Completed");
    b.close();
    b.close();
    b.close();
    b.close();
    b.open(Role::Slot);
    b.style(look::AGENT_RUN);
    text(&mut b, Role::Heading { level: 2 }, "Run 1 · live now".as_bytes());
    text(&mut b, Role::Paragraph, b"Started 10-22 03:48Z");
    b.close();
    b.open(Role::Log);
    b.name(b"Conversation");
    b.open(Role::Region);
    b.style(look::TURN);
    b.open(Role::Slot);
    b.style(look::MESSAGE);
    b.style(look::BY_AGENT);
    b.open(Role::Slot);
    b.style(look::WHO);
    text(&mut b, Role::Paragraph, b"Temper");
    b.close();
    b.open(Role::Slot);
    b.style(look::SAYS);
    skein_ui::markdown::read(b"Checking the credentials.", 3, &mut b);
    b.close();
    b.close();
    b.close();
    b.open(Role::Region);
    b.style(look::TURN);
    b.open(Role::Disclosure);
    b.style(look::WORK);
    b.style(look::TURN_TOOL);
    b.style(look::WORK_RUN);
    b.name(b"read_file");
    b.open(Role::Slot);
    b.style(look::WORK);
    b.style(look::WORK_CALLS);
    b.open(Role::Paragraph);
    b.name(b"read_file: Read token.rs");
    b.text(Run::Code, b"read_file");
    b.open(Role::Text);
    b.style(look::WORK_OUT);
    b.run(Run::Plain, b": Read token.rs");
    b.close();
    b.close();
    b.close();
    b.close();
    b.close();
    b.close();
    b.close();
    b.close();
    b.close();
    b.finish()
}
fn task_link(b: &mut skein_ui::Builder, name: &[u8]) {
    b.open(Role::Link);
    b.name(name);
    b.text(Run::Plain, name);
    b.bind(binding());
    b.close();
}
