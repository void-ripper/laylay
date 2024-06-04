use std::{error::Error, fmt::Display};

use borsh::{BorshDeserialize, BorshSerialize};
use sysinfo::System;

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct Cpu {
    pub name: String,
    pub vendor_id: String,
    pub brand: String,
    pub freq: u64,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct Info {
    pub name: Option<String>,
    pub host_name: Option<String>,
    pub kernel_version: Option<String>,
    pub os_version: Option<String>,
    pub cpu: Cpu,
    pub memory: u64,
}

impl Info {
    //
    // + https://developer.android.com/reference/android/os/Build
    //
    #[cfg(target_os = "android")]
    pub fn new() -> Result<Self, Box<dyn Error>> {
        use jni::objects::JString;

        let ctx = ndk_context::android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
        let mut env = vm.attach_current_thread()?;
        let class_ctx = env.find_class("android/os/Build")?;

        let brand = env.get_static_field(&class_ctx, "BRAND", "Ljava/lang/String;")?;
        let host_name = env.get_static_field(&class_ctx, "HOST", "Ljava/lang/String;")?;
        let hardware = env.get_static_field(&class_ctx, "MODEL", "Ljava/lang/String;")?;
        let device = env.get_static_field(&class_ctx, "DEVICE", "Ljava/lang/String;")?;

        let brand = env.get_string(&JString::from(brand.l()?))?.into();
        let host_name = env.get_string(&JString::from(host_name.l()?))?.into();
        let hardware = env.get_string(&JString::from(hardware.l()?))?.into();
        let device = env.get_string(&JString::from(device.l()?))?.into();

        Ok(Self {
            name: Some(brand),
            host_name: Some(host_name),
            kernel_version: Some(hardware),
            os_version: Some(device),
            cpu: Cpu {
                name: "".to_owned(),
                vendor_id: "".to_owned(),
                brand: "".to_owned(),
                freq: 0,
            },
            memory: 0,
        })
    }

    #[cfg(not(target_os = "android"))]
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let sys = System::new_all();

        let cpu = &sys.cpus()[0];

        Ok(Self {
            name: System::name(),
            host_name: System::host_name(),
            kernel_version: System::kernel_version(),
            os_version: System::os_version(),
            cpu: Cpu {
                name: cpu.name().to_string(),
                vendor_id: cpu.vendor_id().to_string(),
                brand: cpu.brand().to_string(),
                freq: cpu.frequency(),
            },
            memory: sys.total_memory(),
        })
    }
}

impl Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
