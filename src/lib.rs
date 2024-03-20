use std::io::{BufRead, BufReader, Read};
use std::string::FromUtf8Error;

///! `classfile` is a library providing read-only access to a JVM ClassFile structure.
///

pub static MAGIC: [u8; 4] = [0xCA, 0xFE, 0xBA, 0xBE];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClassFileVersion(
    /* Major */ u16,
    /* Minor */ u16);


pub const CP_TAG_UTF8: u8 = 1;
pub const CP_TAG_INTEGER: u8 = 3;
pub const CP_TAG_FLOAT: u8 = 4;
pub const CP_TAG_LONG: u8 = 5;
pub const CP_TAG_DOUBLE: u8 = 6;

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantPoolItem {
    Utf8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassFile {
    pub version: ClassFileVersion,
    pub constant_pool: Vec<ConstantPoolItem>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("i/o error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("utf8 decode error: {0}")]
    Utf8DecodeError(#[from] FromUtf8Error),

    #[error("Invalid magic in file header: {0:?}")]
    InvalidMagic([u8; 4]),
}

trait ReadExt: Read {
    fn read_u8(&mut self) -> Result<u8, std::io::Error>;
    fn read_u16(&mut self) -> Result<u16, std::io::Error>;

    fn read_i32(&mut self) -> Result<i32, std::io::Error>;
    fn read_i64(&mut self) -> Result<i64, std::io::Error>;
    fn read_f32(&mut self) -> Result<f32, std::io::Error>;
    fn read_f64(&mut self) -> Result<f64, std::io::Error>;
}

macro_rules! read_bytes {
    ($self:expr, $ty:ty, $N:expr) => {{
        let mut buf = [0u8; $N];
        $self.read_exact(&mut buf)?;

        Ok(<$ty>::from_be_bytes(buf))
    }};
}

impl<R> ReadExt for R where R: Read {
    fn read_u8(&mut self) -> Result<u8, std::io::Error> {
        read_bytes!(self, u8, 1)
    }

    fn read_u16(&mut self) -> Result<u16, std::io::Error> {
        read_bytes!(self, u16, 2)
    }

    fn read_i32(&mut self) -> Result<i32, std::io::Error> {
        read_bytes!(self, i32, 4)
    }

    fn read_i64(&mut self) -> Result<i64, std::io::Error> {
        read_bytes!(self, i64, 8)
    }

    fn read_f32(&mut self) -> Result<f32, std::io::Error> {
        read_bytes!(self, f32, 4)
    }

    fn read_f64(&mut self) -> Result<f64, std::io::Error> {
        read_bytes!(self, f64, 8)
    }
}


pub fn read_from<R>(reader: R) -> Result<ClassFile, Error>
    where R: Read {
    let mut buf_read = BufReader::new(reader);

    // Try and read until we're able to retrieve a single read var here.
    let mut buf: [u8; 4] = [0u8; 4];

    buf_read.read_exact(&mut buf)?;

    if MAGIC != buf {
        return Err(Error::InvalidMagic(buf));
    }

    // Read major and minor versions
    let minor = buf_read.read_u16()?;
    let major = buf_read.read_u16()?;

    let constant_pool_count = buf_read.read_u16()?;
    let mut constant_pool_items = Vec::new();

    for _ in 0..constant_pool_count {
        constant_pool_items.push(read_constant_pool_item(&mut buf_read)?);
    }

    Ok(ClassFile {
        version: ClassFileVersion(major, minor),
        constant_pool: constant_pool_items,
    })
}

pub fn read_constant_pool_item<R>(mut buf_read: R) -> Result<ConstantPoolItem, Error>
    where R: BufRead,
{
    let type_tag = buf_read.read_u8()?;
    match type_tag {
        CP_TAG_UTF8 => {
            let strlen = buf_read.read_u16()?;
            let mut utf8_bytes = vec![0; strlen as usize];
            buf_read.read_exact(&mut utf8_bytes)?;

            Ok(ConstantPoolItem::Utf8(String::from_utf8(utf8_bytes)?))
        }
        CP_TAG_INTEGER => {
            Ok(ConstantPoolItem::Integer(buf_read.read_i32()?))
        }
        CP_TAG_FLOAT => {
            Ok(ConstantPoolItem::Float(buf_read.read_f32()?))
        }
        CP_TAG_LONG => {
            Ok(ConstantPoolItem::Long(buf_read.read_i64()?))
        }
        CP_TAG_DOUBLE => {
            Ok(ConstantPoolItem::Double(buf_read.read_f64()?))
        }
        _ => Ok(ConstantPoolItem::Unsupported)
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;
    use std::net::{SocketAddr, TcpListener, TcpStream};

    use bytes::{Buf, Bytes};

    use crate::{ClassFile, ClassFileVersion, Error, read_from};

    #[test]
    fn test_invalid_magic() {
        let bytes_reader = Bytes::from_static(&[0u8, 0u8, 0u8, 0u8]);
        let result = read_from(bytes_reader.reader());
        assert!(matches!(result.unwrap_err(), Error::InvalidMagic([0u8, 0u8, 0u8, 0u8])));
    }

    #[test]
    fn test_valid_magic() {
        let bytes_reader = Bytes::from_static(&[0xCA, 0xFE, 0xBA, 0xBE, 0u8, 10u8, 0u8, 10u8]);
        let result = read_from(bytes_reader.reader());
        assert_eq!(result.unwrap(), ClassFile {
            version: ClassFileVersion(10, 10),
            constant_pool: ConstantPool,
        })
    }

    #[test]
    fn test_network() {
        // Fun thing: any std::io::Read type can be used, so we can even implement a TCP server
        // that can receive ClassFile instances sent over a network.
        // This isn't super-duper practical but it sure is neat!
        let addr: SocketAddr = "127.0.0.1:30245".parse().unwrap();

        let server = std::thread::spawn(move || {
            let socket = TcpListener::bind(addr.clone()).unwrap();
            let (stream, _) = socket.accept().unwrap();

            let class_file = read_from(stream).unwrap();

            assert_eq!(class_file, ClassFile {
                version: ClassFileVersion(10, 10),
                constant_pool: ConstantPool,
            });
        });

        let client = std::thread::spawn(move || {
            let mut socket = TcpStream::connect(addr.clone()).unwrap();
            socket.write_all(&[0xCA, 0xFE, 0xBA, 0xBE, 0u8, 10u8, 0u8, 10u8]).unwrap();
        });

        client.join().unwrap();

        // Will rethrow any error thrown from the assert above
        server.join().unwrap();
    }
}