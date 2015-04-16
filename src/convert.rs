extern crate byteorder;
use self::byteorder::{ByteOrder, BigEndian, LittleEndian};

use std;

pub trait NetworkConvert<O, I> {
	fn to_bytes_be(self) -> O;
	fn from_bytes_be(bytes: &[I]) -> Self;
	fn to_bytes_le(self) -> O;
	fn from_bytes_le(bytes: &[I]) -> Self;
}

impl NetworkConvert<[u8; 4], u8> for u32 {
	fn to_bytes_be(self) -> [u8; 4] {
		let mut buf = [0; 4];
		<BigEndian as ByteOrder>::write_u32(&mut buf, self);

		buf
	}

	fn from_bytes_be(bytes: &[u8]) -> Self {
		<BigEndian as ByteOrder>::read_u32(bytes)
	}

	fn to_bytes_le(self) -> [u8; 4] {
		let mut buf = [0; 4];
		<LittleEndian as ByteOrder>::write_u32(&mut buf, self);

		buf
	}

	fn from_bytes_le(bytes: &[u8]) -> Self {
		<LittleEndian as ByteOrder>::read_u32(bytes)
	}
}

pub trait U32Converts {
	fn to_usize(self) -> Option<usize>;
	fn to_u8(self) -> Option<u8>;
}

impl U32Converts for u32 {
	fn to_usize(self) -> Option<usize> {
		if std::mem::size_of::<usize>() > std::mem::size_of::<u32>() {
			Some(self as usize)
		}
		else {
			if self < std::usize::MAX as u32 {
				Some(self as usize)
			}
			else {
				None
			}
		}
	}

	fn to_u8(self) -> Option<u8> {
		if self < std::u8::MAX as u32 {
			Some(self as u8)
		}
		else {
			None
		}
	}
}

pub trait UsizeConverts {
	fn to_u32(self) -> Option<u32>;
	fn to_u8(self) -> Option<u8>;
}

impl UsizeConverts for usize {
	fn to_u32(self) -> Option<u32> {
		if std::mem::size_of::<u32>() > std::mem::size_of::<usize>() {
			Some(self as u32)
		}
		else {
			if self < std::u32::MAX as usize {
				Some(self as u32)
			}
			else {
				None
			}
		}
	}

	fn to_u8(self) -> Option<u8> {
		if self < std::u8::MAX as usize {
			Some(self as u8)
		}
		else {
			None
		}
	}
}