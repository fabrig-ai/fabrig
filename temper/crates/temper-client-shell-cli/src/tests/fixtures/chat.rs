use super::*;

pub(super) struct Read {
    activity: w::ActivityPage,
    run: w::RunState,
}

impl Frame {
    pub fn script_chat_read(&mut self, activity: w::ActivityPage, run: w::RunState) {
        self.chat = Some(Read { activity, run });
    }

    pub(super) fn read_chat_activity(&mut self, fd: Fd) {
        let chat = self.chat.as_ref().expect("scripted chat");
        let request = &self.sockets.get(&fd).expect("activity socket").request;
        let end = skein_lib::bytes::find(request, b"\r\n\r\n").expect("HTTP head");
        let body = request.get(end..).expect("HTTP head end").strip_prefix(b"\r\n\r\n").expect("HTTP separator");
        let owned = w::Owned::decode(&w::CEILINGS, &mut skein_lib::Reader::new(body)).expect("owned read");
        assert_eq!(owned.owner, w::Owner::Jig);
        let read = w::Read::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&owned.record)).expect("activity read");
        assert_eq!(read, w::Read::Activity(w::ReadActivity { task: chat.run.task, before: None }));
        let value = w::ReadValue::Activity(chat.activity.clone());
        let document = w::Owned { owner: w::Owner::Jig, version: w::VERSION, record: encode!(value) };
        self.reply(fd, &encode!(w::ReadAnswer::Value(document)));
    }

    pub(super) fn deliver_chat_run(&mut self, watch: u64, subject: &w::Subject) {
        let run = &self.chat.as_ref().expect("scripted chat").run;
        assert_eq!(*subject, w::Subject::Run(w::SubjectRun { task: run.task, turn: None, offset: 0 }));
        let value = w::DeliveryValue::RunState(run.clone());
        let document = w::Owned { owner: w::Owner::Jig, version: w::VERSION, record: encode!(value) };
        let event = w::StreamEvent::Delivery(w::Delivery { watch, document });
        self.sockets.get_mut(&self.stream.expect("stream")).expect("stream socket").reply.extend(sse(&event));
    }
}
