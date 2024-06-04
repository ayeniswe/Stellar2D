//! The `Instance` is responsible for handling processes and linking modules
use windows::{
    core::PCSTR,
    Win32::{Foundation::HINSTANCE, System::LibraryLoader::GetModuleHandleA},
};
pub(crate) struct Instance<'a>(pub(crate) &'a str);
impl<'a> Instance<'a> {
    /// Get the handle of a process such as a `dll` or `exe`
    pub(crate) fn get_instance(&self) -> HINSTANCE {
        unsafe {
            assert!(!self.0.is_empty());
            let instance = GetModuleHandleA(PCSTR::from_raw(self.0.as_ptr())).unwrap();
            assert!(instance.0 != 0);
            instance.into()
        }
    }
    /// The current instance of this program
    pub(crate) fn this() -> HINSTANCE {
        unsafe {
            let instance = GetModuleHandleA(None).unwrap();
            assert!(instance.0 != 0);
            instance.into()
        }
    }
}
