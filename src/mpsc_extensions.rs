use comm::{Error, Sendable};
use comm::select::{Select, Selectable};
use comm::endpoint;

use super::timer;
use super::support::*;

use std::iter::Iterator;

// TODO: Figure out how to use like <consumer>.recv_with_timeout(<timeout>)
pub fn recv_with_timeout<T: Send, C : endpoint::Consumer<T>>(consumer: &C, timeout: u32) -> Timeout<Result<T, Error>> {
	let timer = timer::oneshot(timeout);

	let select = Select::new();
	select.add(&timer);
	select.add(consumer);

	let id = select.wait(&mut [0])[0];

	if id == timer.id() {
		Timeout::Timeout
	}
	else if id == consumer.id() {
		Timeout::Ok(consumer.recv_async())
	}
	else {
		unreachable!()
	}
}

pub struct RecvIter<'a, T: Sendable> {
	rx: &'a endpoint::Consumer<T>
}

impl<'a, T: Sendable> Iterator for RecvIter<'a, T> {
	type Item = T;

    fn next(&mut self) -> Option<T> { self.rx.recv_sync().ok() }
}

pub fn recv_iter<'a, T: Send, C : endpoint::Consumer<T>>(consumer: &'a C) -> RecvIter<'a, T> {
	RecvIter {
		rx: consumer
	}
}
