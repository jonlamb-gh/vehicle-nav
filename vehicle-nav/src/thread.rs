use crossbeam::channel::{self, select, Receiver, RecvError, SendError, Sender};
use err_derive::Error;
use std::thread::{self, JoinHandle};
use std::{fmt, io};

#[derive(Debug, Error)]
pub enum SendRecvError {
    #[error(display = "Send channel disconnected")]
    SendChannelDisconnected,

    #[error(display = "Recv channel disconnected")]
    RecvChannelDisconnected,
}

impl From<RecvError> for SendRecvError {
    fn from(_e: RecvError) -> Self {
        SendRecvError::RecvChannelDisconnected
    }
}

impl<T> From<SendError<T>> for SendRecvError {
    fn from(_e: SendError<T>) -> Self {
        SendRecvError::SendChannelDisconnected
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ShutdownKind {
    Requested,
    ShutdownRequestChannelDisconnected,
    ChannelDisconnected,
}

impl ShutdownKind {
    fn reason(&self) -> &str {
        match self {
            ShutdownKind::Requested => "requested",
            ShutdownKind::ShutdownRequestChannelDisconnected => {
                "shutdown request channel disconnected"
            }
            ShutdownKind::ChannelDisconnected => "request channel disconnected",
        }
    }
}

impl fmt::Display for ShutdownKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = self.reason();
        f.write_str(reason)
    }
}

#[derive(Debug)]
pub struct ShutdownRequest {}

#[derive(Debug)]
pub struct ShutdownResponse {}

#[derive(Debug)]
pub struct ShutdownHandle {
    thread_name: String,
    req_sender: Sender<ShutdownRequest>,
    resp_recvr: Receiver<ShutdownResponse>,
    join_handle: JoinHandle<()>,
}

impl ShutdownHandle {
    pub fn new(
        thread_name: String,
        req_sender: Sender<ShutdownRequest>,
        resp_recvr: Receiver<ShutdownResponse>,
        join_handle: JoinHandle<()>,
    ) -> Self {
        ShutdownHandle {
            thread_name,
            req_sender,
            resp_recvr,
            join_handle,
        }
    }

    pub fn blocking_shutdown(self) -> Result<(), SendRecvError> {
        log::debug!("Requesting {} shutdown", self.thread_name);
        self.req_sender.send(ShutdownRequest {})?;
        let _ = self.resp_recvr.recv()?;
        if let Err(_e) = self.join_handle.join() {
            // TODO - should probably actually do something here
            log::error!("Thread {} failed to join", self.thread_name);
        }
        //let _ = self.resp_recvr.recv()?;
        Ok(())
    }

    fn spawn_with<F, G>(thread_name: String, f: F) -> Result<Self, io::Error>
    where
        F: FnOnce(String, Receiver<ShutdownRequest>, Sender<ShutdownResponse>) -> G,
        G: FnOnce(),
        G: Send + 'static,
    {
        // Non-blocking shutdown requests
        let (shutdown_req_sender, shutdown_req_recvr) = channel::bounded(1);

        // Blocking shutdown response
        let (shutdown_resp_sender, shutdown_resp_recvr) = channel::bounded(0);

        let join_handle = thread::Builder::new().name(thread_name.clone()).spawn(f(
            thread_name.clone(),
            shutdown_req_recvr,
            shutdown_resp_sender,
        ))?;
        Ok(ShutdownHandle::new(
            thread_name,
            shutdown_req_sender,
            shutdown_resp_recvr,
            join_handle,
        ))
    }
}

pub trait ShutdownHandlingThread {
    type Msg: Send + 'static;
    type ShutdownError: std::error::Error + Send + 'static;

    /// Stuff to do in the thread before the main loop
    fn pre_start(&mut self) -> Result<(), Self::ShutdownError> {
        Ok(())
    }

    /// Stuff to do in the thread before thread shutdown
    fn pre_shutdown(&mut self) {}

    /// Returning an error will shutdown the thread
    fn handle_requests(&mut self, requests: Vec<Self::Msg>) -> Result<(), Self::ShutdownError>;

    fn spawn(
        self,
        name: String,
        msg_receiver: Receiver<Self::Msg>,
    ) -> Result<ShutdownHandle, io::Error>
    where
        Self: Send + Sized + 'static,
    {
        ShutdownHandlingThread::spawn_with(name, msg_receiver, move || self)
    }

    fn spawn_with<F>(
        name: String,
        msg_receiver: Receiver<Self::Msg>,
        f: F,
    ) -> Result<ShutdownHandle, io::Error>
    where
        F: FnOnce() -> Self,
        F: Send + 'static,
        Self: Sized,
    {
        ShutdownHandle::spawn_with(
            name,
            |thread_name, shutdown_req_recvr, shutdown_resp_sender| {
                move || {
                    let context = ShutdownHandlingThreadContext {
                        thread_name,
                        msg_receiver,
                        shutdown_req_recvr,
                        shutdown_resp_sender,
                    };
                    let worker = f();
                    context.thread_loop(worker);
                }
            },
        )
    }
}

struct ShutdownHandlingThreadContext<Msg> {
    thread_name: String,
    msg_receiver: Receiver<Msg>,
    shutdown_req_recvr: Receiver<ShutdownRequest>,
    shutdown_resp_sender: Sender<ShutdownResponse>,
}

impl<Msg: Send + 'static> ShutdownHandlingThreadContext<Msg> {
    fn internal_self_shutdown(
        self,
        mut worker: impl ShutdownHandlingThread<Msg = Msg>,
        shutdown_reason: Result<ShutdownKind, String>,
    ) {
        let ShutdownHandlingThreadContext {
            thread_name,
            msg_receiver,
            shutdown_req_recvr,
            shutdown_resp_sender,
        } = self;
        match shutdown_reason {
            Ok(kind) => log::debug!("Shutting down thread {} because {}", thread_name, kind),
            Err(e) => log::error!("Shutting down thread {} because {}", thread_name, e),
        }
        std::mem::drop(msg_receiver);
        worker.pre_shutdown();
        if let Err(e) = shutdown_resp_sender.send(ShutdownResponse {}) {
            log::error!(
                "Thread {} failed to send shutdown response {}",
                thread_name,
                e
            );
        }
        std::mem::drop(shutdown_req_recvr);
    }

    fn thread_loop(self, mut worker: impl ShutdownHandlingThread<Msg = Msg>) {
        log::debug!("Starting thread {}", self.thread_name);

        if let Err(e) = worker.pre_start() {
            return self.internal_self_shutdown(worker, Err(e.to_string()));
        }

        loop {
            select! {
                recv(self.shutdown_req_recvr) -> msg => {
                    let shutdown_kind = if let Ok(_shutdown_request) = msg {
                          Ok(ShutdownKind::Requested)
                      } else {
                          Ok(ShutdownKind::ShutdownRequestChannelDisconnected)
                      };
                    return self.internal_self_shutdown(worker, shutdown_kind);
                }
                recv(self.msg_receiver) -> msg => {
                    if let Ok(msg) = msg {
                        let mut messages = vec![msg];
                        for extra_message in self.msg_receiver.try_iter() {
                            messages.push(extra_message);
                        }
                        if let Err(e) = worker.handle_requests(messages) {
                            return self.internal_self_shutdown(worker, Err(e.to_string()));
                        }
                    } else {
                        return self.internal_self_shutdown(worker, Ok(ShutdownKind::ChannelDisconnected));
                    }
                }
            }
        }
    }
}
