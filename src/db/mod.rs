cfg_if::cfg_if! {
    if #[cfg(any(feature = "rollup-state-db"))] {
        mod migrator;
        pub use migrator::MIGRATOR;
    }
}

pub type DbType = sqlx::Postgres;
pub type ConnectionType = sqlx::postgres::PgConnection;
pub type PoolOptions = sqlx::postgres::PgPoolOptions;
pub type DBErrType = sqlx::Error;
