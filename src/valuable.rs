use crate::{Map, Value as Json};
use alloc::string::String;

use valuable::{Mappable, Valuable, Value, Visit};

impl Valuable for Json {
    fn as_value(&self) -> Value<'_> {
        match self {
            Json::Array(ref array) => array.as_value(),
            Json::Bool(ref value) => value.as_value(),
            Json::Number(ref num) => {
                if num.is_f64() {
                    Value::F64(num.as_f64().unwrap())
                } else if num.is_i64() {
                    Value::I64(num.as_i64().unwrap())
                } else {
                    unreachable!()
                }
            }
            Json::Null => Value::Unit,
            Json::String(ref s) => s.as_value(),
            Json::Object(ref object) => object.as_value(),
        }
    }

    fn visit(&self, visit: &mut dyn Visit) {
        match self {
            Json::Array(ref array) => array.visit(visit),
            Json::Bool(ref value) => value.visit(visit),
            Json::Number(ref num) => {
                if num.is_f64() {
                    num.as_f64().unwrap().visit(visit)
                } else if num.is_i64() {
                    num.as_i64().unwrap().visit(visit)
                } else {
                    unreachable!()
                }
            }
            Json::Null => Value::Unit.visit(visit),
            Json::String(ref s) => s.visit(visit),
            Json::Object(ref object) => object.visit(visit),
        }
    }
}

impl Valuable for Map<String, Json> {
    fn as_value(&self) -> Value<'_> {
        Value::Mappable(self)
    }

    fn visit(&self, visit: &mut dyn Visit) {
        for (k, v) in self.iter() {
            visit.visit_entry(k.as_value(), v.as_value());
        }
    }
}

impl Mappable for Map<String, Json> {
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

#[cfg(test)]
mod test {
    use crate::json;
    use valuable::{Valuable, Value};

    #[test]
    fn test_json() {
        let j = json!({"a": 100, "b": 1.0, "c": -1});
        let jv = j.as_value();

        assert!(matches!(jv, Value::Mappable(_)));

        assert!(matches!(json!(100).as_value(), Value::I64(_)));
    }
}
