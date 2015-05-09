use super::support::*;
use super::network::*;
use super::convert::*;
use super::io_extensions::*;
use super::mpsc_extensions::*;

use comm::spsc::unbounded::{self, Producer, Consumer};

use std;
use std::thread;
use std::io;

use std::iter::repeat;

#[derive(Debug)]
pub enum Message {
	Handshake(Protocol, Extensions, InfoHash, PeerId),
	KeepAlive,
	Choke,
	Unchoke,
	Interested,
	NotInterested,
	Have(u32),
	Bitfield(Vec<u8>),
	Request(BlockRequest),
	Piece(BlockBegin, Vec<u8>),
	Cancel(BlockRequest),
	Close
}

pub fn create_message_pair<S: 'static + io::Write + Send, R: 'static + io::Read + Send>(mut sending_socket: S, mut listening_socket: R) -> (Producer<Message>, Consumer<Message>) {
	let (client_send, server_listen) = unbounded::new();
	let (server_send, client_listen) = unbounded::new();

	thread::spawn(move || {
		for to_send in recv_iter(&server_listen) {
	    	match to_send {
	    		Message::Close => {
	    			break
	    		}
	    		_ => {
	    			match write_message(to_send, &mut sending_socket) {
	    				Ok(size) => println!("wrote {} bytes", size),
	    				Err(err) => {
	    					// TODO: Is this what we want to do on errors?
	    					println!("error writing to socket {:?}", err);
	    					break;
	    				}
	    			}
	    		}
	    	}
	    }

	    println!("closing send connection")
	});

	thread::spawn(move || {
		// TODO: Deal with connection close

		// Read handshake
		let mut handshake_buffer = [0; 68];
		match read_exact(&mut listening_socket, &mut handshake_buffer, 68) {
			Ok(_) => (),
			Err(err) => {
				// TODO: Is this what we want to do on errors?
				println!("error reading from socket {:?}", err);
				return;
			}
		}

		// Unpack handshake
		// TODO: deal with read & write errors
    	let _ = server_send.send(read_handshake(&handshake_buffer).ok().unwrap());

		// Listen for normal messages

		let mut length_buffer = [0; 4];
	    loop {
	    	// Read length
	    	match read_exact(&mut listening_socket, &mut length_buffer, 4) {
	    		Ok(_) => (),
	    		Err(err) => {
	    			// TODO: Is this what we want to do on errors?
					println!("error reading from socket {:?}", err);
					break;
	    		}
	    	}

	    	let message_length = match u32::from_bytes_be(& length_buffer).to_usize() {
	    		Some(length) => length,
	    		None => {
	    			// TODO: Is this what we want to do on errors?
					println!("packet too large");
	    			break;
	    		}
	    	};

	    	// TODO: Define max packet length so can't allocate memory indefinetly and crash
	    	// Allocate space
	    	let mut message_buffer = Vec::<u8>::new();
	    	message_buffer.extend(repeat(0).take(message_length));

	    	// Read message
	    	match read_exact(&mut listening_socket, &mut message_buffer, message_length) {
	    		Ok(_) => (),
	    		Err(err) => {
	    			// TODO: Is this what we want to do on errors?
					println!("error reading from socket {:?}", err);
					break;
	    		}
	    	}

	    	// We now have the whole message, parse it into a Message enum
	    	// TODO: deal with read & write errors
			let _ = server_send.send(read_message(message_buffer).ok().unwrap());
	    }

	    println!("closing receive connection")
	});

	(client_send, client_listen)
}

fn read_handshake(bytes: &[u8; 68]) -> Result<Message, ()> {
	// TODO: Real errors

	// Protocol
	if bytes[0] != 19 {
		return Err(());
	}

	let protocol = match std::str::from_utf8(&bytes[1..20]) {
		Ok(string) => string,
		Err(_) => return Err(())
	};

	if Protocol::BitTorrent.to_string() != protocol {
		return Err(());
	}

	// Extensions
	let extensions = NONE;

	// Info hash
	let info_hash = match InfoHash::from_bytes(&bytes[28..48]) {
		Some(hash) => hash,
		None => return Err(())
	};

	// Peer Id
	let peer_id = match PeerId::from_bytes(&bytes[48..68]) {
		Some(id) => id,
		None => return Err(())
	};

	Ok(Message::Handshake(Protocol::BitTorrent, extensions, info_hash, peer_id))
}

fn read_message(bytes: Vec<u8>) -> Result<Message, ()> {
	if bytes.len() == 0 {
		return Ok(Message::KeepAlive);
	}

	let id = bytes[0];

	match id {
		0 => Ok(Message::Choke),
		1 => Ok(Message::Unchoke),
		2 => Ok(Message::Interested),
		3 => Ok(Message::NotInterested),
		4 => {
			assert!(bytes.len() == 5);

			Ok(Message::Have(u32::from_bytes_be(&bytes[1..5])))
		}
		5 => Ok(Message::Bitfield(bytes[1..bytes.len()].to_vec())),
		6 => {
			assert!(bytes.len() == 13);

			Ok(Message::Request(BlockRequest {
				start: BlockBegin {
					piece: u32::from_bytes_be(&bytes[1..5]),
					offset: u32::from_bytes_be(&bytes[5..9])
				},
				length: u32::from_bytes_be(&bytes[9..13])
			}))
		}
		7 => {
			assert!(bytes.len() >= 9);

			Ok(Message::Piece(BlockBegin {
					piece: u32::from_bytes_be(&bytes[1..5]),
					offset: u32::from_bytes_be(&bytes[5..9])
				},
				bytes[9..bytes.len()].to_vec()))
		}
		8 => {
			assert!(bytes.len() == 13);

			Ok(Message::Cancel(BlockRequest {
				start: BlockBegin {
					piece: u32::from_bytes_be(&bytes[1..5]),
					offset: u32::from_bytes_be(&bytes[5..9])
				},
				length: u32::from_bytes_be(&bytes[9..13])
			}))
		}
		_ => Err(())
	}
}

fn write_message<T: io::Write + Send>(message: Message, socket: &mut T) -> io::Result<usize> {
	let packet = match message {
		Message::Handshake(protocol, extensions, info_hash, peer_id) => {
			Packet::new()
				.length_prefixed_string(protocol.to_string())
				.bytes(&extensions.to_bytes())
				.bytes(&info_hash.bytes)
				.bytes(&peer_id.bytes)
		}
		Message::KeepAlive => {
			Packet::new()
				.u32(0)
		}
		Message::Choke => {
			Packet::new()
				.u32(1)
				.byte(0)
		}
		Message::Unchoke => {
			Packet::new()
				.u32(1)
				.byte(1)
		}
		Message::Interested => {
			Packet::new()
				.u32(1)
				.byte(2)
		}
		Message::NotInterested => {
			Packet::new()
				.u32(1)
				.byte(3)
		}
		Message::Have(piece) => {
			Packet::new()
				.u32(5)
				.byte(4)
				.u32(piece)
		}
		Message::Bitfield(ref bits) => {
			let length = match (1 + bits.len()).to_u32() {
				Some(size) => size,
				None => panic!("packet too large")
			};

			Packet::new()
				.u32(length)
				.byte(5)
				.bytes_vec(bits)
		}
		Message::Request(request) => {
			Packet::new()
				.u32(13)
				.byte(6)
				.u32(request.start.piece)
				.u32(request.start.offset)
				.u32(request.length)
		}
		Message::Piece(ref start, ref bytes) => {
			let length = match (1 + bytes.len()).to_u32() {
				Some(size) => size,
				None => panic!("packet too large")
			};

			Packet::new()
				.u32(length)
				.byte(7)
				.u32(start.piece)
				.u32(start.offset)
				.bytes_vec(bytes)
		}
		Message::Cancel(request) => {
			Packet::new()
				.u32(13)
				.byte(8)
				.u32(request.start.piece)
				.u32(request.start.offset)
				.u32(request.length)
		}
		Message::Close => {
			unreachable!()
		}
	};

	packet.write(socket)
}
