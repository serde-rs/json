use serde_json::PathOperator;
use serde_json::operator::{JsonPtr, JsonPtrMut};
use serde_json::{json, Value};

#[test]
fn pointer_test() {
    //! review the usage of pointer method.
    let v = json!({"x": {"y": ["z", "zz"]}});
    assert_eq!(v.pointer("").unwrap(), &v);
    //assert_eq!(v.pointer("/").unwrap(), &v);
    assert_eq!(v.pointer("/"), None);
    assert_eq!(v.pointer("/x").unwrap(), &json!({"y": ["z", "zz"]}));
    assert_eq!(v.pointer("/x/y").unwrap(), &json!(["z", "zz"]));
    assert_eq!(v.pointer("/x/y/0").unwrap(), &json!("z"));
    assert_eq!(v.pointer("/x/y/1").unwrap(), &json!("zz"));

    assert_eq!(v.pointer("/x/y/2"), None);
    assert_eq!(v.pointer("x/y/0"), None);

    let v = json!([0, 1, 2, 3]);
    assert_eq!(v.pointer("/0").unwrap(), &json!(0));
    assert_eq!(v.pointer("/1").unwrap(), &json!(1));
    assert_eq!(v.pointer("1"), None);
}

#[test]
fn path_test() {
    //! test path method behaves similar to ponter, with explicit token.
    let v = json!({"x": {"y": ["z", "zz"]}});

    let node = v.path() / "x";
    assert_eq!(node.is_none(), false);
    assert_eq!(node.unwrap(), &json!({"y": ["z", "zz"]}));
    let node = v.path() / "x" / "y";
    assert_eq!(node.unwrap(), &json!(["z", "zz"]));
    let first = node / 0;
    assert_eq!(first.unwrap(), &json!("z"));
    let second = node / 1;
    assert_eq!(second.unwrap(), &json!("zz"));
    let third = node / 2;
    assert_eq!(third.is_none(), true);
    assert_eq!(*third, None);

    let s = 1.to_string();
    assert_eq!(s, "1");

    // use variable for path token, String not implemnt Copy, need reference
    let x = String::from("x");
    let y = String::from("y");
    let i = 1; // as usize;
    let node = v.path() / &x / &y / i;
    assert_eq!(node.unwrap(), &json!("zz"));
}

#[test]
fn pathto_test() {
    //! test path method behaves similar to ponter, with joined token.
    let v = json!({"x": {"y": ["z", "zz"]}});

    // joined path would auto prefix '/' according json pointer standard
    let node = v.path() / "/x/y";
    assert_eq!(node.unwrap(), &json!(["z", "zz"]));
    let node = v.path() / "/x / y";
    assert_eq!(node.is_none(), true);
    let node = v.path() / "x/y";
    assert_eq!(node.is_none(), false);
    assert_eq!(node.unwrap(), &json!(["z", "zz"]));
    let node = v.pathto("/x/y");
    assert_eq!(node.unwrap(), &json!(["z", "zz"]));
}

#[test]
fn path_empty_test() {
    //! test empty path token.
    let v = json!({"x": {"y": ["z", "zz"]}});
    let root = v.path();
    let empty = v.path() / "";
    assert_eq!(empty.is_none(), true);
    assert_eq!(root.unwrap(), &v);

    let p = v.get("");
    assert_eq!(p.is_none(), true);

    let v = json!({"x": {"y": ["z", "zz"]}, "": "abc"});
    let p = v.get("");
    assert_eq!(p.is_none(), false);
    assert_eq!(p.unwrap(), &json!("abc"));

    let node = v.path() / "";
    assert_eq!(node.unwrap(), &json!("abc"));

    let node = v.pathto("");
    assert_eq!(node.unwrap(), &json!("abc"));
}

#[test]
fn path_number_test() {
    //! test numberic path token.
    let v = json!({"1": "a", "2": "b", "array": [1, 2], "3": [3, 4]});

    let p = v.pointer("/1");
    assert_eq!(p.is_none(), false);
    assert_eq!(p.unwrap(), &json!("a"));
    let p = v.pointer("/array/1");
    assert_eq!(p.is_none(), false);
    assert_eq!(p.unwrap(), &json!(2));
    let p = v.pointer("/3/1");
    assert_eq!(p.is_none(), false);
    assert_eq!(p.unwrap(), &json!(4));

    let node = v.path() / "1";
    assert_eq!(node.is_none(), false);
    assert_eq!(node.unwrap(), &json!("a"));
    assert_eq!(node.unwrap().as_str().unwrap(), "a");

    // backforward to pathto() using converted string path
    let node = v.path() / 1;
    assert_eq!(node.is_none(), false);
    assert_eq!(node.unwrap(), &json!("a"));

    let node = v.path() / "3" / 1;
    assert_eq!(node.is_none(), false);
    assert_eq!(node.unwrap(), &json!(4));

    let node = v.pathto("/3/1");
    assert_eq!(node.unwrap(), &json!(4));
}

#[test]
fn pipe_test() {
    //! test basic pipe operator usage.
    let v = json!({"x": {"y": ["z", "zz"]}});

    let val = v.path() / "x" / "y" / 1 | "";
    assert_eq!(val, "zz");
    let val = v.pathto("/x/y/1") | "";
    assert_eq!(val, "zz");

    let v = json!({"misc": {"int": 10, "float": 3.14, "str": "pi", "bool": true}});
    let misc = v.path() / "misc";
    let val = misc / "int" | 0;
    assert_eq!(val, 10);
    let val = misc / "float" | 0.0;
    assert_eq!(val, 3.14);
    let val = misc / "str" | "";
    assert_eq!(val, "pi");
    let val = misc / "bool" | false;
    assert_eq!(val, true);
    let val = misc / "bool" | 0;
    assert_eq!(val, 1); // true cast to 1

    let def = 0;
    let val = misc / "int" | def;
    assert_eq!(val, 10);
}

#[test]
fn pipe_cast_test() {
    //! test |number can fallback to parse from string node.
    let v = json!({"a": 10, "b": 3.14, "c": true, "A": "10", "B": "3.14", "C": "true", "x": "text"});
    let root = v.path();

    assert_eq!(root/"a" | 0, 10);
    assert_eq!(root/"A" | 0, 10);
    assert_eq!(root/"b" | 0.0, 3.14);
    assert_eq!(root/"B" | 0.0, 3.14);
    assert_eq!(root/"x" | 0, 0);
    assert_eq!(root/"x" | 0.0, 0.0);

    let tf: bool = "true".parse().unwrap();
    assert!(tf);
    let tf: bool = "false".parse().unwrap();
    assert!(!tf);

    assert_eq!(root/"c" | false, true);
    assert_eq!(root/"C" | false, true);
    assert_eq!(root/"a" | false, true);
    assert_eq!(root/"A" | false, false);
}

#[test]
fn pipe_string_test() {
    //! test |string to get selectively stringfy for json node.
    let v = json!({"int":3, "float":3.14, "str":"text", "array":[1,null,true]});
    let root = v.path();

    assert_eq!(root/"int" | "", "");
    assert_eq!(root/"int" | "".to_string(), "3");
    assert_eq!(root/"int" | "0".to_string(), "3");
    assert_eq!(root/"int" | "0.0".to_string(), "0.0");

    assert_eq!(root/"float" | "", "");
    assert_eq!(root/"float" | "".to_string(), "3.14");
    assert_eq!(root/"float" | "0".to_string(), "0");
    assert_eq!(root/"float" | "0.0".to_string(), "3.14");

    let vs = (root/"str").unwrap();
    assert_eq!(root/"str" | "", "text");
    assert_eq!(root/"str" | "".to_string(), "text");
    assert_ne!(root/"str" | "".to_string(), vs.to_string());
    assert_eq!(vs.to_string(), "\"text\"");

    assert_eq!(root/"array" | "", "");
    assert_eq!(root/"array" | "".to_string(), "[1,null,true]");
    assert_eq!(root/"array" | "[]".to_string(), "[1,null,true]");
    assert_eq!(root/"array" | "0".to_string(), "0");
    assert_eq!(root/"array" | "any default".to_string(), "any default");
    assert_eq!(root/"array" | "any default", "any default");

    assert_eq!(root | "any default", "any default");
    assert_eq!(root | "".to_string(), root.unwrap().to_string());
    assert_eq!(root | "{}".to_string(), v.to_string());
    assert_eq!(root | "any default".to_string(), "any default");
}

#[test]
fn pipe_str_test() {
    //! test |&str lifetime
    let mut v = json!({"int":10, "float":3.14, "str":"pi", "bool":true});

    let node = v.path() / "str";
    assert_eq!(node | "", "pi");

    let s = String::from("xx");
    let mut val = node | s.as_str();
    assert_eq!(val, "pi");

    {
        let ss = String::from("xx");
        val = node | ss.as_str();
        assert_eq!(val, "pi");
        assert_eq!(v.path() / "int" | ss.as_str(), "xx");
    }

    let node = v.path_mut() / "str";
    {
        let ss = String::from("xx");
        val = node | ss.as_str();
        assert_eq!(val, "pi");
        assert_eq!(v.path_mut() / "int" | ss.as_str(), "xx");
    }

    assert_eq!(s, "xx");
    let val = v.path() / "str" | s;
    assert_eq!(val, "pi");
    // assert_eq!(s, "xx"); //< complie error, s moved by | operator.
}

#[test]
fn path_mut_test() {
    //! test basic mutable path operator.
    let mut v = json!({"x": {"y": ["z", "zz"]}});
    let node = v.path_mut() / "x";
    assert_eq!(node.as_ref().unwrap(), &&json!({"y": ["z", "zz"]}));

    let val = v.path_mut() / "x" / "y" / 1 | "";
    assert_eq!(val, "zz");
    let val = v.pathto_mut("/x/y/1") | "";
    assert_eq!(val, "zz");
    let val = v.path_mut() / "x/y/1" | "";
    assert_eq!(val, "zz");

    let mut v = json!({"misc": {"int":10, "float":3.14, "str":"pi", "bool":true}});
    let misc = v.path_mut() / "misc";
    let val = misc / "int" | 0;
    assert_eq!(val, 10);

    // mutable ptr moved after | operator
    let val = v.path_mut() / "misc" / "float" | 0.0;
    assert_eq!(val, 3.14);
    let val = v.path_mut() / "misc" / "str" | "";
    assert_eq!(val, "pi");
    let val = v.path_mut() / "misc" / "bool" | false;
    assert_eq!(val, true);
    let val = v.path_mut() / "misc" / "bool" | 0;
    assert_eq!(val, 1);

    let def = 0;
    let val = v.path_mut() / "misc" / "int" | def;
    assert_eq!(val, 10);
}

#[test]
fn put_test() {
    //! test operator << to put new value to scalar node, or overwrite any node.
    let mut v = json!({"x": {"y": ["z", "zz"]}});

    // overwrite leaf node data
    let node = v.path_mut() / "x" / "y" / 1;
    let node = node << "AA";
    assert_eq!(node | "", "AA");
    let val = v.pathto("/x/y/1") | "";
    assert_eq!(val, "AA");

    // may also change node type
    let node = v.pathto_mut("/x/y/1") << 12;
    assert_eq!(node.is_none(), false);
    assert_eq!(v.pathto("/x/y/1") | "", "");
    assert_eq!(v.pathto("/x/y/1") | 0, 12);
    let node = v.pathto_mut("x/y") << "array";
    assert_eq!(node | "", "array");

    let mut v = json!({"int":10, "float":3.14, "str":"pi", "bool":true});
    let node = v.path_mut() / "int" << 11;
    assert_eq!(node | 0, 11);
    let node = v.path_mut() / "float" << 31.4;
    assert_eq!(node | 0.0, 31.4);
    let node = v.path_mut() / "str" << "PI";
    assert_eq!(node | "", "PI");
    let node = v.path_mut() / "bool" << false;
    assert_eq!(node | true, false);

    let node = v.path_mut() / "bool" << ();
    assert_eq!(node.is_none(), false);
    assert_eq!(node.is_null(), true);
}

#[test]
fn push_test() {
    //! test operator << to push new item to object or array.
    let mut v = json!({});

    let node = v.path_mut() << ("int", 10) << ("float", 3.14);
    let _ =  node << ("str", "pi") << ("bool", true);

    assert_eq!(v.path()/"int" | 0, 10);
    assert_eq!(v.path()/"float" | 0.0, 3.14);
    assert_eq!(v.path()/"str" | "", "pi");
    assert_eq!(v.path()/"bool" | false, true);

    let node = v.path_mut() << ("array", json!([]));
    let node = node / "array" << [11] << [11.22] << ("PI",) << (true,);
    assert_eq!(node.is_none(), false);

    let array = v.path() / "array";
    assert_eq!(array/0 | 0, 11);
    assert_eq!(array/1 | 0.1, 11.22);
    assert_eq!(array/2 | "", "PI");
    assert_eq!(array/3 | false, true);

    // change int node to array of int
    let node = v.path_mut() / "int";
    let _ode = node << [10] << [20] << [()];
    assert_eq!((v.path()/"int").unwrap(), &json!([10, 20, null]));
    assert_eq!(v["int"], json!([10, 20, null]));

    println!("{}", v);
    assert_eq!(v.pathto("int/2").is_null(), true);

    // Value not implement Copy, need refer with &
    let dint = &v["int"];
    assert_eq!(dint[0], 10);
    assert_eq!(dint.path()/0 | 0, 10);
    assert_eq!(v.path()/"int"/0 | 0, 10);
}

#[test]
fn path_index_test() {
    //! compare path and index syntax.
    let v = json!({"int":10, "float":3.14, "str":"pi", "array":[1,null,true]});
    let root = v.path();

    assert_eq!(root/"array"/0 | 0, 1);
    assert_eq!(v["array"][0], 1);

    let val: i64 = root/"array"/0 | 0;
    assert_eq!(val, 1);
    // let val: i64 = &v["array"][0];
    //^^ compile error, index return Value node, which overload ==
    let val: i64 = v["array"][0].as_i64().unwrap_or(0);
    assert_eq!(val, 1);

    // None pointer for non-existed node.
    let nokey = &v["nokey"]; //< & required
    let nokey_ptr = root/"nokey";
    assert_eq!(nokey.is_null(), true);
    assert_eq!(nokey_ptr.is_none(), true);
    assert_eq!(nokey_ptr.is_null(), false);

    let outrange = &v["array"][3];
    assert_eq!(outrange.is_null(), true);
    let outrange = root/"array"/3;
    assert_eq!(outrange.is_null(), false);
    assert_eq!(outrange.is_none(), true);

    // auto insert for index object, but not path
    let mut v = json!({"array":[1,null,true]});
    let node = v.path_mut() / "new";
    assert_eq!(node.is_none(), true);
    v["new"] = "auto insert".into();
    assert_eq!(v["new"], "auto insert");
    let node = v.path() / "new";
    assert_eq!(node.is_none(), false);
    assert_eq!(node | "", "auto insert");

    // panic when outof range with mutable index
    let outrange = v.path()/"array"/3;
    assert_eq!(outrange.is_none(), true);
    let outrange = &v["array"][3];
    assert_eq!(outrange.is_null(), true);
    // v["array"][3] = "panic".into();
    // ^^ panic at runtime

    let _p = v.path_mut() / "array" << ["new item"];
    assert_eq!(_p/3|"", "new item");
    assert_eq!(v["array"][3], "new item");
}

#[test]
fn check_type_test() {
    //! check type before operator.
    let mut v = json!({"x": {"y": ["z", "zz"]}});

    let node = v.path() / "x" / "y" / 1;
    if node.unwrap().is_string() {
        let val = node | "";
        assert_eq!(val, "zz");
    }

    let node = v.path_mut() / "x" / "y" / 1;
    if node.as_ref().unwrap().is_string() {
        let val = node | "";
        assert_eq!(val, "zz");
    }

    let node = v.path_mut() / "x" / "y" / 1;
    if node.as_ref().unwrap().is_string() {
        let node = node << "AA";
        let val = node | "";
        assert_eq!(val, "AA");
    }

    let node = v.path_mut() / "x" / "y" / 1;
    if node.is_string() {
        let node = node << "BB";
        let val = node | "";
        assert_eq!(val, "BB");
    }
}

#[test]
fn type_test() {
    //! check type method, not overload operator.
    let mut v = json!({"int":10, "float":3.14, "str":"pi", "bool":true});

    assert_eq!(v.pathto("int").is_i64(), true);
    assert_eq!(v.pathto("int").is_u64(), true);
    assert_eq!(v.pathto("float").is_f64(), true);
    assert_eq!(v.pathto("str").is_string(), true);
    assert_eq!(v.pathto("bool").is_boolean(), true);

    assert_eq!(v.pathto_mut("int").is_i64(), true);
    assert_eq!(v.pathto_mut("int").is_u64(), true);
    assert_eq!(v.pathto_mut("float").is_f64(), true);
    assert_eq!(v.pathto_mut("str").is_string(), true);
    assert_eq!(v.pathto_mut("bool").is_boolean(), true);

    assert_eq!(v.pathto("int").is_string(), false);
    assert_eq!(v.pathto("float").is_string(), false);
    assert_eq!(v.pathto("bool").is_string(), false);
    assert_eq!(v.pathto("str").is_i64(), false);
    assert_eq!(v.pathto("float").is_i64(), false);
    assert_eq!(v.pathto("bool").is_i64(), false);

    assert_eq!(v.path().is_object(), true);
    assert_eq!(v.path().is_array(), false);
    assert_eq!(v.path().is_null(), false);
    assert_eq!(v.path_mut().is_object(), true);
    assert_eq!(v.path_mut().is_array(), false);
    assert_eq!(v.path_mut().is_null(), false);
}

#[test]
fn ptr_eq_test() {
    //! test the derived `==` operator trait. compare the value they point to.
    let mut v = json!({"int":10, "float":3.14, "array":["pi", 10, true]});
    let mut v2 = json!({"int":10, "float":3.14, "array":["pi", 10, true]});

    let p1 = v.path() / "int";
    let p2 = v.path() / "float";
    let p3 = v.pathto("int");
    let p4 = v.path() / "array" / 1;
    assert_ne!(p1, p2);
    assert_eq!(p1, p3);
    assert!(p1 == p3);
    assert_eq!(p1, p4);

    let p4 = v2.path() / "int";
    assert_eq!(p1, p4);

    let pn = JsonPtr::new(None);
    let pm = JsonPtr::new(None);
    assert_ne!(p1, pn);
    assert_eq!(pm, pn);

    assert_eq!(&v, &v2);

    let p1 = v.path_mut() / "int";
    let p2 = v2.path_mut() / "int";
    assert_eq!(p1, p2);

    let pn = JsonPtrMut::new(None);
    let pm = JsonPtrMut::new(None);
    assert_ne!(p1, pn);
    assert_eq!(pm, pn);
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

    let pa = va.path_mut() & vb.path() | () | vb.path();
    assert_eq!(pa/1/"value"|0.0, 1.0);
    assert_eq!(va, json!([
            {"name":"PI", "value":3.14},
            {"name":"e", "value":1.0},
            {"name":"const", "value":1.0},
    ]));
}

#[test]
fn pipe_func_test() {
    //! test pipe to closure.
    // double value for int json node.
    let double = |v: &mut Value| {
        if let Some(i) = v.as_i64() {
            *v = (i*2).into();
        }
    };

    let mut v = json!([1, 2, 3.0, "3", false]);
    let i = v.path_mut()/0 | double | 0;
    assert_eq!(i, 2);
    assert_eq!(v, json!([2, 2, 3.0, "3", false]));

    let i = v.path_mut()/1 | double | 0;
    assert_eq!(i, 4);
    assert_eq!(v, json!([2, 4, 3.0, "3", false]));

    // no effect for later items.
    let i = v.path_mut()/2 | double | 0;
    assert_eq!(i, 0);
    assert_eq!(v, json!([2, 4, 3.0, "3", false]));
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
    let mptr = json.path_mut() | |node: &mut Value| {set_int(node, ival);};
    assert_eq!(mptr | 0, -100);
    assert_eq!(json, json!(-100));

    json = json!("100");
    let mptr = json.path_mut() | |node: &mut Value| {set_int(node, ival);};
    assert_eq!(mptr | 0, 100);
    assert_eq!(json, json!("100"));

    // similar json | i64
    let mut oint = 0;
    json = json!(100);
    let _ptr = json.path_mut() | |node: &mut Value| {
        oint = node.as_i64().unwrap_or(1);
    };
    assert_eq!(oint, 100);
    assert_eq!(json, json!(100));
}
