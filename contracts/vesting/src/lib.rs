#![allow(rustdoc::private_intra_doc_links)]
//! Nym vesting contract, providing vesting accounts with ability to stake unvested tokens

#![warn(clippy::expect_used)]
#![warn(clippy::unwrap_used)]

pub mod contract;
mod errors;
mod queued_migrations;
mod storage;
mod support;
mod traits;
pub mod vesting;
