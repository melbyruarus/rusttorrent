extern crate rustc_serialize as serialize;
extern crate byteorder;
extern crate rusttorrent;

use std::net::TcpStream;
use std::str::FromStr;
use rusttorrent::messages::*;
use rusttorrent::support::*;

fn main() {
	let stream = TcpStream::connect("127.0.0.1:8080").unwrap(); //54004
	let stream_clone = stream.try_clone().unwrap();

	let manager = SocketManager::start::<TcpStream, TcpStream>(stream, stream_clone);

	manager.send(Message::Handshake(
									Protocol::BitTorrent,
									NONE,
									InfoHash::from_str("ca669b6679f03a329f25787c761651a8c36a26a4").ok().unwrap(),
									PeerId::from_str("RT097378612376745896").ok().unwrap()));
	manager.send(Message::Unchoke);
	manager.send(Message::Interested);
}