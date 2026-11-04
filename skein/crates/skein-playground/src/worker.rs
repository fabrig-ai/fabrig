//! The worker's messages, slices and one timer (skein/docs/design/playground.md, 3, 5 and 7).

use crate::slices::{Slices, Wake};
use alloc::{boxed::Box, format, rc::Rc, vec::Vec};
use core::cell::RefCell;
use js_sys::{Array, ArrayBuffer, Uint8Array};
use skein_lib::Time;
use skein_shell::Host;
use skein_sim_world::Machine;
use skein_ui_web::bridge::{self, ToPage, ToWorld};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent, Performance};

/// The worker module's loop: posts ready, waits for the start, makes the world with
/// `make`, then runs slices on messages and on its one timer, posting each message encoded,
/// its buffer transferred; a panic posts `Failed` with its words before the module traps.
pub fn run<P: Host + 'static, M: Machine + 'static>(make: fn(&bridge::Start, Time) -> Slices<P, M>) {
    std::panic::set_hook(Box::new(|info| {
        // The hook holds no loop state: a slice can panic while the cell is borrowed.
        if let Ok(scope) = js_sys::global().dyn_into::<DedicatedWorkerGlobalScope>() {
            let failed = ToPage::Failed { words: format!("{info}").into_bytes().into() };
            let posted = post(&scope, &failed.encode());
            // A posting failure must not panic again inside the hook.
            drop(posted);
        }
    }));

    let scope = js_sys::global().dyn_into::<DedicatedWorkerGlobalScope>().expect("dedicated worker scope");
    let performance = scope.performance().expect("worker monotonic browser clock");
    let state = Rc::new(RefCell::new(State {
        scope: scope.clone(),
        performance,
        make,
        slices: None,
        posts: Vec::new(),
        timeout: None,
        timer: None,
    }));
    let timed = Rc::downgrade(&state);
    state.borrow_mut().timer = Some(Closure::new(move || {
        let state = timed.upgrade().expect("the message handler holds the worker loop");
        let mut state = state.borrow_mut();
        state.timeout = None;
        state.slice();
    }));
    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
        state.borrow_mut().message(event.data());
    });
    scope.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    // The scope holds the function; browser GC releases its state with the worker.
    let _onmessage = onmessage.into_js_value();
    post(&scope, bridge::READY).expect("post worker ready");
}

struct State<P, M> {
    scope: DedicatedWorkerGlobalScope,
    performance: Performance,
    make: fn(&bridge::Start, Time) -> Slices<P, M>,
    slices: Option<Slices<P, M>>,
    posts: Vec<ToPage>,
    timeout: Option<i32>,
    timer: Option<Closure<dyn FnMut()>>,
}

impl<P: Host, M: Machine> State<P, M> {
    fn message(&mut self, data: JsValue) {
        if let Some(handle) = self.timeout.take() {
            self.scope.clear_timeout_with_handle(handle);
        }
        let bytes = if data.is_instance_of::<ArrayBuffer>() {
            Uint8Array::new(&data)
        } else {
            data.dyn_into::<Uint8Array>().expect("worker message is a byte buffer")
        };
        let message = ToWorld::decode(&bytes.to_vec()).expect("worker message is encoded by this build");
        let browser = now(&self.performance);
        match self.slices.as_mut() {
            None => match message {
                ToWorld::Start(start) => self.slices = Some((self.make)(&start, browser)),
                ToWorld::Input(_) | ToWorld::Pace(_) => unreachable!("the first worker message is Start"),
            },
            Some(slices) => match message {
                ToWorld::Input(input) => slices.input(input),
                ToWorld::Pace(speed) => slices.pace(speed, browser),
                ToWorld::Start(_) => unreachable!("the worker receives Start only once"),
            },
        }
        self.slice();
    }

    fn slice(&mut self) {
        let browser = now(&self.performance);
        let wake = self.slices.as_mut().expect("Start made the slices").slice(browser, &mut self.posts);
        let elapsed = now(&self.performance).saturating_since(browser);
        let memory = u64::try_from(core::arch::wasm32::memory_size::<0>())
            .expect("wasm32 page count fits u64")
            .checked_mul(65_536)
            .expect("wasm32 linear memory size fits u64");
        match self.posts.last_mut().expect("a slice posts its clock") {
            ToPage::Clock(clock) => {
                clock.memory = memory;
                clock.slice = elapsed;
            }
            ToPage::Report(_) | ToPage::Printed(_) | ToPage::Stamp(_) | ToPage::Stopped | ToPage::Failed { .. } => {
                unreachable!("the clock closes a slice's posts");
            }
        }
        for message in self.posts.drain(..) {
            post(&self.scope, &message.encode()).expect("post worker message");
        }
        let delay: i32 = match wake {
            Wake::Now => 0,
            Wake::At(deadline) => {
                let nanos = deadline.saturating_since(now(&self.performance)).as_nanos();
                i32::try_from(nanos.div_ceil(1_000_000)).unwrap_or(i32::MAX)
            }
            Wake::Message => return,
        };
        self.timeout = Some(
            self.scope
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    self.timer.as_ref().expect("worker timer installed").as_ref().unchecked_ref(),
                    delay,
                )
                .expect("worker timeout"),
        );
    }
}

fn post(scope: &DedicatedWorkerGlobalScope, bytes: &[u8]) -> Result<(), JsValue> {
    let buffer = Uint8Array::from(bytes).buffer();
    scope.post_message_with_transfer(buffer.as_ref(), &Array::of1(buffer.as_ref()))
}

#[expect(
    clippy::float_arithmetic,
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "performance.now is nonnegative milliseconds; the saturating cast bounds its nanoseconds"
)]
fn now(performance: &Performance) -> Time {
    Time::from_nanos((performance.now().max(0.0) * 1_000_000.0) as u64)
}
