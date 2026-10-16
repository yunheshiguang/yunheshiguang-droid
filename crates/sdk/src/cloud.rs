use std::future::Future;
use std::pin::Pin;

pub use crate::contract::yunheshiguang::cloud::v1::{
    CloudPhone, CloudPhoneStatus, CloudProviderInfo,
};
use crate::contract::yunheshiguang::workspace::v1::UiText;
use crate::text::ui_text;

pub type CloudFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, UiText>> + Send + 'a>>;

pub const CLOUD_UNSUPPORTED_KEY: &str = "workspace.cloud.unsupported";

pub trait CloudPhoneProvider: Send + Sync {
    fn describe(&self) -> CloudProviderInfo;

    fn phones(&self) -> CloudFuture<'_, Vec<CloudPhone>>;

    fn start<'a>(&'a self, id: &'a str) -> CloudFuture<'a, CloudPhone> {
        let _ = id;
        Box::pin(async { Err(ui_text(CLOUD_UNSUPPORTED_KEY)) })
    }

    fn stop<'a>(&'a self, id: &'a str) -> CloudFuture<'a, CloudPhone> {
        let _ = id;
        Box::pin(async { Err(ui_text(CLOUD_UNSUPPORTED_KEY)) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Listed;

    impl CloudPhoneProvider for Listed {
        fn describe(&self) -> CloudProviderInfo {
            CloudProviderInfo {
                id: "test.listed".to_string(),
                ..CloudProviderInfo::default()
            }
        }

        fn phones(&self) -> CloudFuture<'_, Vec<CloudPhone>> {
            Box::pin(async { Ok(Vec::new()) })
        }
    }

    #[test]
    fn refuses_to_start_or_stop_unless_the_provider_can() {
        let started = tauri::async_runtime::block_on(Listed.start("a"));
        let stopped = tauri::async_runtime::block_on(Listed.stop("a"));

        assert_eq!(started.unwrap_err().key, CLOUD_UNSUPPORTED_KEY);
        assert_eq!(stopped.unwrap_err().key, CLOUD_UNSUPPORTED_KEY);
        assert!(
            tauri::async_runtime::block_on(Listed.phones())
                .unwrap()
                .is_empty()
        );
    }
}
