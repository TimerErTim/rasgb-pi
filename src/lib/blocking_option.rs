use std::sync::{Arc, Condvar, Mutex, WaitTimeoutResult};
use std::time::Duration;
use tokio::time::timeout;

#[derive(Clone)]
pub struct BlockingOption<T> {
    inner: Arc<(Mutex<Option<T>>, Condvar)>,
}

impl<T> BlockingOption<T> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new((Mutex::new(None), Condvar::new())),
        }
    }

    pub fn new_with(value: T) -> Self {
        let inner = Arc::new((Mutex::new(Some(value)), Condvar::new()));
        Self { inner }
    }

    /// Blocks until a value is available, then takes it.
    pub fn recv_timeout(&self, timeout: Duration) -> Option<T> {
        let (lock, cvar) = &*self.inner;
        let guard = lock.lock().unwrap();

        // Wait until `Some(value)` is set
        let (mut guard, timeout_result) = cvar.wait_timeout_while(
            guard, timeout, 
            |guard| guard.is_none()
        ).unwrap();

        if timeout_result.timed_out() {
            return None
        }
        
        // Extract and return the value, replacing it with None
        guard.take()
    }

    /// Sets a new value and notifies any waiting receiver.
    pub fn send(&self, value: T) {
        let (lock, cvar) = &*self.inner;
        let mut guard = lock.lock().unwrap();
        *guard = Some(value);
        cvar.notify_one(); // Wake up the receiver
    }
}