use tokio::sync::{
    Mutex, broadcast,
    mpsc::{self, UnboundedSender},
};

pub struct WebSocketMessageBus {
    sender: broadcast::Sender<String>,
    receiver: Mutex<mpsc::UnboundedReceiver<String>>,
}

pub struct WebSocketSessionManager {
    sender: mpsc::UnboundedSender<String>,
    receiver: Mutex<broadcast::Receiver<String>>,
}

impl WebSocketMessageBus {
    pub fn new(
        sender: broadcast::Sender<String>,
        receiver: mpsc::UnboundedReceiver<String>,
    ) -> Self {
        Self {
            sender: sender,
            receiver: Mutex::new(receiver),
        }
    }

    pub async fn recv(&self) -> Option<String> {
        let mut rx = { self.receiver.lock().await };
        return rx.recv().await;
    }

    pub fn send(&self, msg: impl Into<String>) {
        self.sender.send(msg.into()).unwrap();
    }

    pub fn get_receiver(&self) -> broadcast::Receiver<String> {
        self.sender.subscribe()
    }
}

impl WebSocketSessionManager {
    pub fn new(sender: UnboundedSender<String>, receiver: broadcast::Receiver<String>) -> Self {
        Self {
            sender,
            receiver: Mutex::new(receiver),
        }
    }

    pub fn send(&self, msg: String) {
        self.sender.send(msg).unwrap();
    }

    pub async fn recv(&self) -> Result<String, broadcast::error::RecvError> {
        let mut rx = { self.receiver.lock().await };
        rx.recv().await
    }

    pub fn get_sender(&self) -> UnboundedSender<String> {
        self.sender.clone()
    }
}
