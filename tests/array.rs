// #![cfg(not(feature = "preserve_order"))] TODO

extern crate serde;

#[macro_use]
extern crate serde_json;

use serde_json::de::ArrayDeserializer;
use serde_json::{de::Read, Deserializer, Value};

fn test_stream<T: Tester>(data: &str) {
    T::test(Deserializer::from_str(data).into_array());
    T::test(Deserializer::from_slice(data.as_bytes()).into_array());
    T::test(Deserializer::from_reader(data.as_bytes()).into_array());
}

trait Tester {
    fn test<'reader, R: Read<'reader>>(stream: ArrayDeserializer<'reader, R>);
}

macro_rules! test_stream {
    ($data:expr, |$stream:ident| $test:block) => {
        {
            struct Test;
            impl Tester for Test {
                fn test<'r, R: Read<'r>>(mut $stream: ArrayDeserializer<'r, R>)
                    $test
            }
            test_stream::<Test>($data);
        }
    };
}

#[test]
fn test_json_array_empty() {
    let data = "[]";

    test_stream!(data, |stream| {
        assert!(stream.next::<Value>().is_none());
    });
}

#[test]
fn test_json_array_empty2() {
    let data = "[]";

    {
        struct Test;
        impl Tester for Test {
            fn test<'a, R: Read<'a>>(mut stream: ArrayDeserializer<'a, R>) {
                assert!(stream.next::<Value>().is_none());
            }
        }
        test_stream::<Test>(data);
    }

    // test_stream!(data, |stream| {
    //     assert!(stream.next::<Value>().is_none());
    // });
}

#[test]
fn test_json_array_whitespace() {
    let data = "\r [\n{\"x\":42}\t, {\"y\":43}\n] \t\n";

    test_stream!(data, |stream| {
        assert_eq!(stream.next::<Value>().unwrap().unwrap()["x"], 42);

        assert_eq!(stream.next::<Value>().unwrap().unwrap()["y"], 43);

        assert!(stream.next::<Value>().is_none());
    });
}

#[test]
fn test_json_array_truncated() {
    let data = "[{\"x\":40},{\"x\":";

    test_stream!(data, |stream| {
        assert_eq!(stream.next::<Value>().unwrap().unwrap()["x"], 40);

        assert!(stream.next::<Value>().unwrap().unwrap_err().is_eof());
    });
}

#[test]
fn test_json_array_primitive() {
    let data = r#"[{}, true, 1, [], 1.0, "hey", [1.0, []], 2.0, null]"#;

    test_stream!(data, |stream| {
        assert_eq!(stream.next::<Value>().unwrap().unwrap(), json!({}));

        assert_eq!(stream.next::<bool>().unwrap().unwrap(), true);

        assert_eq!(stream.next::<u32>().unwrap().unwrap(), 1);

        assert_eq!(stream.next::<Value>().unwrap().unwrap(), json!([]));

        assert_eq!(stream.next::<f32>().unwrap().unwrap(), 1.0);

        assert_eq!(stream.next::<String>().unwrap().unwrap(), "hey");

        {
            let mut sub = stream.next_array();

            assert_eq!(sub.next::<f32>().unwrap().unwrap(), 1.0);

            {
                let mut sub2 = sub.next_array();
                assert!(sub2.next::<f32>().is_none());
            }
            println!("after sub2");
            assert!(sub.next::<f32>().is_none());
            println!("is_none");
        }
        assert_eq!(stream.next::<f32>().unwrap().unwrap(), 2.0);

        assert_eq!(stream.next::<Value>().unwrap().unwrap(), Value::Null);

        assert!(stream.next::<Value>().is_none());
    });
}

#[test]
fn test_json_array_tailing_data() {
    let data = "[]e";

    test_stream!(data, |stream| {
        let second = stream.next::<Value>().unwrap().unwrap_err();
        assert_eq!(second.to_string(), "trailing characters at line 1 column 3");
    });
}

#[test]
fn test_json_array_tailing_comma() {
    let data = "[true,]";

    test_stream!(data, |stream| {
        assert_eq!(stream.next::<Value>().unwrap().unwrap(), true);

        let second = stream.next::<Value>().unwrap().unwrap_err();
        assert_eq!(second.to_string(), "trailing comma at line 1 column 7");
    });
}

#[test]
fn test_json_array_eof() {
    let data = "";

    test_stream!(data, |stream| {
        let second = stream.next::<Value>().unwrap().unwrap_err();
        assert_eq!(
            second.to_string(),
            "EOF while parsing a value at line 1 column 0"
        );
    });
}
