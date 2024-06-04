use window::win::window_manager::WindowManagerBuilder;

pub mod utils;
pub mod window;
fn main() {
    let name = "test";
    let mut builder = WindowManagerBuilder::new();
    builder.set_name(name).build();
    builder.set_name("test").build();
}
