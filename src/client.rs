// Copyright 2019 Petar Dambovaliev. All rights reserved.
// This software may be modified and distributed under the terms
// of the BSD license. See the LICENSE file for details.

use super::constant::{self, Area};
use super::error::{self, Error};
use super::tcp;
use byteorder::{BigEndian, ByteOrder};

pub trait Transport {
    fn send(&mut self, request: &[u8]) -> Result<Vec<u8>, Error>;
    fn pdu_length(&self) -> i32;
}

#[derive(Debug, Clone)]
pub struct Client<T: Transport> {
    transport: T,
}

impl Client<tcp::Transport> {
    pub fn new_tcp(options: tcp::Options) -> Result<Client<tcp::Transport>, Error> {
        let transport = tcp::Transport::connect(options)?;
        Ok(Client { transport })
    }
}

impl<T: Transport> Client<T> {
    pub fn new(transport: T) -> Client<T> {
        Client { transport }
    }

    /// # Examples
    ///
    /// ```
    ///extern crate s7;
    ///use s7::{client::Client, tcp};
    ///use std::time::Duration;
    ///
    /// let mut opts = tcp::Options::new("127.0.0.1:9999".to_string(), tcp::Connection::PG, 5, 5);
    /// opts.read_timeout = Duration::from_secs(2);
    /// opts.write_timeout = Duration::from_secs(2);
    /// let mut cl = Client::new_tcp(opts).unwrap();
    ///
    /// let buffer = &mut vec![0u8; 25];
    ///
    /// match cl.db_read(1, 1, 3, buffer) {
    ///       Ok(()) => println!("buffer: {:?}", buffer),
    ///       Err(e) => println!("error: {:?}", e)
    /// }
    /// ```
    pub fn db_read(
        &mut self,
        db_number: i32,
        start: i32,
        size: i32,
        buffer: &mut Vec<u8>,
    ) -> Result<(), Error> {
        return self.read(
            Area::DataBausteine,
            db_number,
            start,
            size,
            constant::WL_BYTE,
            buffer,
        );
    }

    pub fn db_write(
        &mut self,
        db_number: i32,
        start: i32,
        size: i32,
        buffer: &mut Vec<u8>,
    ) -> Result<(), Error> {
        return self.write(
            Area::DataBausteine,
            db_number,
            start,
            size,
            constant::WL_BYTE,
            buffer,
        );
    }

    pub fn mb_read(&mut self, start: i32, size: i32, buffer: &mut Vec<u8>) -> Result<(), Error> {
        return self.read(Area::Merker, 0, start, size, constant::WL_BYTE, buffer);
    }

    pub fn mb_write(&mut self, start: i32, size: i32, buffer: &mut Vec<u8>) -> Result<(), Error> {
        return self.write(Area::Merker, 0, start, size, constant::WL_BYTE, buffer);
    }

    pub fn eb_read(&mut self, start: i32, size: i32, buffer: &mut Vec<u8>) -> Result<(), Error> {
        return self.read(
            Area::ProcessInput,
            0,
            start,
            size,
            constant::WL_BYTE,
            buffer,
        );
    }

    pub fn eb_write(&mut self, start: i32, size: i32, buffer: &mut Vec<u8>) -> Result<(), Error> {
        return self.write(
            Area::ProcessInput,
            0,
            start,
            size,
            constant::WL_BYTE,
            buffer,
        );
    }

    pub fn ab_read(&mut self, start: i32, size: i32, buffer: &mut Vec<u8>) -> Result<(), Error> {
        return self.read(
            Area::ProcessOutput,
            0,
            start,
            size,
            constant::WL_BYTE,
            buffer,
        );
    }

    pub fn ab_write(&mut self, start: i32, size: i32, buffer: &mut Vec<u8>) -> Result<(), Error> {
        return self.write(
            Area::ProcessOutput,
            0,
            start,
            size,
            constant::WL_BYTE,
            buffer,
        );
    }

    //read generic area, pass result into a buffer
    fn read(
        &mut self,
        area: Area,
        db_number: i32,
        mut start: i32,
        mut amount: i32,
        mut word_len: i32,
        buffer: &mut Vec<u8>,
    ) -> Result<(), Error> {
        // Some adjustment
        match area {
            Area::Counter => word_len = constant::WL_COUNTER,
            Area::Timer => word_len = constant::WL_TIMER,
            _ => {}
        };

        // Calc Word size
        let mut word_size = constant::data_size_byte(word_len);

        if word_size == 0 {
            return Err(Error::Response {
                code: error::ISO_INVALID_DATA_SIZE,
            });
        }

        if word_len == constant::WL_BIT {
            amount = 1; // Only 1 bit can be transferred at time
        } else {
            if word_len != constant::WL_COUNTER && word_len != constant::WL_TIMER {
                amount = amount * word_size;
                word_size = 1;
                word_len = constant::WL_BYTE;
            }
        }

        let pdu_length = self.transport.pdu_length();

        if pdu_length == 0 {
            return Err(Error::PduLength(pdu_length));
        }

        let max_elements = (pdu_length - 18) / word_size; // 18 = Reply telegram header //lth note here

        let mut tot_elements = amount;
        let db_bytes = (db_number as u16).to_be_bytes();
        let mut offset = 0;

        loop {
            if tot_elements < 0 {
                return Ok(());
            }

            let mut num_elements = tot_elements;

            if num_elements > max_elements {
                num_elements = max_elements;
            }

            let size_requested = num_elements * word_size;
            // Setup the telegram
            let mut request =
                constant::READ_WRITE_TELEGRAM[..constant::SIZE_HEADER_READ as usize].to_vec();

            // Set DB Number
            request[25] = db_bytes[0];
            request[26] = db_bytes[1];

            // Set Area
            match area {
                Area::DataBausteine => request[27] = area as u8,
                _ => {}
            }

            // Adjusts Start and word length
            let mut address = match word_len {
                constant::WL_BIT | constant::WL_COUNTER | constant::WL_TIMER => {
                    request[22] = word_len as u8;
                    start
                }
                _ => start << 3,
            };

            // Num elements
            let num_elements_bytes = (num_elements as u16).to_be_bytes();
            request[23] = num_elements_bytes[0];
            request[24] = num_elements_bytes[1];

            // Address into the PLC (only 3 bytes)
            request[30] = (address & 0x0FF) as u8;
            address = address >> 8;
            request[29] = (address & 0x0FF) as u8;
            address = address >> 8;
            request[28] = (address & 0x0FF) as u8;

            let result = self.transport.send(request.as_slice());

            match result {
                Ok(response) => {
                    if response.len() < 25 {
                        return Err(Error::Response {
                            code: error::ISO_INVALID_DATA_SIZE,
                        });
                    }

                    if response[21] != 0xFF {
                        return Err(Error::CPU {
                            code: response[21] as i32,
                        });
                    }
                    let (mut i, end): (usize, usize) = (25, 25 + (size_requested as usize));

                    //copy response to buffer
                    for k in offset..size_requested {
                        if i == end {
                            break;
                        }
                        buffer[k as usize] = response[i];
                        i += 1;
                    }
                    offset += size_requested;
                }
                Err(e) => {
                    return Err(e);
                }
            }

            tot_elements -= num_elements;
            start += num_elements * word_size
        }
    }

    fn write(
        &mut self,
        area: Area,
        db_number: i32,
        mut start: i32,
        mut amount: i32,
        mut word_len: i32,
        buffer: &mut Vec<u8>,
    ) -> Result<(), Error> {
        // Some adjustment
        word_len = match area {
            Area::Counter => constant::WL_COUNTER,
            Area::Timer => constant::WL_TIMER,
            _ => word_len,
        };

        // Calc Word size
        let mut word_size = constant::data_size_byte(word_len);

        if word_size == 0 {
            return Err(Error::Response {
                code: error::ISO_INVALID_DATA_SIZE,
            });
        }

        if word_len == constant::WL_BIT {
            amount = 1; // Only 1 bit can be transferred at time
        } else {
            if word_len != constant::WL_COUNTER && word_len != constant::WL_TIMER {
                amount = amount * word_size;
                word_size = 1;
                word_len = constant::WL_BYTE;
            }
        }

        let (mut address, mut tot_elements, mut offset): (i32, i32, i32) = (0, 0, 0);
        let max_elements = (self.transport.pdu_length() - 35) / word_size; // 35 = Reply telegram header
        tot_elements = amount;

        while tot_elements > 0 {
            let mut num_elements = tot_elements;
            if num_elements > max_elements {
                num_elements = max_elements;
            }
            let data_size = num_elements * word_size;
            let iso_size = constant::SIZE_HEADER_WRITE + data_size;

            // Setup the telegram
            let mut request_data = constant::READ_WRITE_TELEGRAM.to_vec();
            // Whole telegram Size
            BigEndian::write_u16(request_data[2..].as_mut(), iso_size as u16);
            // Data length
            let mut length = data_size + 4;
            BigEndian::write_u16(request_data[15..].as_mut(), length as u16);
            // Function
            request_data[17] = 0x05;
            // Set DB Number
            request_data[27] = area as u8;

            match area {
                Area::DataBausteine => {
                    BigEndian::write_u16(request_data[25..].as_mut(), db_number as u16)
                }
                _ => {}
            }
            // Adjusts start and word length
            match word_len {
                constant::WL_BIT | constant::WL_COUNTER | constant::WL_TIMER => {
                    address = start;
                    length = data_size;
                    request_data[22] = word_len as u8;
                }
                _ => {
                    address = start << 3;
                    length = data_size << 3;
                }
            }

            // Num elements
            BigEndian::write_u16(request_data[23..].as_mut(), num_elements as u16);
            // address into the PLC
            request_data[30] = (address & 0x0FF) as u8;
            address = address >> 8;
            request_data[29] = (address & 0x0FF) as u8;
            address = address >> 8;
            request_data[28] = (address & 0x0FF) as u8;

            // Transport Size
            match word_len {
                constant::WL_BIT => request_data[32] = constant::TS_RES_BIT as u8,
                constant::WL_COUNTER | constant::WL_TIMER => {
                    request_data[32] = constant::TS_RES_OCTET as u8
                }
                _ => request_data[32] = constant::TS_RES_BYTE as u8, // byte/word/dword etc.
            }
            // length
            BigEndian::write_u16(request_data[33..].as_mut(), length as u16);

            //expand values into array
            request_data.splice(
                35..35,
                buffer[offset as usize..offset as usize + data_size as usize].to_vec(),
            );

            self.transport.send(request_data.as_mut_slice())?;

            offset += data_size;
            tot_elements -= num_elements;
            start += num_elements * word_size;
        }
        Ok(())
    }
}

#[test]
fn client_new() {
    use super::tcp;
    let opts = tcp::Options::new("127.0.0.1".to_string(), tcp::Connection::PG, 5, 5);
    let mut cl = Client::new_tcp(opts).unwrap();

    let buffer = &mut vec![0u8; 25];

    match cl.db_read(1, 1, 3, buffer) {
        Ok(()) => assert!(true),
        Err(e) => assert!(false, "{:?}", e),
    }
}