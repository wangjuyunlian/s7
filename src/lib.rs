#![allow(dead_code)]
// Copyright 2019 Petar Dambovaliev. All rights reserved.
// This software may be modified and distributed under the terms
// of the BSD license. See the LICENSE file for details.

pub mod client;
mod constant;
pub mod error;
pub mod field;
pub mod tcp;
pub mod transport;

pub use constant::{Area, BitAddr, DataSizeType};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub address: IpAddr,
    pub port: u16,
    pub rack: u16,
    pub slot: u16,
    pub timeout: Duration,
    pub areas: Vec<Area>,
}
