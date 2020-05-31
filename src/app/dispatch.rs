use std::convert::Into;
use std::pin::Pin;
use futures::future::Future;
use futures::stream::StreamExt;
use futures::channel::mpsc::{Receiver, Sender, channel};


use super::AppAction;

pub type FutureAppAction = Pin<Box<dyn Future<Output=AppAction> + Send>>;

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

// need to generify the above, but it is hard :(

pub type FutureLocalTask = Pin<Box<dyn Future<Output=()>>>;

pub struct LocalTaskLoop {
    future_receiver: Receiver<FutureLocalTask>,
    future_sender: Sender<FutureLocalTask>
}

impl LocalTaskLoop {

    pub fn new() -> Self {
        let (future_sender, future_receiver) = channel::<FutureLocalTask>(0);
        Self { future_receiver, future_sender }
    }

    pub fn make_worker(&self) -> Worker {
        Worker(self.future_sender.clone())
    }

    pub async fn attach(self) {
        self.future_receiver.for_each_concurrent(2, |task| task).await;
    }
}

#[derive(Clone)]
pub struct Worker(Sender<FutureLocalTask>);

impl Worker {

    pub fn send_task<T: Future<Output=()> + 'static>(&self, task: T) -> Option<()> {
        self.0.clone().try_send(Box::pin(task)).ok()
    }
}


