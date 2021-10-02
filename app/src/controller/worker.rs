use crate::config::Config;
use crate::controller::apply::{apply, ApplyResult};
use chrono::Utc;
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError, SyncSender};
use std::time::Duration;

pub struct Worker<T> {
    enabled: bool,
    config: Config,
    receiver: Receiver<Message>,
    on_apply: T,
}

pub enum Message {
    UpdateConfig(Config),
    UpdateEnabled(bool),
    Terminate,
    ForceRefresh,
}

enum LoopAction {
    Continue,
    Break,
}

impl<T> Worker<T>
where
    T: Fn(ApplyResult) + 'static + Send,
{
    pub fn start(config: Config, enabled: bool, on_apply: T) -> SyncSender<Message> {
        let (sender, receiver) = sync_channel(0);
        std::thread::spawn(move || {
            let mut worker = Worker {
                enabled,
                config,
                receiver,
                on_apply,
            };
            loop {
                match worker.tick() {
                    LoopAction::Break => break,
                    LoopAction::Continue => continue,
                }
            }
        });
        sender
    }

    fn tick(&mut self) -> LoopAction {
        let (res, next_run) = apply(&self.config, self.enabled);
        (self.on_apply)(res);

        let wait = next_run.map(|next_run| {
            // Wait for the next run, or a notification
            let unix_time_now = Utc::now().timestamp();
            if next_run > unix_time_now {
                next_run - unix_time_now
            } else {
                0
            }
        });

        let rx_wait = match wait {
            None => {
                log::info!("Brightness Worker sleeping indefinitely");
                self.receiver.recv().map_err(|e| e.into())
            }
            Some(wait) => {
                log::info!("Brightness Worker sleeping for {}s", wait);
                self.receiver.recv_timeout(Duration::from_secs(wait as u64))
            }
        };

        match rx_wait {
            Ok(msg) => match msg {
                Message::Terminate => return LoopAction::Break,
                Message::UpdateConfig(config) => {
                    self.config = config;
                }
                Message::UpdateEnabled(enabled) => self.enabled = enabled,
                Message::ForceRefresh => {}
            },
            Err(e) => {
                if e != RecvTimeoutError::Timeout {
                    panic!("{}", e)
                }
            }
        };

        LoopAction::Continue
    }
}
