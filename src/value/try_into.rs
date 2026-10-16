use core::convert::TryInto;
use alloc::string::String;
use alloc::vec::Vec;
use crate::map::Map;
use crate::error::Error;
use super::Value;

impl TryInto<String> for Value {
    type Error = Error;

    fn try_into(self) -> Result<String, Error> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err(<Error as serde::de::Error>::custom("value is not a string")),
        }
    }
}

impl<'a> TryInto<&'a String> for &'a Value {
    type Error = Error;

    fn try_into(self) -> Result<&'a String, Error> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err(<Error as serde::de::Error>::custom("value is not a string")),
        }
    }
}

impl<'a> TryInto<&'a str> for &'a Value {
    type Error = Error;

    fn try_into(self) -> Result<&'a str, Error> {
        self.as_str().ok_or_else(|| <Error as serde::de::Error>::custom("value is not a string"))
    }
}

impl TryInto<f64> for Value {
    type Error = Error;

    fn try_into(self) -> Result<f64, Error> {
        self.as_f64().ok_or_else(|| <Error as serde::de::Error>::custom("value is not an f64"))
    }
}


impl TryInto<f64> for &Value {
    type Error = Error;

    fn try_into(self) -> Result<f64, Error> {
        self.as_f64().ok_or_else(|| <Error as serde::de::Error>::custom("value is not an f64"))
    }
}

impl TryInto<i64> for Value {
    type Error = Error;

    fn try_into(self) -> Result<i64, Error> {
        self.as_i64().ok_or_else(|| <Error as serde::de::Error>::custom("value is not an i64"))
    }
}

impl TryInto<i64> for &Value {
    type Error = Error;

    fn try_into(self) -> Result<i64, Error> {
        self.as_i64().ok_or_else(|| <Error as serde::de::Error>::custom("value is not an i64"))
    }
}

impl TryInto<u64> for Value {
    type Error = Error;

    fn try_into(self) -> Result<u64, Error> {
        self.as_u64().ok_or_else(|| <Error as serde::de::Error>::custom("value is not an u64"))
    }
}

impl TryInto<u64> for &Value {
    type Error = Error;

    fn try_into(self) -> Result<u64, Error> {
        self.as_u64().ok_or_else(|| <Error as serde::de::Error>::custom("value is not an u64"))
    }
}

impl TryInto<bool> for Value {
    type Error = Error;

    fn try_into(self) -> Result<bool, Error> {
        self.as_bool().ok_or_else(|| <Error as serde::de::Error>::custom("value is not a bool"))
    }
}

impl TryInto<bool> for &Value {
    type Error = Error;

    fn try_into(self) -> Result<bool, Error> {
        self.as_bool().ok_or_else(|| <Error as serde::de::Error>::custom("value is not a bool"))
    }
}

impl<'a> TryInto<&'a Vec<Value>> for &'a Value {
    type Error = Error;

    fn try_into(self) -> Result<&'a Vec<Value>, Error> {
        self.as_array().ok_or_else(|| <Error as serde::de::Error>::custom("value is not an array"))
    }
}

impl<'a> TryInto<&'a [Value]> for &'a Value {
    type Error = Error;

    fn try_into(self) -> Result<&'a [Value], Error> {
        self.as_array().map(|v| v.as_slice()).ok_or_else(|| <Error as serde::de::Error>::custom("value is not an array"))
    }
}

impl TryInto<Vec<Value>> for Value {
    type Error = Error;

    fn try_into(self) -> Result<Vec<Value>, Error> {
        match self {
            Value::Array(a) => Ok(a),
            _ => Err(<Error as serde::de::Error>::custom("value is not an array")),
        }
    }
}

impl TryInto<Map<String, Value>> for Value {
    type Error = Error;

    fn try_into(self) -> Result<Map<String, Value>, Error> {
        match self {
            Value::Object(o) => Ok(o),
            _ => Err(<Error as serde::de::Error>::custom("value is not an object")),
        }
    }
}

impl<'a> TryInto<&'a Map<String, Value>> for &'a Value {
    type Error = Error;

    fn try_into(self) -> Result<&'a Map<String, Value>, Error> {
        self.as_object().ok_or_else(|| <Error as serde::de::Error>::custom("value is not an object"))
    }
}

impl TryInto<()> for Value {
    type Error = Error;

    fn try_into(self) -> Result<(), Error> {
        self.as_null().ok_or_else(|| <Error as serde::de::Error>::custom("value is not a null"))
    }
}

impl TryInto<()> for &Value {
    type Error = Error;

    fn try_into(self) -> Result<(), Error> {
        self.as_null().ok_or_else(|| <Error as serde::de::Error>::custom("value is not a null"))
    }
}
