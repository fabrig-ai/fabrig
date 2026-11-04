use super::pixels;

#[test]
fn whole_css_pixels_for_any_size() {
    for (value, expected) in
        [(f64::NAN, 0), (-0.0, 0), (-3.0, 0), (0.9, 0), (100.9, 100), (70000.0, 65535), (f64::INFINITY, 65535)]
    {
        assert_eq!(pixels(value), expected, "{value:?}");
    }
}
