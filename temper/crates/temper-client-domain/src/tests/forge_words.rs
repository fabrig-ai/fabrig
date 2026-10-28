use crate::forge::words::{self, ResourceWord};
use skein_lib::List;
use skein_ui::phrases::{Hole, HoleKind};
const SEGMENTS: u32 = 3;
const PATH_BYTES: u32 = 128;
fn check(segments: &[&[u8]], expected: ResourceWord, name: Option<&[u8]>, numbers: &[u64]) {
    let mut path = List::with_capacity(SEGMENTS);
    for segment in segments {
        path.push(Box::from(*segment)).expect("three segments");
    }
    let (word, holes) = words::path(path.as_slice(), PATH_BYTES);
    assert_eq!(word, expected);
    let mut number = 0_usize;
    let mut names = 0_usize;
    for hole in &holes {
        match hole {
            Hole::Name(bytes) => {
                assert_eq!(Some(bytes.as_ref()), name);
                names = names.checked_add(1).expect("one name");
            }
            Hole::Number(value) => {
                assert_eq!(Some(value), numbers.get(number));
                number = number.checked_add(1).expect("two numbers");
            }
            Hole::Amount(_) | Hole::Span(_) | Hole::Wall(_) | Hole::Reference { .. } => panic!("resource hole"),
        }
    }
    assert_eq!(number, numbers.len());
    assert_eq!(names, usize::from(name.is_some()));
}
#[test]
fn an_adopted_branch_decodes_the_numbers_and_keeps_its_name() {
    check(
        &[&257_u16.to_be_bytes(), &65537_u32.to_be_bytes(), b"temper/1/c2"],
        ResourceWord::AdoptedBranch,
        Some(b"temper/1/c2"),
        &[65537, 257],
    );
    assert_eq!(
        words::resource_english(ResourceWord::AdoptedBranch),
        (
            [b"branch ".as_slice(), b" of repository ", b" on forge ", b""].as_slice(),
            [HoleKind::Name, HoleKind::Number, HoleKind::Number].as_slice()
        )
    );
}
#[test]
fn an_adopted_repository_still_decodes_its_two_segments() {
    check(&[&257_u16.to_be_bytes(), &65537_u32.to_be_bytes()], ResourceWord::Adopted, None, &[65537, 257]);
}
#[test]
fn wrong_sized_branch_ids_fall_back_to_joined_segments() {
    check(&[b"bad", b"size", b"branch"], ResourceWord::Path, Some(b"bad/size/branch"), &[]);
    check(&[b"ok", b"bad", b"branch"], ResourceWord::Path, Some(b"ok/bad/branch"), &[]);
}
