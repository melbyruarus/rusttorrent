use std::io;
use std::net;
use std::thread;

use std::str::FromStr;

use comm::spsc::unbounded::{self, Producer, Consumer};

use super::messages::{self, Message};
use super::support::*;
use super::mpsc_extensions::*;

pub struct Peer {
	pub send_channel: Producer<Message>,
	pub receive_channel: Consumer<Message>,
	pub peer_id: PeerId,
	pub internal_connection_id: u32,
	pub upload_rate_to_us: u32,
	pub download_rate_from_us: u32,
	pub is_interested: bool,
	pub is_choking: bool,
	pub am_interested: bool,
	pub am_choking: bool
}

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
		internal_connection_id: internal_connection_id,
		upload_rate_to_us: 0,
		download_rate_from_us: 0,
		is_interested: false,
		is_choking: true,
		am_interested: false,
		am_choking: true
	})
}
