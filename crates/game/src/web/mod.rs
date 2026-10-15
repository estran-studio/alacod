use bevy::prelude::*;

pub mod logs;

pub struct WebPlugin;

impl Plugin for WebPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(target_arch = "wasm32")]
        {
            console_error_panic_hook::set_once();
            app.add_plugins(logs::WasmWebLogPlugin {});
            app.add_systems(Update, update_window_size);
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn update_window_size(mut window: Query<&mut Window>) {
    (|| {
        let mut window = window.single_mut().ok()?;
        let browser_window = web_sys::window()?;
        let width = browser_window.inner_width().ok()?.as_f64()?;
        let height = browser_window.inner_height().ok()?.as_f64()?;
        window.resolution.set(width as f32, height as f32);
        Some(())
    })();
}
