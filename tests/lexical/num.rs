use crate::lexical::num::{AsPrimitive, Float, Integer, Number};

fn check_as_primitive<T: AsPrimitive>(t: T) {
    let _: u8 = t.as_u8();
    let _: u16 = t.as_u16();
    let _: u32 = t.as_u32();
    let _: u64 = t.as_u64();
    let _: u128 = t.as_u128();
    let _: usize = t.as_usize();
    let _: i8 = t.as_i8();
    let _: i16 = t.as_i16();
    let _: i32 = t.as_i32();
    let _: i64 = t.as_i64();
    let _: i128 = t.as_i128();
    let _: isize = t.as_isize();
    let _: f32 = t.as_f32();
    let _: f64 = t.as_f64();
}

#[test]
fn as_primitive_test() {
    check_as_primitive(1u8);
    check_as_primitive(1u16);
    check_as_primitive(1u32);
    check_as_primitive(1u64);
    check_as_primitive(1u128);
    check_as_primitive(1usize);
    check_as_primitive(1i8);
    check_as_primitive(1i16);
    check_as_primitive(1i32);
    check_as_primitive(1i64);
    check_as_primitive(1i128);
    check_as_primitive(1isize);
    check_as_primitive(1f32);
    check_as_primitive(1f64);
}

fn check_number<T: Number>(x: T, mut y: T) {
    // Copy, partialeq, partialord
    let _ = x;
    assert!(x < y);
    assert!(x != y);

    // Operations
    let _ = y + x;
    let _ = y - x;
    let _ = y * x;
    let _ = y / x;
    let _ = y % x;
    y += x;
    y -= x;
    y *= x;
    y /= x;
    y %= x;

    // Conversions already tested.
}

#[test]
fn number_test() {
    check_number(1u8, 5);
    check_number(1u16, 5);
    check_number(1u32, 5);
    check_number(1u64, 5);
    check_number(1u128, 5);
    check_number(1usize, 5);
    check_number(1i8, 5);
    check_number(1i16, 5);
    check_number(1i32, 5);
    check_number(1i64, 5);
    check_number(1i128, 5);
    check_number(1isize, 5);
    check_number(1f32, 5.0);
    check_number(1f64, 5.0);
}

fn check_integer<T: Integer>(x: T) {
    // Bitwise operations
    let _ = x & T::ZERO;
}

#[test]
fn integer_test() {
    check_integer(65u8);
    check_integer(65u16);
    check_integer(65u32);
    check_integer(65u64);
    check_integer(65u128);
    check_integer(65usize);
    check_integer(65i8);
    check_integer(65i16);
    check_integer(65i32);
    check_integer(65i64);
    check_integer(65i128);
    check_integer(65isize);
}

fn check_float<T: Float>(x: T) {
    // Check functions
    let _ = x.pow10(5);
    let _ = x.to_bits();
    assert!(T::from_bits(x.to_bits()) == x);

    // Check properties
    let _ = x.to_bits() & T::SIGN_MASK;
    let _ = x.to_bits() & T::EXPONENT_MASK;
    let _ = x.to_bits() & T::HIDDEN_BIT_MASK;
    let _ = x.to_bits() & T::MANTISSA_MASK;
}

#[test]
fn float_test() {
    check_float(123f32);
    check_float(123f64);
}
