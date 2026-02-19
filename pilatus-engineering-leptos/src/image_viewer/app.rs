use egui_pixels::{
    ImageData, ImageId, ImageLoadOk, ImageState, ImageViewerInteraction, MaskImage, PixelArea,
    Tools,
};
use futures::channel::{mpsc, oneshot};
use imbuf::Image;
use leptos::logging::debug_log;

type ChangeItem = Box<dyn FnOnce(&mut App, &egui::Context)>;
pub(super) type ChangeListener = Box<dyn FnMut(&MaskImage)>;

pub struct EframeImageViewer {
    #[cfg(target_arch = "wasm32")]
    _runner: eframe::WebRunner,
    command_send: futures::channel::mpsc::Sender<ChangeItem>,
    ctx: egui::Context,
}

impl EframeImageViewer {
    #[allow(unused_variables)]
    pub async fn create(
        canvas: web_sys::HtmlCanvasElement,
        tools: Tools,
        change_listener: ChangeListener,
    ) -> anyhow::Result<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = canvas;
            panic!("EframeImageViewer::create() is only supported on wasm32")
        }
        #[cfg(target_arch = "wasm32")]
        {
            let (sender, receiver) = futures::channel::mpsc::channel(1);
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
                        Ok(Box::new(App::new(tools, receiver, change_listener)))
                    }),
                )
                .await
                .map_err(|e| anyhow::anyhow!("Couldn't start {e:?}"))?;

            // It is normal for start() to return after initialization... The eventloop continues
            Ok(Self {
                ctx: ctx.take().unwrap(),
                _runner: runner,
                command_send: sender,
            })
        }
    }
    pub fn set_primary(&self, name: String) {
        let enqueue = self
            .command_send
            .clone()
            .try_send(Box::new(move |app, _ctx| {
                let mut primary = app.state.tools.primary();
                let (Some(idx), ImageState::Loaded(loaded)) = (
                    primary.tool_names().position(|x| x == &name),
                    &app.state.image_state,
                ) else {
                    return;
                };
                primary.set_idx(idx, &loaded.image);
            }));
        if !enqueue.is_ok() {
            leptos::logging::error!("Unable to queue set_primary");
        }
    }
    pub async fn replace_image(&self, adjust: Image<[u8; 3], 1>, masks: Vec<PixelArea>) {
        let (r_send, r_recv) = oneshot::channel();
        let set_result = self.command_send.clone().try_send(Box::new(|app, ctx| {
            app.state.image_state.set_image_data(ImageData {
                id: ImageId::from("foo"),
                image: ImageLoadOk {
                    original: egui_pixels::OriginalImage::Rgb8(adjust.clone()),
                    adjust,
                },
                masks,
            });
            ctx.request_repaint();
            debug_log!("Replaced image state");
            r_send.send(()).ok();
        }));
        // Avoid no deadlock
        if set_result.is_ok() {
            self.ctx.request_repaint();
            r_recv.await.ok();
            // Assure it is painted
            self.ctx.request_repaint();
        }
    }
}

pub struct App {
    state: egui_pixels::State,
    receiver: mpsc::Receiver<ChangeItem>,
    change_listener: ChangeListener,
}

impl App {
    pub fn new(
        tools: Tools,
        receiver: mpsc::Receiver<ChangeItem>,
        change_listener: ChangeListener,
    ) -> Self {
        Self {
            state: egui_pixels::State::new(tools),
            receiver,
            change_listener,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(x) = self.receiver.try_recv() {
            x(self, ctx)
        }
        if let ImageState::Loaded(loaded) = &mut self.state.image_state {
            if loaded.masks.is_dirty() {
                (self.change_listener)(&loaded.masks);
                loaded.masks.mark_not_dirty();
            }
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::new()) // Removes padding
            .show(ctx, |ui| {
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
                        .show(ctx, |ui| {
                            egui::Frame::popup(ui.style())
                                .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 200))
                                .show(ui, |ui| {
                                    ui.add(egui::Label::new(format!("x: {x}")).extend());
                                    ui.add(egui::Label::new(format!("y: {y}")).extend());
                                });
                        });
                }
            });
    }
}
