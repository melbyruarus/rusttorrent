use super::support::*;

struct InflightPiece {
	index: u32,
	blocks_to_request: Vec<u32>
}

pub trait PieceSelector {
    fn next_request(&mut self) -> Option<BlockRequest>;
}

struct SequentialPieceSelector {
    piece_size: u32,
    block_size: u32,
	pieces_to_download: Vec<u32>,
	inflight_pieces: Vec<InflightPiece>
}

impl PieceSelector for SequentialPieceSelector {
    fn next_request(&mut self) -> Option<BlockRequest> {
		let inflight_piece_request = match self.inflight_pieces.last_mut() {
			Some(inflight_piece) => {
				match inflight_piece.blocks_to_request.pop() {
					Some(next_block) => {
						Some(BlockRequest {
							start: BlockBegin {
								piece: inflight_piece.index,
								offset: next_block
							},
							length: self.block_size
							})
					}
					None => {
						None
					}
				}
			}
			None => {
				None
			}
		};

		match inflight_piece_request {
			Some(inflight_piece_request) => Some(inflight_piece_request),
			None => {
				match self.pieces_to_download.pop() {
					Some(piece_index) => {
						self.inflight_pieces.push(InflightPiece {
							index: piece_index,
							// This is not what I was lead to beleive, each block is identified by its
							// count from the begining of the piece (assuming 16k blocks), rather than being a direct
							// byte offset.
							blocks_to_request: (1..((self.piece_size/self.block_size)+1)).collect()
							});

						Some(BlockRequest {
							start: BlockBegin {
								piece: piece_index,
								offset: 0
							},
							length: self.block_size
							})
					}
					None => {
						None
					}
				}
			}
		}
	}
}

pub fn new_sequential_selector(piece_count: u32, piece_size: u32, block_size: u32) -> Box<PieceSelector> {
    Box::new(SequentialPieceSelector {
        piece_size: piece_size,
        block_size: block_size,
        pieces_to_download: (0..piece_count).rev().collect(),
        inflight_pieces: vec!()
    })
}
