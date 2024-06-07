use webbrowser::{Browser, BrowserOptions};

pub fn open_url(url: &str, new_tab: bool) {
  let browser = Browser::default();
  let mut options = BrowserOptions::default();
  if new_tab {
    options.with_target_hint("_blank");
  }
  if let Err(cause) = webbrowser::open_browser_with_options(browser, url, &options) {
    tracing::error!(%cause, "failed to open url: {}", cause);
  }
}
