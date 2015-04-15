use super::support::*;
use super::network::*;

use std;
use std::thread::{self, JoinGuard};
use std::sync::mpsc::{channel, Sender, Receiver, SendError};
use std::io;

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
			    			break;
			    		}
			    		_ => {
			    			match SocketManager::write_packet(to_send, &mut sending_socket) {
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
				// TODO: Deal with fragmented messages
				// TODO: Deal with multiple packets
				let mut buffer = [0; 20000];
			    loop {
			    	// TODO: Handle errors
			    	let bytes_read = listening_socket.read(&mut buffer).ok().unwrap();
			    }

			    println!("closing receive connection")
			})
		}
	}

	pub fn send(&self, message: Message) -> Result<(), SendError<Message>> {
		self.send_channel.send(message)
	}

	fn write_packet<T: io::Write + Send>(message: Message, socket: &mut T) -> io::Result<usize> {
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
				let length = 1 + bits.len();

				if length > std::u32::MAX as usize {
					panic!("packet too large");
				}

				Packet::new()
					.u32(length as u32)
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
				let length = 1 + bytes.len();

				if length > std::u32::MAX as usize {
					panic!("packet too large");
				}
				
				Packet::new()
					.u32(length as u32)
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