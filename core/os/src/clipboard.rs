use std::borrow::Cow;

use arboard::Clipboard;

pub fn get_clipboard() -> Box<dyn TextClipboard + Send + 'static> {
  match Clipboard::new() {
    Ok(clipboard) => Box::new(Arboard { clipboard }),
    Err(cause) => {
      tracing::error!(%cause, "failed to create clipboard: {}; falling back to local clipboard", cause);
      Box::new(LocalTextClipboard::default())
    }
  }
}

pub trait TextClipboard {
  fn get(&mut self) -> Option<Cow<str>>;
  fn set(&mut self, text: &str);
}

#[derive(Default, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct LocalTextClipboard {
  pub text: String,
}
impl LocalTextClipboard {
  #[inline]
  pub fn new(text: String) -> Self { Self { text } }
}
impl TextClipboard for LocalTextClipboard {
  #[inline]
  fn get(&mut self) -> Option<Cow<str>> { Some(Cow::from(&self.text)) }
  #[inline]
  fn set(&mut self, text: &str) {
    self.text.clear();
    self.text.push_str(text);
  }
}

pub struct Arboard {
  clipboard: Clipboard,
}
impl TextClipboard for Arboard {
  fn get(&mut self) -> Option<Cow<str>> {
    match self.clipboard.get_text() {
      Ok(text) => Some(Cow::from(text)),
      Err(arboard::Error::ContentNotAvailable) => None,
      Err(cause) => {
        tracing::error!(%cause, "failed to get clipboard text: {}", cause);
        None
      }
    }
  }

  fn set<'a>(&mut self, text: &str) {
    if let Err(cause) = self.clipboard.set_text(text) {
      tracing::error!(%cause, "failed to set clipboard text: {}", cause);
    }
  }
}
