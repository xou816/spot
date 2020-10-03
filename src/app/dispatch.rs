use std::convert::Into;
use std::pin::Pin;
use futures::future::Future;
use futures::stream::StreamExt;
use futures::channel::mpsc::{Receiver, Sender, channel};


use super::AppAction;

pub trait AbstractDispatcher<T> {
    fn dispatch(&self, action: T) -> Option<()>;
}

#[derive(Clone)]
pub struct Dispatcher {
    sender: Sender<AppAction>
}

impl Dispatcher {
    fn new(sender: Sender<AppAction>) -> Self {
        Self { sender }
    }

    pub fn dispatch<T: Into<Option<AppAction>>>(&self, action: T) -> Option<()> {
        if let Some(action) = action.into() {
            self.sender.clone().try_send(action).ok()
        } else {
            println!("No action");
            None
        }
    }
}


impl AbstractDispatcher<AppAction> for Dispatcher {

    fn dispatch(&self, action: AppAction) -> Option<()> {
        self.sender.clone().try_send(action).ok()
    }
}


pub struct DispatchLoop {
    receiver: Receiver<AppAction>,
    sender: Sender<AppAction>
}

impl DispatchLoop {

    pub fn new() -> Self {
        let (sender, receiver) = channel::<AppAction>(0);
        Self { receiver, sender }
    }

    pub fn make_dispatcher(&self) -> Dispatcher {
        Dispatcher::new(self.sender.clone())
    }

    pub async fn attach(self, handler: impl Fn(AppAction) -> ()) {
        self.receiver.for_each(|action| {
            async {
                handler(action);
            }
        }).await;
    }
}


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
        self.future_receiver.for_each_concurrent(1, |task| task).await;
    }
}

#[derive(Clone)]
pub struct Worker(Sender<FutureLocalTask>);

impl Worker {

    pub fn send_task<T: Future<Output=()> + 'static>(&self, task: T) -> Option<()> {
        self.0.clone().try_send(Box::pin(task)).ok()
    }
}


