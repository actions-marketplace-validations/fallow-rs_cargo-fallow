//! A seed with the common item kinds.

use std::collections::BTreeMap;

/// A struct.
#[derive(Debug, Clone)]
pub struct Point<T: Copy> {
    x: T,
    y: T,
}

pub enum Shape {
    Circle { radius: f64 },
    Square(f64),
}

pub trait Area {
    fn area(&self) -> f64;
}

impl Area for Shape {
    fn area(&self) -> f64 {
        match *self {
            Self::Circle { radius } => 3.14 * radius * radius,
            Self::Square(side) => side * side,
        }
    }
}

async fn load<'a>(map: &'a BTreeMap<String, Vec<u8>>, key: &str) -> Option<&'a [u8]> {
    let value = map.get(key)?;
    if let [first, ..] = value.as_slice() && *first > 0 {
        return Some(value);
    }
    let closure = |x: u8| -> u8 { x.wrapping_add(1) };
    let _ = (0..10).map(closure).collect::<Vec<_>>();
    None
}

const LIMIT: usize = 1 << 20;
static NAME: &str = r#"raw "string""#;
unsafe extern "C" {
    fn abs(input: i32) -> i32;
}
