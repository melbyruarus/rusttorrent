extern crate rusttorrent;

use std::str::FromStr;

use rusttorrent::download;
use rusttorrent::support::*;

fn main() {
	let info_hash = InfoHash::from_str("ca669b6679f03a329f25787c761651a8c36a26a4").ok().unwrap();
	let peer_id = PeerId::from_str("-RU0001-965t0j7HrmHh").ok().unwrap();

	let mut downloader = download::new(info_hash, 404, 33792, peer_id);

	downloader.add_peer("127.0.0.1:54004");

	downloader.start();
}
