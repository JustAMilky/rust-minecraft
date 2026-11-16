use winit::window::Window;

pub struct Gui {
    pub context: egui::Context,
    pub state: egui_winit::State,
    pub renderer: egui_wgpu::Renderer,

    pub fps: f32,
    pub tps: f32,
}

impl Gui {
    pub fn new(
        window: &Window,
        device: &wgpu::Device,
        config: wgpu::SurfaceConfiguration,
    ) -> Result<Self, ()> {
        let context = egui::Context::default();

        let state = egui_winit::State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let renderer =
            egui_wgpu::Renderer::new(device, config.format, egui_wgpu::RendererOptions::default());

        Ok(Self {
            context,
            state,
            renderer,
            fps: 0.0,
            tps: 0.0,
        })
    }

    pub fn update(&mut self, fps: f32, tps: f32) {
        self.fps = fps;
        self.tps = tps;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        screen_size: [u32; 2],

        lateral_render_distance: &mut u32,
        vertical_render_distance: &mut u32,
        needs_chunk_load_refresh: &mut bool,

        camera_fly_speed: &mut f64,
    ) {
        let raw_input = self.state.take_egui_input(window);

        let mut full_output = self.context.run_ui(raw_input, |ctx| {
            egui::Window::new("Options")
                .default_pos([10.0, 10.0])
                .resizable(false)
                .show(ctx, |ui| {
                    let mut render_distance_changed = false;

                    render_distance_changed |= ui
                        .add(
                            egui::Slider::new(lateral_render_distance, 1..=64)
                                .text("Horizontal render distance (chunks)"),
                        )
                        .changed();
                    render_distance_changed |= ui
                        .add(
                            egui::Slider::new(vertical_render_distance, 1..=64)
                                .text("Vertical render distance (chunks)"),
                        )
                        .changed();

                    if render_distance_changed {
                        *needs_chunk_load_refresh = true;
                    }

                    ui.add(
                        egui::Slider::new(camera_fly_speed, 1.0..=128.0).text("Camera Fly Speed"),
                    )
                    .changed();
                });

            egui::Area::new(egui::Id::unique("stats_overlay"))
                .anchor(egui::Align2::RIGHT_TOP, [-8.0, 8.0])
                .interactable(false)
                .show(ctx, |ui| {
                    egui::Frame::NONE
                        .fill(egui::Color32::BLACK)
                        .inner_margin(egui::Margin::same(6))
                        .show(ui, |ui| {
                            let lines = [
                                format!("FPS: {:.0}", self.fps),
                                format!("TPS: {:.0}", self.tps),
                                format!("Frame: {:.2} ms", 1000.0 / self.fps.max(0.001)),
                            ];

                            for line in &lines {
                                ui.label(line);
                            }
                        });
                });
        });

        self.state
            .handle_platform_output(window, full_output.platform_output);

        let paint_jobs = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: screen_size,
            pixels_per_point: full_output.pixels_per_point,
        };

        for (id, image_deltas) in full_output.textures_delta.set.drain() {
            for image_delta in &image_deltas {
                self.renderer.update_texture(device, queue, id, image_delta);
            }
        }

        for id in full_output.textures_delta.free.drain() {
            self.renderer.free_texture(&id);
        }

        let user_cmd_buffers =
            self.renderer
                .update_buffers(device, queue, encoder, &paint_jobs, &screen_descriptor);

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            self.renderer.render(
                &mut render_pass.forget_lifetime(),
                &paint_jobs,
                &screen_descriptor,
            );
        }

        queue.submit(user_cmd_buffers);
    }
}
