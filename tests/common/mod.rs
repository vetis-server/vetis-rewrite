#![allow(dead_code)]
use serde::Deserialize;
use vetis_tokio::ServerConfig;

pub(crate) const CA_CERT: &str = "certs/ca.der";

#[allow(unused)]
#[derive(Deserialize, Clone)]
pub struct VetisConfig {
    worker_threads: usize,
    max_blocking_threads: usize,
    server: ServerConfig,
}

impl VetisConfig {
    pub(crate) fn worker_threads(&self) -> usize {
        self.worker_threads
    }

    pub(crate) fn max_blocking_threads(&self) -> usize {
        self.max_blocking_threads
    }

    pub(crate) fn server(&self) -> &ServerConfig {
        &self.server
    }
}
