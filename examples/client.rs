use log::debug;
use pretty_hex::simple_hex;
use s7::field::Bool;
use s7::{client, tcp, transport, BitAddr, DataSizeByte};
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

fn main() {
    custom_utils::logger::logger_stdout_debug();
    let addr = Ipv4Addr::new(192, 168, 1, 222);
    let mut opts = tcp::Options::new(IpAddr::from(addr), 102, 0, 1, transport::Connection::OP);

    opts.read_timeout = Duration::from_secs(2);
    opts.write_timeout = Duration::from_secs(2);

    let t = tcp::TcpTransport::connect(opts).unwrap();

    let mut cl = client::Client::new(t).unwrap();

    // {
    //     // 读DQ0数据
    debug!(
        "{}",
        simple_hex(
            &cl.dq_read(DataSizeByte::Bit {
                addr: 0,
                bit_addr: BitAddr::Addr2
            })
            .unwrap()
        )
    );
    debug!(
        "{}",
        simple_hex(&cl.dq_read(DataSizeByte::Byte { addr: 0, len: 1 }).unwrap())
    );
    // }
    // {
    //     // 读DI数据
    debug!(
        "{}",
        simple_hex(&cl.di_read(DataSizeByte::Byte { addr: 0, len: 1 }).unwrap())
    );
    // }
    {
        // 读DB数据
        debug!(
            "{}",
            simple_hex(
                &cl.db_read(1, DataSizeByte::Word { addr: 300, len: 1 })
                    .unwrap()
            )
        );
    }

    // 写V300的long值
    // let val = 160u32;
    // cl.ag_write(1, 300, 4, &mut val.to_be_bytes().as_slice().to_vec())
    //     .unwrap();
    // 读V300的long值
    // let buffer = &mut vec![0u8; 1];
    // cl.ag_read(1, 300, 1, buffer).unwrap();
    // debug!("{:?}", buffer);
    {
        // let buffer = &mut vec![0u8; Bool::size() as usize];
        // let db = 888;
        // let offset = 8.4;
        // cl.ag_read(db, offset as i32, Bool::size(), buffer).unwrap();
        // let mut lights = Bool::new(db, offset, buffer.to_vec()).unwrap();
    }
}
