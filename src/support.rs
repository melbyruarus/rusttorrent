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

pub struct InfoHash {
	pub bytes: [u8; 20],
	pub string: String
}

impl InfoHash {
	pub fn from_bytes(val: &[u8]) -> Option<InfoHash> {
		if val.len() == 20 {
			Some(InfoHash {
				bytes: to_fixed_size_hash_bytes(val),
				string: val.to_hex()
			})
		}
		else {
			None
		}
	}
}

impl std::fmt::Debug for InfoHash {
	fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		self.string.fmt(formatter)
	}
}

impl FromStr for InfoHash {
	type Err = InfoHashParseError;

	fn from_str(val: &str) -> Result<Self, InfoHashParseError>  {
		if val.len() != 40 {
			return Err(InfoHashParseError { _priv: ()});
		}

		let mut string = String::new();
		string.push_str(val);

		match val.from_hex() {
			Ok(as_vec) => {
				if as_vec.len() != 20 {
					return Err(InfoHashParseError { _priv: ()});
				}

				Ok(InfoHash {
					string: string,
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

pub struct PeerId {
	pub bytes: [u8; 20],
	pub string: String
}

impl PeerId {
	pub fn from_bytes(bytes: &[u8]) -> Option<PeerId> {
		if bytes.len() == 20 {
			let parsed_string = match std::str::from_utf8(&bytes) {
				Ok(val) => val,
				Err(_) => return None
			};

			let mut string = String::new();
			string.push_str(parsed_string);

			Some(PeerId {
				bytes: to_fixed_size_hash_bytes(bytes),
				string: string
			})
		}
		else {
			None
		}
	}
}

impl std::fmt::Debug for PeerId {
	fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		self.string.fmt(formatter)
	}
}

impl FromStr for PeerId {
	type Err = PeerIdParseError;

	fn from_str(val: &str) -> Result<Self, PeerIdParseError> {
		if val.len() != 20 {
			return Err(PeerIdParseError { _priv: ()});
		}

		let mut string = String::new();
		string.push_str(val);

		let as_bytes = val.as_bytes();

		let mut bytes = [0; 20];
		for i in 0..20 {
			bytes[i] = as_bytes[i];
		}

		Ok(PeerId {
			string: string,
			bytes: bytes
		})
	}
}