use crate::names::Names;

#[test]
fn names_are_bijective_and_refusals_leave_both_directions_intact() {
    let mut names = Names::new();
    assert_eq!(names.task("g1"), None);
    assert_eq!(names.logical(7), None);
    names.name("g1", 7).unwrap();
    names.name("g1", 7).expect("the same pair is idempotent");
    assert!(names.name("g1", 8).unwrap_err().contains("g1"));
    assert!(names.name("g2", 7).unwrap_err().contains('7'));
    assert_eq!(names.task("g1"), Some(7));
    assert_eq!(names.logical(7), Some("g1"));
    assert_eq!(names.task("g2"), None);
    assert_eq!(names.logical(8), None);
    names.name("g2", 8).unwrap();
    assert_eq!(names.task("g2"), Some(8));
    assert_eq!(names.logical(8), Some("g2"));
}
