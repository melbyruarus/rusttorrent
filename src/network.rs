use std;
use std::io;

extern crate byteorder;
use self::byteorder::{ByteOrder, BigEndian};

pub struct Packet {
	data: Vec<u8>
}

impl Packet {
	pub fn new() -> Packet {
		Packet { data: Vec::new() }
	}

	pub fn length_prefixed_string(self, string: &str) -> Packet {
		if string.len() > std::u8::MAX as usize {
			panic!("string too long");
		}

		self.byte(string.len() as u8).string(string)
	}

	pub fn string(self, string: &str) -> Packet {
		self.bytes(string.as_bytes())
	}

	pub fn bytes(mut self, bytes: &[u8]) -> Packet {
		for byte in bytes {
			self.data.push(*byte);
		}

		self
	}

	pub fn bytes_vec(mut self, bytes: &Vec<u8>) -> Packet {
		for byte in bytes {
			self.data.push(*byte);
		}

		self
	}

	pub fn u32(self, val: u32) -> Packet {
		let mut buf = [0; 4];
		<BigEndian as ByteOrder>::write_u32(&mut buf, val);

		self.bytes(&buf)
	}

	pub fn byte(mut self, val: u8) -> Packet {
		self.data.push(val);

		self
	}

	pub fn write<T: io::Write + Send>(self, socket: &mut T) -> io::Result<usize> {
		socket.write(&self.data[0..self.data.len()])
	}
}