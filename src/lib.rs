pub mod block;
pub mod cli;
pub mod comms;
pub mod network;
pub mod storage;
pub mod utils;

pub mod tracing {
    use tracing_subscriber::{
        EnvFilter,
        fmt::{self, format::FmtSpan},
        layer::SubscriberExt as _,
    };

    /// Initializes the tracing subscriber with the given environment filter.
    pub fn init(env_filter: &str) {
        // Formats logs with for stdout
        let fmt_layer = fmt::layer()
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .pretty();

        let env_filter = EnvFilter::from(env_filter);
        let subscriber = tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer);

        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set global default subscriber");
    }
}
