use serde_json::{json, Value};

#[test]
fn value_ops_test() {
    //! test use operator directly from &Value without call leading path() method.
    let mut v = json!({"x": {"y": ["z", "zz"]}});

    let node = &v / "x";
    assert_eq!(node.unwrap(), &json!({"y": ["z", "zz"]}));
    let node = &v / "x" / "y";
    assert_eq!(node.unwrap(), &json!(["z", "zz"]));

    let val = &v / "x/y/0" | "";
    assert_eq!(val, "z");
    let val = &v / "x/y" / 1 | "";
    assert_eq!(val, "zz");

    let _ = &mut v / "x/y/1" << "AA";
    assert_eq!(&v/"x/y/1" | "", "AA");

    let _ = &mut v / "x" / "y" << [10] << [3.14] << [true] << ["END"];
    assert_eq!((&v/"x/y").unwrap(), &json!(["z", "AA", 10, 3.14, true, "END"]));
    assert_eq!(&v /"x/y/2" | 0, 10);
    assert_eq!(&v /"x/y/3" | 0.0, 3.14);
    assert_eq!(&v /"x" / "y" / "4" | false, true);

    let _ = &mut v / "x" << ("int", 20) << ("float", 3.14) << ("key", "val");
    assert_eq!(v, json!({
        "x": {
            "y": ["z", "AA", 10, 3.14, true, "END"],
            "int": 20,
            "float": 3.14,
            "key": "val"
        }
    }));

    assert_eq!((&v/"x"/"y").is_array(), true);
    let _ = &mut v / "x" / "y" << "array";
    assert_eq!((&v/"x"/"y").is_array(), false);
    assert_eq!(v, json!({
        "x": {
            "y": "array",
            "int": 20,
            "float": 3.14,
            "key": "val"
        }
    }));

    let _ = &mut v << "object";
    assert_eq!(v, json!("object"));
}

#[test]
fn pipe_json_test() {
    //! test pipe two json operation.
    let mut va = json!([
        {"name":"PI", "value":3.14},
        {"name":"e", "value":null},
        {"name":null, "value":1.0},
        {"name":618, "value":"618"},
        {"name":false, "value":false},
    ]);
    let vb = json!([{"name":"const", "value":1.0}]);

    let ra = &mut va & &vb | () | &vb;
    assert_eq!(ra/1/"value"|0.0, 1.0);
    assert_eq!(va, json!([
            {"name":"PI", "value":3.14},
            {"name":"e", "value":1.0},
            {"name":"const", "value":1.0},
    ]));
}

#[test]
fn pipe_simulate_test() {
    //! simulate some other json operator use pipe closure.

    // set int node only when node is int type.
    // similar json << i64
    let set_int = |node: &mut Value, val: i64| {
        if node.is_i64() {
            *node = val.into();
        }
    };

    let mut json = json!(100);
    let ival = -100;
    let _ref = &mut json | |node: &mut Value| {set_int(node, ival);};
    assert_eq!(json, json!(-100));

    json = json!("100");
    let _ref = &mut json | |node: &mut Value| {set_int(node, ival);};
    assert_eq!(json, json!("100"));

    // similar json | i64
    let mut oint = 0;
    json = json!(100);
    let _ref = &mut json | |node: &mut Value| {
        oint = node.as_i64().unwrap_or(1);
    };
    assert_eq!(oint, 100);
    assert_eq!(json, json!(100));
}
