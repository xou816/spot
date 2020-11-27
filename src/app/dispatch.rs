use std::pin::Pin;
use futures::future::Future;
use futures::stream::StreamExt;
use futures::channel::mpsc::{Receiver, Sender, channel};
use futures::future::LocalBoxFuture;


use super::AppAction;

pub trait ActionDispatcher {
    fn dispatch(&self, action: AppAction);
    fn dispatch_async(&self, action: LocalBoxFuture<'static, Option<AppAction>>);
    fn dispatch_many_async(&self, actions: LocalBoxFuture<'static, Vec<AppAction>>);
    fn box_clone(&self) -> Box<dyn ActionDispatcher>;
}

#[derive(Clone)]
pub struct ActionDispatcherImpl {
    sender: Sender<AppAction>,
    worker: Worker
}

impl ActionDispatcherImpl {
    pub fn new(sender: Sender<AppAction>, worker: Worker) -> Self {
        Self { sender, worker }
    }
}

impl ActionDispatcher for ActionDispatcherImpl {

    fn dispatch(&self, action: AppAction) {
        self.sender.clone().try_send(action).unwrap();
    }

    fn dispatch_async(&self, action: LocalBoxFuture<'static, Option<AppAction>>) {
        let mut clone = self.sender.clone();
        self.worker.send_task(async move {
            if let Some(action) = action.await {
                clone.try_send(action).unwrap();
            }
        });
    }

    fn dispatch_many_async(&self, actions: LocalBoxFuture<'static, Vec<AppAction>>) {
        let sender = self.sender.clone();
        self.worker.send_task(async move {
            for action in actions.await {
                sender.clone().try_send(action).unwrap();
            }
        });
    }

    fn box_clone(&self) -> Box<dyn ActionDispatcher> {
        Box::new(self.clone())
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

    pub fn make_dispatcher(&self) -> Sender<AppAction> {
        self.sender.clone()
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
        self.future_receiver.for_each(|task| task).await;
    }
}

#[derive(Clone)]
pub struct Worker(Sender<FutureLocalTask>);

impl Worker {

    pub fn send_task<T: Future<Output=()> + 'static>(&self, task: T) -> Option<()> {
        self.0.clone().try_send(Box::pin(task)).ok()
    }
}


