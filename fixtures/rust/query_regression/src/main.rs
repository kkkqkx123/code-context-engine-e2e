use foo as bar;
use plain;
use a::{b, c as d};
use x::y::z;
use g::*;
use self::shape;
use super::sibling;
use crate::root_mod;
use outer::{inner::{deep, deeper as d2}};
pub use shared::Reexport;
use with_self::{self};
extern crate once_cell;

fn main() {
    let _ = bar::run();
    let _ = plain::run();
    let _ = b::run();
    let _ = d::run();
    let _ = z::run();
}

fn demo() {
    let _ = Type::new();
    let _ = a::b::c();
    obj.method();
    self.helper();
    chained.first().second();
    generic_fn::<i32>();
    obj.generic_method::<i32>();
    println!("v={}", 1);
    let _ = vec![1, 2];
}

fn helper() {}
