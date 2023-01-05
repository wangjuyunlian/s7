#![allow(dead_code)]
// Copyright 2019 Petar Dambovaliev. All rights reserved.
// This software may be modified and distributed under the terms
// of the BSD license. See the LICENSE file for details.

mod client;
mod constant;
pub mod error;
pub mod field;
pub mod tcp;
pub mod transport;

use crate::transport::Connection;
pub use client::Client;
pub use constant::{Area, BitAddr, DataSizeType};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectParam {
    pub address: Ipv4Addr,
    pub port: u16,
    pub collect_mode: CollectMode,
    pub timeout: Duration,
    pub areas: Vec<Area>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollectMode {
    Tsap {
        conn_type: Connection,
        local_tsap: u16,
        remote_tsap: u16,
    },
    RackSlot {
        conn_type: Connection,
        rack: u16,
        slot: u16,
    },
}
impl CollectMode {
    pub fn init_tsap(conn_type: Connection, local_tsap: u16, remote_tsap: u16) -> Self {
        Self::Tsap {
            conn_type,
            local_tsap,
            remote_tsap,
        }
    }
    pub fn init_rack_slot(conn_type: Connection, rack: u16, slot: u16) -> Self {
        Self::RackSlot {
            conn_type,
            rack,
            slot,
        }
    }
    pub fn conn_type(&self) -> &Connection {
        match self {
            CollectMode::Tsap { conn_type, .. } => conn_type,
            CollectMode::RackSlot { conn_type, .. } => conn_type,
        }
    }
    pub fn local_tsap(&self) -> [u8; 2] {
        match self {
            CollectMode::Tsap { local_tsap, .. } => [(local_tsap >> 8) as u8, *local_tsap as u8],
            CollectMode::RackSlot { .. } => [0x01, 0x00],
        }
    }
    pub fn remote_tsap(&self) -> [u8; 2] {
        let remote_tsap = match self {
            CollectMode::Tsap { remote_tsap, .. } => *remote_tsap,
            CollectMode::RackSlot {
                rack,
                slot,
                conn_type,
            } => ((*conn_type as u16) << 8) + (rack * 0x20) + slot,
        };
        [(remote_tsap >> 8) as u8, remote_tsap as u8]
    }
}
