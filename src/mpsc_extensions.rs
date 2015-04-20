use std::sync::mpsc::{RecvError,Receiver};

use super::timer;
use super::support::*;

pub fn recv_with_timeout<T: Send>(channel: &Receiver<T>, timeout: u32) -> Timeout<Result<T, RecvError>> {
	let timer = timer::oneshot(timeout);

	select!(
		result = channel.recv() => Timeout::Ok(result),
		_ = timer.recv() => Timeout::Timeout
	)
}
