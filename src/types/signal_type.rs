use tokio::sync::{
    Mutex, broadcast,
    mpsc::{self, UnboundedSender},
};

pub struct SignalHub<T> {
    tx: broadcast::Sender<T>,
    rx: Mutex<mpsc::UnboundedReceiver<T>>,
    internal_tx: mpsc::UnboundedSender<T>,
}

pub struct SignalPort<T> {
    tx: mpsc::UnboundedSender<T>,
    rx: Mutex<broadcast::Receiver<T>>,
    internal_tx: broadcast::Sender<T>,
}

impl<T> SignalHub<T>
where
    T: Clone + Send + 'static,
{
    pub fn new() -> Self {
        let tx = broadcast::channel::<T>(256);
        let (in_tx, rx) = mpsc::unbounded_channel();
        Self {
            tx: tx.0,
            rx: Mutex::new(rx),
            internal_tx: in_tx,
        }
    }

    pub fn get_port(&self) -> SignalPort<T> {
        let tx = self.internal_tx.clone();
        let rx = self.tx.subscribe();
        let internal_tx = self.tx.clone();
        SignalPort::new(tx, rx, internal_tx)
    }

    pub fn send(&self, data: impl Into<T>) -> Result<usize, broadcast::error::SendError<T>> {
        self.tx.send(data.into())
    }

    pub async fn recv(&self) -> Option<T> {
        self.rx.lock().await.recv().await
    }
}

impl<T> Default for SignalHub<T>
where
    T: Clone + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SignalPort<T>
where
    T: Clone + Send + 'static,
{
    fn new(
        tx: UnboundedSender<T>,
        rx: broadcast::Receiver<T>,
        internal_tx: broadcast::Sender<T>,
    ) -> Self {
        Self {
            tx,
            rx: Mutex::new(rx),
            internal_tx,
        }
    }

    pub fn send(&self, data: T) -> Result<(), mpsc::error::SendError<T>> {
        self.tx.send(data)
    }

    pub async fn recv(&self) -> Result<T, broadcast::error::RecvError> {
        self.rx.lock().await.recv().await
    }
}

impl<T> Clone for SignalPort<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            rx: Mutex::new(self.internal_tx.subscribe()),
            internal_tx: self.internal_tx.clone(),
        }
    }
}
