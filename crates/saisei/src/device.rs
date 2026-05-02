#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DeviceId {
    Default,
    Exact(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputDevice {
    Default,
    Concrete { id: DeviceId, name: String },
}

impl OutputDevice {
    pub fn id(&self) -> &DeviceId {
        match self {
            OutputDevice::Default => &DeviceId::Default,
            OutputDevice::Concrete { id, .. } => id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            OutputDevice::Default => "(default)",
            OutputDevice::Concrete { name, .. } => name,
        }
    }

    pub(crate) fn from_cubeb(info: &cubeb::DeviceInfo) -> Option<Self> {
        let id = info.device_id()?.to_owned();
        let name = info.friendly_name().unwrap_or(id.as_str()).to_owned();

        Some(Self::Concrete {
            id: DeviceId::Exact(id),
            name,
        })
    }
}
