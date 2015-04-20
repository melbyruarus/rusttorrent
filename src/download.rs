use std::net;
// use std::collections::HashSet;

use super::peer::{self, Peer};
use super::support::*;

// TODO: Temporary
use std::sync::mpsc::Select;

pub struct Downloader {
	info_hash: InfoHash,
	peer_id: PeerId,
	peers: Vec<Peer>,
	internal_connection_counter: u32
}

pub fn new(info_hash: InfoHash, peer_id: PeerId) -> Downloader {
	Downloader {
		info_hash: info_hash,
		peer_id: peer_id,
		peers: Vec::new(),
		internal_connection_counter: 0
	}
}

impl Downloader {
	pub fn add_peer<A: net::ToSocketAddrs>(&mut self, addr: A) {
		let internal_connection_id = self.internal_connection_counter;
		self.internal_connection_counter += 1;

		match peer::connect(addr, self.info_hash, self.peer_id, 30000, internal_connection_id) {
			Some(peer) => {self.peers.push(peer);},
			None => ()
		}
	}

	pub fn start(&self) {
		println!("{:?}", self.peers.len());

		loop {
			let select = Select::new();

			let mut handle = select.handle(&self.peers[0].receive_channel);

			unsafe {
				handle.add();
			}

			select.wait();

			println!("got: {:?}", self.peers[0].receive_channel.recv());
		}
	}
}