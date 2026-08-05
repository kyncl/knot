use notify_rust::{Notification, Timeout};
use tracing::debug;

pub fn send_notification<P: AsRef<str>, Q: AsRef<str>>(summary: P, message: Q) {
    let sum = summary.as_ref();
    let msg = message.as_ref();
    let send_status = Notification::new()
        .summary(sum)
        .body(msg)
        .timeout(Timeout::Milliseconds(3000))
        .show();
    if let Err(err) = send_status {
        debug!("Failed to send notification. Cause: {err}");
    }
}
