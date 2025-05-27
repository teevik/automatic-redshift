use crate::color::{Color, colorramp_fill};
use color_eyre::eyre::bail;
use log::debug;
use std::os::fd::{AsRawFd, RawFd};
use tokio::io::{Interest, unix::AsyncFd};
use wayrs_client::{
    Connection, EventCtx, IoMode,
    global::{Global, GlobalExt},
    protocol::{WlOutput, wl_output, wl_registry},
};
use wayrs_protocols::wlr_gamma_control_unstable_v1::{
    ZwlrGammaControlManagerV1, ZwlrGammaControlV1, zwlr_gamma_control_v1,
};

pub struct Wayland {
    conn: AsyncFd<Connection<WaylandState>>,
    state: WaylandState,
}

pub struct WaylandState {
    pub outputs: Vec<Output>,
    pub gamma_manager: ZwlrGammaControlManagerV1,
}

impl WaylandState {
    /// Returns the average color of all outputs, or the default color if there are no outputs
    pub fn color(&self) -> Color {
        if self.outputs.is_empty() {
            Color::default()
        } else {
            let color = self.outputs.iter().fold(
                Color {
                    inverted: true,
                    brightness: 0.0,
                    temp: 0,
                    gamma: 0.0,
                },
                |color, output| {
                    let output_color = output.color();
                    Color {
                        inverted: color.inverted && output_color.inverted,
                        brightness: color.brightness + output_color.brightness,
                        temp: color.temp + output_color.temp,
                        gamma: color.gamma + output_color.gamma,
                    }
                },
            );

            Color {
                temp: color.temp / self.outputs.len() as u16,
                gamma: color.gamma / self.outputs.len() as f64,
                brightness: color.brightness / self.outputs.len() as f64,
                inverted: color.inverted,
            }
        }
    }

    pub fn set_temperature(&mut self, temp: u16) {
        debug!(
            "Setting temperature to {} K for {} outputs",
            temp,
            self.outputs.len()
        );
        for output in &mut self.outputs {
            let color = output.color();
            debug!(
                "Output {}: changing color from temp {} K to {} K",
                output.reg_name, color.temp, temp
            );
            output.set_color(Color { temp, ..color });
        }
    }
}

impl AsRawFd for Wayland {
    fn as_raw_fd(&self) -> RawFd {
        self.conn.as_raw_fd()
    }
}

impl Wayland {
    pub fn new() -> color_eyre::Result<Self> {
        let mut conn = Connection::connect()?;
        conn.blocking_roundtrip()?;

        let Ok(gamma_manager) = conn.bind_singleton(1) else {
            bail!(
                "Your Wayland compositor is not supported because it does not implement the wlr-gamma-control-unstable-v1 protocol"
            );
        };

        let mut state = WaylandState {
            outputs: Vec::new(),
            gamma_manager,
        };

        conn.add_registry_cb(wl_registry_cb);
        conn.dispatch_events(&mut state);
        conn.flush(IoMode::Blocking)?;

        let conn = AsyncFd::new(conn)?;

        Ok(Self { conn, state })
    }

    pub fn set_temperature(&mut self, temperature: u16) -> color_eyre::Result<()> {
        self.state.set_temperature(temperature);

        self.conn.get_mut().dispatch_events(&mut self.state);

        for output in &mut self.state.outputs {
            if output.color_changed {
                debug!(
                    "Output {}: color_changed flag is set, updating display",
                    output.reg_name
                );
                output.update_displayed_color(self.conn.get_mut())?;
            }
        }

        self.conn.get_mut().flush(IoMode::Blocking)?;

        Ok(())
    }

    pub async fn poll(&mut self) -> color_eyre::Result<()> {
        self.conn
            .async_io_mut(Interest::READABLE, |inner| {
                inner.recv_events(IoMode::NonBlocking)
            })
            .await?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Output {
    reg_name: u32,
    wl: WlOutput,
    name: Option<String>,
    color: Color,
    gamma_control: ZwlrGammaControlV1,
    ramp_size: usize,
    color_changed: bool,
}

impl Output {
    fn bind(
        conn: &mut Connection<WaylandState>,
        global: &Global,
        gamma_manager: ZwlrGammaControlManagerV1,
    ) -> color_eyre::Result<Self> {
        debug!("New output: {}", global.name);
        let output = global.bind_with_cb(conn, 4, wl_output_cb)?;

        Ok(Self {
            reg_name: global.name,
            wl: output,
            name: None,
            color: Color::default(),
            gamma_control: gamma_manager.get_gamma_control_with_cb(conn, output, gamma_control_cb),
            ramp_size: 0,
            color_changed: true,
        })
    }

    fn destroy(self, conn: &mut Connection<WaylandState>) {
        debug!("Output {} removed", self.reg_name);
        self.gamma_control.destroy(conn);
        self.wl.release(conn);
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn set_color(&mut self, color: Color) {
        if color != self.color {
            debug!(
                "Output {}: color changed from {:?} to {:?}",
                self.reg_name, self.color, color
            );
            self.color = color;
            self.color_changed = true;
        } else {
            debug!("Output {}: color unchanged {:?}", self.reg_name, color);
        }
    }

    fn update_displayed_color(
        &mut self,
        conn: &mut Connection<WaylandState>,
    ) -> color_eyre::Result<()> {
        if self.ramp_size == 0 {
            debug!(
                "Output {}: skipping gamma update, ramp_size is 0",
                self.reg_name
            );
            return Ok(());
        }

        debug!(
            "Output {}: updating gamma ramp with color {:?}, ramp_size {}",
            self.reg_name, self.color, self.ramp_size
        );

        let file = shmemfdrs2::create_shmem(c"/ramp-buffer")?;
        file.set_len(self.ramp_size as u64 * 6)?;
        let mut mmap = unsafe { memmap2::MmapMut::map_mut(&file)? };
        let buf = bytemuck::cast_slice_mut::<u8, u16>(&mut mmap);
        let (r, rest) = buf.split_at_mut(self.ramp_size);
        let (g, b) = rest.split_at_mut(self.ramp_size);
        colorramp_fill(r, g, b, self.ramp_size, self.color);

        debug!(
            "Output {}: setting gamma ramp with temp {} K",
            self.reg_name, self.color.temp
        );
        self.gamma_control.set_gamma(conn, file.into());

        self.color_changed = false;
        debug!("Output {}: gamma ramp update completed", self.reg_name);
        Ok(())
    }
}

fn wl_registry_cb(
    conn: &mut Connection<WaylandState>,
    state: &mut WaylandState,
    event: &wl_registry::Event,
) {
    match event {
        wl_registry::Event::Global(global) if global.is::<WlOutput>() => {
            let mut output = Output::bind(conn, global, state.gamma_manager).unwrap();
            output.set_color(state.color());
            output.update_displayed_color(conn).unwrap();
            state.outputs.push(output);
        }
        wl_registry::Event::GlobalRemove(name) => {
            if let Some(output_index) = state.outputs.iter().position(|o| o.reg_name == *name) {
                let output = state.outputs.swap_remove(output_index);
                output.destroy(conn);
            }
        }
        _ => (),
    }
}

fn gamma_control_cb(ctx: EventCtx<WaylandState, ZwlrGammaControlV1>) {
    let output_index = ctx
        .state
        .outputs
        .iter()
        .position(|o| o.gamma_control == ctx.proxy)
        .expect("Received event for unknown output");
    match ctx.event {
        zwlr_gamma_control_v1::Event::GammaSize(size) => {
            let output = &mut ctx.state.outputs[output_index];
            debug!("Output {}: ramp_size = {}", output.reg_name, size);
            output.ramp_size = size as usize;
            output.update_displayed_color(ctx.conn).unwrap();
        }
        zwlr_gamma_control_v1::Event::Failed => {
            let output = ctx.state.outputs.swap_remove(output_index);
            debug!("Output {}: gamma_control::Event::Failed", output.reg_name);
            output.destroy(ctx.conn);
        }
        _ => (),
    }
}

fn wl_output_cb(ctx: EventCtx<WaylandState, WlOutput>) {
    if let wl_output::Event::Name(name) = ctx.event {
        let output = ctx
            .state
            .outputs
            .iter_mut()
            .find(|o| o.wl == ctx.proxy)
            .unwrap();
        let name = String::from_utf8(name.into_bytes()).expect("invalid output name");
        debug!("Output {}: name = {name:?}", output.reg_name);
        output.name = Some(name);
    }
}
