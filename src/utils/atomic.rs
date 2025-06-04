use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub(crate) struct TimeoutAtomicBool {
	value: Arc<AtomicBool>,
	cancel_token: CancellationToken,
	task: Option<JoinHandle<()>>,
	timeout_reset_signal: tokio::sync::mpsc::Sender<()>,
}

impl TimeoutAtomicBool {
	pub(crate) fn new() -> Self {
		let cancel_token = CancellationToken::new();
		let value = Arc::new(AtomicBool::new(false));

		let in_thread_token = cancel_token.clone();
		let in_thread_value = Arc::clone(&value);

		let (wx, mut rx) = tokio::sync::mpsc::channel::<()>(1);

		let task = tokio::spawn(async move {
			loop {
				tokio::select! {
					_ = in_thread_token.cancelled() => {
						break;
					}
					Some(_) = rx.recv() => {
						continue;
					}
					_ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {
						in_thread_value.store(false, Ordering::Relaxed);
					}
				}
			}
		});

		Self {
			cancel_token,
			value,
			task: Some(task),
			timeout_reset_signal: wx,
		}
	}

	pub(crate) async fn set(&self, value: bool) {
		self.value.store(value, Ordering::Relaxed);
		self.timeout_reset_signal.send(()).await.unwrap();
	}

	pub(crate) fn get(&self) -> bool {
		self.value.load(Ordering::Relaxed)
	}
}

impl Drop for TimeoutAtomicBool {
	fn drop(&mut self) {
		self.cancel_token.cancel();

		tokio::task::block_in_place(move || {
			tokio::runtime::Handle::current().block_on(async {
				self.task.take().unwrap().await.unwrap();
			});
		});
	}
}
