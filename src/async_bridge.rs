use crate::ast::Node;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

/// A task sent from the main WGPU thread to the background worker.
pub struct FetchTask {
    pub method: String,
    pub url: String,
    pub callback_node: Box<Node>,
}

/// The result returned from the background worker to the main WGPU thread.
pub struct FetchPayload {
    pub payload: Result<String, String>, // Ok(JSON String) or Err(Error Message)
    pub callback_node: Box<Node>,
}

/// The AsyncBridge handles non-blocking I/O operations by offloading
/// blocking network requests (via `ureq`) to a dedicated background thread.
/// It uses a standard MPSC channel to loop payloads back for the next frame.
pub struct AsyncBridge {
    tx_task: Sender<FetchTask>,
    rx_payload: Receiver<FetchPayload>,
}

impl AsyncBridge {
    pub fn new() -> Self {
        let (tx_task, rx_task) = channel::<FetchTask>();
        let (tx_payload, rx_payload) = channel::<FetchPayload>();

        // Spawn the dedicated background worker thread
        thread::spawn(move || {
            // Processing loop: wait for tasks from the main thread
            while let Ok(task) = rx_task.recv() {
                let payload = match task.method.to_uppercase().as_str() {
                    "GET" => match ureq::get(&task.url).call() {
                        Ok(response) => match response.into_string() {
                            Ok(body) => Ok(body),
                            Err(e) => Err(format!("Failed to read GET response: {}", e)),
                        },
                        Err(ureq::Error::Status(code, response)) => Err(format!(
                            "HTTP {} Error: {}",
                            code,
                            response.into_string().unwrap_or_default()
                        )),
                        Err(e) => Err(format!("GET Request failed: {}", e)),
                    },
                    "POST" => match ureq::post(&task.url).call() {
                        Ok(response) => match response.into_string() {
                            Ok(body) => Ok(body),
                            Err(e) => Err(format!("Failed to read POST response: {}", e)),
                        },
                        Err(ureq::Error::Status(code, response)) => Err(format!(
                            "HTTP {} Error: {}",
                            code,
                            response.into_string().unwrap_or_default()
                        )),
                        Err(e) => Err(format!("POST Request failed: {}", e)),
                    },
                    _ => Err(format!("Unsupported HTTP method: {}", task.method)),
                };

                // Send the payload back to the main thread's Receiver
                let _ = tx_payload.send(FetchPayload {
                    payload,
                    callback_node: task.callback_node,
                });
            }
        });

        AsyncBridge {
            tx_task,
            rx_payload,
        }
    }

    /// Dispatch a request to the background thread without blocking.
    pub fn dispatch_fetch(&self, method: String, url: String, callback_node: Box<Node>) {
        let _ = self.tx_task.send(FetchTask {
            method,
            url,
            callback_node,
        });
    }

    /// Poll for resolved payloads. Returns `Some(FetchPayload)` if a request
    /// has finished since the last poll, or `None` if the queue is empty.
    pub fn try_recv(&self) -> Option<FetchPayload> {
        self.rx_payload.try_recv().ok()
    }
}
