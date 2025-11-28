use egui::{InnerResponse, Sense};

use egui_pixels::{
    ClearTool, ImageData, ImageStateLoaded, ImageViewer, ImageViewerInteraction, Tool, ToolContext,
};

pub struct EframeImageViewer {
    runner: eframe::WebRunner,
}

impl EframeImageViewer {
    pub fn new() -> Self {
        Self {
            runner: eframe::WebRunner::new(),
        }
    }

    pub async fn start(&self, canvas: web_sys::HtmlCanvasElement) {
        let web_options = eframe::WebOptions::default();
        let result = self
            .runner
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    leptos::logging::log!("App creation callback called - eframe instance created");
                    Ok(Box::new(App::new(&cc.egui_ctx)))
                }),
            )
            .await;

        // It is normal for start() to return after initialization... The eventloop continues
        if let Err(e) = result {
            leptos::logging::log!("eframe start() returned error: {e:?}");
        }
    }
}

pub struct App {
    image_state: ImageStateLoaded,
    viewer: ImageViewer,
    tool: Box<dyn Tool>,
}

impl App {
    pub fn new(ctx: &egui::Context) -> Self {
        let image = ImageData::chessboard().next().unwrap();
        let image_state = ImageStateLoaded::from_image_data(image, &ctx);
        Self {
            image_state,
            viewer: ImageViewer::default(),
            tool: Box::new(ClearTool::default()),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                                        ui.label(format!("Pixel Coordinates: ({}, {})", x, y));
                                    });
                            });
                    }
                }
            });
    }
}
