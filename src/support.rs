use std;
use std::str::FromStr;

extern crate rustc_serialize as serialize;
use self::serialize::hex::FromHex;
use self::serialize::hex::ToHex;

use super::macros;



fn to_fixed_size_hash_bytes(bytes: &[u8]) -> [u8; 20] {
	assert!(bytes.len() == 20);
	
	let mut new_bytes = [0; 20];
	for i in 0..20 {
		new_bytes[i] = bytes[i];
	}

	new_bytes
}

pub enum Timeout<T> {
	Ok(T),
	Timeout
}



bitflags!(
	#[derive(Debug)]
    flags Extensions: u16 {
    	const NONE = 0
    }
);

impl Extensions {
	pub fn to_bytes(&self) -> [u8; 8] {
		[0,0,0,0,0,0,0,0]
	}
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct BlockRequest {
	pub start: BlockBegin,
	pub length: u32
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct BlockBegin {
	pub piece: u32,
	pub offset: u32
}

#[derive(Debug)]
pub enum Protocol {
	BitTorrent
}

impl Protocol {
	pub fn to_string(&self) -> &str {
		match *self {
			Protocol::BitTorrent => "BitTorrent protocol"
		}
	}
}

pub struct InfoHashParseError {
	_priv: ()
}

#[derive(Copy, Clone)]
pub struct InfoHash {
	pub bytes: [u8; 20]
}

impl InfoHash {
	pub fn from_bytes(val: &[u8]) -> Option<InfoHash> {
		if val.len() == 20 {
			Some(InfoHash {
				bytes: to_fixed_size_hash_bytes(val)
			})
		}
		else {
			None
		}
	}
}

impl std::fmt::Debug for InfoHash {
	fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		self.to_string().fmt(formatter)
	}
}

impl std::string::ToString for InfoHash {
	fn to_string(&self) -> String {
		self.bytes.to_hex()
	}
}

impl FromStr for InfoHash {
	type Err = InfoHashParseError;

	fn from_str(val: &str) -> Result<Self, InfoHashParseError>  {
		if val.len() != 40 {
			return Err(InfoHashParseError { _priv: ()});
		}

		match val.from_hex() {
			Ok(as_vec) => {
				if as_vec.len() != 20 {
					return Err(InfoHashParseError { _priv: ()});
				}

				Ok(InfoHash {
					bytes: to_fixed_size_hash_bytes(&as_vec[0..20])
				})
			}
			Err(_) => {
				Err(InfoHashParseError { _priv: ()})
			}
		}
	}
}

pub struct PeerIdParseError {
	_priv: ()
}

#[derive(Copy, Clone)]
pub struct PeerId {
	pub bytes: [u8; 20]
}

impl PeerId {
	pub fn from_bytes(bytes: &[u8]) -> Option<PeerId> {
		if bytes.len() == 20 {
			// Check whether this will parse, as we use this in to_string()
			match std::str::from_utf8(&bytes) {
				Err(_) => return None,
				_ => ()
			};

			Some(PeerId {
				bytes: to_fixed_size_hash_bytes(bytes)
			})
		}
		else {
			None
		}
	}
}

impl std::fmt::Debug for PeerId {
	fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		self.to_string().fmt(formatter)
	}
}

impl std::string::ToString for PeerId {
	fn to_string(&self) -> String {
		// This should never panic, because we have a check in from_bytes()
		std::string::String::from_utf8(self.bytes.to_vec()).ok().unwrap()
	}
}

impl FromStr for PeerId {
	type Err = PeerIdParseError;

	fn from_str(val: &str) -> Result<Self, PeerIdParseError> {
		if val.len() != 20 {
			return Err(PeerIdParseError { _priv: ()});
		}

		let as_bytes = val.as_bytes();

		let mut bytes = [0; 20];
		for i in 0..20 {
			bytes[i] = as_bytes[i];
		}

		Ok(PeerId {
			bytes: bytes
		})
	}
}