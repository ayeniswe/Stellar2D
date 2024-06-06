use std::{ops::BitOr, path::Path};
use windows::{
    core::{PCSTR, PCWSTR},
    Win32::{
        Foundation::{HANDLE, HINSTANCE},
        UI::WindowsAndMessaging::*,
    },
};

use super::instance::Instance;
#[derive(Default)]
enum ResourceName<'a> {
    FilePath(&'a Path),
    Ordinal(u32),
    OrdinalW(PCWSTR),
    Name(&'a str),
    #[default]
    Empty,
}
#[derive(Default)]
struct ResourceBuilder {
    flags: IMAGE_FLAGS,
    resource_type: GDI_IMAGE_TYPE,
    dimensions: (i32, i32),
    name: Option<PCSTR>,
    instance: HINSTANCE,
}
impl ResourceBuilder {
    fn new() -> Self {
        Self {
            instance: Instance::this(),
            ..Default::default()
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
        // if self.resource_type != IMAGE_BITMAP {
        //     let rtype;
        //     match self.resource_type {
        //         IMAGE_CURSOR => rtype = "IMAGE_CURSOR",
        //         IMAGE_ICON => rtype = "IMAGE_ICON",
        //         _ => rtype = "UNKNOWN",
        //     }
        //     println!("[Warning] ResourceBuilder::create_dib() DIB section bitmap is no-op on the resource type: '{}'", rtype)
        // }
        self.flags = self.flags.bitor(LR_CREATEDIBSECTION);
        self
    }
    /// Share the image handle with all load instances
    ///
    /// Do not use for images with sizes that may change
    /// after loading or loaded from a file
    fn make_shared(&mut self) -> &mut Self {
        self.flags.bitor(LR_SHARED);
        self
    }
    /// Load image with transparency for every pixel matching the first
    /// pixel in image
    ///
    /// Do not use on bitmap with color depth greater than 8bpp
    fn use_transparent(&mut self) -> &mut Self {
        self.flags.bitor(LR_LOADTRANSPARENT);
        self
    }
    /// Load image gray shades with 3D respective shades
    ///
    /// Do not use on bitmap with color depth greater than 8bpp
    fn use_3d(&mut self) -> &mut Self {
        self.flags.bitor(LR_LOADMAP3DCOLORS);
        self
    }
    /// Load image in black and white
    fn use_mono(&mut self) -> &mut Self {
        self.flags.bitor(LR_MONOCHROME);
        self
    }
    /// Load image with true VGA colors
    fn use_vga(&mut self) -> &mut Self {
        self.flags.bitor(LR_VGACOLOR);
        self
    }
    /// Set the process to control the manager
    ///
    /// Defaults to `this` process if setting is ignored
    fn set_instance(&mut self, module_name: &str) -> &mut Self {
        self.instance = Instance(module_name).get_instance();
        self
    }
    fn set_name(&mut self, name: ResourceName) -> &mut Self {
        self.name = match name {
            ResourceName::FilePath(path) => {
                if let Some(path) = path.to_str() {
                    Some(PCSTR(path.as_ptr()))
                } else {
                    None
                }
            }
            ResourceName::Ordinal(id) => Some(PCSTR(id as *const u8)),
            ResourceName::OrdinalW(id) => Some(PCSTR(id.0 as *const u8)),
            ResourceName::Name(name) => Some(PCSTR(name.as_ptr())),
            ResourceName::Empty => None,
        };
        self
    }
    /// Set resource type as icon
    fn set_typei(&mut self) -> &mut Self {
        self.resource_type = IMAGE_ICON;
        self
    }
    /// Set resource type as cursor
    fn set_typec(&mut self) -> &mut Self {
        self.resource_type = IMAGE_CURSOR;
        self
    }
    /// Set resource type as bitmap
    fn set_typeb(&mut self) -> &mut Self {
        self.resource_type = IMAGE_BITMAP;
        self
    }
    fn load(&mut self) -> &mut Self {
        todo!()
    }
    fn load_file(&mut self) -> Resource {
        self.flags = self.flags.bitor(LR_LOADFROMFILE);
        if let Some(name) = self.name {
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
                Resource::new(handle)
            } else {
                todo!()
            }
        } else {
            todo!()
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
