use std::{sync::{Arc, Mutex}, time::Duration};

use crossbeam_channel::{RecvTimeoutError, SendError, TrySendError};

pub fn bounded<T>() -> (Sender<T>, Receiver<T>) where T: Clone {
    let (tx, rx) = crossbeam_channel::bounded(1);
    Data::from_channel(tx, rx)
}

/// Multi-receiver broadcast for messages.
/// Messages are cloned and sent to each receiver.
struct Data<T> {
    channels: Vec<crossbeam_channel::Sender<T>>,
}

impl<T> Data<T> {
    fn from_channel(tx: crossbeam_channel::Sender<T>, rx: crossbeam_channel::Receiver<T>) -> (Sender<T>, Receiver<T>) where T: Clone {
        let data = Arc::new(Mutex::new(Data { channels: vec![tx] }));
        let sender = Sender {
            data: Arc::clone(&data),
        };
        let receiver = Receiver {
            data,
            channel: rx,
        };
        (sender, receiver)
    }
}

pub struct Sender<T> {
    data: Arc<Mutex<Data<T>>>,
}

impl<T> Sender<T>
where
    T: Clone,
{
    pub fn send(&self, msg: T) -> Result<(), SendError<T>> {
        let mut data = self.data.lock().unwrap();

        data.channels
            .retain(|channel| match channel.try_send(msg.clone()) {
                Ok(()) => true,
                Err(TrySendError::Full(_)) => true,
                Err(TrySendError::Disconnected(_)) => false,
            });

        Ok(())
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}

pub struct Receiver<T> {
    data: Arc<Mutex<Data<T>>>,
    channel: crossbeam_channel::Receiver<T>,
}

impl<T> Receiver<T>
{
    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
        self.channel.recv_timeout(timeout)
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        let (tx, rx) = crossbeam_channel::bounded(1);
        self.data.lock().unwrap().channels.push(tx);
        Self {
            data: Arc::clone(&self.data),
            channel: rx,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_single_receiver() {
        let (sender, receiver) = bounded();
        sender.send("test1").unwrap();
        sender.send("test2").unwrap();
        assert_eq!(receiver.recv_timeout(Duration::from_secs(0)), Ok("test1"));
    }

    #[test]
    fn step_multiple_receivers() {
        let (sender, receiver1) = bounded();
        let receiver2 = receiver1.clone();
        sender.send("test1").unwrap();
        sender.send("test2").unwrap();
        assert_eq!(receiver1.recv_timeout(Duration::from_secs(0)), Ok("test1"));
        assert_eq!(receiver2.recv_timeout(Duration::from_secs(0)), Ok("test1"));
    }
}
