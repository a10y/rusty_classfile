use std::io::{BufRead, BufReader, Read};
use std::string::FromUtf8Error;

#[macro_use]
pub(crate) mod macros;

///! `classfile` is a library providing read-only access to a JVM ClassFile structure.
///

pub static MAGIC: [u8; 4] = [0xCA, 0xFE, 0xBA, 0xBE];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClassFileVersion(
    /* Major */ u16,
    /* Minor */ u16);


reversible_enum! {
    ConstantPoolItemTag as u8,
    {
        Utf8 = 1,
        Integer = 3,
        Float = 4,
        Long = 5,
        Double = 6,
        Class = 7,
        String = 8,
        FieldRef = 9,
        MethodRef = 10,
        InterfaceMethodRef = 11,
        NameAndType = 12,
        MethodHandle = 15,
        MethodType = 16,
        InvokeDynamic = 18,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantPoolItem {
    Utf8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Unsupported,
}

impl ConstantPoolItem {
    pub fn is_8byte(&self) -> bool {
        match &self {
            ConstantPoolItem::Long(_) | ConstantPoolItem::Double(_) => true,
            _ => false
        }
    }
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

    #[error("Invalid constant_pool_item tag: {0}")]
    InvalidConstantPoolItemTag(u8),
}

trait ReadExt: Read {
    fn read_u8(&mut self) -> Result<u8, std::io::Error>;
    fn read_u16(&mut self) -> Result<u16, std::io::Error>;

    fn read_i32(&mut self) -> Result<i32, std::io::Error>;
    fn read_i64(&mut self) -> Result<i64, std::io::Error>;
    fn read_f32(&mut self) -> Result<f32, std::io::Error>;
    fn read_f64(&mut self) -> Result<f64, std::io::Error>;
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

    // NOTE: For some reason the JVM stores this as N+1, and uses 1-based indexing for items.
    let constant_pool_count = buf_read.read_u16()? - 1;
    let mut constant_pool_items = Vec::new();
    println!("count = {constant_pool_count}");

    {
        let mut constant_pool_index = 0;
        loop {
            if constant_pool_index >= constant_pool_count {
                break;
            }

            let item = read_constant_pool_item(&mut buf_read)?;
            // JVM oddity: 64-bit types occupy 2 slots in the constant pool.
            if item.is_8byte() {
                constant_pool_index += 2
            } else {
                constant_pool_index += 1
            }

            constant_pool_items.push(item);
        }
    }

    let access_flags = buf_read.read_u16()?;
    let this_class = buf_read.read_u16()?;
    let super_class = buf_read.read_u16()?;
    let interfaces_count = buf_read.read_u16()?;
    // Read a bunch of interfaces.

    Ok(ClassFile {
        version: ClassFileVersion(major, minor),
        constant_pool: constant_pool_items,
    })
}

pub fn read_constant_pool_item<R>(mut buf_read: R) -> Result<ConstantPoolItem, Error>
    where R: BufRead,
{
    let type_tag = buf_read.read_u8()?;
    let type_tag = ConstantPoolItemTag::try_from(type_tag)?;
    match type_tag {
        ConstantPoolItemTag::Utf8 => {
            let strlen = buf_read.read_u16()?;
            let mut utf8_bytes = vec![0; strlen as usize];
            buf_read.read_exact(&mut utf8_bytes)?;

            Ok(ConstantPoolItem::Utf8(String::from_utf8(utf8_bytes)?))
        }
        ConstantPoolItemTag::Integer => {
            Ok(ConstantPoolItem::Integer(buf_read.read_i32()?))
        }
        ConstantPoolItemTag::Float => {
            Ok(ConstantPoolItem::Float(buf_read.read_f32()?))
        }
        ConstantPoolItemTag::Long => {
            Ok(ConstantPoolItem::Long(buf_read.read_i64()?))
        }
        ConstantPoolItemTag::Double => {
            Ok(ConstantPoolItem::Double(buf_read.read_f64()?))
        }
        ConstantPoolItemTag::Class => {
            // TODO(aduffy): handle CONSTANT_Class_info
            let _index = buf_read.read_u16()?;

            Ok(ConstantPoolItem::Unsupported)
        }
        ConstantPoolItemTag::String => {
            // TODO(aduffy): handle CONSTANT_String_info
            let _string_index = buf_read.read_u16()?;
            Ok(ConstantPoolItem::Unsupported)
        }
        ConstantPoolItemTag::FieldRef => {
            // TODO(aduffy): handle CONSTANT_Fieldref_info
            let _class_index = buf_read.read_u16()?;
            let _name_and_type_index = buf_read.read_u16()?;
            Ok(ConstantPoolItem::Unsupported)
        }
        ConstantPoolItemTag::MethodRef => {
            // TODO(aduffy): handle CONSTANT_Methodref_info
            let _class_index = buf_read.read_u16()?;
            let _name_and_type_index = buf_read.read_u16()?;
            Ok(ConstantPoolItem::Unsupported)
        }
        ConstantPoolItemTag::InterfaceMethodRef => {
            // TODO(aduffy): handle CONSTANT_InterfaceMethodref_info
            let _class_index = buf_read.read_u16()?;
            let _name_and_type_index = buf_read.read_u16()?;
            Ok(ConstantPoolItem::Unsupported)
        }
        ConstantPoolItemTag::NameAndType => {
            // TODO(aduffy): handle CONSTANT_NameAndType_info
            let _name_index = buf_read.read_u16()?;
            let _descriptor_index = buf_read.read_u16()?;
            Ok(ConstantPoolItem::Unsupported)
        }
        ConstantPoolItemTag::MethodHandle => {
            // TODO(aduffy): handle CONSTANT_MethodHandle_info
            let _reference_kind = buf_read.read_u8()?;
            let _reference_index = buf_read.read_u16()?;
            Ok(ConstantPoolItem::Unsupported)
        }
        ConstantPoolItemTag::MethodType => {
            // TODO(aduffy): handle CONSTANT_MethodType_info
            let _descriptor_index = buf_read.read_u16()?;
            Ok(ConstantPoolItem::Unsupported)
        }
        ConstantPoolItemTag::InvokeDynamic => {
            // TODO(aduffy): handle CONSTANT_InvokeDynamic_info
            let _bootstrap_method_attr_index = buf_read.read_u16()?;
            let _name_and_type_index = buf_read.read_u16()?;
            Ok(ConstantPoolItem::Unsupported)
        }
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
        let bytes_reader = Bytes::from_static(&[0xCA, 0xFE, 0xBA, 0xBE, 0u8, 10u8, 0u8, 10u8, 0u8, 0u8]);
        let result = read_from(bytes_reader.reader());
        assert_eq!(result.unwrap(), ClassFile {
            version: ClassFileVersion(10, 10),
            constant_pool: Vec::new(),
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
                constant_pool: Vec::new(),
            });
        });

        let client = std::thread::spawn(move || {
            let mut socket = TcpStream::connect(addr.clone()).unwrap();
            socket.write_all(&[0xCA, 0xFE, 0xBA, 0xBE, 0u8, 10u8, 0u8, 10u8, 0u8, 0u8]).unwrap();
        });

        client.join().unwrap();

        // Will rethrow any error thrown from the assert above
        server.join().unwrap();
    }
}