use std::convert::Into;
use futures::future::Future;
use futures::stream::StreamExt;
use futures::channel::mpsc::{Receiver, Sender, channel};

use super::AppAction;

pub type FutureAppAction = std::pin::Pin<Box<dyn Future<Output=AppAction> + Send>>;

#[derive(Clone)]
pub struct Dispatcher {
    future_sender: Sender<FutureAppAction>
}


impl Dispatcher {
    fn new(future_sender: Sender<FutureAppAction>) -> Self {
        Self { future_sender }
    }

    pub fn dispatch<T: Into<Option<AppAction>>>(&self, action: T) -> Option<()> {
        if let Some(action) = action.into() {
            self.dispatch_async(Box::pin(async {
                action
            }))
        } else {
            println!("No action");
            None
        }
    }

    pub fn dispatch_async(&self, action: FutureAppAction) -> Option<()> {
        self.future_sender.clone().try_send(action).ok()
    }
}


pub struct DispatchLoop {
    future_receiver: Receiver<FutureAppAction>,
    future_sender: Sender<FutureAppAction>
}

impl DispatchLoop {

    pub fn new() -> Self {
        let (future_sender, future_receiver) = channel::<FutureAppAction>(0);
        Self { future_receiver, future_sender }
    }

    pub fn make_dispatcher(&self) -> Dispatcher {
        Dispatcher::new(self.future_sender.clone())
    }

    pub async fn attach(self, handler: impl Fn(AppAction) -> ()) {
        self.future_receiver.for_each_concurrent(2, |action| {
            async {
                handler(action.await);
            }
        }).await;
    }
}

