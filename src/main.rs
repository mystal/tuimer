use notify_rust::Notification;

fn main() -> anyhow::Result<()> {
    Notification::new()
        .summary("Timer finished")
        .action("dismiss", "Dismiss")
        .show()?;
    Ok(())
}
