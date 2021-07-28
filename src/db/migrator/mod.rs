cfg_if::cfg_if! {
    if #[cfg(feature = "rollup-state-db")] {
        mod rollup_state_migrator;
        pub use rollup_state_migrator::MIGRATOR;
    }
}
