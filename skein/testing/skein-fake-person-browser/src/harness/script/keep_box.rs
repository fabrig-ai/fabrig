//! Keep a semantic node's viewport box (ui/web/README.md, section 5).
use crate::BrowserFace;
use skein_fake_person::{Find, Found};

pub(super) struct KeepBox {
    find: Find,
    name: Box<[u8]>,
}
impl KeepBox {
    pub(super) fn new(find: Find, name: Box<[u8]>) -> Self {
        Self { find, name }
    }
    pub(super) fn next(&self, face: &mut BrowserFace<'_>) -> bool {
        match face.find_boxed(&self.find) {
            Found::Pending => return false,
            Found::Absent => face.fail(format!("missing box {:?}", self.find)),
            Found::Node(node) => {
                if let Some(rect) = face.rect(node) {
                    face.keep_value(self.name.clone(), format!("{} {}", rect.y, rect.height).into_bytes().into());
                } else {
                    face.fail(format!("node {:?} has no box", self.find));
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::KeepBox;
    use crate::tests::{face, found, seen};
    use skein_browser::boundary::{Rect, Request};
    use skein_fake_person::Find;
    use skein_ui::Role;

    fn step() -> KeepBox {
        KeepBox::new(Find::exactly_named(Role::TextArea, b"Title"), b"line".as_slice().into())
    }
    #[test]
    fn keep_box_requests_viewport_boxes_and_keeps_signed_y_and_height() {
        let (mut face, step) = (face(), step());
        assert!(!step.next(&mut face));
        let Request::Find { query, .. } = face.requests().next().expect("box query") else { panic!("find") };
        assert!(query.boxes);
        // Supply a fresh response to the same query on a second face.
        let mut face = crate::tests::face();
        step.next(&mut face);
        let mut target = seen(b"");
        target.rect = Some(Rect { x: 10, y: -123, width: 400, height: 56 });
        found(&mut face, Some(target));
        assert!(step.next(&mut face));
        assert_eq!(face.kept(), Some((b"line".as_slice().into(), b"-123 56".as_slice().into())));
    }
    #[test]
    fn keep_box_fails_when_the_find_is_absent() {
        let (mut face, step) = (face(), step());
        step.next(&mut face);
        found(&mut face, None);
        assert!(step.next(&mut face));
        assert!(face.failure().expect("missing find").contains("missing box"));
    }
    #[test]
    fn keep_box_fails_when_the_node_has_no_box() {
        let (mut face, step) = (face(), step());
        step.next(&mut face);
        found(&mut face, Some(seen(b"")));
        assert!(step.next(&mut face));
        assert!(face.failure().expect("missing box").contains("has no box"));
    }
}
