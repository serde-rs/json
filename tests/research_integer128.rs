#[test]
fn integer128_boundaries() {
    for (source, expected) in [
        ("0", 0i128),
        ("-0", 0),
        ("1", 1),
        ("-1", -1),
        ("170141183460469231731687303715884105727", i128::MAX),
        ("-170141183460469231731687303715884105728", i128::MIN),
    ] {
        assert_eq!(serde_json::from_str::<i128>(source).unwrap(), expected);
    }

    for (source, expected) in [
        ("0", 0u128),
        ("1", 1),
        ("340282366920938463463374607431768211455", u128::MAX),
    ] {
        assert_eq!(serde_json::from_str::<u128>(source).unwrap(), expected);
    }

    for source in [
        "170141183460469231731687303715884105728",
        "-170141183460469231731687303715884105729",
        "999999999999999999999999999999999999999999999999999",
        "01",
        "1e",
        "1.0",
        "-",
    ] {
        assert!(serde_json::from_str::<i128>(source).is_err(), "{source}");
    }
    for source in [
        "340282366920938463463374607431768211456",
        "-1",
        "01",
        "1e",
        "1.0",
    ] {
        assert!(serde_json::from_str::<u128>(source).is_err(), "{source}");
    }
}

#[test]
fn integer128_error_positions_and_input_sources() {
    use std::io::Cursor;
    let cases = [
        "0",
        "-0",
        "1",
        "-1",
        "01",
        "-01",
        "-",
        "+1",
        "1e2",
        "1.0",
        "170141183460469231731687303715884105727",
        "170141183460469231731687303715884105728",
        "-170141183460469231731687303715884105728",
        "-170141183460469231731687303715884105729",
        "340282366920938463463374607431768211455",
        "340282366920938463463374607431768211456",
        "9999999999999999999999999999999999999999999999999999999999",
        "9999999999999999999999999999999999999999999999999999999999x",
        "1\n2",
    ];
    for input in cases {
        let s =
            serde_json::from_str::<i128>(input).map_err(|e| (e.classify(), e.line(), e.column()));
        let b = serde_json::from_slice::<i128>(input.as_bytes())
            .map_err(|e| (e.classify(), e.line(), e.column()));
        let r = serde_json::from_reader::<_, i128>(Cursor::new(input.as_bytes()))
            .map_err(|e| (e.classify(), e.line(), e.column()));
        assert_eq!(s, b, "slice {input:?}");
        assert_eq!(s.map_err(|e| e.0), r.map_err(|e| e.0), "reader {input:?}");
        let s =
            serde_json::from_str::<u128>(input).map_err(|e| (e.classify(), e.line(), e.column()));
        let b = serde_json::from_slice::<u128>(input.as_bytes())
            .map_err(|e| (e.classify(), e.line(), e.column()));
        let r = serde_json::from_reader::<_, u128>(Cursor::new(input.as_bytes()))
            .map_err(|e| (e.classify(), e.line(), e.column()));
        assert_eq!(s, b, "u128 slice {input:?}");
        assert_eq!(
            s.map_err(|e| e.0),
            r.map_err(|e| e.0),
            "u128 reader {input:?}"
        );
    }
}
