extern crate rustc_serialize as serialize;
extern crate byteorder;
extern crate rusttorrent;

use std::net::TcpStream;
use std::str::FromStr;
use rusttorrent::messages::*;
use rusttorrent::support::*;
use std::thread;

fn main() {
	let addr = "127.0.0.1:54004";
	let stream = match TcpStream::connect(addr) {
		Ok(s) => s,
		Err(e) => panic!("Couldn't connect to {}, got {}", addr, e)
	};
	let stream_clone = stream.try_clone().unwrap();

	let manager = SocketManager::start::<TcpStream, TcpStream>(stream, stream_clone);

	manager.send(Message::Handshake(
									Protocol::BitTorrent,
									NONE,
									InfoHash::from_str("ca669b6679f03a329f25787c761651a8c36a26a4").ok().unwrap(),
									PeerId::from_str("RT097378612376745896").ok().unwrap()));
	manager.send(Message::Unchoke);
	manager.send(Message::Interested);

	thread::scoped(move || {
		match manager.recv() {
			Ok(message) => {
				match message {
					Message::Handshake(protocol, extensions, info_hash, peer_id) => {
						println!("Handshake({:?}, {:?}, {:?}, {:?}", protocol, extensions, info_hash, peer_id);
					},
					_ => {
						println!("Unexpected message {:?}", message);
						return;
					}
				}
			}
			Err(err) => {
				println!("Error listening for data {:?}", err);
				return;
			}
		}

		for message in manager.recv_iter() {
			println!("{:?}", message);
		}
	});
}
