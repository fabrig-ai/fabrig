//! Order of the key against a chat's work (temper/docs/design/client/tui/box.md, 7 and 11).
use super::{BoxStage, Host, Key, Report, Told, admit, delivery, give, records, signed_host, text, w, watch};
use crate::{
    host::Stop,
    testing::quit::{CHAT, PROJECT, Read},
};
use temper_client_domain::{Address, pages::Act};
use temper_client_tui::Event;

struct Front {
    host: Host,
    watch: u64,
    activity: u64,
}

impl Front {
    fn with_chat() -> Self {
        let (mut host, _, _) = signed_host(true);
        host.tell(Told::Front { set_up: false, box_log: Box::default() });
        host.report(Report::Stage(BoxStage::Running));
        let duties = give(&mut host, Event::Act { act: Act::Go { address: Address::Chat { task: CHAT } } });
        let duties = admit(&mut host, duties);
        let watch = watch(&duties, &w::Subject::Task(w::SubjectTask { task: CHAT }));
        Self { host, watch, activity: 7 }
    }

    fn read(&mut self, read: Read) {
        self.activity += 1;
        let (facts, waits_on) = read.facts(records::wire_owner_chip_details_row());
        let mut task = records::task_record();
        task.project = PROJECT;
        task.chip = records::chip_from_rows(&facts);
        task.waits_on = waits_on;
        task.last_activity = self.activity;
        let duties = delivery(&mut self.host, self.watch, &w::DeliveryValue::Task(Box::new(task)));
        assert_eq!(duties.stop, None, "{read:?} delivery must not stop the box");
        assert!(!duties.close);
        let root = &self.host.root;
        let read_task = root.pages().chat().expect("open chat").task(root.client()).expect("read chat");
        assert_eq!(read_task.last_activity.as_nanos(), self.activity, "fresh {read:?} read");
        self.host.frame();
    }

    fn key_asks(&mut self) {
        let duties = self.host.report(Report::Key(Key::Control(b'c')));
        assert!(!duties.close);
        assert_eq!(duties.stop, None);
        assert_eq!(self.host.state().box_stage(), &BoxStage::Running);
        assert!(self.host.state().quitting());
    }

    fn key_stops(&mut self) {
        let duties = self.host.report(Report::Key(Key::Control(b'c')));
        assert_eq!(duties.stop, Some(Stop::First));
        assert!(!duties.close);
        assert_eq!(self.host.state().box_stage(), &BoxStage::Stopping);
        assert!(!self.host.state().quitting());
    }

    fn head_is(&mut self, expected: &str) {
        let frame = text(&mut self.host);
        assert_eq!(frame.lines().map(str::trim).find(|line| line.starts_with("Stop the box?")), Some(expected));
    }
}

#[test]
fn work_ends_before_ctrl_c_and_the_box_stops_at_once() {
    let mut front = Front::with_chat();
    front.read(Read::Working);
    front.read(Read::Parked);
    front.key_stops();
}

#[test]
fn ctrl_c_names_the_chat_parked_on_a_live_delegate() {
    let mut front = Front::with_chat();
    front.read(Read::ParkedOnDelegate);
    front.key_asks();
    front.head_is("Stop the box? csv-export's running task stops with it.");
}

#[test]
fn ctrl_c_before_work_ends_leaves_the_question_standing() {
    let mut front = Front::with_chat();
    front.read(Read::Working);
    front.key_asks();
    front.head_is("Stop the box? csv-export's running task stops with it.");
    front.read(Read::Parked);
    assert!(front.host.state().quitting());
    front.head_is("Stop the box?");
    front.key_stops();
}
