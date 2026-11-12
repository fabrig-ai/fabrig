use crate::BrowserFace;
use skein_browser::boundary::{Event, Refusal, Request};
use skein_lib::Token;
fn one(face: &mut BrowserFace<'_>) -> Request {
    let mut r = face.requests();
    let request = r.next().expect("request");
    assert!(r.next().is_none());
    request
}
#[test]
fn looks_restore_viewport_even_after_refusal_or_cancel() {
    let mut face = BrowserFace::new(Token::new(1), "example");
    face.snapshot();
    let Request::Snapshot { op, .. } = one(&mut face) else { panic!("snapshot") };
    face.event(&Event::Snapshot { op, text: b"main\n  text Hello".as_slice().into() });
    assert!(!face.pending());
    assert_eq!(face.take_snapshot().expect("snapshot").as_ref(), b"main\n  text Hello");
    assert!(face.take_snapshot().is_none());
    face.viewport(900, 700);
    let Request::Viewport { op, .. } = one(&mut face) else { panic!("viewport") };
    face.event(&Event::Done { op });
    assert_eq!(face.viewport_size(), (900, 700));
    face.screenshot_at(1000);
    let Request::Viewport { op, width: 1000, height: 700, .. } = one(&mut face) else { panic!("look keeps height") };
    face.event(&Event::Done { op });
    let Request::Screenshot { op, .. } = one(&mut face) else { panic!("PNG") };
    face.event(&Event::Screenshot { op, png: b"PNG".as_slice().into() });
    assert!(face.pending());
    assert!(face.take_screenshot().is_none());
    let Request::Viewport { op, width: 900, height: 700, .. } = one(&mut face) else { panic!("restore") };
    face.event(&Event::Done { op });
    assert!(!face.pending());
    assert_eq!(face.take_screenshot().expect("PNG").as_ref(), b"PNG");
    assert_eq!(face.viewport_size(), (900, 700));
    for cancel in [false, true] {
        face.screenshot_at(1000);
        let Request::Viewport { op, .. } = one(&mut face) else { panic!("resize") };
        face.event(&Event::Done { op });
        let Request::Screenshot { op: image, .. } = one(&mut face) else { panic!("image") };
        if cancel {
            face.cancel();
        } else {
            face.event(&Event::Refused { op: image, why: Refusal::Limit, detail: Box::new([]) });
        }
        assert!(face.pending());
        let Request::Viewport { op, width: 900, height: 700, .. } = one(&mut face) else {
            panic!("restore even on failure")
        };
        face.event(&Event::Screenshot { op: image, png: b"late".as_slice().into() });
        assert!(face.pending());
        face.event(&Event::Done { op });
        assert!(!face.pending());
        assert!(face.take_screenshot().is_none());
        assert!(face.failure().is_some());
    }
}
