// Copyright 2019 Petar Dambovaliev. All rights reserved.
// This software may be modified and distributed under the terms
// of the BSD license. See the LICENSE file for details.

//! TCP transport implementation

extern crate byteorder;

use super::error::{self, Error};
use super::transport::{self, Transport};
use crate::transport::Connection;
use crate::CollectParam;
use byteorder::{BigEndian, ByteOrder};
use std::io::{Read, Write};
use std::net::IpAddr;
use std::net::TcpStream;
use std::sync::Mutex;
use std::time::Duration;

/// Default TCP timeout
pub const TIMEOUT: Duration = Duration::from_secs(10);
/// Default TCP idle timeout
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(60);
pub const MAX_LENGTH: usize = 2084;
//messages
const PDU_SIZE_REQUESTED: u16 = 480;
// const ISO_TCP: u16 = 102; //default isotcp port
const ISO_HEADER_SIZE: u16 = 7; // TPKT+COTP Header Size
const MIN_PDU_SIZE: u16 = 16;

pub struct TcpTransport {
    options: Options,
    stream: Mutex<TcpStream>,
}

/// a set of options for the TCP connection
#[derive(Debug, Clone)]
pub struct Options {
    pub read_timeout: Duration,
    pub write_timeout: Duration,
    address: IpAddr,
    port: u16,
    pub conn_type: transport::Connection,
    //Transport Service Access Point
    local_tsap_high: u8,
    local_tsap_low: u8,
    remote_tsap_high: u8,
    remote_tsap_low: u8,
    last_pdu_type: u8,
    //PDULength variable to store pdu length after connect
    pdu_length: u16,
}

impl Options {
    // pub fn new(address: IpAddr, port: u16, rack: u16, slot: u16, conn_type: Connection) -> Options {
    //     let remote_tsap = ((conn_type as u16) << 8) + (rack * 0x20) + slot;
    //     Options {
    //         read_timeout: Duration::new(0, 0),
    //         write_timeout: Duration::new(0, 0),
    //         port,
    //         address, //ip:102,
    //         conn_type,
    //         local_tsap_high: 0x01,
    //         local_tsap_low: 0x00,
    //         remote_tsap_high: (remote_tsap >> 8) as u8,
    //         remote_tsap_low: remote_tsap as u8,
    //         last_pdu_type: 0,
    //         pdu_length: 256,
    //     }
    // }
    pub fn init_from_config(config: &CollectParam) -> Options {
        let remote_tsap = config.collect_mode.remote_tsap();
        let local_tsap = config.collect_mode.local_tsap();
        Options {
            read_timeout: config.timeout,
            write_timeout: config.timeout,
            port: config.port,
            address: config.address.into(), //ip:102,
            conn_type: config.collect_mode.conn_type().clone(),
            local_tsap_high: local_tsap[0],
            local_tsap_low: local_tsap[1],
            remote_tsap_high: remote_tsap[0],
            remote_tsap_low: remote_tsap[1],
            last_pdu_type: 0,
            pdu_length: 256,
        }
    }
}

impl TcpTransport {
    pub fn connect(options: Options) -> Result<TcpTransport, Error> {
        let tcp_client = TcpStream::connect((options.address, options.port))?;

        tcp_client.set_read_timeout(Some(options.read_timeout))?;
        tcp_client.set_write_timeout(Some(options.write_timeout))?;
        Ok(TcpTransport {
            options,
            stream: Mutex::new(tcp_client),
        })
    }

    fn iso_connect(&mut self) -> Result<(), Error> {
        let mut msg = transport::ISO_CONNECTION_REQUEST_TELEGRAM.to_vec();

        msg[16] = self.options.local_tsap_high;
        msg[17] = self.options.local_tsap_low;
        msg[20] = self.options.remote_tsap_high;
        msg[21] = self.options.remote_tsap_low;

        let r = self.send(msg.as_slice());

        let n = match r {
            Ok(n) => n.len(),
            Err(e) => return Err(Error::Connect(e.to_string())),
        };

        // Sends the connection request telegram
        if n != msg.len() {
            return Err(Error::PduLength(n as u16));
        }

        if self.options.last_pdu_type != transport::CONFIRM_CONNECTION {
            return Err(Error::Iso);
        }
        Ok(())
    }

    fn negotiate_pdu_length(&mut self) -> Result<(), Error> {
        // Set PDU Size Requested //lth
        let mut pdu_size_package = transport::PDU_NEGOTIATION_TELEGRAM.to_vec();
        BigEndian::write_u16(pdu_size_package[23..].as_mut(), PDU_SIZE_REQUESTED as u16);

        // Sends the connection request telegram
        let response = self.send(pdu_size_package.as_slice())?;
        if response.len() == 27 && response[17] == 0 && response[18] == 0 {
            // 20 = size of Negotiate Answer
            // Get PDU Size Negotiated
            self.options.pdu_length = BigEndian::read_u16(&response[25..]);
            if self.options.pdu_length <= 0 {
                return Err(Error::Response {
                    code: error::CLI_NEGOTIATING_PDU,
                });
            }
        } else {
            return Err(Error::Response {
                code: error::CLI_NEGOTIATING_PDU,
            });
        }
        Ok(())
    }
}

impl Transport for TcpTransport {
    fn send(&mut self, request: &[u8]) -> Result<Vec<u8>, Error> {
        // Send sends data to server and ensures response length is greater than header length.
        let mut stream = match self.stream.lock() {
            Ok(s) => s,
            Err(_) => return Err(Error::Lock),
        };
        stream.write(request)?;

        let mut data = vec![0u8; MAX_LENGTH];
        let mut length;

        loop {
            // Get TPKT (4 bytes)
            stream.read(&mut data[..4])?;

            // Read length, ignore transaction & protocol id (4 bytes)
            length = BigEndian::read_u16(&data[2..]);
            let length_n = length;

            if length_n == ISO_HEADER_SIZE {
                stream.read(&mut data[4..7])?;
            } else {
                if length_n > PDU_SIZE_REQUESTED + ISO_HEADER_SIZE || length_n < MIN_PDU_SIZE {
                    return Err(Error::PduLength(length_n));
                }
                break;
            }
        }

        // Skip remaining 3 COTP bytes
        stream.read(&mut data[4..7])?;
        self.options.last_pdu_type = data[5]; // Stores PDU Type, we need it for later

        // Receives the S7 Payload
        stream.read(&mut data[7..length as usize])?;
        Ok(data[0..length as usize].to_vec())
    }

    fn pdu_length(&self) -> u16 {
        self.options.pdu_length
    }

    fn negotiate(&mut self) -> Result<(), Error> {
        self.iso_connect()?;
        self.negotiate_pdu_length()
    }

    fn connection_type(&self) -> Connection {
        self.options.conn_type
    }
}
