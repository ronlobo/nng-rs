//! A simple PUB/SUB demonstration application.
//!
//! This application simply publishes current number of subscribers every few
//! seconds.
extern crate byteorder;
extern crate nng;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{env, mem, process, thread};

use byteorder::{ByteOrder, LittleEndian};
use nng::options::protocol::pubsub::Subscribe;
use nng::options::Options;
use nng::{Message, PipeEvent, Protocol, Socket};

/// Entry point of the application.
fn main() -> Result<(), nng::Error>
{
	// Begin by parsing the arguments to determine whether this is the
	// subscriber or the publisher and what URL to connect with.
	let args: Vec<_> = env::args().take(3).collect();

	match &args[..] {
		[_, t, url] if t == "publisher" => publisher(url),
		[_, t, url] if t == "subscriber" => subscriber(url),
		_ => {
			println!("Usage: pubsub publisher|subscriber <url>");
			process::exit(1);
		},
	}
}

/// Run the publisher portion of the program.
fn publisher(url: &str) -> Result<(), nng::Error>
{
	let mut s = Socket::new(Protocol::Pub0)?;
	let count = Arc::new(AtomicUsize::new(0));
	let count_clone = count.clone();

	s.pipe_notify(move |_, ev| {
		match ev {
			PipeEvent::AddPost => count_clone.fetch_add(1, Ordering::Relaxed),
			PipeEvent::RemovePost => count_clone.fetch_sub(1, Ordering::Relaxed),
			_ => 0,
		};
	})?;

	s.listen(url)?;

	loop {
		// Sleep for a little bit before sending the next message.
		thread::sleep(Duration::from_secs(3));

		// Load the number of subscribers and send the value across
		let data = count.load(Ordering::Relaxed) as u64;
		let mut msg = Message::zeros(mem::size_of::<u64>())?;
		LittleEndian::write_u64(&mut msg, data);

		println!("PUBLISHER: SENDING {}", data);
		s.send(msg)?;
	}
}

/// Run the subscriber portion of the program.
fn subscriber(url: &str) -> Result<(), nng::Error>
{
	let mut s = Socket::new(Protocol::Sub0)?;
	s.dial(url)?;

	println!("SUBSCRIBER: SUBSCRIBING TO ALL TOPICS");
	let all_topics = vec![];
	s.set_opt::<Subscribe>(all_topics)?;

	loop {
		let msg = s.recv()?;
		let subs = LittleEndian::read_u64(&msg);
		println!("SUBSCRIBER: THERE ARE {} SUBSCRIBERS", subs);
	}
}