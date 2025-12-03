use std::{
    cell::Cell,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use futures::Stream;
use gloo_events::EventListener;
use gloo_net::websocket::{Message, WebSocketError, futures::WebSocket};
use gloo_timers::callback::Timeout;
use pin_project_lite::pin_project;

#[derive(Debug, thiserror::Error)]
pub enum SuspensibleError {
    #[error("WebSocket error: {0}")]
    WebSocket(anyhow::Error),
    #[error("WebSocket suspended while application was in background")]
    Suspended,
}

struct SharedState {
    visible: Cell<bool>,
    waker: Cell<Option<Waker>>,
}

pin_project! {
    /// WebSocket stream that suspends when the page is hidden and yields
    /// between messages to prevent executor starvation.
    pub struct SuspensibleWebSocket {
        url: String,
        #[pin]
        ws: Option<WebSocket>,
        state: State,
        shared: Rc<SharedState>,
        _listener: EventListener,
        // Stored to prevent memory leak - dropped when next timeout is set or struct is dropped
        pending_wake: Option<Timeout>,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Active { was_visible: bool },
    Yield,
    EmitSuspended,
    Closed,
}

impl SuspensibleWebSocket {
    pub fn new(url: String) -> anyhow::Result<Self> {
        let document = web_sys::window().unwrap().document().unwrap();
        let visible = document.visibility_state() == web_sys::VisibilityState::Visible;

        let shared = Rc::new(SharedState {
            visible: Cell::new(visible),
            waker: Cell::new(None),
        });

        let shared_clone = shared.clone();
        let doc_clone = document.clone();
        let listener = EventListener::new(&document, "visibilitychange", move |_| {
            shared_clone
                .visible
                .set(doc_clone.visibility_state() == web_sys::VisibilityState::Visible);
            if let Some(w) = shared_clone.waker.take() {
                w.wake();
            }
        });

        let ws = visible.then(|| WebSocket::open(&url)).transpose()?;

        Ok(Self {
            url,
            ws,
            state: State::Active {
                was_visible: visible,
            },
            shared,
            _listener: listener,
            pending_wake: None,
        })
    }
}

impl Stream for SuspensibleWebSocket {
    type Item = Result<Message, SuspensibleError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        // Handle non-Active states first
        match *this.state {
            State::Yield => {
                *this.state = State::Active {
                    was_visible: this.shared.visible.get(),
                };
                // Defer wake to next event loop tick to truly yield to other tasks
                let waker = cx.waker().clone();
                *this.pending_wake = Some(Timeout::new(0, move || waker.wake()));
                return Poll::Pending;
            }
            State::Closed => return Poll::Ready(None),
            State::EmitSuspended => {
                *this.state = State::Active { was_visible: true };
                this.ws.set(None);
                return Poll::Ready(Some(Err(SuspensibleError::Suspended)));
            }
            State::Active { was_visible } => {
                this.shared.waker.set(Some(cx.waker().clone()));
                let visible = this.shared.visible.get();

                // Handle visibility transitions
                match (was_visible, visible) {
                    (true, false) => {
                        this.ws.set(None);
                        *this.state = State::Active { was_visible: false };
                        return Poll::Pending;
                    }
                    (false, true) => {
                        *this.state = State::EmitSuspended;
                        cx.waker().wake_by_ref();
                        return Poll::Pending;
                    }
                    (false, false) => return Poll::Pending,
                    (true, true) => {} // Continue to poll WebSocket
                }
            }
        }

        // Ensure connection exists
        if this.ws.as_ref().is_none() {
            match WebSocket::open(this.url) {
                Ok(ws) => this.ws.set(Some(ws)),
                Err(e) => {
                    *this.state = State::Closed;
                    return Poll::Ready(Some(Err(SuspensibleError::WebSocket(e.into()))));
                }
            }
        }

        // Poll the underlying WebSocket
        let Some(ws) = this.ws.as_mut().as_pin_mut() else {
            return Poll::Pending;
        };
        match ws.poll_next(cx) {
            Poll::Ready(Some(Ok(msg))) => {
                *this.state = State::Yield;
                Poll::Ready(Some(Ok(msg)))
            }
            Poll::Ready(Some(Err(e))) => {
                *this.state = State::Closed;
                this.ws.set(None);
                Poll::Ready(Some(Err(SuspensibleError::WebSocket(e.into()))))
            }
            Poll::Ready(None) => {
                *this.state = State::Closed;
                this.ws.set(None);
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
