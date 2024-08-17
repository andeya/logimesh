#![allow(deprecated)]

use std::fmt::Formatter;

#[lrcall::service(derive_serde = false)]
trait Foo {
    async fn foo();
}

fn foo(f: &mut Formatter) {
    let x = FooRequest::Foo {};
    lrcall::serde::Serialize::serialize(&x, f);
}

fn main() {}
