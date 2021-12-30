use std::{
    sync::mpsc::{sync_channel, Receiver},
    thread::JoinHandle,
};

pub enum JobPollError {
    Finished,
}

impl std::fmt::Display for JobPollError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Finished => write!(f, "Can't poll a finished job."),
        }
    }
}

impl std::fmt::Debug for JobPollError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <JobPollError as std::fmt::Display>::fmt(&self, f)
    }
}
impl std::error::Error for JobPollError {}

pub struct JobHandle<T> {
    handle: Option<JoinHandle<()>>,
    receiver: Receiver<T>,
}

impl<T> JobHandle<T> {
    pub fn poll(&mut self) -> Result<Option<T>, JobPollError> {
        if self.handle.is_none() {
            return Err(JobPollError::Finished);
        }

        match self.receiver.try_recv() {
            Ok(v) => {
                self.handle.take().unwrap().join().unwrap();
                Ok(Some(v))
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => Ok(None),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => panic!("Job disconnected!"),
        }
    }

    pub fn finished(&self) -> bool {
        self.handle.is_none()
    }
}

pub fn spawn_job<T, FThread>(thread: FThread) -> JobHandle<T>
where
    FThread: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = sync_channel::<T>(0);
    let handle = std::thread::spawn(move || {
        let _ = tx.send(thread());
    });

    JobHandle {
        handle: Some(handle),
        receiver: rx,
    }
}
