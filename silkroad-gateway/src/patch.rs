use crate::config::PatchConfig;
use silkroad_protocol::login::PatchFile;

pub(crate) enum PatchInformation {
    UpToDate,
    RequiredFiles(Vec<PatchFile>),
    Outdated,
}

pub(crate) struct Patcher {
    config: PatchConfig,
}

impl Patcher {
    pub(crate) fn new(config: PatchConfig) -> Self {
        Patcher { config }
    }

    pub fn get_patch_information(&self, version: u32) -> PatchInformation {
        if version == self.config.expected_client_version {
            return PatchInformation::UpToDate;
        } else if version < self.config.expected_client_version
            && version >= self.config.minimum_client_version
        {
            return PatchInformation::RequiredFiles(self.get_patches_for(version));
        }
        PatchInformation::Outdated
    }

    fn get_patches_for(&self, version: u32) -> Vec<PatchFile> {
        todo!()
    }

    pub(crate) fn current_version(&self) -> u32 {
        self.config.expected_client_version
    }

    pub(crate) fn patch_host(&self) -> String {
        self.config.remote_url.clone()
    }
}
