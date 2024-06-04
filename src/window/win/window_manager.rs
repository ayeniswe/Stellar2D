//! The `WindowManager` is responsible for creating, managing, and destroying windows.
//! The `WindowManager` abstracts away the registering of a window class
//! Compatible with `Windows` only; all other platforms will be no-op.
use super::{instance::Instance, window::Window};
use std::{
    ffi::CString,
    ops::{BitAnd, BitOr},
    sync::Arc,
};
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::{ValidateRect, HBRUSH},
        UI::WindowsAndMessaging::*,
    },
};
#[derive(Debug, Default)]
pub struct WindowManagerBuilder<'a> {
    style: WNDCLASS_STYLES,
    lpfnWndProc: WNDPROC,
    metadata: i32,
    window_metadata: i32,
    instance: HINSTANCE,
    hIcon: HICON,
    hCursor: HCURSOR,
    hbrBackground: HBRUSH,
    menuname: Option<&'a str>,
    classname: &'a str,
}
impl<'a> WindowManagerBuilder<'a> {
    pub fn new() -> Self {
        Self {
            instance: Instance::this(),
            ..Default::default()
        }
    }
    /// Set the process to control the manager
    ///
    /// Defaults to `this` process if setting is ignored
    pub fn set_instance(&mut self, module_name: &str) -> &mut Self {
        self.instance = Instance(module_name).get_instance();
        self
    }
    /// Set the name of the manager
    ///
    /// Name must be unique
    pub fn set_name(&mut self, name: &'a str) -> &mut Self {
        self.classname = name;
        self
    }
    /// Set default menu
    ///
    /// Name must be unique
    pub fn set_menu(&mut self, name: &'a str) -> &mut Self {
        self.menuname = Some(name);
        self
    }
    /// Aligns the managers children window's client area by byte boundary on x-axis
    pub fn align_byte_client(&mut self) -> &mut Self {
        self.style = self.style.bitor(CS_BYTEALIGNCLIENT);
        self
    }
    /// Aligns the managers children window's by byte boundary on x-axis
    pub fn align_byte_window(&mut self) -> &mut Self {
        self.style = self.style.bitor(CS_BYTEALIGNWINDOW);
        self
    }
    /// Listen for double clicks within any windows in manager
    pub fn listen_to_dbclick(&mut self) -> &mut Self {
        self.style = self.style.bitor(CS_DBLCLKS);
        self
    }
    /// Enable drop shadow effect
    ///
    /// Windows created from the manager must be top-level windows (parent or root); they may not be child windows.
    pub fn enable_drop_shadow(&mut self) -> &mut Self {
        self.style = self.style.bitor(CS_DROPSHADOW);
        self
    }
    /// Make manager global
    ///
    /// see more - Application Global Class
    pub fn make_global(&mut self) -> &mut Self {
        self.style = self.style.bitor(CS_GLOBALCLASS);
        self
    }
    /// Listen for a movement or size adjustment change in the height of the client area
    ///
    /// The entire window will be redrawn
    pub fn listen_to_vert(&mut self) -> &mut Self {
        self.style = self.style.bitor(CS_VREDRAW);
        self
    }
    /// Listen for a movement or size adjustment change in the width of the client area.
    ///
    /// The entire window will be redrawn
    pub fn listen_to_hori(&mut self) -> &mut Self {
        self.style = self.style.bitor(CS_HREDRAW);
        self
    }
    /// Disable close on a window menu
    pub fn disable_close(&mut self) -> &mut Self {
        self.style = self.style.bitor(CS_NOCLOSE);
        self
    }
    // Check if a single device context has already been set
    fn is_dc_set(&self, class1: WNDCLASS_STYLES, class2: WNDCLASS_STYLES) -> bool {
        let class1_dc = self.style.bitand(class1) != WNDCLASS_STYLES(0);
        let class2_dc = self.style.bitand(class2) != WNDCLASS_STYLES(0);
        if class1_dc || class2_dc {
            let class = if class1_dc {
                stringify!(class1_dc)
            } else {
                stringify!(class2_dc)
            };
            format!(
                "[WARNING] The device context has already been set to '{}'",
                class
            );
            return true;
        }
        return false;
    }
    /// Create a single shared device context for all windows in manager
    pub fn create_single_dc(&mut self) -> &mut Self {
        if self.is_dc_set(CS_OWNDC, CS_PARENTDC) {
            return self;
        };
        self.style = self.style.bitor(CS_CLASSDC);
        self
    }
    /// Allocate an unique private device context to each window
    pub fn create_unique_dc(&mut self) -> &mut Self {
        if self.is_dc_set(CS_CLASSDC, CS_PARENTDC) {
            return self;
        };
        self.style = self.style.bitor(CS_OWNDC);
        self
    }
    /// Parent and child windows use cache system device contexts
    pub fn create_cache_dc(&mut self) -> &mut Self {
        if self.is_dc_set(CS_CLASSDC, CS_OWNDC) {
            return self;
        };
        self.style = self.style.bitor(CS_PARENTDC);
        self
    }
    /// Redraw screen images obscured by any window using cache bitmaps
    /// ## Performance
    /// - Eliminates the need of screen redraws
    /// - Memory intensive
    pub fn save_bitmap(&mut self) -> &mut Self {
        self.style = self.style.bitor(CS_SAVEBITS);
        self
    }
    /// Allocate bytes of memory to store metadata per window
    pub fn allocate_window_metadata(&mut self, bytes: i32) -> &mut Self {
        self.window_metadata = bytes;
        self
    }
    /// Allocate bytes of memory to store metadata
    ///
    /// Shared among all windows created
    pub fn allocate_metadata(&mut self, bytes: i32) -> &mut Self {
        self.metadata = bytes;
        self
    }
    pub fn build(&self) -> WindowManager {
        assert!(
            !self.classname.is_empty(),
            "[Error] Window Manager name can not be empty"
        );
        let mut class = WNDCLASSA::default();
        class.lpszClassName = PCSTR::from_raw(self.classname.as_ptr());
        if let Some(menuname) = self.menuname {
            assert!(
                !menuname.is_empty(),
                "[Error] Window Manager Menu name can not be empty"
            );
            class.lpszMenuName = PCSTR::from_raw(self.menuname.unwrap().as_ptr());
        }
        class.hInstance = self.instance;
        class.style = self.style;
        class.cbClsExtra = self.metadata;
        class.cbWndExtra = self.window_metadata;
        // class.hbrBackground =
        // class.hCursor =
        // class.hIcon =
        // class.lpfnWndProc =
        let atom = unsafe { RegisterClassA(&class) };
        assert!(
            atom != 0,
            "[Error] Window Manager '{}' already exists",
            self.classname
        );
        WindowManager::new(&self.classname)
    }
}
#[derive(Debug, Default)]
pub struct WindowManager<'a> {
    name: &'a str,
    windows: Vec<Window>,
}
impl<'a> WindowManager<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name: name,
            ..Default::default()
        }
    }
}
pub extern "system" fn wndproc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match message {
            WM_PAINT => {
                println!("WM_PAINT");
                _ = ValidateRect(window, None);
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}
#[cfg(test)]
mod window_manager_builder_dc_tests {
    use super::WindowManagerBuilder;
    use windows::Win32::UI::WindowsAndMessaging::{CS_CLASSDC, CS_OWNDC, CS_PARENTDC};
    #[test]
    fn test_create_unique_dc() {
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder.create_unique_dc();

        assert_eq!(manager_builder.style, CS_OWNDC)
    }
    #[test]
    fn test_create_single_dc() {
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder.create_single_dc();

        assert_eq!(manager_builder.style, CS_CLASSDC)
    }
    #[test]
    fn test_create_cache_dc() {
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder.create_cache_dc();

        assert_eq!(manager_builder.style, CS_PARENTDC)
    }
    #[test]
    fn test_single_dc_is_set() {
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder
            .create_cache_dc()
            .create_single_dc()
            .create_unique_dc();

        // Device context should be based on the first one set
        assert_eq!(manager_builder.style, CS_PARENTDC)
    }
}
#[cfg(test)]
mod window_manager_builder_tests {
    use super::WindowManagerBuilder;
    use windows::Win32::UI::WindowsAndMessaging::{
        CS_BYTEALIGNCLIENT, CS_BYTEALIGNWINDOW, CS_DBLCLKS, CS_DROPSHADOW, CS_GLOBALCLASS,
        CS_HREDRAW, CS_NOCLOSE, CS_SAVEBITS, CS_VREDRAW,
    };
    #[test]
    fn test_add_byte_align_clients() {
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder.align_byte_client();

        assert_eq!(manager_builder.style, CS_BYTEALIGNCLIENT)
    }
    #[test]
    fn test_add_byte_align_window() {
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder.align_byte_window();

        assert_eq!(manager_builder.style, CS_BYTEALIGNWINDOW)
    }
    #[test]
    fn test_listen_to_dbclick() {
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder.listen_to_dbclick();

        assert_eq!(manager_builder.style, CS_DBLCLKS)
    }
    #[test]
    fn test_enable_drop_shadow() {
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder.enable_drop_shadow();

        assert_eq!(manager_builder.style, CS_DROPSHADOW)
    }
    #[test]
    fn test_make_global() {
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder.make_global();

        assert_eq!(manager_builder.style, CS_GLOBALCLASS)
    }
    #[test]
    fn test_listen_to_vert() {
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder.listen_to_vert();

        assert_eq!(manager_builder.style, CS_VREDRAW)
    }
    #[test]
    fn test_listen_to_hori() {
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder.listen_to_hori();

        assert_eq!(manager_builder.style, CS_HREDRAW)
    }
    #[test]
    fn test_disable_close() {
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder.disable_close();

        assert_eq!(manager_builder.style, CS_NOCLOSE)
    }
    #[test]
    fn test_save_bitmap() {
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder.save_bitmap();

        assert_eq!(manager_builder.style, CS_SAVEBITS)
    }
    #[test]
    fn test_multiple_options() {
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder
            .align_byte_client()
            .disable_close()
            .enable_drop_shadow()
            .save_bitmap();

        assert_eq!(manager_builder.style.0, 137728)
    }
}
#[cfg(test)]
mod window_manager_builder_class_tests {
    use super::WindowManagerBuilder;
    #[test]
    #[should_panic(expected = "[Error] Window Manager name can not be empty")]
    fn test_set_name_empty() {
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder.set_name("").build();
    }
    #[test]
    fn test_set_name_not_exists() {
        let name = "test-name-not-exists";
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder.set_name(name).build();

        assert!(manager_builder.classname == name)
    }
    #[test]
    #[should_panic(expected = "[Error] Window Manager 'test-name-exists' already exists")]
    fn test_set_name_exists() {
        let name = "test-name-exists";
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder.set_name(name).build();
        manager_builder.set_name(name).build();
    }
    #[test]
    #[should_panic(expected = "[Error] Window Manager Menu name can not be empty")]
    fn test_set_menu_empty() {
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder
            .set_name("test-menu-empty")
            .set_menu("")
            .build();
    }
    #[test]
    fn test_set_menu_not_exists() {
        let name = "test";
        let mut manager_builder = WindowManagerBuilder::new();
        manager_builder
            .set_name("test-menu-not-exists")
            .set_menu(name)
            .build();

        assert!(manager_builder.menuname.unwrap() == name)
    }
}
