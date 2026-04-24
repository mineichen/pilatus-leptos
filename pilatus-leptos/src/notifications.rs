use std::time::Duration;

use leptos::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub enum NotificationLevel {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug, PartialEq)]
struct NotificationEntry {
    pub id: Uuid,
    pub message: String,
    pub level: NotificationLevel,
}

#[derive(Clone, Copy, Default)]
pub struct NotificationContext {
    notifications: RwSignal<Vec<NotificationEntry>>,
}

impl NotificationContext {
    pub fn push(&self, level: NotificationLevel, message: impl Into<String>, ttl: Duration) {
        let id = Uuid::new_v4();
        let entry = NotificationEntry {
            id,
            message: message.into(),
            level,
        };
        self.notifications.update(|notifications| {
            notifications.push(entry);
        });

        let notifications = self.notifications;
        leptos::task::spawn_local(async move {
            gloo_timers::future::sleep(ttl).await;
            notifications.update(|n| {
                n.retain(|e| e.id != id);
            });
        });
    }

    pub fn error(&self, message: impl Into<String>, ttl: Duration) {
        self.push(NotificationLevel::Error, message, ttl);
    }
}

#[component]
pub fn Notifications() -> impl IntoView {
    let ctx: NotificationContext = expect_context();
    let notifications = ctx.notifications;

    view! {
        <div
            style="position: fixed; top: 1rem; right: 1rem; z-index: 9998; pointer-events: none; display: flex; flex-direction: column; gap: 0.5rem;"
        >
            {move || {
                leptos::logging::log!("Notifications changed: {} ", notifications.get().len());
                notifications.get().into_iter().map(|entry| {
                    let (bg, border) = match entry.level {
                        NotificationLevel::Error => ("#dc2626", "#ef4444"),
                        NotificationLevel::Warning => ("#ca8a04", "#eab308"),
                        NotificationLevel::Info => ("#2563eb", "#3b82f6"),
                    };
                    let message = entry.message.clone();
                    view! {
                        <div
                            style=format!(
                                "background-color: {bg}; border: 1px solid {border}; \
                                 border-radius: 0.5rem; padding: 0.75rem 1rem; \
                                 color: white; font-size: 0.875rem; font-weight: 500; \
                                 min-width: 280px; max-width: 420px; \
                                 box-shadow: 0 4px 12px rgba(0,0,0,0.3); \
                                 pointer-events: auto;"
                            )
                        >
                            {message}
                        </div>
                    }
                }).collect_view()
            }}
        </div>
    }
}
