pub mod cli;
pub mod comms;
pub mod storage;

pub mod tracing {
    use tracing_subscriber::{EnvFilter, layer::SubscriberExt as _};

    /// Initializes the tracing subscriber with the given environment filter.
    pub fn init(env_filter: &str) {
        let env_filter = EnvFilter::from(env_filter);
        let subscriber = tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer());
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set global default subscriber");
    }
}
