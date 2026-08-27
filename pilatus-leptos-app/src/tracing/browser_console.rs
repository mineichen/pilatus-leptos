pub(crate) struct BrowserConsoleLayer;

impl<S> tracing_subscriber::Layer<S> for BrowserConsoleLayer
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let message = visitor.message.unwrap_or_default();

        match *event.metadata().level() {
            tracing::Level::ERROR => web_sys::console::error_1(&message.into()),
            tracing::Level::WARN => web_sys::console::warn_1(&message.into()),
            tracing::Level::INFO => web_sys::console::info_1(&message.into()),
            tracing::Level::DEBUG => web_sys::console::log_1(&message.into()),
            tracing::Level::TRACE => web_sys::console::debug_1(&message.into()),
        }
    }
}

#[derive(Default)]
struct MessageVisitor {
    message: Option<String>,
}

impl tracing::field::Visit for MessageVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_owned());
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{value:?}"));
        }
    }
}
