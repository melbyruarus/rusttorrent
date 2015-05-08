use comm::spsc::{one_space,ring_buf};
use std::thread;

pub fn oneshot(ms: u32) -> one_space::Consumer<()> {
	let (tx, rx) = one_space::new();
    thread::spawn(move || {
        thread::sleep_ms(ms);
        tx.send(());
    });
    rx
}

pub fn repeating(ms: u32) -> ring_buf::Consumer<()> {
	let (tx, rx) = ring_buf::new(1);
    thread::spawn(move || {
    	loop {
	        thread::sleep_ms(ms);
	        tx.send(());
	    }
    });
    rx
}
