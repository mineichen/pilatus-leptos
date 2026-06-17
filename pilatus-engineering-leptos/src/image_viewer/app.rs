use chrono::DateTime;
use futures_channel::{mpsc, oneshot};
use futures_util::future::LocalBoxFuture;
use imanot::{
    AsyncTask, ImageData, ImageId, ImageLoadOk, ImageState, ImageStateLoaded,
    ImageViewerInteraction, PixelArea, Tools,
};
use imbuf::Image;
use leptos::prelude::{Set, SignalSetter};
use leptos::logging::{debug_log, warn};

type ChangeItem = Box<dyn FnOnce(&mut App, &egui::Context) + Send>;
pub(super) type ChangeListener = Box<
    dyn FnMut(
        &mut ImageStateLoaded,
        imanot::AffectedLayer,
    ) -> LocalBoxFuture<'static, anyhow::Result<()>>,
>;

#[derive(Clone)]
pub struct ViewerHandle {
    command_send: mpsc::Sender<ChangeItem>,
    ctx: egui::Context,
}

impl ViewerHandle {
    pub fn set_primary(&self, name: String) {
        let enqueue = self
            .command_send
            .clone()
            .try_send(Box::new(move |app, _ctx| {
                let mut primary = app.state.tools.primary();
                let (Some(idx), ImageState::Loaded(loaded)) = (
                    primary.tool_names().position(|x| x == name),
                    &app.state.image_state,
                ) else {
                    return;
                };
                primary.set_idx(idx, &loaded.image);
            }));
        if enqueue.is_err() {
            leptos::logging::error!("Unable to queue set_primary");
        }
    }

    pub async fn replace_image(&self, adjust: Image<[u8; 3], 1>, masks: Vec<PixelArea>) {
        let (r_send, r_recv) = oneshot::channel();
        let set_result = self.command_send.clone().try_send(Box::new(|app, ctx| {
            app.state.image_state.set_image_data(ImageData {
                id: ImageId::from("foo"),
                image: ImageLoadOk {
                    original: imanot::OriginalImage::Rgb8(adjust.clone()),
                    adjust,
                },
                masks: masks.into(),
            });
            ctx.request_repaint();
            debug_log!("Replaced image state");
            r_send.send(()).ok();
        }));
        if set_result.is_ok() {
            self.ctx.request_repaint();
            r_recv.await.ok();
            self.ctx.request_repaint();
        }
    }

    pub fn with_loaded_state(&self, f: impl FnOnce(&mut ImageStateLoaded) + Send + 'static) {
        let set_result = self
            .command_send
            .clone()
            .try_send(Box::new(move |app, ctx| {
                if let ImageState::Loaded(loaded) = &mut app.state.image_state {
                    f(loaded);
                    ctx.request_repaint();
                } else {
                    app.pending_on_load.push(Box::new(f));
                }
            }));
        if set_result.is_ok() {
            self.ctx.request_repaint();
        } else {
            leptos::logging::error!("Unable to queue with_loaded_state");
        }
    }
}

pub struct EframeImageViewer {
    #[cfg(target_arch = "wasm32")]
    _runner: eframe::WebRunner,
    handle: ViewerHandle,
}

impl EframeImageViewer {
    #[allow(unused_variables)]
    pub async fn create(
        canvas: web_sys::HtmlCanvasElement,
        tools: Tools,
        change_listener: ChangeListener,
        active_layer: Option<SignalSetter<Option<usize>>>,
    ) -> anyhow::Result<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (canvas, tools, change_listener, active_layer);
            panic!("EframeImageViewer::create() is only supported on wasm32")
        }
        #[cfg(target_arch = "wasm32")]
        {
            let (sender, receiver) = futures_channel::mpsc::channel(1);
            let web_options = eframe::WebOptions::default();
            let runner = eframe::WebRunner::new();
            let ctx = std::rc::Rc::new(std::cell::Cell::new(None));
            let ctx_start = ctx.clone();
            runner
                .start(
                    canvas,
                    web_options,
                    Box::new(move |cc| {
                        leptos::logging::log!(
                            "App creation callback called - eframe instance created"
                        );
                        ctx_start.set(Some(cc.egui_ctx.clone()));
                        Ok(Box::new(App::new(tools, receiver, change_listener, active_layer)))
                    }),
                )
                .await
                .map_err(|e| anyhow::anyhow!("Couldn't start {e:?}"))?;

            Ok(Self {
                _runner: runner,
                handle: ViewerHandle {
                    ctx: ctx.take().unwrap(),
                    command_send: sender,
                },
            })
        }
    }

    pub fn handle(&self) -> &ViewerHandle {
        &self.handle
    }
}

pub struct App {
    state: imanot::State,
    receiver: mpsc::Receiver<ChangeItem>,
    change_listener: ChangeListener,
    change_listener_task: Option<(imanot::AsyncTask<anyhow::Result<()>>, i64)>,
    pending_on_load: Vec<Box<dyn FnOnce(&mut ImageStateLoaded) + Send>>,
    active_layer: Option<SignalSetter<Option<usize>>>,
    last_active_subgroup: Option<usize>,
}

impl App {
    #[cfg(target_arch = "wasm32")]
    pub fn new(
        tools: Tools,
        receiver: mpsc::Receiver<ChangeItem>,
        change_listener: ChangeListener,
        active_layer: Option<SignalSetter<Option<usize>>>,
    ) -> Self {
        Self {
            state: imanot::State::new(tools),
            receiver,
            change_listener,
            change_listener_task: None,
            pending_on_load: Vec::new(),
            active_layer,
            last_active_subgroup: None,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if let Ok(x) = self.receiver.try_recv() {
            x(self, &ctx)
        }
        if let ImageState::Loaded(loaded) = &mut self.state.image_state {
            if let Some(affected) = loaded.masks.take_dirty() {
                let time = chrono::Utc::now()
                    .signed_duration_since(DateTime::UNIX_EPOCH)
                    .num_seconds();

                web_sys::console::log_1(
                    &format!("Dirty {}", self.change_listener_task.is_some()).into(),
                );
                if let Some((_task, start_time)) = &mut self.change_listener_task {
                    warn!(
                        "Still waiting for previous toolchain-change-future to finish (since {}s)... Change is ignored",
                        time - *start_time
                    );
                } else {
                    self.change_listener_task = Some((
                        AsyncTask::new((self.change_listener)(loaded, affected)),
                        time,
                    ));
                }
            }
            if let Some((task, _)) = &mut self.change_listener_task {
                if let Some(x) = task.data() {
                    self.change_listener_task = None;
                    if let Err(e) = x {
                        leptos::logging::error!("Error in change_listener: {e}");
                    }
                } else {
                    ctx.request_repaint_after_secs(0.5);
                }
            }
            let mut pending = std::mem::take(&mut self.pending_on_load).into_iter();
            if let Some(f) = pending.next() {
                f(loaded);
                for f in pending {
                    f(loaded);
                }
                ctx.request_repaint();
            }
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::new())
            .show_inside(ui, |ui| {
                let viewer_result = self.state.ui(ui);
                if let Some(ImageViewerInteraction {
                    cursor_image_pos: Some((x, y)),
                    ..
                }) = viewer_result.inner
                {
                    let image_rect = viewer_result.response.rect;
                    let offset_pos = egui::pos2(image_rect.max.x - 5.0, image_rect.max.y - 5.0);

                    egui::Area::new(egui::Id::new("pixel_coords_overlay"))
                        .fixed_pos(offset_pos)
                        .pivot(egui::Align2::RIGHT_BOTTOM)
                        .order(egui::Order::Foreground)
                        .show(&ctx, |ui| {
                            egui::Frame::popup(ui.style())
                                .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 200))
                                .show(ui, |ui| {
                                    ui.add(egui::Label::new(format!("x: {x}")).extend());
                                    ui.add(egui::Label::new(format!("y: {y}")).extend());
                                });
                        });
                }
            });

        let current_active = match &self.state.image_state {
            ImageState::Loaded(loaded) => loaded.masks.active_subgroup(),
            _ => None,
        };
        if current_active != self.last_active_subgroup {
            self.last_active_subgroup = current_active;
            if let Some(setter) = &self.active_layer {
                setter.set(current_active);
            }
        }
    }
}
