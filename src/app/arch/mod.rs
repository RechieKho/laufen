use crate::adapter::shared;

pub mod consumer;
pub mod provider;
pub mod relay;

pub trait Pollable {
    fn poll(&mut self, p_delta: std::time::Duration) -> anyhow::Result<()>;
}

#[derive(getset::Getters)]
pub struct PollHandle<T: Pollable + 'static + Send> {
    #[getset(get = "pub")]
    shared_pollable: shared::Shared<T>,
    join_handle: tokio::task::JoinHandle<()>,
}

impl<T: Pollable + 'static + Send> PollHandle<T> {
    pub fn new(p_shared_pollable: shared::Shared<T>, p_delta: std::time::Duration) -> Self {
        let join_handle = {
            let shared_pollable = p_shared_pollable.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(p_delta);

                loop {
                    shared_pollable.lock().unwrap().poll(p_delta).unwrap();
                    interval.tick().await;
                }
            })
        };

        Self {
            shared_pollable: p_shared_pollable,
            join_handle,
        }
    }

    pub fn shut_down(&self) {
        self.join_handle.abort_handle().abort();
    }
}
