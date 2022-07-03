use tokio::sync::{oneshot, watch, mpsc};

pub struct Shutdown {
    pub trigger_send: oneshot::Sender<()>,
    pub trigger_recv: oneshot::Receiver<()>,
    pub signal_send: watch::Sender<bool>,
    pub signal_recv: watch::Receiver<bool>,
    pub ack_send: mpsc::UnboundedSender<()>,
    pub ack_recv: mpsc::UnboundedReceiver<()>,
}

impl Shutdown {

    pub fn new() -> Shutdown {
        let (trigger_send, trigger_recv) = oneshot::channel::<()>();
        let (signal_send, signal_recv) = watch::channel(false);
        let (ack_send, ack_recv) = mpsc::unbounded_channel::<()>();

        Shutdown {
            trigger_send,
            trigger_recv,
            signal_send,
            signal_recv,
            ack_send,
            ack_recv,
        }
    }

}
