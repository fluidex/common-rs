cfg_if::cfg_if! {
    if #[cfg(feature = "rollup-state-db")] {
        mod rollup_state_migrator;

        pub use rollup_state_migrator::MIGRATOR;
        pub use rollup_state_migrator::DbType;
        pub use rollup_state_migrator::ConnectionType;
        pub use rollup_state_migrator::PoolOptions;
        pub use rollup_state_migrator::DBErrType;
    }
}
