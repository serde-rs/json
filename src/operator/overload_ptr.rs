// Not sub mod but seperate file for operator overload interface.
// Used by include! macro in operator mod.

/* ------------------------------------------------------------ */

/// Overload `*` deref operator to treate pointer as `Option<&json::Value>`.
impl<'tr> Deref for JsonPtr<'tr>
{
    type Target = Option<&'tr Value>;
    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}

/// Path operator `/`, visit sub-node by string key for object or index for array.
/// 
/// First try directly json index, then try json pointer syntax, otherwise
/// return `None` if both fail.
///
/// ```rust
/// # use serde_json::json;
/// # use serde_json::PathOperator;
/// let v = json!({"i":1,"f":3.14,"a":["pi",null,true]});
/// let p = v.path() / "i";
/// assert_eq!(p.unwrap(), &v["i"]);
/// let p = v.path() / "a" / 0;
/// assert_eq!(p.unwrap(), &v["a"][0]);
/// let p = v.path() / "a/0";
/// assert_eq!(p.unwrap(), &v["a"][0]);
/// ```
impl<'tr, Rhs> Div<Rhs> for JsonPtr<'tr> where Rhs: Index + Copy + ToString
{
    type Output = Self;
    fn div(self, rhs: Rhs) -> Self::Output {
        self.path(rhs)
    }
}

/// Pipe operator `|` to get string refer or default `rhs`
/// when invalid pointer or the json type is not string.
/// Usually used with literal `|"default"` or just simple `|""`.
///
/// ```rust
/// # use serde_json::json;
/// # use serde_json::PathOperator;
/// let v = json!({"sub":{"key":"val"}});
/// assert_eq!(v.path()/"sub" | "", "");
/// assert_eq!(v.path()/"sub"/"key" | "", "val");
/// assert_eq!(v.path()/"sub"/"any" | "xxx", "xxx");
/// ```
impl<'tr> BitOr<&'tr str> for JsonPtr<'tr> {
    type Output = &'tr str;
    fn bitor(self, rhs: &'tr str) -> Self::Output {
        self.get_str(rhs)
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
/// # use serde_json::PathOperator;
/// let v = json!({"int":3, "float":3.14, "str":"null", "array":[1,null,"null"]});
/// 
/// assert_eq!(v.path()/"str" | "".to_string(), "null");
/// assert_eq!(v.path()/"int" | "".to_string(), "3");
/// assert_eq!(v.path()/"int" | "0".to_string(), "3");
/// assert_eq!(v.path()/"int" | "0.0".to_string(), "0.0");
/// assert_eq!(v.path()/"array" | "".to_string(), "[1,null,\"null\"]");
/// assert_eq!(v.path()/"array" | "[]".to_string(), "[1,null,\"null\"]");
/// ```
///
/// Note that the `rhs` string would be moved.
/// If want to only get content from string node, use `| &str` version instead.
impl<'tr> BitOr<String> for JsonPtr<'tr> {
    type Output = String;
    fn bitor(self, rhs: String) -> Self::Output {
        self.get_string(rhs)
    }
}

/// Pipe operator `|` to get integer value if the json node don't hold a integer
/// or string which can parse to integer, or bool which conver to 1 or 0,
/// otherwise return default `rhs`. 
///
/// ```rust
/// # use serde_json::json;
/// # use serde_json::PathOperator;
/// let v = json!({"a":1, "b":"2", "c":"nan", "e":true});
/// assert_eq!(v.path()/"a" | 0, 1);
/// assert_eq!(v.path()/"b" | 0, 2);
/// assert_eq!(v.path()/"c" | 0, 0);
/// assert_eq!(v.path()/"d" | -1, -1);
/// assert_eq!(v.path()/"e" | -1, 1);
/// ```
///
/// Not support `| u64` overload, only support `| i64`
/// to make `| 0` as simple as possible in most use case.
impl<'tr> BitOr<i64> for JsonPtr<'tr> {
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
/// # use serde_json::PathOperator;
/// let v = json!({"a":1.0, "b":"2.0", "c":"not"});
/// assert_eq!(v.path()/"a" | 0.0, 1.0);
/// assert_eq!(v.path()/"b" | 0.0, 2.0);
/// assert_eq!(v.path()/"c" | 0.0, 0.0);
/// assert_eq!(v.path()/"d" | -1.0, -1.0);
/// ```
impl<'tr> BitOr<f64> for JsonPtr<'tr> {
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
/// # use serde_json::PathOperator;
/// let v = json!({"a":1, "b":"2", "c":"nan", "e":true});
/// assert_eq!(v.path()/"a" | false, true);
/// assert_eq!(v.path()/"b" | false, false);
/// assert_eq!(v.path()/"c" | false, false);
/// assert_eq!(v.path()/"e" | false, true);
/// ```
impl<'tr> BitOr<bool> for JsonPtr<'tr> {
    type Output = bool;
    fn bitor(self, rhs: bool) -> Self::Output {
        self.get_bool(rhs)
    }
}

/* ------------------------------------------------------------ */

/// Overload `*` deref operator to treate pointer as `Option<&mut json::Value>`.
impl<'tr> Deref for JsonPtrMut<'tr> {
    type Target = Option<&'tr mut Value>;
    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}

/// Overload `*` deref operator to treate pointer as `Option<&mut json::Value>`.
impl<'tr> DerefMut for JsonPtrMut<'tr> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ptr
    }
}

/// Path operator `/`, visit sub-node by string key for object or index for array.
/// Can chained as `jsonptr / "path" / "to" / "node"` or `jsonptr / "path/to/node"`.
/// First try directly json index, then try json pointer syntax, then
/// return `None` if both fail.
/// Hope to change the node it point to, otherwise better to use immutable `JsonPtr`.
///
/// ```rust
/// # use serde_json::json;
/// # use serde_json::PathOperator;
/// let mut v = json!({"i":1,"f":3.14,"a":["pi",null,true]});
/// let p = v.path_mut() / "i";
/// assert_eq!(p | 0, 1);
/// let p = v.path_mut() / "a" / 0;
/// assert_eq!(p | "", "pi");
/// let p = v.path_mut() / "a/0";
/// assert_eq!(p | "", "pi");
/// ```
impl<'tr, Rhs> Div<Rhs> for JsonPtrMut<'tr> where Rhs: Index + Copy + ToString
{
    type Output = Self;
    fn div(mut self, rhs: Rhs) -> Self::Output {
        self.path(rhs)
    }
}

/// Pipe operator `|` to get string refer or default `rhs`.
/// 
/// Behaves the same as `JsonPtr | &str`, except that
/// the `JsonPtrMut` in `lhs` would be moved and cannot used any more.
impl<'tr> BitOr<&'tr str> for JsonPtrMut<'tr> {
    type Output = &'tr str;
    fn bitor(mut self, rhs: &'tr str) -> Self::Output {
        self.immut().bitor(rhs)
    }
}

/// Proxy of `|` operator overload for mutable json pointer.
/// Would expand for String, i64, f64, bool.
macro_rules! bitor_mut {
    ($rhs:ty) => {
        impl<'tr> BitOr<$rhs> for JsonPtrMut<'tr> {
            type Output = $rhs;
            fn bitor(mut self, rhs: $rhs) -> Self::Output {
                self.immut().bitor(rhs)
            }
        }
    };
}

bitor_mut!(String);
bitor_mut!(i64);
bitor_mut!(f64);
bitor_mut!(bool);

/// Operator `<<` to put a scalar value into json node, what supported type inclde:
/// &str, String, i64, f64, bool, and unit() for json null.
/// It will consume the `lhs` pointer and return a new one point to the same node
/// after modify it's content and may type.
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
impl<'tr, Rhs> Shl<Rhs> for JsonPtrMut<'tr> where Rhs: JsonScalar, Value: From<Rhs> {
    type Output = Self;
    fn shl(mut self, rhs: Rhs) -> Self::Output {
        self.put_value(rhs)
    }
}

/// Operator `<<` to push key-value pair (tuple) into json object.
///
/// It will consume the `lhs` pointer and return a new one point to the same node
/// after modify it's content and type.
/// If the node is object, the new pair is insert to it,
/// otherwise change the node to object with only one new pair.
/// The key and value will be moved into json node, and key require String conversion.
///
/// ```rust
/// # use serde_json::json;
/// # use serde_json::PathOperator;
/// let mut v = json!("init string node");
/// 
/// let _ = v.path_mut() << ("i", 1) << ("f", 3.14);
/// assert_eq!(v, json!({"i":1,"f":3.14}));
/// ```
///
/// It can chain `<<` to object with several pairs, while it may be not good enough
/// to use in large loop.
impl<'tr, K: ToString, T> Shl<(K, T)> for JsonPtrMut<'tr> where Value: From<T> {
    type Output = Self;
    fn shl(mut self, rhs: (K, T)) -> Self::Output {
        self.push_object(rhs.0, rhs.1)
    }
}

/// Operator `<<` to push one value tuple into json array.
///
/// It will consume the `lhs` pointer and return a new one point to the same node
/// after modify it's content and type, and so is chainable.
/// If the node is array, the new item is push back to it,
/// otherwise change the node to array with only one new item.
///
/// ```rust
/// # use serde_json::json;
/// # use serde_json::PathOperator;
/// let mut v = json!("init string node");
/// 
/// let _ = v.path_mut() << ("i",) << (1,) << ("f",) << (3.14,);
/// assert_eq!(v, json!(["i", 1,"f", 3.14]));
/// ```
///
/// Note that use single tuple to distinguish with pushing one value to node.
/// Can also use the other overload `<< ["val"]` instead of `("val",)` which may be
/// more clear to express the meanning for one item in array.
impl<'tr, T> Shl<(T,)> for JsonPtrMut<'tr> where Value: From<T> {
    type Output = Self;
    fn shl(mut self, rhs: (T,)) -> Self::Output {
        self.push_array(rhs.0)
    }
}

/// Operator `<<` to push one item to json array.
///
/// It will consume the `lhs` pointer and return a new one point to the same node
/// after modify it's content and type, and so is chainable.
/// If the node is array, the new item is push back to it,
/// otherwise change the node to array with only one new item.
///
/// ```rust
/// # use serde_json::json;
/// # use serde_json::PathOperator;
/// let mut v = json!("init string node");
/// 
/// let _ = v.path_mut() << ["i"] << [1] << ["f"] << [3.14];
/// assert_eq!(v, json!(["i", 1,"f", 3.14]));
/// ```
impl<'tr, T: Copy> Shl<[T;1]> for JsonPtrMut<'tr> where Value: From<T> {
    type Output = Self;
    fn shl(mut self, rhs: [T;1]) -> Self::Output {
        self.push_array(rhs[0])
    }
}

/// Operator `<<` to push a slice to json array.
///
/// It will consume the `lhs` pointer and return a new one point to the same node
/// after modify it's content and type, and so can chain further.
/// If the node is array, the new items is append back,
/// otherwise change the node to array with only the new items.
///
/// ```rust
/// # use serde_json::json;
/// # use serde_json::PathOperator;
/// let mut v = json!("init string node");
/// 
/// let vi = vec![1, 2, 3, 4];
/// let _ = v.path_mut() << &vi[..] << [5] << (6,);
/// assert_eq!(v, json!([1,2,3,4,5,6]));
/// ```
impl<'tr, T: Copy> Shl<&[T]> for JsonPtrMut<'tr> where Value: From<T> {
    type Output = Self;
    fn shl(mut self, rhs: &[T]) -> Self::Output {
        for item in rhs {
            self = self.push_array(*item);
        }
        self
    }
}

/* ------------------------------------------------------------ */

/// Operator `JsonPtrMut | FnOnce`, perform some action to json tree.
///
/// Return a new mutable pointer that points to the same modified tree,
/// and then can chain with other operator.
///
/// ```rust
/// # use serde_json::{json, Value};
/// # use serde_json::PathOperator;
/// let remove_null = |v: &mut Value| {
///     if let Value::Array(array) = v {
///         array.retain(|x| !x.is_null())
///     }
/// };
///
/// let mut v = json!(["pi", null, 3.14]);
/// let second = (v.path_mut() | remove_null) / 1 | 0.0;
/// assert_eq!(second, 3.14);
/// assert_eq!(v, json!(["pi", 3.14]));
/// ```
///
/// Note in above that, the first `|` pipe json tree to function, 
/// while the last one pipe leaf node to scalar to read the value in it,
/// and the operator `/` has higher priority than `|` so () is required.
///
/// See also operator `JsonPtrMut | ()` to remove null recursively.
impl<'tr, F> BitOr<F> for JsonPtrMut<'tr> where F: FnOnce(&mut Value) {
    type Output = Self;
    fn bitor(self, rhs: F) -> Self::Output {
        self.pipe_fn(rhs)
    }
}

/// Operator `JsonPtrMut | JsonPtrMut`, merge rhs to lhs recursively.
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
/// # use serde_json::PathOperator;
/// let mut va = json!({"name": "pi"});
/// let mut vb = json!({"name": "PI", "value": "3.14"});
/// let _ = va.path_mut() | vb.path_mut();
/// assert_eq!(va, json!({"name":"pi", "value":"3.14"}));
/// assert_eq!(vb, json!({"name": "PI", "value":null}));
/// 
/// va = json!(["pi", null, null, 2.72]);
/// vb = json!(["PI", 3.14, "e", 2.71, false, null, 6.18]);
/// let _ = va.path_mut() | vb.path_mut();
/// assert_eq!(va, json!(["pi", 3.14, "e", 2.72, false, null, 6.18]));
/// assert_eq!(vb, json!(["PI", null, null, 2.71, null, null, null]));
/// ```
impl<'tr> BitOr<JsonPtrMut<'tr>> for JsonPtrMut<'tr> {
    type Output = Self;
    fn bitor(self, rhs: JsonPtrMut<'tr>) -> Self::Output {
        self.merge_move(rhs)
    }
}

/// Operator `JsonPtrMut | JsonPtr`, merge rhs to lhs recursively.
///
/// Each leaf node in `rhs` would copy to `lhs` if the corresponding node in `lhs`
/// is null or absent.
/// When the array in `rhs` has only one item then it reeatedly compare 
/// with each array item in `lhs`, otherwise compare one by one and the
/// extra items in `rhs` copy to the end of `rhs` array.
///
/// ```rust
/// # use serde_json::json;
/// # use serde_json::PathOperator;
/// let mut va = json!({"name": "pi"});
/// let mut vb = json!({"name": "PI", "value": "3.14"});
/// let _ = va.path_mut() | vb.path();
/// assert_eq!(va, json!({"name":"pi", "value":"3.14"}));
/// assert_eq!(vb, json!({"name": "PI", "value":"3.14"}));
/// 
/// va = json!(["pi", null, null, 2.72]);
/// vb = json!(["PI", 3.14, "e", 2.71, false, null, 6.18]);
/// let _ = va.path_mut() | vb.path();
/// assert_eq!(va, json!(["pi", 3.14, "e", 2.72, false, null, 6.18]));
/// assert_eq!(vb, json!(["PI", 3.14, "e", 2.71, false, null, 6.18]));
/// ```
impl<'tr> BitOr<JsonPtr<'tr>> for JsonPtrMut<'tr> {
    type Output = Self;
    fn bitor(self, rhs: JsonPtr<'tr>) -> Self::Output {
        self.merge_copy(rhs)
    }
}

/// Operator `JsonPtrMut & JsonPtr`, filter `lhs` recursively by `rhs`
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
/// # use serde_json::PathOperator;
/// let mut va = json!({"name": "pi", "VALUE": 3.14});
/// let mut vb = json!({"name": "PI", "value": "3.14"});
/// let _ = va.path_mut() & vb.path();
/// assert_eq!(va, json!({"name":"pi", "VALUE": null}));
/// 
/// va = json!(["pi", "314", null, 2.72, true, "xx", 6.18]);
/// vb = json!(["PI", 3.14, "e", 2.71, false, null]);
/// let _ = va.path_mut() & vb.path();
/// assert_eq!(va, json!(["pi", null, null, 2.72, true, null, null]));
/// 
/// va = json!(["pi", "314", null, 2.72, true, "xx", 6.18]);
/// let _ = va.path_mut() & vb.path() | ();
/// assert_eq!(va, json!(["pi", 2.72, true]));
/// ```
///
/// Note that the bitand `&` operator is selected after `|` to perform
/// different operation on two json tree, and they both are special case
/// for pipe to function `JsonPtrMut | FnOnce`.
impl<'tr> BitAnd<JsonPtr<'tr>> for JsonPtrMut<'tr> {
    type Output = Self;
    fn bitand(self, rhs: JsonPtr<'tr>) -> Self::Output {
        self.filter_shape(rhs)
    }
}

/// Operator `JsonPtrMut | ()`, remove null node recursively.
///
/// ```rust
/// # use serde_json::json;
/// # use serde_json::PathOperator;
/// let mut v = json!({"null": "null", "Null": null,
///     "array": ["null", null, {"i": 10, "n": null, "s": "null"}],
///     "object": {"1":null, "a":[,{}, {}], "3":{}}
/// });
/// let _ = v.path_mut() | ();
/// assert_eq!(v, json!({"null":"null","array":["null", {"i":10,"s":"null"}]}))
/// ```
impl<'tr> BitOr<()> for JsonPtrMut<'tr> {
    type Output = Self;
    fn bitor(self, _rhs: ()) -> Self::Output {
        self.remove_null()
    }
}
