use serde_json::Stream;

#[test]
fn test() {
    let deserializer = serde_json::Deserializer::from_str(
        r#"[1, 5, {"a": 1, "b": 2, "c": [1, 2], "d": 3}, 0]
        {
            "a"
            :
            { "b" : 1 },
            "d": 5
        }
        "#,
    );

    let stream = Stream::new(deserializer);

    // [1, 5, {a: 1, b: 2, c: [1, 2], d: 3}, 0]
    let mut stream = stream.enter_array().unwrap();
    assert_eq!(stream.next_value::<usize>().unwrap(), 1);
    assert_eq!(stream.next_value::<usize>().unwrap(), 5);

    let mut stream = stream.enter_map().unwrap();
    assert_eq!(stream.next_value::<usize>().unwrap(), ("a".to_string(), 1));
    assert_eq!(stream.next_value::<usize>().unwrap(), ("b".to_string(), 2));
    let (key, mut stream) = stream.enter_array().unwrap();
    assert_eq!(key, "c".to_string());
    assert_eq!(stream.next_value::<usize>().unwrap(), 1);
    assert_eq!(stream.next_value::<usize>().unwrap(), 2);

    let mut stream = stream.end_array().unwrap();
    assert_eq!(stream.next_value::<usize>().unwrap(), ("d".to_string(), 3));
    let mut stream = stream.end_map().unwrap();

    assert_eq!(stream.next_value::<usize>().unwrap(), 0);
    let stream = stream.end_array().unwrap();

    // { a: {b: 1}, d: 5 }
    let stream = stream.enter_map().unwrap();

    let (key, mut stream) = stream.enter_map().unwrap();
    assert_eq!(key, "a".to_string());
    assert_eq!(stream.next_value::<usize>().unwrap(), ("b".to_string(), 1));
    let mut stream = stream.end_map().unwrap();

    assert_eq!(stream.next_value::<usize>().unwrap(), ("d".to_string(), 5));
    let stream = stream.end_map().unwrap();

    stream.end().unwrap();
}

#[test]
fn simple_iterator() {
    let deserializer = serde_json::Deserializer::from_str(r#"[1, 2, 5, 6, 0]5"#);

    let stream = Stream::new(deserializer);

    let mut stream = stream.enter_array().unwrap();
    let mut iter = stream.iter::<usize>();
    assert_eq!(iter.next().unwrap().unwrap(), 1);
    assert_eq!(iter.next().unwrap().unwrap(), 2);
    assert_eq!(iter.next().unwrap().unwrap(), 5);
    assert_eq!(iter.next().unwrap().unwrap(), 6);
    assert_eq!(iter.next().unwrap().unwrap(), 0);
    assert!(iter.next().is_none());

    let mut stream = stream.end_array().unwrap();
    assert_eq!(stream.next_value::<usize>().unwrap(), 5);
    stream.end().unwrap();
}
