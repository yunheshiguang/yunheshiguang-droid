use super::manifest::MANIFEST;

pub const DEFAULT_REPOSITORY: &str = "yunheshiguang/yunheshiguang-android";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    Nightly,
    Release,
    Local,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildInfo {
    pub channel: String,
    pub version: String,
    pub commit: String,
    pub repository: String,
}

impl BuildInfo {
    pub fn current() -> Self {
        Self::new(
            option_env!("YUNHESHIGUANG_BUILD_CHANNEL").unwrap_or_default(),
            env!("CARGO_PKG_VERSION"),
            option_env!("YUNHESHIGUANG_BUILD_COMMIT").unwrap_or_default(),
            option_env!("YUNHESHIGUANG_BUILD_REPOSITORY").unwrap_or_default(),
        )
    }

    pub fn new(channel: &str, version: &str, commit: &str, repository: &str) -> Self {
        let commit = if is_commit(commit) { commit } else { "" };
        let channel = match channel {
            "nightly" if !commit.is_empty() => "nightly",
            "release" => "release",
            _ => "",
        };
        let repository = if is_repository(repository) {
            repository
        } else {
            DEFAULT_REPOSITORY
        };

        Self {
            channel: channel.to_string(),
            version: version.to_string(),
            commit: commit.to_string(),
            repository: repository.to_string(),
        }
    }

    pub fn channel(&self) -> Channel {
        match self.channel.as_str() {
            "nightly" => Channel::Nightly,
            "release" => Channel::Release,
            _ => Channel::Local,
        }
    }

    pub fn releases_url(&self) -> String {
        format!("https://github.com/{}/releases", self.repository)
    }

    pub fn manifest_url(&self) -> Option<String> {
        let releases = self.releases_url();
        match self.channel() {
            Channel::Nightly => Some(format!("{releases}/download/nightly/{MANIFEST}")),
            Channel::Release => Some(format!("{releases}/latest/download/{MANIFEST}")),
            Channel::Local => None,
        }
    }
}

pub fn is_commit(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

fn is_repository(value: &str) -> bool {
    let part = |part: &str| {
        (1..=100).contains(&part.len())
            && part != "."
            && part != ".."
            && part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    };

    value
        .split_once('/')
        .is_some_and(|(owner, name)| part(owner) && part(name))
}
