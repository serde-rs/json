// Not sub mod but seperate file for operator overload interface.
// Used by include! macro in operator mod.

/* ------------------------------------------------------------ */

/// Path operator `/` create a json pointer to sub-node.
/// 
/// First try directly json index, then try json pointer syntax,
/// return pointer in `JsonPtr` struct, may `None` if both fail.
/// Use `path()` method to return pointer to self node.
///
/// ```rust
/// # use serde_json::json;
/// let v = json!({"i":1,"f":3.14,"a":["pi",null,true]});
/// let p = &v / "i";
/// assert_eq!(p.unwrap(), &v["i"]);
/// let p = &v / "a" / 0;
/// assert_eq!(p.unwrap(), &v["a"][0]);
/// let p = &v / "a/0";
/// assert_eq!(p.unwrap(), &v["a"][0]);
/// ```
impl<'tr, Rhs> Div<Rhs> for &'tr Value where Rhs: Index + Copy + ToString {
    type Output = JsonPtr<'tr>;
    fn div(self, rhs: Rhs) -> Self::Output {
        self.path().path(rhs)
    }
}

/// Pipe operator `|` try to get string from a json node or default `rhs`.
///
/// * if `lhs` is really string node, return the string content.
/// * if `rhs` is empty, stringfy the json for any other type.
/// * if `rhs` is "0", only stringfy node for i64 or u64 type.
/// * if `rhs` is "0.0", only stringfy node for float type.
/// * if `rhs` is "bool", only stringfy node for bool type.
/// * if `rhs` is "[]", only stringfy node for array type.
/// * if `rhs` is "{}", only stringfy node for object type.
/// * otherwise, return `rhs` as default.
///
/// ```rust
/// # use serde_json::json;
/// let v = json!({"int":3, "float":3.14, "str":"null", "array":[1,null,"null"]});
/// 
/// assert_eq!(&v/"str" | "".to_string(), "null");
/// assert_eq!(&v/"int" | "".to_string(), "3");
/// assert_eq!(&v/"int" | "0".to_string(), "3");
/// assert_eq!(&v/"int" | "0.0".to_string(), "0.0");
/// assert_eq!(&v/"array" | "".to_string(), "[1,null,\"null\"]");
/// assert_eq!(&v/"array" | "[]".to_string(), "[1,null,\"null\"]");
/// ```
///
/// Note that the `rhs` string would be moved.
/// If want to only get content from string node, use `| &str` version instead.
impl<'tr> BitOr<String> for &'tr Value {
    type Output = String;
    fn bitor(self, rhs: String) -> Self::Output {
        self.get_string(rhs)
    }
}

/// Pipe operator `|` to get string reference or default `rhs`
/// if the json node type is not string.
/// Usually used with literal `|"default"` or just simple `|""`.
///
/// ```rust
/// # use serde_json::json;
/// let v = json!({"sub":{"key":"val"}});
/// assert_eq!(&v/"sub" | "", "");
/// assert_eq!(&v/"sub"/"key" | "", "val");
/// assert_eq!(&v/"sub"/"any" | "xxx", "xxx");
/// ```
impl<'tr> BitOr<&'tr str> for &'tr Value {
    type Output = &'tr str;
    fn bitor(self, rhs: &'tr str) -> Self::Output {
        self.get_str(rhs)
    }
}

/// Pipe operator `|` to get integer value if the json node hold a integer
/// or string which can parse to integer, or bool which conver to 1 or 0,
/// otherwise return default `rhs`. 
///
/// ```rust
/// # use serde_json::json;
/// let v = json!({"a":1, "b":"2", "c":"nan", "e":true});
/// assert_eq!(&v/"a" | 0, 1);
/// assert_eq!(&v/"b" | 0, 2);
/// assert_eq!(&v/"c" | 0, 0);
/// assert_eq!(&v/"d" | -1, -1);
/// assert_eq!(&v/"e" | -1, 1);
/// ```
///
/// Not support `| u64` overload, only support `| i64`
/// to make `| 0` as simple as possible in most use case.
impl BitOr<i64> for &Value {
    type Output = i64;
    fn bitor(self, rhs: i64) -> Self::Output {
        self.get_i64(rhs)
    }
}

/// Pipe operator `|` to get float value if the json node hold a float
/// or string which can parse to float, otherwise default `rhs`. 
///
/// ```rust
/// # use serde_json::json;
/// let v = json!({"a":1.0, "b":"2.0", "c":"not"});
/// assert_eq!(&v/"a" | 0.0, 1.0);
/// assert_eq!(&v/"b" | 0.0, 2.0);
/// assert_eq!(&v/"c" | 0.0, 0.0);
/// assert_eq!(&v/"d" | -1.0, -1.0);
/// ```
impl BitOr<f64> for &Value {
    type Output = f64;
    fn bitor(self, rhs: f64) -> Self::Output {
        self.get_f64(rhs)
    }
}

/// Pipe operator `|` to get bool value if the json node hold a bool
/// or string which can parse to bool, otherwise default `rhs`. 
/// And for integer node, non-zero value is treated as true, zero is false.
///
/// ```rust
/// # use serde_json::json;
/// let v = json!({"a":1, "b":"2", "c":"nan", "e":true});
/// assert_eq!(&v/"a" | false, true);
/// assert_eq!(&v/"b" | false, false);
/// assert_eq!(&v/"c" | false, false);
/// assert_eq!(&v/"e" | false, true);
/// ```
impl BitOr<bool> for &Value {
    type Output = bool;
    fn bitor(self, rhs: bool) -> Self::Output {
        self.get_bool(rhs)
    }
}

/* ------------------------------------------------------------ */

/// Path operator `/` create a json pointer to sub-node.
/// 
/// First try directly json index, then try json pointer syntax,
/// return mutable pointer in `JsonPtrMut` struct, may `None` if both fail.
/// Use `path_mut()` method to return pointer to self node.
/// Hope to change the node it point to, otherwise better to use immutable version.
///
/// ```rust
/// # use serde_json::json;
/// let mut v = json!({"i":1,"f":3.14,"a":["pi",null,true]});
/// let p = &mut v / "i";
/// assert_eq!(p | 0, 1);
/// let p = &mut v / "a" / 0;
/// assert_eq!(p | "", "pi");
/// let p = &mut v / "a/0";
/// assert_eq!(p | "", "pi");
/// ```
impl<'tr, Rhs> Div<Rhs> for &'tr mut Value where Rhs: Index + Copy + ToString {
    type Output = JsonPtrMut<'tr>;
    fn div(self, rhs: Rhs) -> Self::Output {
        self.path_mut().path(rhs)
    }
}

/// Operator `<<` to put a scalar value into json node, what supported type inclde:
/// &str, String, i64, f64, bool, and unit`()` for json null.
///
/// ```rust
/// # use serde_json::json;
/// # use serde_json::PathOperator;
/// let mut v = json!({});
/// 
/// let _ = v.path_mut() << "pi";
/// assert!(v.is_string());
/// let _ = v.path_mut() << 1;
/// assert!(v.is_i64());
/// let _ = v.path_mut() << 3.14;
/// assert!(v.is_f64());
/// let _ = v.path_mut() << true;
/// assert!(v.is_boolean());
/// let _ = v.path_mut() << ();
/// assert!(v.is_null());
///
/// let pi = String::from("PI");
/// let _ = v.path_mut() << "pi" << 3.14 << pi;
/// assert_eq!(v, "PI");
/// ```
///
/// Though put operator `<<` can be chained, the later one overwrite the previous value.
impl<Rhs> Shl<Rhs> for &mut Value where Rhs: JsonScalar, Value: From<Rhs> {
    type Output = Self;
    fn shl(self, rhs: Rhs) -> Self::Output {
        self.put_value(rhs)
    }
}

/// Operator `<<` to push key-value pair (tuple) into json object.
///
/// If the node is object, the new pair is insert to it,
/// otherwise change the node to object with only one new pair.
/// The key and value will be moved into json node, and key require String conversion.
///
/// ```rust
/// # use serde_json::json;
/// let mut v = json!("init string node");
/// 
/// let _ = &mut v << ("i", 1) << ("f", 3.14);
/// assert_eq!(v, json!({"i":1,"f":3.14}));
/// ```
///
/// It can chain `<<` to object with several pairs, while it may be not good enough
/// to use in large loop.
impl<K: ToString, T> Shl<(K, T)> for &mut Value where Value: From<T> {
    type Output = Self;
    fn shl(self, rhs: (K, T)) -> Self::Output {
        self.push_object(rhs.0, rhs.1)
    }
}

/// Operator `<<` to push one value tuple into json array.
///
/// If the node is array, the new item is push back to it,
/// otherwise change the node to array with only one new item.
///
/// ```rust
/// # use serde_json::json;
/// let mut v = json!("init string node");
/// 
/// let _ = &mut v << ("i",) << (1,) << ("f",) << (3.14,);
/// assert_eq!(v, json!(["i", 1,"f", 3.14]));
/// ```
///
/// Note that use single tuple to distinguish with pushing one value to node.
/// Can also use the other overload `<< ["val"]` instead of `("val",)` which may be
/// more clear to express the meanning for one item in array.
impl<T> Shl<(T,)> for &mut Value where Value: From<T> {
    type Output = Self;
    fn shl(self, rhs: (T,)) -> Self::Output {
        self.push_array(rhs.0)
    }
}

/// Operator `<<` to push one item to json array.
///
/// If the node is array, the new item is push back to it,
/// otherwise change the node to array with only one new item.
///
/// ```rust
/// # use serde_json::json;
/// let mut v = json!("init string node");
/// 
/// let _ = &mut v << ["i"] << [1] << ["f"] << [3.14];
/// assert_eq!(v, json!(["i", 1,"f", 3.14]));
/// ```
impl<T: Copy> Shl<[T;1]> for &mut Value where Value: From<T> {
    type Output = Self;
    fn shl(self, rhs: [T;1]) -> Self::Output {
        self.push_array(rhs[0])
    }
}

/// Operator `<<` to push a slice to json array.
///
/// If the node is array, the new items is append back,
/// otherwise change the node to array with only the new items.
///
/// ```rust
/// # use serde_json::json;
/// let mut v = json!("init string node");
/// 
/// let vi = vec![1, 2, 3, 4];
/// let _ = &mut v << &vi[..] << [5] << (6,);
/// assert_eq!(v, json!([1,2,3,4,5,6]));
/// ```
impl<T: Copy> Shl<&[T]> for &mut Value where Value: From<T> {
    type Output = Self;
    fn shl(self, rhs: &[T]) -> Self::Output {
        for item in rhs {
            self.push_array(*item);
        }
        self
    }
}

/* ------------------------------------------------------------ */

/// Operator `&mut Value | FnOnce`, perform some action to json tree.
///
/// Return mutable reference to the same modified json tree,
/// and then can chain with other operator.
///
/// ```rust
/// # use serde_json::{json, Value};
/// let remove_null = |v: &mut Value| {
///     if let Value::Array(array) = v {
///         array.retain(|x| !x.is_null())
///     }
/// };
///
/// let mut v = json!(["pi", null, 3.14]);
/// let second = (&mut v | remove_null) / 1 | 0.0;
/// assert_eq!(second, 3.14);
/// assert_eq!(v, json!(["pi", 3.14]));
/// ```
///
/// Note in above that, the first `|` pipe json tree to function, 
/// while the last one pipe leaf node to scalar to read the value in it,
/// and the operator `/` has higher priority than `|` so () is required.
///
/// See also operator `&mut Value | ()` to remove null recursively.
impl<F> BitOr<F> for &mut Value where F: FnOnce(&mut Value) {
    type Output = Self;
    fn bitor(self, rhs: F) -> Self::Output {
        self.pipe_fn(rhs)
    }
}

/// Operator `&mut Value | &mut Value`, merge rhs to lhs recursively.
///
/// Each leaf node in `rhs` would move to `lhs` if the corresponding node in `lhs`
/// is null or absent.
/// When the array in `rhs` has only one item then it reeatedly compare 
/// with each array item in `lhs` and switch to copy to `lhs` as needed,
/// otherwise compare one by one and the extra items in `rhs` move to 
/// the end of `rhs` array.
///
/// ```rust
/// # use serde_json::json;
/// let mut va = json!({"name": "pi"});
/// let mut vb = json!({"name": "PI", "value": "3.14"});
/// let _ = &mut va | &mut vb;
/// assert_eq!(va, json!({"name":"pi", "value":"3.14"}));
/// assert_eq!(vb, json!({"name": "PI", "value":null}));
/// 
/// va = json!(["pi", null, null, 2.72]);
/// vb = json!(["PI", 3.14, "e", 2.71, false, null, 6.18]);
/// let _ = &mut va | &mut vb;
/// assert_eq!(va, json!(["pi", 3.14, "e", 2.72, false, null, 6.18]));
/// assert_eq!(vb, json!(["PI", null, null, 2.71, null, null, null]));
/// ```
impl BitOr<&mut Value> for &mut Value {
    type Output = Self;
    fn bitor(self, rhs: &mut Value) -> Self::Output {
        self.merge_move(rhs)
    }
}

/// Operator `&mut Value | &Value`, merge rhs to lhs recursively.
///
/// Each leaf node in `rhs` would copy to `lhs` if the corresponding node in `lhs`
/// is null or absent.
/// When the array in `rhs` has only one item then it reeatedly compare 
/// with each array item in `lhs`, otherwise compare one by one and the
/// extra items in `rhs` copy to the end of `rhs` array.
///
/// ```rust
/// # use serde_json::json;
/// let mut va = json!({"name": "pi"});
/// let mut vb = json!({"name": "PI", "value": "3.14"});
/// let _ = &mut va | &vb;
/// assert_eq!(va, json!({"name":"pi", "value":"3.14"}));
/// 
/// va = json!(["pi", null, null, 2.72]);
/// vb = json!(["PI", 3.14, "e", 2.71, false, null, 6.18]);
/// let _ = &mut va | &vb;
/// assert_eq!(va, json!(["pi", 3.14, "e", 2.72, false, null, 6.18]));
/// ```
impl BitOr<&Value> for &mut Value {
    type Output = Self;
    fn bitor(self, rhs: &Value) -> Self::Output {
        self.merge_copy(rhs)
    }
}

/// Operator `&mut Value & &Value`, filter `lhs` recursively by `rhs`
/// as template or sample structure.
///
/// Each leaf node in `lhs` would set to null if the corresponding node in `rhs`
/// has different type or absent. The null node only marked while not actually remove
/// from it's parent node, may chain to `| ()` to achieve that purpose.
/// When the array in `rhs` has only one item then it reeatedly compare 
/// with each array item in `lhs`, otherwise compare one by one.
///
/// ```rust
/// # use serde_json::json;
/// let mut va = json!({"name": "pi", "VALUE": 3.14});
/// let mut vb = json!({"name": "PI", "value": "3.14"});
/// let _ = &mut va & &vb;
/// assert_eq!(va, json!({"name":"pi", "VALUE": null}));
/// 
/// va = json!(["pi", "314", null, 2.72, true, "xx", 6.18]);
/// vb = json!(["PI", 3.14, "e", 2.71, false, null]);
/// let _ = &mut va & &vb;
/// assert_eq!(va, json!(["pi", null, null, 2.72, true, null, null]));
/// 
/// va = json!(["pi", "314", null, 2.72, true, "xx", 6.18]);
/// let _ = &mut va & &vb | ();
/// assert_eq!(va, json!(["pi", 2.72, true]));
/// ```
///
/// Note that the bitand `&` operator is selected after `|` to perform
/// different operation on two json tree, and they both are special case
/// for pipe to function `&mut Value | FnOnce`.
/// Since `&` is also used for reference type, this bitand operator may be
/// more clear when used with `JsonPtrMut` type.
impl BitAnd<&Value> for &mut Value {
    type Output = Self;
    fn bitand(self, rhs: &Value) -> Self::Output {
        self.filter_shape(rhs)
    }
}

/// Operator `&mut Value | ()`, remove null node recursively.
///
/// ```rust
/// # use serde_json::json;
/// let mut v = json!({"null": "null", "Null": null,
///     "array": ["null", null, {"i": 10, "n": null, "s": "null"}],
///     "object": {"1":null, "a":[,{}, {}], "3":{}}
/// });
/// let _ = &mut v | ();
/// assert_eq!(v, json!({"null":"null","array":["null", {"i":10,"s":"null"}]}))
/// ```
impl BitOr<()> for &mut Value {
    type Output = Self;
    fn bitor(self, _rhs: ()) -> Self::Output {
        self.remove_null()
    }
}
