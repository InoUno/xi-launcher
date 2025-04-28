#![allow(unused)]

pub const fn default_true() -> bool {
    true
}

pub const fn is_true(value: &bool) -> bool {
    *value
}

pub const fn is_false(value: &bool) -> bool {
    !*value
}

pub fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}

pub fn vec_is_empty<T>(value: &Vec<T>) -> bool {
    value.is_empty()
}

pub const fn default_minus_one() -> i32 {
    -1
}

pub const fn is_minus_one(value: &i32) -> bool {
    *value == -1
}
