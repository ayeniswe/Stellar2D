use windows::Win32::{
    Foundation::COLORREF,
    Graphics::Gdi::{CreateSolidBrush, HBRUSH},
};

// Create handle for window paint brush
fn create_brush(r: u8, g: u8, b: u8) -> HBRUSH {
    let color = ((b as u32) << 16) | ((g as u32) << 8) | r as u32;
    unsafe { CreateSolidBrush(COLORREF(color)) }
}
