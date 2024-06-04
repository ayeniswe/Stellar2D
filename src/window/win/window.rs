#[derive(Debug)]
pub(crate) struct Window {
    title: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    windows: Vec<Window>,
}
