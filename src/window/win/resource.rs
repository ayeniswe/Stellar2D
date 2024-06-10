use crate::utils::logger::Logger;

use super::instance::Instance;
use std::{
    io::Write,
    ops::{BitAnd, BitOr},
    path::Path,
};
use windows::{
    core::{PCSTR, PCWSTR},
    Win32::{
        Foundation::{HANDLE, HINSTANCE},
        UI::WindowsAndMessaging::*,
    },
};
#[derive(Default)]
/// All resource types accepted
///
/// `WinOEM` corresponds to Windows OEM icons, bitmaps, and cursors
///
/// `WinIC` corresponds to Windows Standard icons and cursors
enum ResourceName<'a> {
    FilePath(&'a Path),
    WinOEM(u32),
    WinIC(PCWSTR),
    Name(&'a str),
    #[default]
    Empty,
}
#[derive(Default)]
struct ResourceBuilder<'a, T: Write> {
    flags: IMAGE_FLAGS,
    resource_type: GDI_IMAGE_TYPE,
    dimensions: (i32, i32),
    name: ResourceName<'a>,
    instance: HINSTANCE,
    logger: Logger<T>,
}
impl<'a, T: Write> ResourceBuilder<'a, T> {
    fn new(logger: Logger<T>) -> Self {
        Self {
            logger,
            instance: Instance::this(),
            flags: Default::default(),
            resource_type: Default::default(),
            dimensions: Default::default(),
            name: Default::default(),
        }
    }
    ///  Set the width and height of the icon or image
    ///
    /// No-op for bitmap
    fn set_dimensions(&mut self, w: i32, h: i32) -> &mut Self {
        self.dimensions = (w, h);
        self
    }
    /// Use the system default size for the resource
    fn use_default(&mut self) -> &mut Self {
        self.flags = self.flags.bitor(LR_DEFAULTSIZE);
        self
    }
    /// Use a DIB section bitmap rather than compatible
    fn use_dib(&mut self) -> &mut Self {
        self.flags = self.flags.bitor(LR_CREATEDIBSECTION);
        self
    }
    /// Share the image handle with all load instances
    ///
    /// Do not use for images with sizes that may change
    /// after loading or loaded from a file
    fn make_shared(&mut self) -> &mut Self {
        self.flags = self.flags.bitor(LR_SHARED);
        self
    }
    /// Load image with transparency for every pixel matching the first
    /// pixel in image
    ///
    /// Do not use on bitmap with color depth greater than 8bpp
    fn use_transparent(&mut self) -> &mut Self {
        self.flags = self.flags.bitor(LR_LOADTRANSPARENT);
        self
    }
    /// Load image gray shades with 3D respective shades
    ///
    /// Do not use on bitmap with color depth greater than 8bpp
    fn use_3d(&mut self) -> &mut Self {
        self.flags = self.flags.bitor(LR_LOADMAP3DCOLORS);
        self
    }
    /// Load image in black and white
    fn use_mono(&mut self) -> &mut Self {
        self.flags = self.flags.bitor(LR_MONOCHROME);
        self
    }
    /// Load image with true VGA colors
    fn use_vga(&mut self) -> &mut Self {
        self.flags = self.flags.bitor(LR_VGACOLOR);
        self
    }
    /// Set the process to control the manager
    ///
    /// Defaults to `this` process if setting is ignored
    fn set_instance(&mut self, module_name: &str) -> &mut Self {
        self.instance = Instance(module_name).get_instance();
        self
    }
    /// Set the resource name
    fn set_name(&mut self, name: ResourceName<'a>) -> &mut Self {
        self.name = name;
        self
    }
    /// Set resource type as icon
    ///
    /// The last type to be set will be used
    fn set_typei(&mut self) -> &mut Self {
        self.resource_type = IMAGE_ICON;
        self
    }
    /// Set resource type as cursor
    ///
    /// The last type to be set will be used
    fn set_typec(&mut self) -> &mut Self {
        self.resource_type = IMAGE_CURSOR;
        self
    }
    /// Set resource type as bitmap
    ///
    /// The last type to be set will be used
    fn set_typeb(&mut self) -> &mut Self {
        self.resource_type = IMAGE_BITMAP;
        self
    }
    /// Convert stored `ResourceName` -> PCSTR
    fn name_as_pcstr(&mut self) -> Option<PCSTR> {
        let name = match self.name {
            ResourceName::FilePath(path) => {
                if let Some(path) = path.to_str() {
                    Some(PCSTR(path.as_ptr()))
                } else {
                    self.logger
                        .elogln("ResourceBuilder::name_as_pcstr() File could not be found");
                    None
                }
            }

            ResourceName::WinOEM(id) => Some(PCSTR(id as *const u8)),
            ResourceName::WinIC(id) => Some(PCSTR(id.0 as *const u8)),
            ResourceName::Name(name) => {
                self.logger
                    .elogln("ResourceBuilder::name_as_pcstr() Name can not be empty");
                Some(PCSTR(name.as_ptr()))
            }
            ResourceName::Empty => {
                self.logger
                    .elogln("ResourceBuilder::name_as_pcstr() Resource name is empty");
                None
            }
        };
        name
    }
    /// Check if flag is set
    fn is_flag(&self, flag: IMAGE_FLAGS) -> bool {
        self.flags.0.bitand(flag.0) == flag.0
    }
    /// Validate builder settings
    fn validator(&mut self) {
        // Dimensions
        let (width, height) = self.dimensions;
        let mut check_dimensions = |dimension: i32, dimension_name| -> () {
            if dimension == Default::default() {
                if self.is_flag(LR_DEFAULTSIZE) {
                    self.logger.wlogln(
                        format!(
                            "ResourceBuilder::validator() The default system {} will be used",
                            dimension_name
                        )
                        .as_str(),
                    )
                } else {
                    self.logger.wlogln(
                        format!(
                            "ResourceBuilder::validator() The original image {} will be used",
                            dimension_name
                        )
                        .as_str(),
                    )
                }
            }
        };
        check_dimensions(width, "width");
        check_dimensions(height, "height");
        // Bitmap
        if self.resource_type != IMAGE_BITMAP {
            let rtype;
            match self.resource_type {
                IMAGE_CURSOR => rtype = "IMAGE_CURSOR",
                IMAGE_ICON => rtype = "IMAGE_ICON",
                _ => rtype = "UNKNOWN",
            }
            self.logger.wlogln(
                format!("ResourceBuilder::validator() DIB section bitmap is no-op with resource type: '{}'",
                rtype).as_str(),
            )
        }
        // Handles
        if !self.is_flag(LR_SHARED) {
            match self.name {
                ResourceName::WinOEM(_) | ResourceName::WinIC(_) => self.logger.elogln("ResourceBuilder::validator() Windows icons, cursors, and bitmaps must be shared"),
                _ => ()
            }
        }
        // Color
        if self.is_flag(LR_MONOCHROME) {
            if self.is_flag(LR_LOADMAP3DCOLORS) || self.is_flag(LR_VGACOLOR) {
                self.logger.wlogln(
                    "ResourceBuilder::validator() 3D and VGA color are no-op when mono is used",
                )
            }
        }
    }
    fn load(&mut self) -> Option<Resource> {
        if let Some(name) = self.name_as_pcstr() {
            let handle = unsafe {
                LoadImageA(
                    self.instance,
                    name,
                    self.resource_type,
                    self.dimensions.0,
                    self.dimensions.1,
                    self.flags,
                )
            }
            .ok();
            if let Some(handle) = handle {
                Some(Resource::new(handle))
            } else {
                self.logger
                    .elogln("ResourceBuilder::load() Failed to create a handle for the resource");
                None
            }
        } else {
            None
        }
    }
    // Load resource from a file path
    fn load_file(&mut self) -> Option<Resource> {
        match self.name {
            ResourceName::FilePath(_) => {
                self.flags = self.flags.bitor(LR_LOADFROMFILE);
                self.load()
            }
            _ => {
                self.logger.elogln(
                    "ResourceBuilder::load_file() ResourceName::FilePath variance must be used",
                );
                None
            }
        }
    }
}
struct Resource {
    id: HANDLE,
}
impl Resource {
    fn new(id: HANDLE) -> Self {
        Self { id }
    }
}

#[cfg(test)]
mod resource_builder_tests {
    use super::*;
    use regex::Regex;
    #[test]
    fn test_name_as_pcstr_is_empty() {
        let mut buffer = Vec::new();
        let logger = Logger::new(&mut buffer, 3);
        let mut builder = ResourceBuilder::new(logger);
        let resource = builder.set_name(ResourceName::Empty).load();
        let log_re = Regex::new(
          r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::name_as_pcstr\(\) Resource name is empty\n",
      )
      .unwrap();
        let log = String::from_utf8_lossy(&buffer);

        assert!(log_re.is_match(&log));
        assert!(resource.is_none());
    }
    #[test]
    fn test_name_as_pcstr_is_empty_str() {
        let mut buffer = Vec::new();
        let logger = Logger::new(&mut buffer, 3);
        let mut builder = ResourceBuilder::new(logger);
        let resource = builder.set_name(ResourceName::Name("")).load();
        let log_re = Regex::new(
            r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::name_as_pcstr\(\) Name can not be empty\n",
        )
        .unwrap();
        let log = String::from_utf8_lossy(&buffer);

        assert!(log_re.is_match(&log))
    }
    // #[test]
    // fn test_name_as_pcstr_is_file_not_found() {
    //     let mut buffer = Vec::new();
    //     let logger = Logger::new(&mut buffer, 3);
    //     let mut builder = ResourceBuilder::new(logger);
    //     let resource = builder
    //         .set_name(ResourceName::FilePath(&Path::new("foo.txt")))
    //         .load();
    //     let log_re = Regex::new(
    //         r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::name_as_pcstr\(\) File could not be found\n",
    //     )
    //     .unwrap();
    //     let log = String::from_utf8_lossy(&buffer);

    //     assert!(log_re.is_match(&log))
    // }
}
