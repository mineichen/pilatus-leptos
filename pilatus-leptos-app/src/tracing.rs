use tracing_log::log::{self, Level, Log, Metadata, Record};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod browser_console;

pub fn init_logging() {
    console_error_panic_hook::set_once();

    // tracing_wasm::set_as_global_default();
    tracing_subscriber::registry()
        .with(browser_console::BrowserConsoleLayer)
        .init();
    let tracer = tracing_log::LogTracer::new();
    if let Err(e) = log::set_boxed_logger(Box::new(ErrorOnlyLog(tracer))) {
        leptos::logging::error!("Error setting up logger bridge: {e:?}")
    }
    log::set_max_level(log::LevelFilter::Trace);
}

struct ErrorOnlyLog(tracing_log::LogTracer);

impl Log for ErrorOnlyLog {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        let is_wgpu = || {
            let target = metadata.target();
            target.starts_with("wgpu") || target.starts_with("naga")
        };
        if metadata.level() >= Level::Debug && is_wgpu() {
            return false;
        }
        self.0.enabled(metadata)
    }
    fn log(&self, record: &Record<'_>) {
        #[cfg(not(debug_assertions))]
        let record = {
            let meta = record.metadata();
            Record::builder()
                .metadata(
                    Metadata::builder()
                        .level(meta.level())
                        .target(meta.target())
                        .build(),
                )
                .args(*record.args())
                .level(record.level())
                .build()
        };
        if self.enabled(record.metadata()) {
            self.0.log(&record);
        }
    }
    fn flush(&self) {
        self.0.flush();
    }
}
