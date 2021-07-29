cfg_if::cfg_if! {
    if #[cfg(feature = "db")] {
        pub type ConnectionType = sqlx::postgres::PgConnection;
        pub type DBErrType = sqlx::Error;
        pub type DbType = sqlx::Postgres;
        pub type PoolOptions = sqlx::postgres::PgPoolOptions;
        pub type TimestampDbType = chrono::NaiveDateTime;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(feature = "rollup-state-db"))] {
        mod migrator;
        pub use migrator::MIGRATOR;
    }
}

pub mod models;
