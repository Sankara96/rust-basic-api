use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::{Config, NoTls};

#[derive(Clone)]
pub struct DbPool(pub Pool);

impl DbPool {
    pub fn from_env() -> anyhow::Result<Self> {
        let mut cfg = Config::new();
        use std::env;
        cfg.host(env::var("PGHOST").as_deref().unwrap_or("localhost"));
        cfg.user(&env::var("PGUSER")?);
        cfg.password(&env::var("PGPASSWORD")?);
        cfg.dbname(&env::var("PGDATABASE")?);
        cfg.port(env::var("PGPORT").ok().and_then(|p| p.parse().ok()).unwrap_or(5432));
        // sslmode is handled outside; using NoTls here for simplicity.

        let mgr = Manager::from_config(cfg, NoTls, ManagerConfig {
            recycling_method: RecyclingMethod::Fast
        });
        let pool = Pool::builder(mgr)
            .max_size(8)
            .build()
            .unwrap();
        Ok(DbPool(pool))
    }
}
