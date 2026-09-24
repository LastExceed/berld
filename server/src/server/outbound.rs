use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use tokio::io::{AsyncWriteExt as _, BufWriter};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::{Notify, Semaphore};
use tokio::task::JoinHandle;
use tokio::time::timeout;

use protocol::WriteCwData;
use protocol::packet::FromServer;
use protocol::utils::io_extensions::WritePacket;

pub type Frame = Arc<[u8]>;

// queue budget to prevent unbounded allocation
const QUEUE_BUDGET: usize = 512 * 1024;
const MAX_AGE: Duration = Duration::from_secs(10);

const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

pub const FLUSH_GRACE: Duration = Duration::from_secs(3);

const WRITE_BUFFER: usize = 4 * 1024;

// exempt oversized frames (large models)
fn budgeted(frame: &[u8]) -> usize {
	if frame.len() > QUEUE_BUDGET { 0 } else { frame.len() }
}

#[derive(Debug, Default)]
struct Queue {
	frames: VecDeque<(Instant, Frame)>,
	bytes: usize,
	draining: bool,
	failed: bool
}

impl Queue {
	fn stale(&self) -> bool {
		self.frames.front().is_some_and(|(queued_at, _)| queued_at.elapsed() > MAX_AGE)
	}

	fn seal(&mut self) {
		self.draining = true;
		self.frames.clear();
		self.bytes = 0;
	}
}

#[derive(Debug)]
struct Shared {
	queue: Mutex<Queue>,
	wake: Notify,
	// semaphore (not Notify): notify_waiters only wakes already waiting tasks, so a late closed() could hang forever
	finished: Semaphore
}

impl Shared {
	fn lock(&self) -> MutexGuard<'_, Queue> {
		self.queue.lock().unwrap_or_else(PoisonError::into_inner)
	}
}

#[derive(Debug, Clone)]
pub struct Outbound {
	shared: Arc<Shared>
}

impl Outbound {
	pub fn new(write_half: OwnedWriteHalf) -> (Self, JoinHandle<()>) {
		let shared = Arc::new(Shared {
			queue: Mutex::new(Queue::default()),
			wake: Notify::new(),
			finished: Semaphore::new(0)
		});

		let handle = tokio::spawn(run_writer(Arc::clone(&shared), write_half));

		(Self { shared }, handle)
	}

	pub fn enqueue(&self, frame: Frame) -> bool {
		let accepted = {
			let mut queue = self.shared.lock();

			if queue.draining {
				return false;
			}

			if queue.stale() || queue.bytes + budgeted(&frame) > QUEUE_BUDGET {
				queue.failed = true;
				queue.seal();
				false
			} else {
				queue.bytes += budgeted(&frame);
				queue.frames.push_back((Instant::now(), frame));
				true
			}
		};

		self.shared.wake.notify_one();

		accepted
	}

	pub fn begin_shutdown(&self) {
		self.shared.lock().draining = true;
		self.shared.wake.notify_one();
	}

	pub async fn closed(&self) {
		#[expect(let_underscore_drop, clippy::let_underscore_must_use, reason = "only ever resolves as Err(closed)")]
		let _ = self.shared.finished.acquire().await;
	}
}

pub async fn serialize<P: FromServer>(packet: &P) -> Frame
	where Vec<u8>: WriteCwData<P>
{
	let mut buffer = vec![];
	buffer
		.write_packet(packet)
		.await
		.expect("failed to serialize a packet in-memory");

	buffer.into()
}

async fn run_writer(shared: Arc<Shared>, write_half: OwnedWriteHalf) {
	let mut socket = BufWriter::with_capacity(WRITE_BUFFER, write_half);

	let clean = loop {
		let next = {
			let mut queue = shared.lock();

			if queue.failed || queue.stale() {
				break false;
			}

			match queue.frames.pop_front() {
				Some((_queued_at, frame)) => {
					queue.bytes -= budgeted(&frame);
					Some(frame)
				}
				None if queue.draining => break true,
				None => None
			}
		};

		let idle = next.is_none();

		let write = match next {
			Some(frame) => timeout(WRITE_TIMEOUT, socket.write_all(&frame)).await,
			None => timeout(WRITE_TIMEOUT, socket.flush()).await
		};

		if matches!(write, Err(_) | Ok(Err(_))) {
			break false;
		}

		if idle {
			shared.wake.notified().await;
		}
	};

	// the order matters here; seal before the socket work so
	// enqueue stops accepting frames when nothing drains them
	shared.lock().seal();

	if clean {
		_ = timeout(FLUSH_GRACE, socket.shutdown()).await;
	}

	shared.finished.close();
}