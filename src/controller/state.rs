use crate::config::Config;
use std::sync::{Arc, Weak};
use std::time::SystemTime;

#[derive(Clone)]
pub struct LastResult {
    pub brightness: u32,
    pub expiry: SystemTime,
    pub time: SystemTime,
    pub sunrise: SystemTime,
    pub sunset: SystemTime,
    pub visible: bool,
}

pub struct State {
    last_result: Option<LastResult>,
    enabled: bool,
    config: Config,
    observers: Vec<Weak<dyn Observer + Send + Sync>>,
}

pub trait Observer {
    fn did_set_enabled(&self, running: bool);
    fn did_set_last_result(&self, last_calculation: &LastResult);
    fn did_set_config(&self, config: &Config);
}

impl State {
    pub fn new(config: Config) -> Self {
        Self {
            last_result: None,
            enabled: true,
            config,
            observers: vec![],
        }
    }

    pub fn get_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn get_last_result(&self) -> &Option<LastResult> {
        &self.last_result
    }

    pub fn register(&mut self, o: Weak<dyn Observer + Send + Sync>) {
        self.clean_observers();
        self.observers.push(o);
    }

    #[allow(unused)]
    pub fn unregister(&mut self, o: Weak<dyn Observer + Send + Sync>) {
        let observers = std::mem::take(&mut self.observers);
        self.observers = observers
            .into_iter()
            .filter(|o2| !o2.ptr_eq(&o) && o2.upgrade().is_some())
            .collect()
    }

    /// Enable or disable solar screen brightness, returns the previous value
    pub fn set_enabled(&mut self, enabled: bool) -> bool {
        let before = self.enabled;
        self.enabled = enabled;
        self.clean_observers();
        self.notify_observers(|o| o.did_set_enabled(enabled));
        before
    }

    /// Update the solar screen brightness config, returns the previous config
    pub fn set_config(&mut self, config: Config) -> Config {
        let before = std::mem::replace(&mut self.config, config);
        self.clean_observers();
        self.notify_observers(|o| o.did_set_config(&self.config));
        before
    }

    pub(super) fn set_last_result(&mut self, last_result: LastResult) {
        self.last_result = Some(last_result);
        self.clean_observers();
        self.notify_observers(|o| o.did_set_last_result(self.last_result.as_ref().unwrap()));
    }

    fn clean_observers(&mut self) {
        let observers = std::mem::take(&mut self.observers);
        self.observers = observers
            .into_iter()
            .filter(|p| p.upgrade().is_some())
            .collect();
    }

    fn notify_observers<F>(&self, f: F)
    where
        F: Fn(Arc<dyn Observer + Send + Sync>),
    {
        self.observers.iter().for_each(|p| match p.upgrade() {
            None => {}
            Some(p) => {
                f(p);
            }
        })
    }
}
