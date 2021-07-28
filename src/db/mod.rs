cfg_if::cfg_if! {
    if #[cfg(any(feature = "rollup-state-db"))] {
        mod migrator;

        pub use migrator::MIGRATOR;
        pub use migrator::DbType;
        pub use migrator::ConnectionType;
        pub use migrator::PoolOptions;
        pub use migrator::DBErrType;
    }
}
