use std::{net::IpAddr, sync::Arc};

use governor::DefaultKeyedRateLimiter;
use sqlx::PgPool;
use crate::config::Config;

pub type IpRateLimiter = DefaultKeyedRateLimiter<IpAddr>;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub rate_limiter: Arc<IpRateLimiter>,
}