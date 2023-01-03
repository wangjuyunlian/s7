use crate::error::Error;

// Area ID
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub(crate) enum Area {
    ProcessInput = 0x81,
    ProcessOutput = 0x82,
    /// Merkers are address registers within the CPU.
    /// The number of available flag bytes depends on the respective CPU and can be taken from the technical data.
    /// You can use flag bits, flag bytes, flag words or flag double words in a PLC program.
    Merker = 0x83,
    /// German thing, means building blocks
    /// This is your storage  
    DataBausteine = 0x84,
    Counter = 0x1C,
    Timer = 0x1D,
    Unknown,
}
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum BitAddr {
    Addr0 = 0,
    Addr1 = 1,
    Addr2 = 2,
    Addr3 = 3,
    Addr4 = 4,
    Addr5 = 5,
    Addr6 = 6,
    Addr7 = 7,
}

#[derive(Debug, Copy, Clone)]
pub enum DataSizeByte {
    Bit { addr: u16, bit_addr: BitAddr },
    Byte { addr: u16, len: u16 },
    Char { addr: u16, len: u16 },
    Word { addr: u16, len: u16 },
    Int { addr: u16, len: u16 },
    DWord { addr: u16, len: u16 },
    DInt { addr: u16, len: u16 },
    Real { addr: u16, len: u16 },
    Counter { addr: u16, len: u16 },
    Timer { addr: u16, len: u16 },
}
impl DataSizeByte {
    /// 类型对应的字节长度
    pub fn length(&self) -> u16 {
        use DataSizeByte::*;
        match self {
            Bit { .. } | Byte { .. } | Char { .. } => 1,
            Word { .. } | Int { .. } | Counter { .. } | Timer { .. } => 2,
            DWord { .. } | DInt { .. } | Real { .. } => 4,
        }
    }
    /// 位的偏移位置
    pub fn bit_addr(&self) -> u8 {
        use DataSizeByte::*;
        match self {
            Bit { bit_addr, .. } => *bit_addr as u8,
            _ => 0x00,
        }
    }
    /// 读取的单位长度
    pub fn len(&self) -> u16 {
        use DataSizeByte::*;
        match self {
            Bit { bit_addr, .. } => 1u16,
            Byte { len, .. } => *len,
            Char { len, .. } => *len,
            Word { len, .. } => *len,
            Int { len, .. } => *len,
            DWord { len, .. } => *len,
            DInt { len, .. } => *len,
            Real { len, .. } => *len,
            Counter { len, .. } => *len,
            Timer { len, .. } => *len,
        }
    }
    /// 用于返回后的byte长度 = 读取长度 * 单位字节数
    pub fn byte_len(&self) -> usize {
        (self.len()  * self.length() ) as usize
    }
    pub fn addr(&self) -> [u8; 3] {
        use DataSizeByte::*;
        let byte_addr = match self {
            Bit { addr, .. } => *addr,
            Byte { addr, .. } => *addr,
            Char { addr, .. } => *addr,
            Word { addr, .. } => *addr,
            Int { addr, .. } => *addr,
            DWord { addr, .. } => *addr,
            DInt { addr, .. } => *addr,
            Real { addr, .. } => *addr,
            Counter { addr, .. } => *addr,
            Timer { addr, .. } => *addr,
        };
        let mut address = (byte_addr as u32) << 3 + self.bit_addr();
        [
            (address & 0xFF0000 >> 16) as u8,
            (address & 0xFF00 >> 8) as u8,
            (address & 0xFF) as u8,
        ]
    }
    pub fn data(&self) -> u8 {
        use DataSizeByte::*;
        match self {
            Bit { .. } => 0x01,
            Byte { .. } => 0x02,
            Char { .. } => 0x03,
            Word { .. } => 0x04,
            Int { .. } => 0x05,
            DWord { .. } => 0x06,
            DInt { .. } => 0x07,
            Real { .. } => 0x08,
            Counter { .. } => 0x1C,
            Timer { .. } => 0x1D,
        }
    }
}

// Word Length
// pub const WL_BIT: i32 = 0x01; //Bit (inside a word)
// pub const WL_BYTE: i32 = 0x02; //Byte (8 bit)
// pub const WL_CHAR: i32 = 0x03;
// pub const WL_WORD: i32 = 0x04; //Word (16 bit)
// pub const WL_INT: i32 = 0x05;
// pub const WL_DWORD: i32 = 0x06; //Double Word (32 bit)
// pub const WL_DINT: i32 = 0x07;
// pub const WL_REAL: i32 = 0x08; //Real (32 bit float)
// pub const WL_COUNTER: i32 = 0x1C; //Counter (16 bit)
// pub const WL_TIMER: i32 = 0x1D; //Timer (16 bit)
//
// //dataSize to number of byte accordingly
// pub fn data_size_byte(word_length: i32) -> i32 {
//     match word_length {
//         WL_BIT | WL_BYTE | WL_CHAR => 1,
//         WL_WORD | WL_INT | WL_COUNTER | WL_TIMER => 2,
//         WL_DWORD | WL_DINT | WL_REAL => 4,
//         _ => 0,
//     }
// }

// PLC Status
pub enum CpuStatus {
    Unknown = 0,
    Stop = 4,
    Run = 8,
}

impl CpuStatus {
    pub(crate) fn from_u8(value: u8) -> Result<CpuStatus, Error> {
        match value {
            0 => Ok(CpuStatus::Unknown),
            4 => Ok(CpuStatus::Stop),
            8 => Ok(CpuStatus::Run),
            _ => Err(Error::InvalidCpuStatus(value)),
        }
    }
}

//size header
pub const SIZE_HEADER_READ: i32 = 31; // Header Size when Reading
pub const SIZE_HEADER_WRITE: i32 = 35; // Header Size when Writing

// Result transport size
pub const TS_RES_BIT: i32 = 3;
pub const TS_RES_BYTE: i32 = 4;
#[allow(dead_code)]
pub const TS_RES_INT: i32 = 5;
//todo implement read write multi
#[allow(dead_code)]
pub const TS_RES_REAL: i32 = 7;
pub const TS_RES_OCTET: i32 = 9;
