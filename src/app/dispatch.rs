use std::convert::Into;
use futures::future::Future;
use futures::stream::StreamExt;
use futures::channel::mpsc::{Receiver, Sender, channel};

use super::AppAction;

pub type FutureAppAction = std::pin::Pin<Box<dyn Future<Output=AppAction> + Send>>;

#[derive(Clone)]
pub struct Dispatcher {
    sender: glib::Sender<AppAction>,
    future_sender: Sender<FutureAppAction>
}


impl Dispatcher {
    fn new(sender: glib::Sender<AppAction>, future_sender: Sender<FutureAppAction>) -> Self {
        Self { sender, future_sender }
    }

    pub fn dispatch<T: Into<Option<AppAction>>>(&self, action: T) -> Option<()> {
        if let Some(action) = action.into() {
            self.sender.send(action).ok()
        } else {
            None
        }
    }

    pub fn dispatch_async(&self, action: FutureAppAction) -> Option<()> {
        self.future_sender.clone().try_send(action).ok()
    }
}


pub struct DispatchLoop {
    sender: glib::Sender<AppAction>,
    future_receiver: Receiver<FutureAppAction>,
    future_sender: Sender<FutureAppAction>
}

impl DispatchLoop {

    pub fn wrap(sender: glib::Sender<AppAction>) -> Self {
        let (future_sender, future_receiver) = channel::<FutureAppAction>(0);
        Self { sender, future_receiver, future_sender }
    }

    pub fn make_dispatcher(&self) -> Dispatcher {
        Dispatcher::new(self.sender.clone(), self.future_sender.clone())
    }

    pub async fn future(self) {
        let sender = self.sender.clone();
        self.future_receiver.for_each_concurrent(2, move |action| {
            let sender = sender.clone();
            async move {
                let action = action.await;
                sender.send(action).expect("Error!");
            }
        }).await
    }
}

