use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::future::Future;
use futures::future::{BoxFuture, LocalBoxFuture};
use futures::stream::StreamExt;
use std::pin::Pin;

use super::AppAction;

pub trait ActionDispatcher {
    fn dispatch(&self, action: AppAction);
    fn dispatch_local_async(&self, action: LocalBoxFuture<'static, Option<AppAction>>);
    fn dispatch_async(&self, action: BoxFuture<'static, Option<AppAction>>);
    fn dispatch_many_async(&self, actions: BoxFuture<'static, Vec<AppAction>>);
    fn box_clone(&self) -> Box<dyn ActionDispatcher>;
}

#[derive(Clone)]
pub struct ActionDispatcherImpl {
    sender: UnboundedSender<AppAction>,
    worker: Worker,
}

impl ActionDispatcherImpl {
    pub fn new(sender: UnboundedSender<AppAction>, worker: Worker) -> Self {
        Self { sender, worker }
    }
}

impl ActionDispatcher for ActionDispatcherImpl {
    fn dispatch(&self, action: AppAction) {
        self.sender.unbounded_send(action).unwrap();
    }

    fn dispatch_local_async(&self, action: LocalBoxFuture<'static, Option<AppAction>>) {
        let clone = self.sender.clone();
        self.worker.send_local_task(async move {
            if let Some(action) = action.await {
                clone.unbounded_send(action).unwrap();
            }
        });
    }

    fn dispatch_async(&self, action: BoxFuture<'static, Option<AppAction>>) {
        let clone = self.sender.clone();
        self.worker.send_task(async move {
            if let Some(action) = action.await {
                clone.unbounded_send(action).unwrap();
            }
        });
    }

    fn dispatch_many_async(&self, actions: BoxFuture<'static, Vec<AppAction>>) {
        let clone = self.sender.clone();
        self.worker.send_task(async move {
            for action in actions.await.into_iter() {
                clone.unbounded_send(action).unwrap();
            }
        });
    }

    fn box_clone(&self) -> Box<dyn ActionDispatcher> {
        Box::new(self.clone())
    }
}

pub struct DispatchLoop {
    receiver: UnboundedReceiver<AppAction>,
    sender: UnboundedSender<AppAction>,
}

impl DispatchLoop {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded::<AppAction>();
        Self { receiver, sender }
    }

    pub fn make_dispatcher(&self) -> UnboundedSender<AppAction> {
        self.sender.clone()
    }

    pub async fn attach(self, mut handler: impl FnMut(AppAction)) {
        self.receiver
            .for_each(|action| {
                handler(action);
                async move {}
            })
            .await;
    }
}

pub type FutureTask = Pin<Box<dyn Future<Output = ()> + Send>>;
pub type FutureLocalTask = Pin<Box<dyn Future<Output = ()>>>;

pub fn spawn_task_handler(context: &glib::MainContext) -> Worker {
    let (future_local_sender, future_local_receiver) = unbounded::<FutureLocalTask>();
    context.spawn_local_with_priority(
        glib::source::PRIORITY_DEFAULT_IDLE,
        future_local_receiver.for_each(|t| t),
    );

    let (future_sender, future_receiver) = unbounded::<FutureTask>();
    context.spawn_with_priority(
        glib::source::PRIORITY_DEFAULT_IDLE,
        future_receiver.for_each(|t| t),
    );

    Worker(future_local_sender, future_sender)
}

#[derive(Clone)]
pub struct Worker(
    UnboundedSender<FutureLocalTask>,
    UnboundedSender<FutureTask>,
);

impl Worker {
    pub fn send_local_task<T: Future<Output = ()> + 'static>(&self, task: T) -> Option<()> {
        self.0.unbounded_send(Box::pin(task)).ok()
    }

    pub fn send_task<T: Future<Output = ()> + Send + 'static>(&self, task: T) -> Option<()> {
        self.1.unbounded_send(Box::pin(task)).ok()
    }
}
