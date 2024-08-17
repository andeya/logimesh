#![no_implicit_prelude]
extern crate lrcall as some_random_other_name;

#[cfg(feature = "serde1")]
mod serde1_feature {
    #[::lrcall::derive_serde]
    #[derive(Debug, PartialEq, Eq)]
    pub enum TestData {
        Black,
        White,
    }
}

#[::lrcall::service]
pub trait ColorProtocol {
    async fn get_opposite_color(color: u8) -> u8;
}
