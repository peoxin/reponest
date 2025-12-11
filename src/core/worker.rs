//! This module provides a reusable worker pattern for processing tasks
//! in parallel with automatic shutdown when complete.
//!
//! Example:
//!
//! ```ignore
//! use std::sync::Arc;
//!
//! // Create a worker with a custom processor function
//! let worker = Arc::new(Worker::new(|number: i32| {
//!     Ok(number * 2)
//! }));
//!
//! // Submit tasks
//! for i in 0..10 {
//!     worker.submit(i).unwrap();
//! }
//! worker.finish_submitting();
//!
//! // Poll for results
//! loop {
//!     let results = worker.poll_results();
//!     for result in results {
//!         println!("{:?}", result);
//!     }
//!     if worker.is_complete() {
//!         break;
//!     }
//! }
//! ```

use crossbeam_channel::{Receiver, Sender, unbounded};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

/// Generic background worker for parallel task processing
///
/// Type Parameters:
/// - `I`: Input type (what you send to the worker)
/// - `O`: Output type (what the worker produces)
pub struct Worker<I, O>
where
    I: Send + 'static,
    O: Send + 'static,
{
    result_rx: Receiver<Result<O, String>>,
    task_tx: Sender<I>,
    pending_tasks: Arc<AtomicUsize>,
    completed_tasks: Arc<AtomicUsize>,
    shutdown: Arc<AtomicBool>,
    submitting_finished: Arc<AtomicBool>,
}

impl<I, O> Worker<I, O>
where
    I: Send + 'static,
    O: Send + 'static,
{
    /// Create a new worker with a custom processor function
    pub fn new<F>(processor: F) -> Self
    where
        F: Fn(I) -> Result<O, String> + Send + Sync + 'static,
    {
        let (task_tx, task_rx) = unbounded::<I>();
        let (result_tx, result_rx) = unbounded::<Result<O, String>>();
        let shutdown = Arc::new(AtomicBool::new(false));
        let pending_tasks = Arc::new(AtomicUsize::new(0));
        let completed_tasks = Arc::new(AtomicUsize::new(0));
        let submitting_finished = Arc::new(AtomicBool::new(false));

        let shutdown_clone = shutdown.clone();
        let pending_clone = pending_tasks.clone();
        let completed_clone = completed_tasks.clone();
        let submitting_clone = submitting_finished.clone();
        let processor = Arc::new(processor);

        // Background dispatcher thread
        std::thread::spawn(move || {
            while !shutdown_clone.load(Ordering::Relaxed) {
                match task_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(input) => {
                        let tx = result_tx.clone();
                        let completed = completed_clone.clone();
                        let processor = processor.clone();

                        // Spawn parallel task using rayon
                        rayon::spawn(move || {
                            let result = processor(input);
                            let _ = tx.send(result);
                            completed.fetch_add(1, Ordering::Relaxed);
                        });
                    }
                    Err(_) => {
                        // Check if all tasks are done
                        if submitting_clone.load(Ordering::Relaxed) {
                            let pending = pending_clone.load(Ordering::Relaxed);
                            let completed = completed_clone.load(Ordering::Relaxed);
                            if pending > 0 && pending == completed {
                                // All tasks completed, shutdown
                                shutdown_clone.store(true, Ordering::Relaxed);
                                break;
                            }
                        }
                        continue;
                    }
                }
            }
        });

        Self {
            result_rx,
            task_tx,
            pending_tasks,
            completed_tasks,
            shutdown,
            submitting_finished,
        }
    }

    /// Submit a task for background processing
    ///
    /// Should only be called before `finish_submitting()`. Returns an error
    /// if called after `finish_submitting()` or if the channel is disconnected.
    pub fn submit(&self, input: I) -> Result<(), String> {
        if self.submitting_finished.load(Ordering::Relaxed) {
            return Err("Cannot submit after finish_submitting() was called".to_string());
        }

        self.task_tx
            .send(input)
            .map_err(|e| format!("Failed to submit task: {}", e))?;

        self.pending_tasks.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Mark that all tasks have been submitted
    ///
    /// After calling this, the worker will auto-shutdown when all tasks complete.
    /// No more tasks can be submitted after this call.
    pub fn finish_submitting(&self) {
        self.submitting_finished.store(true, Ordering::Relaxed);
    }

    /// Poll for completed results (non-blocking)
    ///
    /// Returns all currently available results without waiting.
    /// This can be called multiple times to collect results as they arrive.
    pub fn poll_results(&self) -> Vec<Result<O, String>> {
        let mut results = Vec::new();
        while let Ok(result) = self.result_rx.try_recv() {
            results.push(result);
        }
        results
    }

    /// Check if all tasks are complete
    ///
    /// Returns true when all submitted tasks have been processed.
    /// This will only return true after `finish_submitting()` has been called
    /// and all pending tasks have completed.
    pub fn is_complete(&self) -> bool {
        if !self.submitting_finished.load(Ordering::Relaxed) {
            return false;
        }
        let pending = self.pending_tasks.load(Ordering::Relaxed);
        let completed = self.completed_tasks.load(Ordering::Relaxed);
        pending > 0 && pending == completed
    }

    /// Gracefully shutdown the worker
    ///
    /// Signals the background thread to stop processing new tasks.
    /// Automatically called when the worker is dropped.
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

impl<I, O> Drop for Worker<I, O>
where
    I: Send + 'static,
    O: Send + 'static,
{
    fn drop(&mut self) {
        self.shutdown();
    }
}
