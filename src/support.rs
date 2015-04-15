use std::str::FromStr;

extern crate rustc_serialize as serialize;
use self::serialize::hex::FromHex;

use super::macros;

bitflags!(
    flags Extensions: u16 {
    	const NONE = 0
    }
);

impl Extensions {
	pub fn to_bytes(&self) -> [u8; 8] {
		[0,0,0,0,0,0,0,0]
	}
}

pub enum Protocol {
	BitTorrent
}

impl Protocol {
	pub fn to_string(&self) -> &str {
		match *self {
			Protocol::BitTorrent => "BitTorrent"
		}
	}
}

pub struct InfoHash {
	pub bytes: [u8; 20],
	pub string: String
}

pub struct InfoHashParseError {
	_priv: ()
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

				let mut bytes = [0; 20];

				for i in 0..20 {
					bytes[i] = as_vec[i];
				}

				Ok(InfoHash {
					string: string,
					bytes: bytes
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