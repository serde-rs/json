use super::SpannedValue;
use alloc::string::String;

fn eq_i64(value: &SpannedValue, other: i64) -> bool {
    value.as_i64() == Some(other)
}

fn eq_u64(value: &SpannedValue, other: u64) -> bool {
    value.as_u64() == Some(other)
}

fn eq_f32(value: &SpannedValue, other: f32) -> bool {
    match value {
        SpannedValue::Number(n) => n.as_f32() == Some(other),
        _ => false,
    }
}

fn eq_f64(value: &SpannedValue, other: f64) -> bool {
    value.as_f64() == Some(other)
}

fn eq_bool(value: &SpannedValue, other: bool) -> bool {
    value.as_bool() == Some(other)
}

fn eq_str(value: &SpannedValue, other: &str) -> bool {
    value.as_str() == Some(other)
}

impl PartialEq<str> for SpannedValue {
    fn eq(&self, other: &str) -> bool {
        eq_str(self, other)
    }
}

impl PartialEq<&str> for SpannedValue {
    fn eq(&self, other: &&str) -> bool {
        eq_str(self, *other)
    }
}

impl PartialEq<SpannedValue> for str {
    fn eq(&self, other: &SpannedValue) -> bool {
        eq_str(other, self)
    }
}

impl PartialEq<SpannedValue> for &str {
    fn eq(&self, other: &SpannedValue) -> bool {
        eq_str(other, *self)
    }
}

impl PartialEq<String> for SpannedValue {
    fn eq(&self, other: &String) -> bool {
        eq_str(self, other.as_str())
    }
}

impl PartialEq<SpannedValue> for String {
    fn eq(&self, other: &SpannedValue) -> bool {
        eq_str(other, self.as_str())
    }
}

macro_rules! partialeq_numeric {
    ($($eq:ident [$($ty:ty)*])*) => {
        $($(
            impl PartialEq<$ty> for SpannedValue {
                fn eq(&self, other: &$ty) -> bool {
                    $eq(self, *other as _)
                }
            }

            impl PartialEq<SpannedValue> for $ty {
                fn eq(&self, other: &SpannedValue) -> bool {
                    $eq(other, *self as _)
                }
            }

            impl<'a> PartialEq<$ty> for &'a SpannedValue {
                fn eq(&self, other: &$ty) -> bool {
                    $eq(*self, *other as _)
                }
            }

            impl<'a> PartialEq<$ty> for &'a mut SpannedValue {
                fn eq(&self, other: &$ty) -> bool {
                    $eq(*self, *other as _)
                }
            }
        )*)*
    }
}

partialeq_numeric! {
    eq_i64[i8 i16 i32 i64 isize]
    eq_u64[u8 u16 u32 u64 usize]
    eq_f32[f32]
    eq_f64[f64]
    eq_bool[bool]
}
