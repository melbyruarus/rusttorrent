use std::io;

// TODO: Can you implement a trait for a trait?
pub fn read_exact<T: io::Read>(stream: &mut T, buffer: &mut [u8], amount: usize) -> io::Result<usize> {
	assert!(buffer.len() >= amount);

	let mut amount_read = 0;
	while amount_read < amount {
		// TODO: Only care about second half
		let (_, sliced) = buffer.split_at_mut(amount_read);

		amount_read += match stream.read(sliced) {
			Ok(size) => size,
			Err(err) => return Err(err)
		};
	}

	Ok(amount_read)
}