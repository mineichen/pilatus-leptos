use egui::{InnerResponse, Sense};

use egui_pixels::{
    ClearTool, ImageData, ImageId, ImageLoadOk, ImageStateLoaded, ImageViewer,
    ImageViewerInteraction, Tool, ToolContext,
};
use futures::channel::{mpsc, oneshot};
use image_buffer::Image;
use leptos::logging::debug_log;

type ChangeItem = Box<dyn FnOnce(&mut App, &egui::Context)>;

pub struct EframeImageViewer {
    _runner: eframe::WebRunner,
    command_send: futures::channel::mpsc::Sender<ChangeItem>,
    ctx: egui::Context,
}

impl EframeImageViewer {
    pub async fn create(canvas: web_sys::HtmlCanvasElement) -> anyhow::Result<Self> {
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
                    leptos::logging::log!("App creation callback called - eframe instance created");
                    ctx_start.set(Some(cc.egui_ctx.clone()));
                    Ok(Box::new(App::new(&cc.egui_ctx, receiver)))
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
    pub async fn replace_image(&self, adjust: Image<[u8; 3], 1>) {
        let (r_send, r_recv) = oneshot::channel();
        let set_result = self.command_send.clone().try_send(Box::new(|app, ctx| {
            app.image_state = ImageStateLoaded::from_image_data(
                ImageData {
                    id: ImageId::from("foo"),
                    image: ImageLoadOk {
                        original: egui_pixels::OriginalImage::Rgb8(adjust.clone()),
                        adjust,
                    },
                    masks: Vec::new(),
                },
                ctx,
            );
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
    image_state: ImageStateLoaded,
    viewer: ImageViewer,
    tool: Box<dyn Tool>,
    receiver: mpsc::Receiver<ChangeItem>,
}

impl App {
    pub fn new(ctx: &egui::Context, receiver: mpsc::Receiver<ChangeItem>) -> Self {
        let image = ImageData::chessboard().next().unwrap();
        let image_state = ImageStateLoaded::from_image_data(image, ctx);
        Self {
            image_state,
            viewer: ImageViewer::default(),
            tool: Box::new(ClearTool::default()),
            receiver,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(Some(x)) = self.receiver.try_next() {
            x(self, ctx)
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                if let InnerResponse {
                    inner:
                        Some(ImageViewerInteraction {
                            original_image_size: _,
                            cursor_image_pos,
                        }),
                    response,
                } = self
                    .viewer
                    .ui(ui, self.image_state.sources(ui.ctx()), Some(Sense::click()))
                {
                    // Store the image rect before response is moved
                    let image_rect = response.rect;

                    if let Some(cursor_image_pos) = cursor_image_pos {
                        self.tool.handle_interaction(ToolContext::new(
                            &mut self.image_state,
                            response,
                            cursor_image_pos,
                            ctx,
                        ));
                    }

                    // Overlay pixel coordinates on top of the image
                    if let Some((x, y)) = cursor_image_pos {
                        let offset_pos = egui::pos2(image_rect.max.x - 5.0, image_rect.max.y - 5.0);
                        egui::Area::new(egui::Id::new("pixel_coords_overlay"))
                            .fixed_pos(offset_pos)
                            .pivot(egui::Align2::RIGHT_BOTTOM)
                            .order(egui::Order::Foreground)
                            .show(ctx, |ui| {
                                egui::Frame::popup(ui.style())
                                    .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 200))
                                    .show(ui, |ui| {
                                        ui.label(format!("x: {} y: {}", x, y));
                                    });
                            });
                    }
                }
            });
    }
}
