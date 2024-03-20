use std::io::{BufReader, Read};

///! `classfile` is a library providing read-only access to a JVM ClassFile structure.
///

pub static MAGIC: [u8; 4] = [0xCA, 0xFE, 0xBA, 0xBE];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClassFileVersion(
    /* Major */ u16,
    /* Minor */ u16);

#[derive(Debug, Clone, PartialEq)]
pub struct ConstantPool;

#[derive(Debug, Clone, PartialEq)]
pub struct ClassFile {
    pub version: ClassFileVersion,
    pub constant_pool: ConstantPool,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("i/o error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid magic in file header: {0:?}")]
    InvalidMagic([u8; 4]),
}

trait ReadExt: Read {
    // fn read_u8(&mut self) -> Result<u8, std::io::Error>;
    fn read_u16(&mut self) -> Result<u16, std::io::Error>;
    // fn read_u32(&mut self) -> Result<u32, std::io::Error>;
}

macro_rules! read_bytes {
    ($self:expr, $ty:ty, $N:expr) => {{
        let mut buf = [0u8; $N];
        $self.read_exact(&mut buf)?;

        Ok(<$ty>::from_be_bytes(buf))
    }};
}

impl<R> ReadExt for R where R: Read {
    // fn read_u8(&mut self) -> Result<u8, std::io::Error> {
    //     read_bytes!(self, u8, 1)
    // }

    fn read_u16(&mut self) -> Result<u16, std::io::Error> {
        read_bytes!(self, u16, 2)
    }

    // fn read_u32(&mut self) -> Result<u32, std::io::Error> {
    //     read_bytes!(self, u32, 4)
    // }
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

    Ok(ClassFile {
        version: ClassFileVersion(major, minor),
        constant_pool: ConstantPool,
    })
}

#[cfg(test)]
mod test {
    use std::io::Write;
    use std::net::{SocketAddr, TcpListener, TcpStream};

    use bytes::{Buf, Bytes};

    use crate::{ClassFile, ClassFileVersion, ConstantPool, Error, read_from};

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