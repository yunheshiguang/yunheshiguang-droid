use std::time::Duration;

use crate::workspace::contract::{RunOptions, UiText};
use crate::workspace::messages::AUTOMATION_OPTIONS_TOO_LARGE_KEY;
use crate::workspace::text::ui_text;

pub const MAX_REPEAT: u32 = 1000;
pub const MAX_INTERVAL_SECONDS: u32 = 86_400;
pub const MAX_RETRY_ATTEMPTS: u32 = 10;
pub const MAX_RETRY_DELAY_SECONDS: u32 = 3_600;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunPlan {
    pub iterations: u32,
    pub interval: Duration,
    pub retries: u32,
    pub retry_delay: Duration,
}

pub fn check_options(options: Option<&RunOptions>) -> Result<RunPlan, UiText> {
    let options = options.cloned().unwrap_or_default();

    if options.repeat > MAX_REPEAT
        || options.repeat_interval_seconds > MAX_INTERVAL_SECONDS
        || options.retry_attempts > MAX_RETRY_ATTEMPTS
        || options.retry_delay_seconds > MAX_RETRY_DELAY_SECONDS
    {
        return Err(ui_text(AUTOMATION_OPTIONS_TOO_LARGE_KEY));
    }

    Ok(RunPlan {
        iterations: options.repeat.max(1),
        interval: Duration::from_secs(options.repeat_interval_seconds.into()),
        retries: options.retry_attempts,
        retry_delay: Duration::from_secs(options.retry_delay_seconds.into()),
    })
}
