use super::*;

macro_rules! test_write {
    ( $name:ident, $func:ident, $input:expr, $expected_output:expr ) => {
        #[test]
        fn $name()
        {
            let mut veccy = Vec::new();
            $func($input, &mut veccy);
            assert_eq!(veccy, $expected_output, "Ouput was {:?} but expected {:?}", veccy, $expected_output);
        }
}}

test_write!(write2, write_u128, 128, vec![128, 1]);
test_write!(write3, write_u8, 0, vec![0]);
test_write!(write4, write_u8, 0_u8, vec![0]);
test_write!(write5, write_u8, 1_u8, vec![0b000_0001]);
test_write!(write6, write_i8, 1_i8, vec![0b000_0010]);
test_write!(write7, write_i64, 1_i64, vec![0b000_0010]);
test_write!(write8, write_u32, 300_u32, vec![0b1010_1100, 0b0000_0010]);
test_write!(write9, write_i8, -1_i8, vec![0b000_0001]);
test_write!(write10, write_i32, 63_i32, vec![0x7e]);
test_write!(write11, write_u32, 63_u32, vec![63]);
test_write!(write12, write_i32, -64_i32, vec![0x7f]);
test_write!(write13, write_u32, 127, vec![0b0111_1111]);
test_write!(write14, write_u32, 128, vec![0b1000_0000, 0b0000_0001]);
test_write!(write15, write_usize, 127, vec![0b0111_1111]);
test_write!(write16, write_usize, 128, vec![0b1000_0000, 0b0000_0001]);
test_write!(write17, write_usize, 7681, vec![129, 60]);
test_write!(write18, write_i8, 0, vec![0]);
test_write!(write19, write_i32, 0, vec![0]);
test_write!(write20, write_u32, 0, vec![0]);
test_write!(write21, write_usize, 12345, vec![185, 96]);
test_write!(write22, write_isize, 12345, vec![242, 192, 1]);

test_write!(write23, write_i32, -1649404, vec![247, 171, 201, 1]);
test_write!(write24, write_i32, 2001649404, vec![248, 251, 245, 244, 14]);


macro_rules! test_read {
    ( $name:ident, $func:ident, $input:expr, $expected_output:expr, $expected_output_buf:expr ) => {
        #[test]
        fn $name()
        {
            let output = $func($input);
            assert!(output.is_ok(), "Expected {:?} to be OK, but got an error", $input);
            let output = output.unwrap();
            assert_eq!($expected_output, output.0, "Expected output {:?} but got {:?}", $expected_output, output.0);
            assert_eq!($expected_output_buf, output.1, "Expected the rest of the buffer to be {:?} but got {:?}", $expected_output_buf, output.1);
        }
}}

macro_rules! test_cant_read {
    ( $name:ident, $func:ident, $input:expr ) => {
        #[test]
        fn $name()
        {
            let output = $func($input);
            assert!(output.is_err());
        }
}}

test_read!(read2, read_usize, &[185, 96], 12345, &[] as &[u8]);
test_read!(read3, read_usize, &[127], 127, &[] as &[u8]);
test_read!(read4, read_usize, &[0b1000_0000, 0b0000_0001], 128, &[] as &[u8]);

test_read!(read5, read_u8, &[0], 0, &[] as &[u8]);
test_read!(read6, read_u8, &[0, 200], 0, vec![200]);
test_read!(read7, read_u8, &[1], 1, &[] as &[u8]);
test_cant_read!(empty, read_u8, &[]);

test_read!(read9, read_i64, &[0, 200], 0, vec![200]);
test_read!(read10, read_isize, &[242, 192, 1], 12345, &[] as &[u8]);
test_read!(read11, read_usize, &[188, 1, 105, 117, 121], 188, &[105, 117, 121]);
test_read!(read12, read_i8, &[0x00], 0, &[] as &[u8]);
test_read!(read13, read_i8, &[0x01], -1, &[] as &[u8]);
test_read!(read14, read_i8, &[0x02], 1, &[] as &[u8]);
test_read!(read15, read_i8, &[0x03], -2, &[] as &[u8]);
test_read!(read16, read_i8, &[0x04], 2, &[] as &[u8]);

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
