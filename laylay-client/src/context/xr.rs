use openxr::{ExtensionSet, Instance, SystemId};

use crate::errors::ClientError;

pub struct XrContext {
    instance: Instance,
    system: SystemId,
}

impl XrContext {
    pub fn new() -> Result<Self, ClientError> {
        let xr_entry = unsafe { openxr::Entry::load()? };
        let aviable = xr_entry.enumerate_extensions()?;
        let mut extensions = ExtensionSet::default();

        if !aviable.khr_vulkan_enable2 {
            tracing::warn!("khr_vulkan_enable2 NOT supported");
        }

        extensions.khr_vulkan_enable2 = aviable.khr_vulkan_enable2;

        #[cfg(target_os = "android")]
        {
            extensions.khr_android_create_instance = true;
            xr_entry.initialize_android_loader()?;
        }

        tracing::info!("aviable exntensions: {:?}", aviable);

        let xr_app_info = openxr::ApplicationInfo {
            api_version: openxr::Version::new(1, 0, 0),
            application_name: "LayLay",
            application_version: 1,
            engine_name: "LayLay",
            engine_version: 1,
        };

        let instance = xr_entry.create_instance(&xr_app_info, &extensions, &[])?;
        let system = instance.system(openxr::FormFactor::HEAD_MOUNTED_DISPLAY)?;

        Ok(Self { instance, system })
    }
}
