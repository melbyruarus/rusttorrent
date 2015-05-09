use rand::{self, Rng};
use comm::select::{Select, Selectable};
use comm::endpoint::Consumer;

use std::net;

use super::peer::{self, Peer};
use super::support::*;
use super::timer;
use super::messages::Message;

pub struct Downloader {
	info_hash: InfoHash,
	peer_id: PeerId,
	peers: Vec<Peer>,
	internal_connection_counter: u32,
	choke_algorithm_counter: u8,
	select: Select
}

pub fn new(info_hash: InfoHash, peer_id: PeerId) -> Downloader {
	Downloader {
		info_hash: info_hash,
		peer_id: peer_id,
		peers: Vec::new(),
		internal_connection_counter: 0,
		choke_algorithm_counter: 0,
		select: Select::new()
	}
}

impl Downloader {
	pub fn add_peer<A: net::ToSocketAddrs>(&mut self, addr: A) {
		let internal_connection_id = self.internal_connection_counter;
		self.internal_connection_counter += 1;

		match peer::connect(addr, self.info_hash, self.peer_id, 30000, internal_connection_id) {
			Some(peer) => {
				self.select.add(&peer.receive_channel);
				self.peers.push(peer)
			},
			None => ()
		}
	}

	fn peer_index_for_channel_id(&self, id: usize) -> Option<usize> {
		let mut index = 0;

		for peer in self.peers.iter() {
			if id == peer.receive_channel.id() {
				return Some(index)
			}

			index += 1;
		}

		None
	}

	pub fn start(&mut self) {
		println!("{:?}", self.peers.len());

		let choke_algorithm_timer = timer::repeating(10000);
		self.select.add(&choke_algorithm_timer);

		loop {
			let id = self.select.wait(&mut [0])[0];

			if let Some(peer_index) = self.peer_index_for_channel_id(id) {
				self.process_peer_message_for_index(peer_index);
			}
			else if id == choke_algorithm_timer.id() {
				let _ = choke_algorithm_timer.recv_sync();
				self.run_choke_algorithm();
			}
		}
	}

	fn process_peer_message_for_index(&mut self, peer_index: usize) {
		let peer = &mut self.peers[peer_index];

		match peer.receive_channel.recv_sync() {
			Ok(message) => {
				println!("got: {:?}", message);

				match message {
					Message::KeepAlive => (),
					Message::Handshake(_, _, _, _) => (), // Should never happen
					Message::Choke => peer.is_choking = true,
					Message::Unchoke => peer.is_choking = false,
					Message::Interested => peer.is_interested = true,
					Message::NotInterested => peer.is_interested = false,
					Message::Have(_) => (), // TODO: Implement
					Message::Bitfield(_) => (), // TODO: Implement
					Message::Request(_) => (), // TODO: Implement
					Message::Piece(_, _) => (), // TODO: Implement
					Message::Cancel(_) => (), // TODO: Implement
					Message::Close => (), // TODO: Implement
				}
			}
			Err(_) => {
				// TODO: remove peer?
			}
		}
	}

	fn run_choke_algorithm(&mut self) {
		self.choke_algorithm_counter += 1;

		let should_optimistic_unchoke = if self.choke_algorithm_counter == 3 {
			self.choke_algorithm_counter = 0;
			true
		}
		else {
			false
		};

		println!("run choke algritithm: {:?}", should_optimistic_unchoke);

		// Todo use something more efficient, needs to update each time. Good sort for partially sorted data?
		self.peers.sort_by(|a, b| a.upload_rate_to_us.cmp(&b.upload_rate_to_us));

		// These must be vectors, as we may need to pop the last element off in the case where we optimistically
		// unchoke a peer who is interested in us.
		let mut indexes_to_unchoke = Vec::new();
		let mut indexes_to_choke = Vec::new();

		let mut num_of_unchoked_and_interested = 0;
		let mut index_of_peer_with_worst_upload_rate_to_us = 0;
		let mut index = 0;

		// Unchoke everyone with a good upload_rate_to_us rate until we have four peers who are interested. Everyone worse than that get choked
		for peer in self.peers.iter() {
			if num_of_unchoked_and_interested < 4 {
				if peer.is_interested {
					num_of_unchoked_and_interested += 1;
					index_of_peer_with_worst_upload_rate_to_us = index;
				}

				if peer.am_choking {
					indexes_to_unchoke.push(index);
				}
			}
			else {
				if !peer.am_choking {
					indexes_to_choke.push(index);
				}
			}

			index += 1;
		}

		// Do the optimistic unchoke
		if should_optimistic_unchoke && index_of_peer_with_worst_upload_rate_to_us+1 < self.peers.len() {
			let peer_to_optimistically_unchoke = rand::thread_rng().gen_range(index_of_peer_with_worst_upload_rate_to_us+1, self.peers.len());

			if self.peers[peer_to_optimistically_unchoke].is_interested {
				// Choke worst peer we aren't choking

				if indexes_to_unchoke.last() == Some(&index_of_peer_with_worst_upload_rate_to_us) {
					indexes_to_unchoke.pop();
				}

				if !self.peers[index_of_peer_with_worst_upload_rate_to_us].am_choking {
					indexes_to_choke.push(index_of_peer_with_worst_upload_rate_to_us);
				}
			}
		}

		// Apply results of algorithm
		for index in indexes_to_unchoke {
			let peer = &mut self.peers[index];
			peer.am_choking = false;
			// TODO: Deal with write errors
			let _ = peer.send_channel.send(Message::Unchoke);
		}

		for index in indexes_to_choke {
			let peer = &mut self.peers[index];
			peer.am_choking = true;
			// TODO: Deal with write errors
			let _ = peer.send_channel.send(Message::Choke);
		}
	}
}
