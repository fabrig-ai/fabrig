//! Browser push glue (temper/docs/design/client/web/README.md, 2.7).
//! Permission is asked synchronously in carry, before a promise is observed.
use crate::Report as ShellReport;
use js_sys::{Promise, Reflect, Uint8Array};
use skein_ui_web::inbox::Inbox;
use std::rc::Rc;
use temper_client_domain::pages::told::{Browser, Duty, Permission, Refusal, Report, Subscription};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{
    Notification, NotificationPermission, PushEncryptionKeyName, PushSubscription, PushSubscriptionOptionsInit,
    ServiceWorkerRegistration,
};

type Mail = Rc<Inbox<ShellReport>>;
fn report(inbox: &Mail, report: Report) {
    inbox.raise(ShellReport::Push(report));
}
fn capable() -> bool {
    let window = web_sys::window().expect("window");
    Reflect::has(&window, &"PushManager".into()).unwrap_or(false)
        && Reflect::has(&window.navigator(), &"serviceWorker".into()).unwrap_or(false)
        && Reflect::has(&window, &"Notification".into()).unwrap_or(false)
}
fn browser(endpoint: Option<Box<[u8]>>) -> Browser {
    let capable = capable();
    let permission = if !capable {
        Permission::Ask
    } else if Notification::permission() == NotificationPermission::Denied {
        Permission::Denied
    } else if Notification::permission() == NotificationPermission::Granted {
        Permission::Granted
    } else {
        Permission::Ask
    };
    Browser {
        capable,
        permission,
        endpoint,
        user_agent: web_sys::window().expect("window").navigator().user_agent().unwrap_or_default().into_bytes().into(),
    }
}
/// Add the manifest at shell start: the bundler's default page has no manifest input.
pub fn manifest() {
    let document = web_sys::window().expect("window").document().expect("document");
    if document.query_selector("link[rel=manifest]").expect("manifest selector").is_none() {
        let link = document.create_element("link").expect("manifest link");
        link.set_attribute("rel", "manifest").expect("manifest relation");
        link.set_attribute("href", "/manifest.webmanifest").expect("manifest address");
        document.head().expect("head").append_child(&link).expect("append manifest");
    }
}
/// Inspect without registering or prompting. An absent registration has no subscription.
pub fn state(inbox: Mail) {
    if !capable() {
        report(&inbox, Report::State(browser(None)));
        return;
    }
    let promise = web_sys::window().expect("window").navigator().service_worker().get_registration();
    listen(
        &promise,
        move |value| {
            let Ok(registration) = value.dyn_into::<ServiceWorkerRegistration>() else {
                report(&inbox, Report::State(browser(None)));
                return;
            };
            let Ok(manager) = registration.push_manager() else {
                let mut unavailable = browser(None);
                unavailable.capable = false;
                report(&inbox, Report::State(unavailable));
                return;
            };
            // A failed read says nothing new about the subscription: keep the last report.
            let Ok(promise) = manager.get_subscription() else { return };
            let inbox = Rc::clone(&inbox);
            listen(
                &promise,
                move |value| {
                    let endpoint = value
                        .dyn_into::<PushSubscription>()
                        .ok()
                        .map(|subscription| subscription.endpoint().into_bytes().into_boxed_slice());
                    report(&inbox, Report::State(browser(endpoint)));
                },
                |_| {},
            );
        },
        |_| {},
    );
}
pub fn carry(inbox: Mail, duty: Duty) {
    match duty {
        Duty::Subscribe { token, key } => {
            if !capable() {
                fail(&inbox, token, Refusal::Failed);
                return;
            }
            // This call is in Dom's press -> Inbox::raise -> Shell::carry, with no wait.
            let Ok(permission) = Notification::request_permission() else {
                fail(&inbox, token, Refusal::Blocked);
                return;
            };
            let failed = Rc::clone(&inbox);
            listen(
                &permission,
                move |value| {
                    if value.as_string().as_deref() == Some("granted") {
                        register(Rc::clone(&inbox), token, key);
                    } else {
                        fail(&inbox, token, Refusal::Blocked);
                    }
                },
                move |_| fail(&failed, token, Refusal::Blocked),
            );
        }
        Duty::Drop { token } => drop_subscription(inbox, token),
    }
}
fn register(inbox: Mail, token: u64, key: [u8; 65]) {
    let workers = web_sys::window().expect("window").navigator().service_worker();
    let promise = workers.register("/sw.js");
    let failed = Rc::clone(&inbox);
    listen(
        &promise,
        move |_| {
            let Ok(ready) = workers.ready() else {
                fail(&inbox, token, Refusal::Failed);
                return;
            };
            let failed = Rc::clone(&inbox);
            let inbox = Rc::clone(&inbox);
            listen(
                &ready,
                move |value| {
                    let Ok(registration) = value.dyn_into::<ServiceWorkerRegistration>() else {
                        fail(&inbox, token, Refusal::Failed);
                        return;
                    };
                    subscribe(Rc::clone(&inbox), token, key, &registration);
                },
                move |_| fail(&failed, token, Refusal::Failed),
            );
        },
        move |_| fail(&failed, token, Refusal::Failed),
    );
}
fn subscribe(inbox: Mail, token: u64, key: [u8; 65], registration: &ServiceWorkerRegistration) {
    let Ok(manager) = registration.push_manager() else {
        fail(&inbox, token, Refusal::Failed);
        return;
    };
    let options = PushSubscriptionOptionsInit::new();
    options.set_user_visible_only(true);
    let key = Uint8Array::from(key.as_slice());
    if Reflect::set(options.as_ref(), &"applicationServerKey".into(), &key).is_err() {
        fail(&inbox, token, Refusal::Failed);
        return;
    }
    let Ok(promise) = manager.subscribe_with_options(&options) else {
        fail(&inbox, token, Refusal::Failed);
        return;
    };
    let failed = Rc::clone(&inbox);
    listen(
        &promise,
        move |value| {
            let subscription = value.dyn_into::<PushSubscription>().ok().and_then(|subscription| {
                Some(Subscription {
                    endpoint: subscription.endpoint().into_bytes().into_boxed_slice(),
                    p256dh: Uint8Array::new(&subscription.get_key(PushEncryptionKeyName::P256dh).ok()??.into())
                        .to_vec()
                        .try_into()
                        .ok()?,
                    auth: Uint8Array::new(&subscription.get_key(PushEncryptionKeyName::Auth).ok()??.into())
                        .to_vec()
                        .try_into()
                        .ok()?,
                })
            });
            if let Some(subscription) = subscription {
                report(&inbox, Report::Subscribed { token, subscription });
                state(Rc::clone(&inbox));
            } else {
                fail(&inbox, token, Refusal::Malformed);
            }
        },
        move |_| fail(&failed, token, Refusal::Failed),
    );
}
fn drop_subscription(inbox: Mail, token: u64) {
    if !capable() {
        fail(&inbox, token, Refusal::Failed);
        return;
    }
    let promise = web_sys::window().expect("window").navigator().service_worker().get_registration();
    let failed = Rc::clone(&inbox);
    listen(
        &promise,
        move |value| {
            let Ok(registration) = value.dyn_into::<ServiceWorkerRegistration>() else {
                dropped(&inbox, token);
                return;
            };
            let Ok(manager) = registration.push_manager() else {
                fail(&inbox, token, Refusal::Failed);
                return;
            };
            let Ok(promise) = manager.get_subscription() else {
                fail(&inbox, token, Refusal::Failed);
                return;
            };
            let failed = Rc::clone(&inbox);
            let inbox = Rc::clone(&inbox);
            listen(
                &promise,
                move |value| {
                    let Ok(subscription) = value.dyn_into::<PushSubscription>() else {
                        dropped(&inbox, token);
                        return;
                    };
                    let Ok(promise) = subscription.unsubscribe() else {
                        fail(&inbox, token, Refusal::Failed);
                        return;
                    };
                    let failed = Rc::clone(&inbox);
                    let inbox = Rc::clone(&inbox);
                    listen(&promise, move |_| dropped(&inbox, token), move |_| fail(&failed, token, Refusal::Failed));
                },
                move |_| fail(&failed, token, Refusal::Failed),
            );
        },
        move |_| fail(&failed, token, Refusal::Failed),
    );
}
fn dropped(inbox: &Mail, token: u64) {
    report(inbox, Report::Dropped { token });
    state(Rc::clone(inbox));
}
fn fail(inbox: &Mail, token: u64, why: Refusal) {
    report(inbox, Report::Refused { token, why });
    state(Rc::clone(inbox));
}
fn listen(promise: &Promise, resolved: impl FnMut(JsValue) + 'static, rejected: impl FnMut(JsValue) + 'static) {
    let resolve = Closure::<dyn FnMut(JsValue)>::new(resolved);
    let reject = Closure::<dyn FnMut(JsValue)>::new(rejected);
    let _next = promise.then2(&resolve, &reject);
    let _resolve = resolve.into_js_value();
    let _reject = reject.into_js_value();
}
