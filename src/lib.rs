//! Read & Write varint to/from bytes
//!
//! # Writing bytes
//!
//! ```rust
//! use vartyint;
//! let mut my_bytes = Vec::new();
//! vartyint::write_i32(1000, &mut my_bytes);
//! assert_eq!(my_bytes, &[0xd0, 0x0f]);
//! ```
//!
//! # Reading
//!
//! Read an integer from a slice of bytes (`&[u8]`). Upon success, the number, as well as the rest
//! of the bytes is returned. You can "pop off" numbers like this.
//!
//! ```rust
//! use vartyint;
//! let my_bytes = vec![0x18, 0x01, 0xBF, 0xBB, 0x01];
//!
//! let (num1, my_bytes) = vartyint::read_i32(&my_bytes).unwrap();
//! assert_eq!(num1, 12);
//! assert_eq!(my_bytes, &[0x01, 0xBF, 0xBB, 0x01]);
//!
//! let (num2, my_bytes) = vartyint::read_i32(&my_bytes).unwrap();
//! assert_eq!(num2, -1);
//! assert_eq!(my_bytes, &[0xBF, 0xBB, 0x01]);
//!
//! let (num3, my_bytes) = vartyint::read_i32(&my_bytes).unwrap();
//! assert_eq!(num3, -12_000);
//! assert_eq!(my_bytes, &[]);
//!
//! // Can't read any more
//! assert_eq!(vartyint::read_i32(&my_bytes), Err(vartyint::VartyIntError::EmptyBuffer));
//! ```
//!
//!

/// Error type
#[derive(Debug,PartialEq,Eq)]
pub enum VartyIntError {

    /// Attempted to read from an empty buffer. No bytes, so cannot return anything
    EmptyBuffer,

    /// There are not enough bytes for the integer.
    NotEnoughBytes,

    /// Attempted to read an integer that is too small for the data
    TooManyBytesForType,
}

impl std::fmt::Display for VartyIntError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{:?}", self)
    }
}

impl std::error::Error for VartyIntError {}

macro_rules! write_unsigned {
    ( $name:ident, $type:ty ) => {

/// Write an integer to this buffer
pub fn $name(mut val: $type, buf: &mut Vec<u8>) {
    if val == 0 {
        buf.push(0);
        return;
    }

    while val != 0 {
        let mut num = (val & 0b0111_1111) as u8;
        val >>= 7;
        if val != 0 {
            num |= 0b1000_0000;
        }
        buf.push(num);
    }

}
}}

write_unsigned!(write_u8, u8);
write_unsigned!(write_u16, u16);
write_unsigned!(write_u32, u32);
write_unsigned!(write_u64, u64);
write_unsigned!(write_usize, usize);
write_unsigned!(write_u128, u128);

macro_rules! read_unsigned {
    ( $name:ident, $type:ty ) => {

/// Read an integer from this buffer
pub fn $name(mut buf: &[u8]) -> Result<($type, &[u8]), VartyIntError> {
    if buf.is_empty() {
        return Err(VartyIntError::EmptyBuffer);
    }
    let mut num_bits_read = 0;
    let mut val: $type = 0;
    let mut is_last: bool;
    let mut byte: $type;

    loop {
        if buf.is_empty() {
            return Err(VartyIntError::NotEnoughBytes)
        }
        byte = buf[0] as $type;
        buf = &buf[1..];

        is_last = byte >> 7 == 0;
        byte &= 0b0111_1111;

        byte = match byte.checked_shl(num_bits_read) {
            None => {
                return Err(VartyIntError::TooManyBytesForType);
            },
            Some(v) => v,
        };
        val = val | byte;
        num_bits_read += 7;
        if is_last {
            // last byte
            break
        }

    }

    Ok((val, buf))
}

}}

read_unsigned!(read_u8, u8);
read_unsigned!(read_u16, u16);
read_unsigned!(read_u32, u32);
read_unsigned!(read_u64, u64);
read_unsigned!(read_u128, u128);
read_unsigned!(read_usize, usize);

macro_rules! read_signed {
    ( $name:ident, $type:ty, $bits:expr ) => {

/// Read an integer from this buffer
pub fn $name(mut buf: &[u8]) -> Result<($type, &[u8]), VartyIntError> {
    if buf.is_empty() {
        return Err(VartyIntError::EmptyBuffer);
    }
    let mut num_bits_read = 0;
    let mut val: $type = 0;
    let mut is_last: bool;

    let mut byte: $type;

    loop {
        if buf.is_empty() {
            return Err(VartyIntError::NotEnoughBytes)
        }
        byte = buf[0] as $type;
        buf = &buf[1..];

        is_last = byte >> 7 == 0;
        byte &= 0b0111_1111;

        byte = match byte.checked_shl(num_bits_read) {
            None => {
                return Err(VartyIntError::TooManyBytesForType);
            },
            Some(v) => v,
        };
        val = val | byte;

        num_bits_read += 7;
        if is_last  {
            break;
        }
    }

    let val = (val >> 1) ^ -(val & 1);

    Ok((val, buf))
}

}}

read_signed!(read_i8, i8, 8);
read_signed!(read_i16, i16, 16);
read_signed!(read_i32, i32, 32);
read_signed!(read_i64, i64, 64);
read_signed!(read_i128, i128, 128);
read_signed!(read_isize, isize, std::mem::size_of::<isize>()*8);

macro_rules! write_signed {
    ( $name:ident, $type:ty ) => {

/// Write an integer to this buffer
pub fn $name(val: $type, buf: &mut Vec<u8>) {
    if val == 0 {
        buf.push(0);
        return;
    }

    let mut val = (val << 1) ^ (val >> std::mem::size_of::<$type>()*8-1);
    let mut num: u8;

    while val != 0 {
        num = (val & 0b0111_1111) as u8;
        val >>= 7;
        if val != 0 {
            num |= 0b1000_0000;
        }
        buf.push(num);
    }
}
}}

write_signed!(write_i8, i8);
write_signed!(write_i16, i16);
write_signed!(write_i32, i32);
write_signed!(write_i64, i64);
write_signed!(write_i128, i128);
write_signed!(write_isize, isize);

trait VarInt {
    fn as_varint(&self) -> Vec<u8>;
    fn from_varint(buf: &[u8]) -> Result<(Self, &[u8]), VartyIntError> where Self: Sized;
}

pub enum VartyIntReadError {
    VartyIntError(VartyIntError),
    ReadError(std::io::Error),
}

trait ReadVarInt {
    fn read_varint_i32(&mut self) -> Result<i32, VartyIntError>;
}

macro_rules! trait_impl {
    ( $type:ty, $read: ident, $write: ident ) => {

impl VarInt for $type {
    fn as_varint(&self) -> Vec<u8> {
        let mut vec = vec![];
        $write(*self, &mut vec);
        vec
    }
    fn from_varint(buf: &[u8]) -> Result<(Self, &[u8]), VartyIntError> {
        $read(buf)
    }
}


}}

trait_impl!(i8, read_i8, write_i8);
trait_impl!(i16, read_i16, write_i16);
trait_impl!(i32, read_i32, write_i32);
trait_impl!(i64, read_i64, write_i64);
trait_impl!(i128, read_i128, write_i128);

trait_impl!(u8, read_u8, write_u8);
trait_impl!(u16, read_u16, write_u16);
trait_impl!(u32, read_u32, write_u32);
trait_impl!(u64, read_u64, write_u64);
trait_impl!(u128, read_u128, write_u128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write1() {
        macro_rules! assert_write {
            ( $func:ident, $input:expr, $expected_output:expr ) => {
                {
                    let mut veccy = Vec::new();
                    $func($input, &mut veccy);
                    assert_eq!(veccy, $expected_output, "Ouput was {:?} but expected {:?}", veccy, $expected_output);
                }
        }}

        assert_write!(write_u128, 128, vec![128, 1]);
        assert_write!(write_u8, 0, vec![0]);
        assert_write!(write_u8, 0_u8, vec![0]);
        assert_write!(write_u8, 1_u8, vec![0b000_0001]);
        assert_write!(write_i8, 1_i8, vec![0b000_0010]);
        assert_write!(write_i64, 1_i64, vec![0b000_0010]);
        assert_write!(write_u32, 300_u32, vec![0b1010_1100, 0b0000_0010]);
        assert_write!(write_i8, -1_i8, vec![0b000_0001]);
        assert_write!(write_i32, 63_i32, vec![0x7e]);
        assert_write!(write_u32, 63_u32, vec![63]);
        assert_write!(write_i32, -64_i32, vec![0x7f]);

        assert_write!(write_u32, 127, vec![0b0111_1111]);
        assert_write!(write_u32, 128, vec![0b1000_0000, 0b0000_0001]);

        assert_write!(write_usize, 127, vec![0b0111_1111]);
        assert_write!(write_usize, 128, vec![0b1000_0000, 0b0000_0001]);

        assert_write!(write_usize, 7681, vec![129, 60]);

        assert_write!(write_i8, 0, vec![0]);
        assert_write!(write_i32, 0, vec![0]);
        assert_write!(write_u32, 0, vec![0]);
        assert_write!(write_usize, 12345, vec![185, 96]);
        assert_write!(write_isize, 12345, vec![242, 192, 1]);

    }

    #[test]
    fn read1() {
        macro_rules! assert_read {
            ( $func:ident, $input:expr, $expected_output:expr, $expected_output_buf:expr ) => {
                {
                    let output = $func($input);
                    assert!(output.is_ok(), "Expected {:?} to be OK, but got an error", $input);
                    let output = output.unwrap();
                    assert_eq!($expected_output, output.0, "Expected output {:?} but got {:?}", $expected_output, output.0);
                    assert_eq!($expected_output_buf, output.1, "Expected the rest of the buffer to be {:?} but got {:?}", $expected_output_buf, output.1);
                }
        }}

        macro_rules! assert_cant_read {
            ( $func:ident, $input:expr ) => {
                {
                    let output = $func($input);
                    assert!(output.is_err());

                }
        }}

        assert_read!(read_usize, &[185, 96], 12345, &[] as &[u8]);
        assert_read!(read_usize, &[127], 127, &[] as &[u8]);

        assert_read!(read_usize, &[0b1000_0000, 0b0000_0001], 128, &[] as &[u8]);

        assert_read!(read_u8, &[0], 0, &[] as &[u8]);
        assert_read!(read_u8, &[0, 200], 0, vec![200]);
        assert_read!(read_u8, &[1], 1, &[] as &[u8]);
        assert_cant_read!(read_u8, &[]);

        assert_read!(read_i64, &[0, 200], 0, vec![200]);

        assert_read!(read_isize, &[242, 192, 1], 12345, &[] as &[u8]);
        assert_read!(read_usize, &[188, 1, 105, 117, 121], 188, &[105, 117, 121]);

        assert_read!(read_i8, &[0x00], 0, &[] as &[u8]);
        assert_read!(read_i8, &[0x01], -1, &[] as &[u8]);
        assert_read!(read_i8, &[0x02], 1, &[] as &[u8]);
        assert_read!(read_i8, &[0x03], -2, &[] as &[u8]);
        assert_read!(read_i8, &[0x04], 2, &[] as &[u8]);

    }

    #[test]
    /// Ensure we get the same result out as in.
    fn varint_idempotent1() {
        macro_rules! assert_same {
            ( $reader:ident, $writer:ident, $input:expr ) => {
                {
                    let mut veccy = Vec::new();
                    $writer($input, &mut veccy);
                    let res = $reader(&veccy);
                    assert!(res.is_ok());
                    let (num, rest) = res.unwrap();
                    assert_eq!(num, $input);
                    assert!(rest.is_empty(), "Expected no further bytes, got {:?} instead", rest);
                }
        }}

        assert_same!(read_isize, write_isize, 12_345);
        assert_same!(read_isize, write_isize, -12_345);
        assert_same!(read_i8, write_i8, 1);

        assert_same!(read_usize, write_usize, 127);
        assert_same!(read_usize, write_usize, 128);

        assert_same!(read_usize, write_usize, 12_345);

        assert_same!(read_i64, write_i64, 50<<10);
    }

    #[test]
    fn bad1(){
        assert_eq!(read_u32(&[0b1010_1100]), Err(VartyIntError::NotEnoughBytes));
        assert_eq!(read_i32(&[0b1010_1100]), Err(VartyIntError::NotEnoughBytes));

        assert_eq!(read_i32(&[]), Err(VartyIntError::EmptyBuffer));

    }

    #[test]
    fn bad2() {
        assert_eq!(read_u8(&[128, 173, 226, 4]), Err(VartyIntError::TooManyBytesForType));
        assert_eq!(read_i8(&[128, 173, 226, 4]), Err(VartyIntError::TooManyBytesForType));
        assert_eq!(read_i16(&[128, 173, 226, 4]), Err(VartyIntError::TooManyBytesForType));
        assert_eq!(read_u16(&[128, 173, 226, 4]), Err(VartyIntError::TooManyBytesForType));
        assert_eq!(read_i32(&[128, 173, 226, 4]), Ok((5_000_000, &[] as &[u8])));
        assert_eq!(read_u32(&[128, 173, 226, 4]), Ok((10_000_000, &[] as &[u8])));
    }

    #[test]
    fn traits1() {
        let x: i32 = 1;
        assert_eq!(x.as_varint(), vec![0x02]);

        assert_eq!(i32::from_varint(&[0x02]), Ok((1, &[] as &[u8])));        
    }

}
