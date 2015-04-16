use super::support::*;
use super::network::*;
use super::convert::*;
use super::io_extensions::*;

use std;
use std::thread::{self, JoinGuard};
use std::sync::mpsc::{channel, Sender, Receiver, SendError};
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
	Request(u32, u32, u32),
	Piece(u32, u32, Vec<u8>),
	Cancel(u32, u32, u32),
	Close
}

pub struct SocketManager<'a> {
	sending: JoinGuard<'a ()>,
	receiving: JoinGuard<'a ()>,
	send_channel: Sender<Message>,
	receive_channel: Receiver<Message>
}

impl<'a> SocketManager<'a> {
	pub fn start<S: 'a + io::Write + Send, R: 'a + io::Read + Send>(mut sending_socket: S, mut listening_socket: R) -> SocketManager<'a> {
		let (client_send, server_listen) = channel::<Message>();
		let (server_send, client_listen) = channel::<Message>();

		SocketManager {
			send_channel: client_send,
			receive_channel: client_listen,
			sending: thread::scoped(move || {
			    for to_send in server_listen.iter() {
			    	match to_send {
			    		Message::Close => {
			    			// TODO: This all we do?
			    			break;
			    		}
			    		_ => {
			    			match SocketManager::write_message(to_send, &mut sending_socket) {
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
			}),
			receiving: thread::scoped(move || {
				// TODO: Deal with connection close

				// Read handshake
				let mut handshake_buffer = [0; 68];
				read_exact(&mut listening_socket, &mut handshake_buffer, 68);

				// Unpack handshake
				// TODO: deal with read errors
		    	server_send.send(SocketManager::read_handshake(&handshake_buffer).ok().unwrap());

				// Listen for normal messages

				let mut length_buffer = [0; 4];
			    loop {
			    	// Read length
			    	match read_exact(&mut listening_socket, &mut length_buffer, 4) {
			    		Ok(usize) => (),
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
			    		Ok(usize) => (),
			    		Err(err) => {
			    			// TODO: Is this what we want to do on errors?
							println!("error reading from socket {:?}", err);
							break;
			    		}
			    	}

			    	// We now have the whole message, parse it into a Message enum
			    	// TODO: deal with read errors
			    	server_send.send(SocketManager::read_message(message_buffer).ok().unwrap());
			    }

			    println!("closing receive connection")
			})
		}
	}

	pub fn send(&self, message: Message) -> Result<(), SendError<Message>> {
		self.send_channel.send(message)
	}

	pub fn recv_iter(&self) -> std::sync::mpsc::Iter<Message> {
		self.receive_channel.iter()
	}

	pub fn recv(&self) -> Result<Message, std::sync::mpsc::RecvError> {
		self.receive_channel.recv()
	}

	fn read_handshake(bytes: &[u8; 68]) -> Result<Message, ()> {
		// TODO: Real errors

		// Protocol
		if bytes[0] != 19 {
			return Err(());
		}

		let protocol = match std::str::from_utf8(&bytes[1..20]) {
			Ok(string) => string,
			Err(err) => return Err(())
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

				Ok(Message::Request(u32::from_bytes_be(&bytes[1..5]),
									u32::from_bytes_be(&bytes[5..9]),
									u32::from_bytes_be(&bytes[9..13])))
			}
			7 => {
				assert!(bytes.len() >= 9);

				Ok(Message::Piece(u32::from_bytes_be(&bytes[1..5]),
								  u32::from_bytes_be(&bytes[5..9]),
								  bytes[9..bytes.len()].to_vec()))
			}
			8 => {
				assert!(bytes.len() == 13);

				Ok(Message::Cancel(u32::from_bytes_be(&bytes[1..5]),
								   u32::from_bytes_be(&bytes[5..9]),
								   u32::from_bytes_be(&bytes[9..13])))
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
			Message::Request(index, begin, length) => {
				Packet::new()
					.u32(13)
					.byte(6)
					.u32(index)
					.u32(begin)
					.u32(length)
			}
			Message::Piece(index, begin, ref bytes) => {
				let length = match (1 + bytes.len()).to_u32() {
					Some(size) => size,
					None => panic!("packet too large")
				};
				
				Packet::new()
					.u32(length)
					.byte(7)
					.u32(index)
					.u32(begin)
					.bytes_vec(bytes)
			}
			Message::Cancel(index, begin, length) => {
				Packet::new()
					.u32(13)
					.byte(8)
					.u32(index)
					.u32(begin)
					.u32(length)
			}
			Message::Close => {
				unreachable!()
			}
		};

		packet.write(socket)
	}
}