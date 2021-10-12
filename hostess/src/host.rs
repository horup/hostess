use crate::HostInfo;

pub struct Host {
    pub info:HostInfo
}

impl Host {
    pub fn new(info:HostInfo) -> Self {
        Self {
            info
        }
    }
}