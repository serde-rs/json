/// Construct a `serde_json::Value` from a JSON literal.
///
/// ```rust
/// # #![allow(unused_variables)]
/// # #[macro_use] extern crate serde_json;
/// # fn main() {
/// let value = json!({
///     "code": 200,
///     "success": true,
///     "payload": {
///         "features": [
///             "serde",
///             "json"
///         ]
///     }
/// });
/// # }
/// ```
///
/// Any variable or expression that implements Serde's `Serialize` trait can be
/// interpolated into the JSON literal just by referring to it.
///
/// ```rust
/// # #![allow(unused_variables)]
/// # #[macro_use] extern crate serde_json;
/// # fn main() {
/// let code = 200;
/// let features = vec!["serde", "json"];
///
/// let value = json!({
///    "code": code,
///    "success": code == 200,
///    "payload": {
///        features[0]: features[1]
///    }
/// });
/// # }
/// ```
///
/// Trailing commas are allowed inside both arrays and objects.
///
/// ```rust
/// # #![allow(unused_variables)]
/// # #[macro_use] extern crate serde_json;
/// # fn main() {
/// let value = json!([
///     "notice",
///     "the",
///     "trailing",
///     "comma -->",
/// ]);
/// # }
/// ```
#[macro_export]
macro_rules! json {
    // Hide distracting implementation details from the generated rustdoc.
    ($($json:tt)+) => {
        json_internal!($($json)+)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! json_internal {
    (null) => {
        $crate::Value::Null
    };

    (true) => {
        $crate::Value::Bool(true)
    };

    (false) => {
        $crate::Value::Bool(false)
    };

    ([]) => {
        $crate::Value::Array(Vec::new())
    };

    ([ $($tt:tt)+ ]) => {
        $crate::Value::Array({
            let mut array = Vec::new();
            json_within_array!(array () $($tt)+);
            array
        })
    };

    ({}) => {
        $crate::Value::Object($crate::Map::new())
    };

    ({ $($tt:tt)+ }) => {
        $crate::Value::Object({
            let mut object = $crate::Map::new();
            json_within_object!(object () () $($tt)+);
            object
        })
    };

    // Any Serialize type: numbers, strings, struct literals, variables etc.
    ($other:expr) => {
        $crate::to_value($other).unwrap()
    };
}

// TT muncher for parsing the inside of an array [...]. Each element is pushed
// into the given array variable.
//
// Must be invoked as: json_within_array!(var () $($tt)*)
#[macro_export]
#[doc(hidden)]
macro_rules! json_within_array {
    // Done.
    ($array:ident ()) => {};

    // Push a single element. The element must be more than zero tokens.
    ($array:ident ($($elem:tt)+)) => {
        $array.push(json!($($elem)+));
    };

    // Misplaced comma. Trigger a reasonable error message by failing to match
    // the comma in the recursive call.
    ($array:ident () , $($rest:tt)*) => {
        json_within_array!($array ,);
    };

    // Found a comma separator. Push whatever we have so far and move on to
    // remaining elements. Trailing comma is allowed.
    ($array:ident ($($elem:tt)+) , $($rest:tt)*) => {
        json_within_array!($array ($($elem)+));
        json_within_array!($array () $($rest)*);
    };

    // Munch a token into the current element.
    ($array:ident ($($elem:tt)*) $tt:tt $($rest:tt)*) => {
        json_within_array!($array ($($elem)* $tt) $($rest)*);
    };
}

// TT muncher for parsing the inside of an object {...}. Each entry is inserted
// into the given map variable.
//
// Must be invoked as: json_within_object!(var () () $($tt)*)
#[macro_export]
#[doc(hidden)]
macro_rules! json_within_object {
    // Done.
    ($object:ident () ()) => {};

    // Insert a single entry. The key and value must both be more than zero
    // tokens. The key must be Into-convertible to String.
    ($object:ident ($($key:tt)+) : ($($value:tt)+)) => {
        $object.insert(($($key)+).into(), json!($($value)+));
    };

    // Misplaced colon. Trigger a reasonable error message by failing to match
    // the colon in the recursive call.
    ($object:ident () () : $($rest:tt)*) => {
        json_within_object!($object :);
    };

    // Found a comma inside a key. Trigger a reasonable error message by failing
    // to match the comma in the recursive call.
    ($object:ident ($($key:tt)*) () , $($rest:tt)*) => {
        json_within_object!($object ,);
    };

    // Found a colon after a key. Move on to the value.
    ($object:ident ($($key:tt)+) () : $($rest:tt)*) => {
        json_within_object!($object ($($key)+) : () $($rest)*);
    };

    // Misplaced comma. Trigger a reasonable error message by failing to match
    // the comma in the recursive call.
    ($object:ident ($($key:tt)+) : () , $($rest:tt)*) => {
        json_within_object!($object ,);
    };

    // Found a comma after a value. Insert whatever we have so far and move on
    // to remaining elements. Trailing comma is allowed.
    ($object:ident ($($key:tt)+) : ($($value:tt)+) , $($rest:tt)*) => {
        json_within_object!($object ($($key)+) : ($($value)+));
        json_within_object!($object () () $($rest)*);
    };

    // Munch a token into the current key.
    ($object:ident ($($key:tt)*) () $tt:tt $($rest:tt)*) => {
        json_within_object!($object ($($key)* $tt) () $($rest)*)
    };

    // Munch a token into the current value.
    ($object:ident ($($key:tt)+) : ($($value:tt)*) $tt:tt $($rest:tt)*) => {
        json_within_object!($object ($($key)+) : ($($value)* $tt) $($rest)*)
    };
}
