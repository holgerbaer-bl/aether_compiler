use crate::executor::ExecutionEngine;
use crate::ast::Node;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::platform::run_on_demand::EventLoopExtRunOnDemand;
use winit::window::WindowId;

impl ExecutionEngine {
    pub fn run_event_loop(&mut self, body: &Node) {
        if let Some(mut event_loop) = self.event_loop.take() {
            let mut app = KnotenApp {
                engine: self,
                body,
                exit: false,
            };
            let _ = event_loop.run_app_on_demand(&mut app);
            self.event_loop = Some(event_loop);
        }
    }
}

pub struct KnotenApp<'a> {
    pub engine: &'a mut ExecutionEngine,
    pub body: &'a Node,
    pub exit: bool,
}

impl<'a> ApplicationHandler for KnotenApp<'a> {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let (Some(state), Some(window)) = (&mut self.engine.egui_state, &self.engine.window) {
            let _ = state.on_window_event(window.as_ref(), &event);
        }
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                self.exit = true;
            }
            WindowEvent::KeyboardInput { event: key_ev, .. } => {
                let is_pressed = key_ev.state == winit::event::ElementState::Pressed;
                if let winit::keyboard::Key::Named(k) = &key_ev.logical_key {
                    if is_pressed && let winit::keyboard::NamedKey::Backspace = k {
                        let mut kb = self.engine.keyboard_buffer.lock().unwrap();
                        kb.pop();
                    }
                } else if let winit::keyboard::Key::Character(c) = &key_ev.logical_key {
                    if is_pressed {
                        let mut kb = self.engine.keyboard_buffer.lock().unwrap();
                        kb.push_str(c);
                    }
                }
                if let winit::keyboard::PhysicalKey::Code(code) = key_ev.physical_key {
                    match code {
                        winit::keyboard::KeyCode::KeyW => self.engine.input_w = is_pressed,
                        winit::keyboard::KeyCode::KeyA => self.engine.input_a = is_pressed,
                        winit::keyboard::KeyCode::KeyS => self.engine.input_s = is_pressed,
                        winit::keyboard::KeyCode::KeyD => self.engine.input_d = is_pressed,
                        winit::keyboard::KeyCode::Space => self.engine.input_space = is_pressed,
                        winit::keyboard::KeyCode::ShiftLeft => self.engine.input_shift = is_pressed,
                        _ => {}
                    }
                }
            }
            WindowEvent::Resized(physical_size) => {
                if let (Some(surface), Some(device), Some(config)) = (&self.engine.surface, &self.engine.device, &mut self.engine.config) {
                    config.width = physical_size.width.max(1);
                    config.height = physical_size.height.max(1);
                    surface.configure(device, config);
                    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("Depth Texture"),
                        size: wgpu::Extent3d { width: config.width, height: config.height, depth_or_array_layers: 1 },
                        mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Depth32Float,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    });
                    self.engine.depth_texture_view = Some(depth_texture.create_view(&wgpu::TextureViewDescriptor::default()));
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: winit::event::DeviceEvent) {
        if self.engine.mouse_grab_enabled {
            if let winit::event::DeviceEvent::MouseMotion { delta } = event {
                self.engine.mouse_delta.0 += delta.0 as f32;
                self.engine.mouse_delta.1 += delta.1 as f32;
                if self.engine.camera_active {
                    self.engine.camera_yaw += delta.0 as f32 * 0.002;
                    self.engine.camera_pitch -= delta.1 as f32 * 0.002;
                    let limit = std::f32::consts::FRAC_PI_2 - 0.01;
                    self.engine.camera_pitch = self.engine.camera_pitch.clamp(-limit, limit);
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.engine.camera_active {
            let speed = 0.05;
            let yaw = self.engine.camera_yaw;
            let (sy, cy) = yaw.sin_cos();
            let mut dx = 0.0;
            let mut dz = 0.0;

            if self.engine.input_w { dx -= sy * speed; dz -= cy * speed; }
            if self.engine.input_s { dx += sy * speed; dz += cy * speed; }
            if self.engine.input_a { dx -= cy * speed; dz += sy * speed; }
            if self.engine.input_d { dx += cy * speed; dz -= sy * speed; }

            if self.engine.physics_enabled {
                self.engine.velocity_y -= 0.008;
                if self.engine.input_space && self.engine.is_grounded {
                    self.engine.velocity_y = 0.15;
                    self.engine.is_grounded = false;
                }
                let mut new_pos = self.engine.camera_pos;
                new_pos[1] += self.engine.velocity_y;
                let foot_y = (new_pos[1] - 1.6).floor() as i64;
                let check_x = new_pos[0].floor() as i64;
                let check_z = new_pos[2].floor() as i64;

                if self.engine.voxel_map.contains_key(&[check_x, foot_y, check_z]) {
                    if self.engine.velocity_y < 0.0 {
                        new_pos[1] = (foot_y + 1) as f32 + 1.6;
                        self.engine.velocity_y = 0.0;
                        self.engine.is_grounded = true;
                    }
                } else {
                    self.engine.is_grounded = false;
                }
                
                let try_x = new_pos[0] + dx;
                let try_z = new_pos[2] + dz;
                let ty = (new_pos[1] - 0.5).floor() as i64;
                if !self.engine.voxel_map.contains_key(&[try_x.floor() as i64, ty, check_z]) { new_pos[0] = try_x; }
                if !self.engine.voxel_map.contains_key(&[check_x, ty, try_z.floor() as i64]) { new_pos[2] = try_z; }
                self.engine.camera_pos = new_pos;
            } else {
                self.engine.camera_pos[0] += dx;
                self.engine.camera_pos[2] += dz;
            }
        }

        // GUI & Rendering
        if let (Some(ctx), Some(state), Some(window)) = (&self.engine.egui_ctx, &mut self.engine.egui_state, &self.engine.window) {
            ctx.begin_pass(state.take_egui_input(window.as_ref()));
        }

        if let Some(window) = &self.engine.window {
            if self.engine.mouse_grab_enabled {
                let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Locked);
                window.set_cursor_visible(false);
            } else {
                let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
                window.set_cursor_visible(true);
            }
        }

        // Frame Clear + Evaluate Body
        if let (Some(surface), Some(_device), Some(_queue)) = (&self.engine.surface, &self.engine.device, &self.engine.queue) {
            if let Ok(frame) = surface.get_current_texture() {
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                self.engine.current_canvas_frame = Some(frame);
                self.engine.current_canvas_view = Some(view);
            }
        }

        let _ = self.engine.evaluate(self.body);
        self.engine.present_frame();
    }
}
