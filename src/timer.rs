use std::sync::mpsc;
use std::thread;

pub fn oneshot(ms: u32) -> mpsc::Receiver<()> {
	let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        thread::sleep_ms(ms);
        tx.send(());
    });
    rx
}

pub fn repeating(ms: u32) -> mpsc::Receiver<()> {
	let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
    	loop {
	        thread::sleep_ms(ms);
	        tx.send(());
	    }
    });
    rx
}