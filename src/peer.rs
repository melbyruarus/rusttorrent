use std::io;
use std::net;
use std::thread;
use std::sync::mpsc::{Sender,Receiver};
// use std::hash::{Hash, Hasher};

use std::str::FromStr;

use super::messages::{self, Message};
use super::support::*;
use super::mpsc_extensions::*;

pub struct Peer {
	pub send_channel: Sender<Message>,
	pub receive_channel: Receiver<Message>,
	pub peer_id: PeerId,
	pub internal_connection_id: u32
}

// impl Eq for Peer {
// }

// impl PartialEq for Peer {
// 	fn eq(&self, other: &Self) -> bool {
// 		self.internal_connection_id == other.internal_connection_id
// 	}

//     fn ne(&self, other: &Self) -> bool {
//     	!self.eq(other)
//     }
// }

// impl Hash for Peer {
// 	fn hash<H>(&self, state: &mut H) where H: Hasher {
// 		self.internal_connection_id.hash(state)
// 	}
// }

pub fn connect<A: net::ToSocketAddrs>(addr: A, info_hash: InfoHash, peer_id: PeerId, timeout: u32, internal_connection_id: u32) -> Option<Peer> {
	let stream = match net::TcpStream::connect(addr) {
		Ok(s) => s,
		Err(_) => return None
	};

	let stream_clone = match stream.try_clone() {
		Ok(s) => s,
		Err(_) =>  return None
	};

	let (send, receive) = messages::create_message_pair::<net::TcpStream, net::TcpStream>(stream, stream_clone);

	let protocol = Protocol::BitTorrent;
	let extensions = NONE;

	send.send(Message::Handshake(protocol, extensions, info_hash, peer_id));

   	match recv_with_timeout(&receive, timeout) {
		Timeout::Ok(result) => match result {
			Ok(message) => {
				match message {
					Message::Handshake(protocol, extensions, info_hash, peer_id) => {
						println!("Handshake({:?}, {:?}, {:?}, {:?}", protocol, extensions, info_hash, peer_id);
					},
					// TODO: Log?
					_ =>  return None
				}
			}
			// TODO: Log?
			Err(err) =>  return None
		},
		// TODO: Log?
		Timeout::Timeout =>  return None
	}

	Some(Peer {
		send_channel: send,
		receive_channel: receive,
		peer_id: peer_id,
		internal_connection_id: internal_connection_id
	})
}