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

#[cfg(test)]
mod tests;

/// Error type
#[derive(Debug, PartialEq, Eq)]
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
            while val >= 0b1000_0000 {
                buf.push((val as u8) | 0b1000_0000);
                val >>= 7;
            }
            buf.push(val as u8);
        }
    };
}

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
            let mut val: $type = 0;
            let mut shift = 0;
            let mut byte: $type;
            let mut is_last: bool;
            loop {
                if buf.is_empty() {
                    return Err(VartyIntError::NotEnoughBytes);
                }
                byte = buf[0] as $type;
                is_last = byte >> 7 == 0;
                byte &= 0b0111_1111;
                buf = &buf[1..];
                byte = match byte.checked_shl(shift) {
                    None => {
                        return Err(VartyIntError::TooManyBytesForType);
                    }
                    Some(b) => b,
                };
                val |= byte;
                shift += 7;
                if is_last {
                    break;
                }
            }

            Ok((val, buf))
        }
    };
}

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
            let mut val: i128 = 0;
            let mut is_last: bool;

            let mut byte: i128;

            loop {
                if buf.is_empty() {
                    return Err(VartyIntError::NotEnoughBytes);
                }
                byte = buf[0] as i128;
                buf = &buf[1..];

                is_last = byte >> 7 == 0;
                byte &= 0b0111_1111;

                byte = match byte.checked_shl(num_bits_read) {
                    None => {
                        return Err(VartyIntError::TooManyBytesForType);
                    }
                    Some(v) => v,
                };
                val |= byte;

                num_bits_read += 7;
                if is_last {
                    break;
                }
            }

            let mut sign = 1;
            if val & 0b0000_0001 == 1 {
                sign = -1;
                val += 1;
            }
            val >>= 1;
            val *= sign;

            match val.try_into() {
                Err(_) => Err(VartyIntError::TooManyBytesForType),
                Ok(val) => Ok((val, buf)),
            }
        }
    };
}

read_signed!(read_i8, i8, 8);
read_signed!(read_i16, i16, 16);
read_signed!(read_i32, i32, 32);
read_signed!(read_i64, i64, 64);
read_signed!(read_i128, i128, 128);
read_signed!(read_isize, isize, std::mem::size_of::<isize>() * 8);

macro_rules! write_signed {
    ( $name:ident, $type:ty ) => {
        /// Write an integer to this buffer
        pub fn $name(val: $type, buf: &mut Vec<u8>) {
            if val == 0 {
                buf.push(0);
                return;
            }

            // to prevent around overflows, work with i128 version of numbers
            // TODO What happens with i128 numbers & overflowing?
            let val: i128 = val as i128;
            // convert it to zig zag encoding
            let mut val = (val << 1) ^ (val >> std::mem::size_of::<$type>() * 8 - 1);
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
    };
}

write_signed!(write_i8, i8);
write_signed!(write_i16, i16);
write_signed!(write_i32, i32);
write_signed!(write_i64, i64);
write_signed!(write_i128, i128);
write_signed!(write_isize, isize);

pub trait VarInt: std::fmt::Debug + Copy {
    fn zero() -> Self;
    fn as_varint(&self) -> Vec<u8>;
    fn write_varint(&self, buf: &mut Vec<u8>);

    fn from_varint(buf: &[u8]) -> Result<(Self, &[u8]), VartyIntError>
    where
        Self: Sized;

    fn read_varint(buf: &[u8]) -> Result<(Self, &[u8]), VartyIntError>
    where
        Self: Sized,
    {
        Self::from_varint(buf)
    }
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
            fn zero() -> Self {
                0
            }
            fn as_varint(&self) -> Vec<u8> {
                let mut vec = vec![];
                $write(*self, &mut vec);
                vec
            }
            fn from_varint(buf: &[u8]) -> Result<(Self, &[u8]), VartyIntError> {
                $read(buf)
            }

            fn write_varint(&self, buf: &mut Vec<u8>) {
                $write(*self, buf)
            }
        }
    };
}

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

pub fn write_many_new<T>(nums: &[T]) -> Vec<u8>
where
    T: VarInt,
{
    let mut buf = vec![];
    write_many(nums, &mut buf);
    buf
}
pub fn write_many<T>(nums: &[T], buf: &mut Vec<u8>)
where
    T: VarInt,
{
    for num in nums.iter() {
        num.write_varint(buf);
    }
}

/// Read many different integers from this list of bytes, one after the other.
pub fn read_many<T>(buf: &[u8]) -> impl Iterator<Item = Result<T, VartyIntError>> + '_
where
    T: VarInt,
{
    let mut buf = buf;
    std::iter::from_fn(move || {
        if buf.is_empty() {
            return None;
        }
        match T::read_varint(buf) {
            Err(VartyIntError::EmptyBuffer) => None,
            Err(e) => Some(Err(e)),
            Ok((num, newbuf)) => {
                buf = newbuf;
                Some(Ok(num))
            }
        }
    })
}

pub fn write_many_delta_new<T>(nums: &[T]) -> Vec<u8>
where
    T: VarInt + std::ops::Sub<T, Output = T> + Copy,
{
    let mut buf = Vec::with_capacity(nums.len());
    write_many_delta(nums, &mut buf);
    buf
}

pub fn write_many_delta<T>(nums: &[T], buf: &mut Vec<u8>)
where
    T: VarInt + std::ops::Sub<T, Output = T>,
{
    let mut last: T = T::zero();
    for num in nums {
        (*num - last).write_varint(buf);
        last = *num
    }
}

/// Read many different integers from this list of bytes, one after the other, where the integers
/// are stores as offsets from each other. This is very effecient when a lot of integers are
/// incrementing
pub fn read_many_delta<'a, T>(buf: &'a [u8]) -> impl Iterator<Item = Result<T, VartyIntError>> + 'a
where
    T: VarInt + std::ops::Add<T, Output = T> + Copy + 'a,
{
    let mut buf = buf;
    let mut last = T::zero();
    std::iter::from_fn(move || {
        if buf.is_empty() {
            return None;
        }
        match T::read_varint(buf) {
            Err(VartyIntError::EmptyBuffer) => None,
            Err(e) => Some(Err(e)),
            Ok((num, newbuf)) => {
                buf = newbuf;
                last = last + num;
                Some(Ok(last))
            }
        }
    })
}

/// Read many different integers from this list of bytes, one after the other, where the integers
/// are stores as offsets from each other. This is very effecient when a lot of integers are
/// incrementing. Like `read_many_delta`, but returns the allocated vec for you.
pub fn read_many_delta_new<'a, T>(buf: &'a [u8]) -> Result<Vec<T>, VartyIntError>
where
    T: VarInt + std::ops::Add<T, Output = T> + Copy + 'a,
{
    read_many_delta(buf).collect::<Result<Vec<_>, _>>()
}
