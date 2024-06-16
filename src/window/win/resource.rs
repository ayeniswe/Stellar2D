use super::instance::Instance;
use crate::utils::logger::Logger;
use std::{
    borrow::Cow,
    fs::metadata,
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

enum ResourceName<'a> {
    File(&'a str),
    /// Windows OEM Bitmaps
    WinOBM(u32),
    /// Windows OEM Icons
    WinOIC(u32),
    /// Windows OEM Cursors
    WinOCR(u32),
    /// Windows Standard Icons
    WinIDI(PCWSTR),
    /// Windows Standard Cursors
    WinIDC(PCWSTR),
    Name(&'a str),
}

struct ResourceBuilder<'a, T: Write> {
    flags: IMAGE_FLAGS,
    resource_type: GDI_IMAGE_TYPE,
    dimensions: (i32, i32),
    name: ResourceName<'a>,
    instance: HINSTANCE,
    logger: Logger<T>,
}
impl<'a, T: Write> ResourceBuilder<'a, T> {
    pub fn new(logger: Logger<T>) -> Self {
        Self {
            logger,
            instance: Instance::this(),
            flags: Default::default(),
            resource_type: Default::default(),
            dimensions: Default::default(),
            name: ResourceName::Name(""),
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
    fn use_sysdefault(&mut self) -> &mut Self {
        self.flags = self.flags.bitor(LR_DEFAULTSIZE);
        self
    }

    /// Use a DIB section bitmap rather than compatible
    fn use_dib(&mut self) -> &mut Self {
        self.flags = self.flags.bitor(LR_CREATEDIBSECTION);
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

    /// Set the process to hold the resource
    ///
    /// Default is `this` process
    fn set_instance(&mut self, module_name: &str) -> &mut Self {
        self.instance = Instance(module_name).get_instance();
        self
    }

    /// Set the resource name
    ///
    /// `Resource::Name` should end in `BMP`, `ICO`, or `CUR`
    ///
    /// `Resource::File` extension should end in `.cur`, `.ico`, or `.bmp`
    ///
    /// ## Example
    /// ```
    /// let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 2));
    /// let resource1 = builder.set_name(Resource::Name("TestBMP\0")).load()
    /// let resource2 = builder.set_name(Resource::File("test.bmp\0")).load()
    ///
    /// assert!(resource1.is_some())
    /// assert!(resource2.is_some())
    /// ```
    fn set_name(&mut self, name: ResourceName<'a>) -> &mut Self {
        self.name = name;
        self
    }

    /// Convert stored `ResourceName` to PCSTR
    fn name_as_pcstr(&mut self) -> Option<PCSTR> {
        let name = match self.name {
            ResourceName::File(file) => {
                if !file.is_empty() {
                    if file.ends_with('\0') {
                        let path = Path::new(file.trim_end_matches('\0'));
                        if let Some(ext) = path.extension() {
                            let ext = ext.to_string_lossy();
                            match ext {
                                Cow::Borrowed("cur") => self.resource_type = IMAGE_CURSOR,
                                Cow::Borrowed("ico") => self.resource_type = IMAGE_ICON,
                                Cow::Borrowed("bmp") => self.resource_type = IMAGE_BITMAP,
                                _ => {
                                    self.logger.elogln(
                                        format!(
                                        "ResourceBuilder::name_as_pcstr() File extension is not valid: .{}",
                                        ext
                                    )
                                        .as_str(),
                                    );
                                    return None;
                                }
                            }
                        } else {
                            self.logger
                                .elogln("ResourceBuilder::name_as_pcstr() No file extension");
                            return None;
                        }

                        let path_string = path.to_string_lossy();
                        if !path_string.contains("�") {
                            if metadata(path).is_ok() {
                                Some(PCSTR(file.as_ptr()))
                            } else {
                                self.logger.elogln(
                                    format!(
                                        "ResourceBuilder::name_as_pcstr() File does not exist: {}",
                                        path_string
                                    )
                                    .as_str(),
                                );
                                None
                            }
                        } else {
                            self.logger.elogln(
                                format!(
                                    "ResourceBuilder::name_as_pcstr() File should not have invalid Unicode: {}",
                                    path_string
                                )
                                .as_str(),
                            );
                            None
                        }
                    } else {
                        self.logger.elogln(
                            format!(
                                r"ResourceBuilder::name_as_pcstr\(\) Filename needs to end in '\0': {} \n",
                                file
                            )
                            .as_str(),
                        );
                        None
                    }
                } else {
                    self.logger
                        .elogln("ResourceBuilder::name_as_pcstr() Filename can not be empty");
                    None
                }
            }
            ResourceName::WinOIC(id) => {
                self.resource_type = IMAGE_ICON;
                self.flags = self.flags.bitor(LR_SHARED);
                self.instance = Default::default();

                Some(PCSTR(id as *const u8))
            }
            ResourceName::WinOCR(id) => {
                self.resource_type = IMAGE_CURSOR;
                self.flags = self.flags.bitor(LR_SHARED);
                self.instance = Default::default();

                let res = match id {
                    32641u32 => None,
                    32647u32 => None,
                    32640u32 => None,
                    _ => Some(PCSTR(id as *const u8)),
                };

                if res.is_none() {
                    self.logger
                        .elogln("ResourceBuilder::name_as_pcstr() OCR_NO, OCR_SIZE, and OCR_ICOCUR are no-op with ResourceName::WinOCR");
                }

                res
            }
            ResourceName::WinOBM(id) => {
                self.resource_type = IMAGE_BITMAP;
                self.flags = self.flags.bitor(LR_SHARED);
                self.instance = Default::default();

                Some(PCSTR(id as *const u8))
            }
            ResourceName::WinIDC(id) => {
                self.resource_type = IMAGE_CURSOR;
                self.flags = self.flags.bitor(LR_SHARED);
                self.instance = Default::default();

                Some(PCSTR(id.0 as *const u8))
            }
            ResourceName::WinIDI(id) => {
                self.resource_type = IMAGE_ICON;
                self.flags = self.flags.bitor(LR_SHARED);
                self.instance = Default::default();

                Some(PCSTR(id.0 as *const u8))
            }
            ResourceName::Name(name) => {
                if !name.is_empty() {
                    if name.ends_with('\0') {
                        match name.to_uppercase() {
                            n if n.contains("BMP") => self.resource_type = IMAGE_BITMAP,
                            n if n.contains("CUR") => self.resource_type = IMAGE_CURSOR,
                            n if n.contains("ICO") => self.resource_type = IMAGE_ICON,
                            _ => {
                                self.logger.elogln(
                                    format!(
                                        "ResourceBuilder::name_as_pcstr() Name is invalid: {}",
                                        name
                                    )
                                    .as_str(),
                                );
                                return None;
                            }
                        };
                        Some(PCSTR(name.as_ptr()))
                    } else {
                        self.logger.elogln(
                            format!(
                                r"ResourceBuilder::name_as_pcstr() Name needs to end in '\0': {}",
                                name
                            )
                            .as_str(),
                        );
                        None
                    }
                } else {
                    self.logger
                        .elogln("ResourceBuilder::name_as_pcstr() Name can not be empty");
                    None
                }
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
        if self.is_flag(LR_CREATEDIBSECTION) {
            match self.resource_type {
                IMAGE_CURSOR => {
                    self.logger.wlogln(
                        format!("ResourceBuilder::validator() DIB section bitmap is no-op with resource type: 'IMAGE_CURSOR'"
                        ).as_str(),
                    )
                }
                IMAGE_ICON => {
                    self.logger.wlogln(
                        format!("ResourceBuilder::validator() DIB section bitmap is no-op with resource type: 'IMAGE_ICON'"
                        ).as_str(),
                    )
                }
                _ => (),
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
        match self.name {
            ResourceName::File(_) => {
                self.flags = self.flags.bitor(LR_LOADFROMFILE);
            }
            _ => (),
        }

        if let Some(name) = self.name_as_pcstr() {
            self.validator();

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

    fn load_icon(&mut self) -> Option<HICON> {
        match self.name {
            ResourceName::WinIDI(_) | ResourceName::WinOIC(_) => {
                let name = self.name_as_pcstr().unwrap_or(PCSTR::null());
                if let Some(handle) = unsafe { LoadIconA(self.instance, name) }.ok() {
                    Some(handle)
                } else {
                    self.logger.elogln(
                        "ResourceBuilder::load_icon() Failed to create a handle for the icon",
                    );
                    None
                }
            }
            _ => {
                self.logger.elogln(
                    "ResourceBuilder::load_icon() 'ResourceName::WinIDI' or 'ResourceName::WinOIC' should be used",
                );
                None
            }
        }
    }

    fn load_cursor(&mut self) -> Option<HCURSOR> {
        match self.name {
            ResourceName::WinIDC(_) | ResourceName::WinOCR(_) => {
                let name = self.name_as_pcstr().unwrap_or(PCSTR::null());
                if let Some(handle) = unsafe { LoadCursorA(self.instance, name) }.ok() {
                    Some(handle)
                } else {
                    self.logger.elogln(
                        "ResourceBuilder::load_cursor() Failed to create a handle for the cursor",
                    );
                    None
                }
            }
            _ => {
                self.logger.elogln(
                    "ResourceBuilder::load_cursor() 'ResourceName::WinIDC' or 'ResourceName::WinOCR' should be used",
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

    fn assert_log(expected: &str, actual: &Vec<u8>) {
        match Regex::new(expected) {
            Ok(r) => assert!(r.is_match(&String::from_utf8_lossy(actual))),
            Err(e) => println!("{}", e),
        }
    }
    fn assert_log_cnt(expected: &str, actual: &Vec<u8>, count: usize) {
        match Regex::new(expected) {
            Ok(r) => assert!(r.find_iter(&String::from_utf8_lossy(actual)).count() == count),
            Err(e) => println!("{}", e),
        }
    }

    mod load_tests {
        use super::*;

        #[test]
        fn test_load_cursor() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let cursor1 = builder
                .set_name(ResourceName::WinOCR(OCR_CROSS.0))
                .load_cursor();
            let cursor2 = builder
                .set_name(ResourceName::WinIDC(IDC_CROSS))
                .load_cursor();

            assert!(&buffer.is_empty());
            assert!(cursor1.is_some());
            assert!(cursor2.is_some());
        }

        #[test]
        fn test_load_cursor_failed() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let cursor1 = builder
                .set_name(ResourceName::WinIDC(PCWSTR(7821 as *const u16)))
                .load_cursor();
            let cursor2 = builder.set_name(ResourceName::WinOCR(7821)).load_cursor();

            assert_log_cnt(
                r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::load_cursor\(\) Failed to create a handle for the cursor\n",
                &buffer,
                2,
            );
            assert!(cursor1.is_none());
            assert!(cursor2.is_none());
        }

        #[test]
        fn test_load_cursor_incompatible_name() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let cursor1 = builder
                .set_name(ResourceName::Name("TestBMP\0"))
                .load_cursor();
            let cursor2 = builder
                .set_name(ResourceName::File("test.bmp\0"))
                .load_cursor();
            let cursor3 = builder
                .set_name(ResourceName::WinOBM(OBM_BTNCORNERS))
                .load_cursor();
            let cursor4 = builder
                .set_name(ResourceName::WinOIC(OIC_HAND))
                .load_cursor();
            let cursor5 = builder
                .set_name(ResourceName::WinIDI(IDI_EXCLAMATION))
                .load_cursor();

            assert_log_cnt(
                r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::load_cursor\(\) 'ResourceName::WinIDC' or 'ResourceName::WinOCR' should be used\n",
                &buffer,
                5,
            );
            assert!(cursor1.is_none());
            assert!(cursor2.is_none());
            assert!(cursor3.is_none());
            assert!(cursor4.is_none());
            assert!(cursor5.is_none());
        }

        #[test]
        fn test_load_icon() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let icon1 = builder
                .set_name(ResourceName::WinIDI(IDI_APPLICATION))
                .load_icon();
            let icon2 = builder.set_name(ResourceName::WinOIC(OIC_HAND)).load_icon();

            assert!(&buffer.is_empty());
            assert!(icon1.is_some());
            assert!(icon2.is_some());
        }

        #[test]
        fn test_load_icon_failed() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let icon1 = builder
                .set_name(ResourceName::WinIDI(PCWSTR(7821 as *const u16)))
                .load_icon();
            let icon2 = builder.set_name(ResourceName::WinOIC(7821)).load_icon();

            assert_log_cnt(
                r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::load_icon\(\) Failed to create a handle for the icon\n",
                &buffer,
                2,
            );
            assert!(icon1.is_none());
            assert!(icon2.is_none());
        }

        #[test]
        fn test_load_icon_incompatible_name() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let icon1 = builder
                .set_name(ResourceName::Name("TestBMP\0"))
                .load_icon();
            let icon2 = builder
                .set_name(ResourceName::File("test.bmp\0"))
                .load_icon();
            let icon3 = builder
                .set_name(ResourceName::WinOBM(OBM_BTNCORNERS))
                .load_icon();
            let icon4 = builder
                .set_name(ResourceName::WinOCR(OCR_APPSTARTING.0))
                .load_icon();
            let icon5 = builder
                .set_name(ResourceName::WinIDC(IDC_APPSTARTING))
                .load_icon();

            assert_log_cnt(
                r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::load_icon\(\) 'ResourceName::WinIDI' or 'ResourceName::WinOIC' should be used\n",
                &buffer,
                5,
            );
            assert!(icon1.is_none());
            assert!(icon2.is_none());
            assert!(icon3.is_none());
            assert!(icon4.is_none());
            assert!(icon5.is_none());
        }

        #[test]
        fn test_load_failed() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let resource = builder.set_name(ResourceName::Name("TestTestBMP\0")).load();

            assert_log(
                r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::load\(\) Failed to create a handle for the resource",
                &buffer,
            );
            assert!(resource.is_none())
        }
    }

    mod flags_tests {
        use super::*;

        #[test]
        fn test_use_dimensions() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            builder.set_dimensions(10, 10);

            assert_eq!(builder.dimensions, (10, 10))
        }

        #[test]
        fn test_use_transparent() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            builder.use_transparent();

            assert_eq!(builder.flags, LR_LOADTRANSPARENT)
        }

        #[test]
        fn test_use_sysdefault() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            builder.use_sysdefault();

            assert_eq!(builder.flags, LR_DEFAULTSIZE)
        }

        #[test]
        fn test_use_dib() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            builder.use_dib();

            assert_eq!(builder.flags, LR_CREATEDIBSECTION)
        }

        #[test]
        fn test_use_vga() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            builder.use_vga();

            assert_eq!(builder.flags, LR_VGACOLOR)
        }

        #[test]
        fn test_use_3d() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            builder.use_3d();

            assert_eq!(builder.flags, LR_LOADMAP3DCOLORS)
        }

        #[test]
        fn test_use_mono() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            builder.use_mono();

            assert_eq!(builder.flags, LR_MONOCHROME)
        }
    }

    mod name_as_pcstr_test {
        use super::*;

        #[test]
        fn test_load() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let resource = builder.set_name(ResourceName::Name("TestTestBMP\0")).load();

            assert_log(
                r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::load\(\) Failed to create a handle for the resource",
                &buffer,
            );
            assert!(resource.is_none())
        }

        #[test]
        fn test_name_as_pcstr_is_not_valid_name() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let resource = builder.set_name(ResourceName::Name("Test\0")).load();

            assert_log(
                r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::name_as_pcstr\(\) Name is invalid: Test",
                &buffer,
            );
            assert!(resource.is_none())
        }

        #[test]
        fn test_name_as_pcstr_is_valid_name() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let resource1 = builder.set_name(ResourceName::Name("TestBMP\0")).load();
            let resource2 = builder.set_name(ResourceName::Name("TestCUR\0")).load();
            let resource3 = builder.set_name(ResourceName::Name("TestICO\0")).load();
            assert!(&buffer.is_empty());
            assert!(resource1.is_some());
            assert!(resource2.is_some());
            assert!(resource3.is_some());
        }

        #[test]
        fn test_name_as_pcstr_name_not_null_terminating() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let resource: Option<Resource> = builder.set_name(ResourceName::Name("test")).load();

            assert_log(
                r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::name_as_pcstr\(\) Name needs to end in '\0': foot.txt\n",
                &buffer,
            );
            assert!(resource.is_none())
        }

        #[test]
        fn test_name_as_pcstr_name_empty() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 3));
            let resource = builder.set_name(ResourceName::Name("")).load();

            assert_log(
                r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::name_as_pcstr\(\) Name can not be empty\n",
                &buffer,
            );
            assert!(resource.is_none());
        }

        #[test]
        fn test_name_as_pcstr_is_valid_file_extension() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let resource1: Option<Resource> = builder
                .set_name(ResourceName::File("tests\\resources\\sample.ico\0"))
                .load();
            let resource2 = builder
                .set_name(ResourceName::File("tests\\resources\\sample.cur\0"))
                .load();
            let resource3 = builder
                .set_name(ResourceName::File("tests\\resources\\sample.bmp\0"))
                .load();

            assert!(&buffer.is_empty());
            assert!(resource1.is_some());
            assert!(resource2.is_some());
            assert!(resource3.is_some());
        }

        #[test]
        fn test_name_as_pcstr_file_empty() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let resource: Option<Resource> = builder.set_name(ResourceName::File("")).load();

            assert_log(
                r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::name_as_pcstr\(\) Filename can not be empty\n",
                &buffer,
            );
            assert!(resource.is_none())
        }

        #[test]
        fn test_name_as_pcstr_file_not_null_terminating() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let resource: Option<Resource> = builder.set_name(ResourceName::File("foo.txt")).load();

            assert_log(
                r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::name_as_pcstr\(\) Filename needs to end in '\0': foot.txt\n",
                &buffer,
            );
            assert!(resource.is_none())
        }

        #[test]
        fn test_name_as_pcstr_is_not_vaild_file_extension() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let resource: Option<Resource> =
                builder.set_name(ResourceName::File("foo.txt\0")).load();

            assert_log(
                r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::name_as_pcstr\(\) File extension is not valid: .txt\n",
                &buffer,
            );
            assert!(resource.is_none())
        }

        #[test]
        fn test_name_as_pcstr_no_file_extension() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let resource: Option<Resource> = builder.set_name(ResourceName::File("foo\0")).load();

            assert_log(
                r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::name_as_pcstr\(\) No file extension\n",
                &buffer,
            );
            assert!(resource.is_none())
        }

        #[test]
        fn test_name_as_pcstr_file_does_not_exist() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let resource: Option<Resource> =
                builder.set_name(ResourceName::File("foo.bmp\0")).load();

            assert_log(
                r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::name_as_pcstr\(\) File does not exist: foo.bmp\n",
                &buffer,
            );
            assert!(resource.is_none())
        }

        #[test]
        fn test_name_as_pcstr_invaild_unicode() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let resource = builder.set_name(ResourceName::File("foo�.bmp\0")).load();

            assert_log(
                r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::name_as_pcstr\(\) File should not have invalid Unicode: foo�.bmp\n",
                &buffer,
            );
            assert!(resource.is_none())
        }

        #[test]
        fn test_name_as_pcstr_win_resources() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let resource1 = builder.set_name(ResourceName::WinIDC(IDC_ARROW)).load();
            let resource2 = builder.set_name(ResourceName::WinIDI(IDC_ARROW)).load();
            let resource3 = builder.set_name(ResourceName::WinOBM(OBM_CHECK)).load();
            let resource4 = builder.set_name(ResourceName::WinOCR(OCR_NO.0)).load();
            let resource5 = builder.set_name(ResourceName::WinOIC(OIC_BANG)).load();

            assert!(&buffer.is_empty());
            assert!(resource1.is_some());
            assert!(resource2.is_some());
            assert!(resource3.is_some());
            assert!(resource4.is_some());
            assert!(resource5.is_some());
        }

        #[test]
        fn test_name_as_pcstr_win_cursors_no_op() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 1));
            let resource1 = builder.set_name(ResourceName::WinOCR(OCR_ICOCUR)).load();
            let resource2 = builder.set_name(ResourceName::WinOCR(OCR_SIZE)).load();
            let resource3 = builder.set_name(ResourceName::WinOCR(OCR_ICON)).load();

            assert_log(
                r"\[ERROR\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::name_as_pcstr\(\) OCR_NO, OCR_SIZE, and OCR_ICOCUR are no-op with ResourceName::WinOCR\n",
                &buffer,
            );
            assert!(resource1.is_none());
            assert!(resource2.is_none());
            assert!(resource3.is_none());
        }
    }

    mod validator_tests {
        use super::*;

        #[test]
        fn test_validator_use_orginal_dimensions() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 2));
            let resource = builder.set_name(ResourceName::WinIDI(IDI_ERROR)).load();

            assert_log(
                r"\[WARNING\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::validator\(\) The original image height will be used\n",
                &buffer,
            );
            assert_log(
                r"\[WARNING\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::validator\(\) The original image width will be used\n",
                &buffer,
            );
            assert!(resource.is_some());
        }

        #[test]
        fn test_validator_use_system_dimensions() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 2));
            let resource = builder
                .use_sysdefault()
                .set_name(ResourceName::WinIDI(IDI_APPLICATION))
                .load();

            assert_log(
                r"\[WARNING\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::validator\(\) The default system height will be used\n",
                &buffer,
            );
            assert_log(
                r"\[WARNING\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::validator\(\) The default system width will be used\n",
                &buffer,
            );
            assert!(resource.is_some());
        }

        #[test]
        fn test_validator_no_op_dib() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 2));
            let resource1 = builder
                .set_name(ResourceName::WinIDI(IDI_EXCLAMATION))
                .use_dib()
                .load();
            let resource2 = builder
                .set_name(ResourceName::WinIDC(IDC_APPSTARTING))
                .use_dib()
                .load();

            assert_log(
                r"\[WARNING\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::validator\(\) DIB section bitmap is no-op with resource type: 'IMAGE_ICON'\n",
                &buffer,
            );
            assert_log(
                r"\[WARNING\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::validator\(\) DIB section bitmap is no-op with resource type: 'IMAGE_CURSOR'\n",
                &buffer,
            );
            assert!(resource1.is_some());
            assert!(resource2.is_some());
        }

        #[test]
        fn test_validator_use_dib() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 2));
            let resource = builder
                .set_name(ResourceName::WinOBM(OBM_CHECKBOXES))
                .set_dimensions(10, 10)
                .use_dib()
                .load();

            assert!(resource.is_some());
            assert!(&buffer.is_empty());
        }

        #[test]
        fn test_validator_no_op_3d_or_vga() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 2));
            let resource = builder
                .set_name(ResourceName::WinIDC(IDC_HAND))
                .use_3d()
                .use_mono()
                .load();

            assert_log(
                r"\[WARNING\] \d{4}-\d{1,2}-\d{1,2} \d{1,2}:\d{1,2}:\d{1,2}.\d{1,3}: ResourceBuilder::validator\(\) 3D and VGA color are no-op when mono is used\n",
                &buffer,
            );
            assert!(resource.is_some());
        }

        #[test]
        fn test_validator_use_3d_or_vga() {
            let mut buffer = Vec::new();

            let mut builder = ResourceBuilder::new(Logger::new(&mut buffer, 2));
            let resource1 = builder
                .set_name(ResourceName::WinIDI(IDI_EXCLAMATION))
                .set_dimensions(10, 10)
                .use_3d()
                .load();
            let resource2: Option<Resource> = builder
                .set_name(ResourceName::WinOBM(OBM_BTSIZE))
                .set_dimensions(10, 10)
                .use_vga()
                .load();

            assert!(&buffer.is_empty());
            assert!(resource1.is_some());
            assert!(resource2.is_some());
        }
    }
}
