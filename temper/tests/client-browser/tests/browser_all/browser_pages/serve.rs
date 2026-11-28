//! Hold the reading story's serve without stopping its browser (temper/docs/design/client/web/testing.md, 6).
use skein_fake_person_browser::{Play, Script, Step};
use skein_io::kernel::Signal;
use temper_client_browser_world::Tier;

pub(super) struct ReadingSteps<'a> {
    pub browser: Vec<Step>,
    tier: &'a Tier,
    #[cfg(target_os = "linux")]
    switches: std::collections::VecDeque<(u32, Signal)>,
}

impl<'a> ReadingSteps<'a> {
    pub fn new(tier: &'a Tier, browser: Vec<Step>) -> Self {
        Self {
            browser,
            tier,
            #[cfg(target_os = "linux")]
            switches: std::collections::VecDeque::new(),
        }
    }

    pub fn stop(&mut self) {
        self.switch(Signal::Stop);
    }

    pub fn resume(&mut self) {
        self.switch(Signal::Continue);
    }

    fn switch(&mut self, signal: Signal) {
        #[cfg(target_os = "linux")]
        self.switches.push_back((u32::try_from(self.browser.len()).expect("script length fits"), signal));
        #[cfg(not(target_os = "linux"))]
        self.browser.extend(pkill(self.tier, signal));
    }

    pub fn player(self) -> impl Play + use<> {
        let script = Script::new(self.browser.into_boxed_slice());
        #[cfg(target_os = "linux")]
        return linux::Reading::new(script, self.switches, self.tier);
        #[cfg(not(target_os = "linux"))]
        script
    }
}

#[cfg(not(target_os = "linux"))]
fn pkill(tier: &Tier, signal: Signal) -> [Step; 2] {
    use skein_fake_person_browser::{Command, Stdin};
    use temper_client_browser_world::{WITHIN, temper};
    let signal = match signal {
        Signal::Stop => "-STOP",
        Signal::Continue => "-CONT",
        _ => unreachable!("a reading hold stops or continues serve"),
    };
    let args = tier.box_serve();
    let args = args.iter().map(|arg| arg.to_str().expect("fixture argument")).collect::<Vec<_>>().join(" ");
    let command = format!("{} {args}", temper().display());
    let mut pattern = String::from("^");
    for ch in command.chars() {
        if ".^$*+?()[]{}|\\".contains(ch) {
            pattern.push('\\');
        }
        pattern.push(ch);
    }
    pattern.push('$');
    [
        Step::Run(Command {
            program: "/usr/bin/pkill".into(),
            args: vec![signal.into(), "-f".into(), pattern.into()],
            env: vec![],
            stdin: Stdin::Closed,
            opens: None,
            within: WITHIN,
        }),
        Step::Exited { code: 0, within: WITHIN },
    ]
}

#[cfg(target_os = "linux")]
mod linux;
