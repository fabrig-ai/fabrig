// Codec owners' largest-root memory checks (data/codec/golden.md, section 1).
// Included by each world's test binary, with its counting allocator installed.

/// Find the largest common clamp that fits, bisecting the remaining interval.
pub(crate) fn select<L>(
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
    assert_eq!(Some(ceiling_bounds), pin, "{family}/{root}: ceiling bounds (64-bit sizes)");
    assert!(bounds(&lower(0)).0 <= cap, "{family}/{root}: even zero limits exceed cap {cap}");
    let mut low = 0;
    let mut high = u32::MAX;
    while low < high {
        let middle =
            low.checked_add(high.checked_sub(low).expect("ordered interval").div_ceil(2)).expect("midpoint fits");
        if bounds(&lower(middle)).0 <= cap {
            low = middle;
        } else {
            high = middle.checked_sub(1).expect("middle is above low");
        }
    }
    (lower(low), Some(low))
}

macro_rules! round_trip {
    ($value:ident, $limits:ident, $bytes:ident, $label:ident) => {{
        let length = $value.measure(&$limits).unwrap_or_else(|why| panic!("{}: measure: {why:?}", $label));
        assert_eq!(usize::try_from(length).expect("u32 fits usize"), $bytes.len(), "{}: measure", $label);
        let encoded = $value.encode(&$limits).unwrap_or_else(|why| panic!("{}: encode: {why:?}", $label));
        assert_eq!(encoded.as_ref(), $bytes, "{}: round trip", $label);
    }};
}

/// The type list dispatches every schema root through its generated decoder;
/// the exhaustive Limits literal makes a newly added limit a compile error.
macro_rules! family {
    ($cap:expr, $text:expr, $module:path,
     limits [$($field:ident),* $(,)?], roots [$($root:ident $(=> $pin:expr)?),+ $(,)?]) => {{
        use $module as codec;
        let schema = &skein_codegen::values($text).schema;
        let lower = |k| codec::Limits { $($field: codec::CEILINGS.$field.min(k)),* };
        let roots = skein_codegen::roots(schema);
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
                    let variants = skein_codegen::largest_variants(schema, &root, values.as_slice())
                        .unwrap_or_else(|why| panic!("{label}: largest variants: {why:?}"));
                    let encoded = codec::$root::encoded_bound(&limits).expect("encoded bound");
                    let bound = codec::$root::decoded_bound(&limits).expect("decoded bound");
                    let longest = variants.iter().map(|bytes| u64::try_from(bytes.len()).expect("length fits")).max();
                    assert_eq!(longest, Some(encoded), "{label}: largest");
                    assert!(encoded <= $cap, "{label}: cap");
                    for (index, bytes) in variants.into_iter().enumerate() {
                        let label = format!("{label} variant {index}");
                        let encoded = bytes.len();
                        let mut reader = skein_lib::Reader::new(&bytes);
                        let meter = skein_heap::Meter::new();
                        let measured = {
                            meter.start();
                            let value = codec::$root::decode(&limits, &mut reader);
                            let measured = meter.end();
                            let value = value.unwrap_or_else(|why| panic!("{label}: decode at k={k:?}: {why:?}"));
                            assert!(reader.is_empty(), "{label}: whole encoding consumed");
                            assert!(measured.peak() <= bound,
                                "{label}: k={k:?}, limits={:?}, peak={}, bound={bound}",
                                values.as_slice(), measured.peak());
                            super::codec_memory::round_trip!(value, limits, bytes, label);
                            measured
                        };
                        assert_eq!(meter.held(), 0, "{label}: decoded allocations released");
                        eprintln!("{label}: k={k:?}, cap={}, encoded={encoded}, decoded={bound}, peak={}, ceilings=({}, {})",
                            $cap, measured.peak(),
                            codec::$root::encoded_bound(&codec::CEILINGS).expect("ceiling encoded bound"),
                            codec::$root::decoded_bound(&codec::CEILINGS).expect("ceiling decoded bound"));
                    }
                }),+
                _ => panic!("{label}: missing generated decoder"),
            }
        }
    }};
    (@pin $pin:expr) => { Some($pin) };
    (@pin) => { None };
}

pub(crate) use {family, round_trip};
