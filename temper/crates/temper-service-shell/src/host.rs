//! The host drives the binary's service table (temper/docs/design/protocol/services.md, 7).
//! Start refusals name their cause (temper/docs/design/protocol/binaries.md).
use crate::Served;
use skein_io::kernel::{Complete, Exit, Submit};
use skein_lib::{Queue, Time, Wall};
use std::io::{self, Write};
use temper_service_core::long_pass::{LONG_PASS, LongPass, Part, Watch};
use temper_shell::directories::Directories;
use temper_shell::directories::EnvironmentEntry;

pub struct Host<S> {
    pub service: S,
    served: Served<S>,
    binary: String,
    directories: Directories,
    pub(super) store: Box<[u8]>,
    pub(super) link_file: Option<std::path::PathBuf>,
    output: Box<dyn Write>,
    output_failed: bool,
    wait_check: bool,
    cut_at_listen: Option<std::path::PathBuf>,
    now: Time,
    watch: Watch,
    clock: skein_shell::Clock,
}
impl<S> Host<S> {
    #[must_use]
    pub fn new(service: S, served: Served<S>, directories: Directories) -> Self {
        Self::with_binary("temper", service, served, directories, &[])
    }
    #[must_use]
    pub fn with_binary(
        binary: &str,
        service: S,
        served: Served<S>,
        directories: Directories,
        environment: &[EnvironmentEntry],
    ) -> Self {
        let store = directories.data.clone();
        Self {
            service,
            served,
            binary: binary.into(),
            directories,
            store,
            link_file: None,
            output: Box::new(io::stdout()),
            output_failed: false,
            // QA launches sanitized child environments. Test/debug programs
            // always check; release programs can opt in with the same flag.
            wait_check: wait_check(environment, cfg!(debug_assertions)),
            cut_at_listen: super::restart_fault::marker(environment),
            now: Time::ZERO,
            watch: Watch::new(),
            clock: skein_shell::Clock::new(),
        }
    }
    fn ending(&self) -> (String, u8) {
        (self.served.ending)(&self.service, &self.binary, self.output_failed, &self.directories, &self.store)
    }
    pub(super) fn keep_link(&self, url: &[u8]) {
        if let Some(path) = &self.link_file
            && let Err(why) = super::link_file::keep(path, url, skein_shell::effective_user())
        {
            eprintln!("{} serve: link file {why}", self.binary);
        }
    }
}

pub(crate) fn wait_check(environment: &[EnvironmentEntry], debug: bool) -> bool {
    debug || crate::WAIT_CHECK.value(environment).is_some()
}

impl<S> skein_shell::Host for Host<S> {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.now = now;
        self.watch.begin(now);
        for &stage in self.served.stages {
            (self.served.iterate_stage)(&mut self.service, stage, now, wall);
            self.watch.ended(Part::Service(stage), self.clock.now().now);
        }
        (self.served.resolve)(&mut self.service);
        self.watch.ended(Part::Names, self.clock.now().now);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        (self.served.completions)(&mut self.service)
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        (self.served.submissions)(&mut self.service)
    }
    fn work_pending(&self, now: Time) -> bool {
        (self.served.work_pending)(&self.service, now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        (self.served.next_mechanism_deadline)(&self.service)
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        (self.served.next_policy_deadline)(&self.service)
    }
    fn is_empty(&self) -> bool {
        (self.served.is_empty)(&self.service)
    }
    fn operations(&self) -> u32 {
        (self.served.operations)(&self.service)
    }
    fn exit(&self) -> Option<Exit> {
        if !(self.served.is_empty)(&self.service) {
            return None;
        }
        if let Some(long) = self.watch.held() {
            eprint!("{}", long_pass_line(&self.binary, long));
        }
        let (lines, code) = self.ending();
        eprint!("{lines}");
        Some(Exit::Code(code))
    }
    fn drain(&mut self) {
        self.drain_with(&mut io::stderr());
    }
}

impl<S> Host<S> {
    fn drain_with(&mut self, diagnostics: &mut dyn Write) {
        (self.served.handover)(&mut self.service);
        while let Some((lines, clients)) = (self.served.listening)(&mut self.service, &self.binary) {
            diagnostics.write_all(lines.as_bytes()).expect("printing to stderr failed");
            if self.wait_check && clients {
                super::restart_fault::cut_at_listen(self.cut_at_listen.as_deref());
            }
        }
        while let Some(lines) = (self.served.note)(&mut self.service, &self.binary) {
            diagnostics.write_all(lines.as_bytes()).expect("printing to stderr failed");
        }
        while let Some(link) = (self.served.link)(&mut self.service) {
            if self
                .output
                .write_all(&link)
                .and_then(|()| self.output.write_all(b"\n"))
                .and_then(|()| self.output.flush())
                .is_err()
            {
                self.output_failed = true;
                (self.served.shutdown)(&mut self.service);
            }
        }
        while let Some(link) = (self.served.local_link)(&mut self.service) {
            self.keep_link(&link);
        }
        if self.wait_check {
            (self.served.check_wait_boundary)(&self.service, self.now);
        }
        self.watch.ended(Part::Drain, self.clock.now().now);
        if let Some(long) = self.watch.pass_ended() {
            diagnostics.write_all(long_pass_line(&self.binary, long).as_bytes()).expect("printing to stderr failed");
        }
    }
}

fn long_pass_line(binary: &str, long: LongPass) -> String {
    use std::fmt::Write as _;
    let mut line = format!(
        "{binary} serve: long pass: {} ms, {} {} ms of it, past the {} ms bound",
        long.took.as_millis(),
        long.part.name(),
        long.part_took.as_millis(),
        LONG_PASS.as_millis(),
    );
    if long.more != 0 {
        write!(line, "; {} more since the last line", long.more).expect("format long pass count");
    }
    line.push('\n');
    line
}
#[test]
fn long_pass_line_names_the_pass_its_longest_part_and_the_bound() {
    let long = LongPass {
        took: skein_lib::Duration::from_millis(1250),
        part: Part::Service(temper_service_core::Stage::Engine),
        part_took: skein_lib::Duration::from_millis(1200),
        more: 0,
    };
    assert_eq!(
        long_pass_line("temper", long),
        "temper serve: long pass: 1250 ms, engine 1200 ms of it, past the 1000 ms bound\n"
    );
}

#[test]
fn long_pass_line_counts_the_other_long_passes() {
    let long = LongPass {
        took: skein_lib::Duration::from_millis(1250),
        part: Part::Service(temper_service_core::Stage::Engine),
        part_took: skein_lib::Duration::from_millis(1200),
        more: 3,
    };
    assert_eq!(
        long_pass_line("temper", long),
        "temper serve: long pass: 1250 ms, engine 1200 ms of it, past the 1000 ms bound; 3 more since the last line\n"
    );
}
#[cfg(test)]
pub(super) mod adapter_tests {
    use super::*;
    use skein_shell::Host as _;

    pub(crate) struct OneStage {
        stages: u32,
        at: Option<Time>,
        stopped: bool,
        completions: Queue<Complete>,
        submissions: Queue<Submit>,
        links: Queue<Box<[u8]>>,
        notes: Queue<&'static str>,
    }
    impl OneStage {
        fn new() -> Self {
            Self {
                stages: 0,
                at: None,
                stopped: false,
                completions: Queue::uncapped(),
                submissions: Queue::uncapped(),
                links: Queue::uncapped(),
                notes: Queue::uncapped(),
            }
        }
    }
    pub(crate) const SERVED: Served<OneStage> = Served {
        stages: &[temper_service_core::Stage::Route],
        iterate_stage: |service, stage, now, wall| {
            assert_eq!(stage, temper_service_core::Stage::Route);
            service.at = Some(now);
            assert_eq!(wall, Wall::EPOCH);
            service.stages += 1;
        },
        resolve: |_| {},
        completions: |service| &mut service.completions,
        submissions: |service| &mut service.submissions,
        operations: |_| 7,
        work_pending: |service, _| service.stages == 0,
        next_mechanism_deadline: |_| Some(Time::ZERO),
        next_policy_deadline: |_| None,
        is_empty: |service| service.stages == 1,
        shutdown: |service| service.stopped = true,
        ending: |service, binary, failed, _, store| {
            assert_eq!(service.stages, 1);
            assert_eq!(binary, "own-service");
            assert_eq!(store, b"/d");
            (format!("{binary}: one stage ended\n"), u8::from(failed))
        },
        handover: |_| {},
        listening: |_, _| None,
        note: |service, binary| service.notes.pop().map(|note| format!("{binary} serve: {note}\n")),
        link: |service| service.links.pop(),
        local_link: |_| None,
        check_wait_boundary: |service, _| assert_eq!(service.stages, 1),
        adopt_listener: |_, _, _| Ok(()),
        adopt_signals: |_, _| Ok(()),
    };
    fn host() -> Host<OneStage> {
        let arguments = ["--config", "/c", "--data", "/d", "--cache", "/k"].map(|word| Box::from(word.as_bytes()));
        let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("directories");
        Host::with_binary("own-service", OneStage::new(), SERVED, directories, &[])
    }
    #[test]
    fn supplied_service_stage_and_ending_drive_the_host() {
        let mut host = host();
        assert_eq!(host.exit(), None);
        assert!(host.work_pending(Time::ZERO));
        assert_eq!(host.operations(), 7);
        assert_eq!(host.next_mechanism_deadline(), Some(Time::ZERO));
        assert_eq!(host.next_policy_deadline(), None);
        assert!(host.completions().is_empty());
        assert!(host.submissions().is_empty());
        let now = host.clock.now().now;
        host.iterate(now, Wall::EPOCH);
        host.drain();
        assert_eq!(host.service.at, Some(now));
        assert!(host.is_empty());
        assert!(!host.work_pending(Time::ZERO));
        assert_eq!(host.ending(), ("own-service: one stage ended\n".into(), 0));
        assert_eq!(host.exit(), Some(Exit::Code(0)));
    }
    #[test]
    fn supplied_notes_print_in_order_under_the_binarys_prefix() {
        let mut host = host();
        host.service.notes.push("first fixed line");
        host.service.notes.push("second fixed line");
        let now = host.clock.now().now;
        host.iterate(now, Wall::EPOCH);
        let mut diagnostics = Vec::new();
        host.drain_with(&mut diagnostics);
        assert_eq!(diagnostics, b"own-service serve: first fixed line\nown-service serve: second fixed line\n");
        assert!(host.service.notes.is_empty());
    }

    struct BrokenOutput;
    impl Write for BrokenOutput {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn output_failure_shuts_down_the_supplied_service() {
        let mut host = host();
        host.output = Box::new(BrokenOutput);
        host.service.links.push(b"a link".as_slice().into());
        let now = host.clock.now().now;
        host.iterate(now, Wall::EPOCH);
        host.drain();
        assert!(host.service.stopped, "failed output must ask the supplied service to shut down");
        assert_eq!(host.exit(), Some(Exit::Code(1)));
    }
}
