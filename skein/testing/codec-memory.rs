//! Codec owners' largest-root memory checks (data/codec/README.md, section 6).
//! Included by each world's test binary, with its counting allocator installed.

/// Find the largest common clamp that fits, bisecting the remaining interval.
pub(crate) fn select<L: Copy>(
    family: &str,
    root: &str,
    cap: u64,
    ceilings: L,
    lower: impl Fn(u32) -> L,
    bounds: impl Fn(&L) -> (u64, u64),
    pin: Option<(u64, u64)>,
) -> (L, Option<u32>) {
    let ceiling_bounds = bounds(&ceilings);
    if ceiling_bounds.0 <= cap {
        assert_eq!(pin, None, "{family}/{root}: a ceiling-sized root needs no pin");
        return (ceilings, None);
    }
    // Decoded pins describe the generated types on a 64-bit target.
    if let Some(pin) = pin {
        assert_eq!(ceiling_bounds, pin, "{family}/{root}: ceiling bounds (64-bit sizes)");
    } else {
        eprintln!("PIN {family}/{root}: {ceiling_bounds:?}");
    }
    assert!(bounds(&lower(0)).0 <= cap, "{family}/{root}: even zero limits exceed cap {cap}");
    let mut low = 0;
    let mut high = u32::MAX;
    while low < high {
        let middle = low + (high - low).div_ceil(2);
        if bounds(&lower(middle)).0 <= cap {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    (lower(low), Some(low))
}

macro_rules! round_trip {
    (sealed, $value:ident, $limits:ident, $bytes:ident, $label:ident) => {{
        let length = usize::try_from($value.measure()).expect("u32 fits usize");
        assert_eq!(length, $bytes.len(), "{}: measure", $label);
        let mut writer = skein_lib::Writer::new(length);
        $value.encode(&mut writer).unwrap_or_else(|why| panic!("{}: encode: {why:?}", $label));
        assert_eq!(writer.finish().as_ref(), $bytes, "{}: round trip", $label);
    }};
    (lean, $value:ident, $limits:ident, $bytes:ident, $label:ident) => {{
        let length = $value.measure(&$limits).unwrap_or_else(|why| panic!("{}: measure: {why:?}", $label));
        assert_eq!(usize::try_from(length).expect("u32 fits usize"), $bytes.len(), "{}: measure", $label);
        let encoded = $value.encode(&$limits).unwrap_or_else(|why| panic!("{}: encode: {why:?}", $label));
        assert_eq!(encoded.as_ref(), $bytes, "{}: round trip", $label);
    }};
}

/// The type list dispatches every schema root through its generated decoder;
/// the exhaustive Limits literal makes a newly added limit a compile error.
macro_rules! family {
    ($cap:expr, $text:expr, $module:path, $form:ident,
     limits [$($field:ident),* $(,)?], roots [$($root:ident $(=> $pin:expr)?),+ $(,)?]) => {{
        use $module as codec;
        let schema = skein_codegen::parse($text).expect("family schema");
        let lower = |k| codec::Limits { $($field: codec::CEILINGS.$field.min(k)),* };
        let roots = skein_codegen::roots(&schema);
        let expected = [$(stringify!($root)),+];
        assert_eq!(roots, expected, "{}: generated decoder coverage", schema.family);
        for root in roots {
            let label = format!("{}/{}", schema.family, root);
            match root.as_str() {
                $(stringify!($root) => {
                    let (limits, k) = super::codec_memory::select(
                        &schema.family, &root, $cap, codec::CEILINGS,
                        lower,
                        |limits| (
                            codec::$root::encoded_bound(limits).expect("valid encoded limits"),
                            codec::$root::decoded_bound(limits).expect("valid decoded limits"),
                        ),
                        super::codec_memory::family!(@pin $($pin)?),
                    );
                    let values = limits.values();
                    let bytes = skein_codegen::largest(&schema, &root, values.as_slice())
                        .unwrap_or_else(|why| panic!("{label}: largest: {why:?}"));
                    let encoded = codec::$root::encoded_bound(&limits).expect("encoded bound");
                    let bound = codec::$root::decoded_bound(&limits).expect("decoded bound");
                    assert_eq!(u64::try_from(bytes.len()).expect("length fits"), encoded, "{label}: largest");
                    assert!(encoded <= $cap, "{label}: cap");
                    let mut reader = skein_lib::Reader::new(&bytes);
                    let meter = skein_heap::Meter::new();
                    meter.start();
                    let value = codec::$root::decode(&limits, &mut reader);
                    let measured = meter.end();
                    let value = value.unwrap_or_else(|why| panic!("{label}: decode at k={k:?}: {why:?}"));
                    assert!(reader.is_empty(), "{label}: whole encoding consumed");
                    assert!(measured.peak() <= bound,
                        "{label}: k={k:?}, limits={:?}, peak={}, bound={bound}",
                        values.as_slice(), measured.peak());
                    super::codec_memory::round_trip!($form, value, limits, bytes, label);
                    drop(value);
                    assert_eq!(meter.held(), 0, "{label}: decoded allocations released");
                    eprintln!("{label}: k={k:?}, cap={}, encoded={encoded}, decoded={bound}, peak={}",
                        $cap, measured.peak());
                }),+
                _ => panic!("{label}: missing generated decoder"),
            }
        }
    }};
    (@pin $pin:expr) => { Some($pin) };
    (@pin) => { None };
}

pub(crate) use {family, round_trip};
