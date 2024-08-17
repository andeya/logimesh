#![allow(deprecated)]
#[lrcall::service(derive = [Clone], derive_serde = true)]
trait Foo {
    async fn foo();
}

fn main() {}
