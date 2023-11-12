use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // Sets info level as defauld if level not specified by RUST_LOG environment variable
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    // Sends formatted JSON spans to sink
    // in out case to: stdout
    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    // Creates processing pipeline for logs from layers
    Registry::default()
        .with(env_filter)
        // Stores spans for further layers
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Collects all logs from actix_web
    LogTracer::init().expect("Failed to set logger.");

    // Register default subscriber to process logs
    set_global_default(subscriber).expect("Failed to set subscriber");
}
