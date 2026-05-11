/// macOS桌面通知
use notify_rust::Notification;

/// 发送桌面通知
pub fn send_alert(title: &str, message: &str) {
    if let Err(e) = Notification::new()
        .summary(title)
        .body(message)
        .timeout(5000)
        .show()
    {
        eprintln!("通知发送失败: {}", e);
    }
}
