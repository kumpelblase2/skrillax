use crate::config::PatchConfig;
use silkroad_gateway_protocol::PatchFile;

pub(crate) enum PatchInformation {
    UpToDate,
    RequiresUpdate {
        files: Vec<PatchFile>,
        target_version: u32,
        host: String,
    },
    Outdated,
}

pub(crate) enum Patcher {
    AcceptAll,
    AcceptMatching {
        min: u32,
        current: u32,
        dir: String,
        remote: String,
    },
}

impl Patcher {
    pub(crate) fn new(config: PatchConfig) -> Self {
        Patcher::AcceptMatching {
            min: config.minimum_client_version,
            current: config.expected_client_version,
            dir: config.dir,
            remote: config.remote_url,
        }
    }

    pub(crate) fn allow_all() -> Self {
        Patcher::AcceptAll
    }

    pub fn get_patch_information(&self, version: u32) -> PatchInformation {
        match &self {
            Patcher::AcceptAll => PatchInformation::UpToDate,
            Patcher::AcceptMatching {
                min, current, remote, ..
            } => {
                if version == *current {
                    PatchInformation::UpToDate
                } else if version < *current && version >= *min {
                    PatchInformation::RequiresUpdate {
                        files: self.get_patches_for(version),
                        target_version: *current,
                        host: remote.clone(),
                    }
                } else {
                    PatchInformation::Outdated
                }
            },
        }
    }

    fn get_patches_for(&self, version: u32) -> Vec<PatchFile> {
        todo!("Load patches from dir and check them")
    }
}
