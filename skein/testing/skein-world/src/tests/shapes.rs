use crate::sweep::{SHAPES_KEPT, Shapes};

fn sample() -> Shapes {
    Shapes::new("S", "smith/1")
}

fn lines(shapes: &Shapes) -> Vec<String> {
    let mut lines = Vec::new();
    shapes.write_to(|line| lines.push(line.to_string()));
    lines
}

#[test]
fn repeated_shapes_count_once_per_case() {
    let mut shapes = sample();
    shapes.ran(&["a", "a", "b"]).unwrap();
    shapes.ran(&["b"]).unwrap();
    assert_eq!((shapes.cases(), shapes.reached("a"), shapes.reached("b")), (2, 1, 2));
    assert_eq!(
        lines(&shapes),
        [
            "fuzzy shapes: S smith/1 cases 2 distinct 2 beyond 0",
            "fuzzy shape: S smith/1 2 b",
            "fuzzy shape: S smith/1 1 a"
        ]
    );
}

#[test]
fn empty_cases_count_toward_the_sample_size() {
    let mut shapes = sample();
    shapes.ran(&[]).unwrap();
    assert_eq!(shapes.cases(), 1);
}

#[test]
fn lines_order_shapes_by_count_then_by_bytes() {
    let mut shapes = sample();
    shapes.ran(&["b", "c", "a", "a"]).unwrap();
    shapes.ran(&["b"]).unwrap();
    assert_eq!(
        lines(&shapes),
        [
            "fuzzy shapes: S smith/1 cases 2 distinct 3 beyond 0",
            "fuzzy shape: S smith/1 2 b",
            "fuzzy shape: S smith/1 1 a",
            "fuzzy shape: S smith/1 1 c",
        ]
    );
}

#[test]
fn excess_shapes_count_once_and_kept_shapes_still_count() {
    const FIRST: &str = "z-kept-first";
    const LAST: &str = "z-kept-last";
    const EXCESS: &str = "a-not-kept";
    let mut shapes = sample();
    let middle: Vec<_> = (0..SHAPES_KEPT - 2).map(|index| format!("middle-{index}")).collect();
    let mut first_case = vec![FIRST];
    first_case.extend(middle.iter().map(String::as_str));
    first_case.extend([LAST, EXCESS]);
    shapes.ran(&first_case).unwrap();
    shapes.ran(&[EXCESS, "b-not-kept", EXCESS, FIRST]).unwrap();
    assert_eq!((shapes.reached(FIRST), shapes.reached(LAST), shapes.reached(EXCESS)), (2, 1, 0));
    assert_eq!(
        lines(&shapes).first(),
        Some(&format!("fuzzy shapes: S smith/1 cases 2 distinct {SHAPES_KEPT} beyond 2"))
    );
}

#[test]
fn bad_shape_text_refuses_the_whole_case_without_echoing_it() {
    const TOO_LONG: usize = 1025;
    let long = "x".repeat(TOO_LONG);
    let mut shapes = sample();
    for (text, why) in [
        ("", "empty text"),
        (long.as_str(), "more than 1024 bytes"),
        ("private\nline", "line feed"),
        ("private é", "non-printable ASCII byte"),
    ] {
        assert_eq!(shapes.ran(&["good", text]), Err(format!("sweep S: shape not in form: {why}")));
    }
    assert_eq!(lines(&shapes), ["fuzzy shapes: S smith/1 cases 0 distinct 0 beyond 0"]);
}

#[test]
fn shape_text_accepts_its_length_and_ascii_bounds() {
    const AT_BOUND: usize = 1024;
    let longest = " ~".repeat(AT_BOUND / 2);
    let mut shapes = sample();
    assert_eq!(shapes.ran(&["a", &longest]), Ok(()));
}
