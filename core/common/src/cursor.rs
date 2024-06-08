use std::ops::Deref;

pub use cursor_icon::CursorIcon;

/// Newtype for cursor icons, enabling [`Into`] and [`From`] implementations.
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Icon(pub CursorIcon);

impl Icon {
  #[inline]
  pub fn new(cursor_icon: CursorIcon) -> Self { Self(cursor_icon) }

  #[inline]
  pub fn into_cursor_icon(self) -> CursorIcon { self.0 }
}

impl Deref for Icon {
  type Target = CursorIcon;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.0 }
}

// Conversions from others into `Icon`.
impl From<CursorIcon> for Icon {
  #[inline]
  fn from(cursor_icon: CursorIcon) -> Self { Self(cursor_icon) }
}

// Conversions from `Icon` into others.
impl From<Icon> for CursorIcon {
  #[inline]
  fn from(cursor_icon: Icon) -> Self { cursor_icon.0 }
}
#[cfg(feature = "winit")]
impl From<Icon> for winit::window::Cursor {
  #[inline]
  fn from(value: Icon) -> Self {
    winit::window::Cursor::Icon(value.into())
  }
}
#[cfg(feature = "egui")]
impl From<Icon> for egui::CursorIcon {
  fn from(cursor_icon: Icon) -> Self {
    cursor_icon_to_egui_cursor_icon(cursor_icon.into()).into()
  }
}


/// Newtype for optional cursor icons, enabling [`Into`] and [`From`] implementations.
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OptionIcon(pub Option<CursorIcon>);

impl OptionIcon {
  #[inline]
  pub fn new(option: Option<CursorIcon>) -> Self { Self(option) }
  #[inline]
  pub fn from_cursor_icon(cursor_icon: CursorIcon) -> Self { Self(Some(cursor_icon)) }
  #[inline]
  pub fn from_none() -> Self { Self(None) }

  #[inline]
  pub fn into_option(self) -> Option<CursorIcon> { self.0 }
  #[inline]
  pub fn as_ref(&self) -> Option<&CursorIcon> { self.0.as_ref() }
}

// Conversions from others into `OptionCursorIcon`.
impl From<Option<CursorIcon>> for OptionIcon {
  #[inline]
  fn from(option: Option<CursorIcon>) -> Self { Self(option) }
}
impl From<CursorIcon> for OptionIcon {
  #[inline]
  fn from(cursor_icon: CursorIcon) -> Self { Self(Some(cursor_icon)) }
}
impl From<Option<Icon>> for OptionIcon {
  #[inline]
  fn from(option: Option<Icon>) -> Self { Self(option.map(|i| i.into())) }
}
impl From<Icon> for OptionIcon {
  #[inline]
  fn from(icon: Icon) -> Self { Self(Some(icon.into())) }
}
#[cfg(feature = "winit")]
impl From<winit::window::Cursor> for OptionIcon {
  fn from(winit_cursor: winit::window::Cursor) -> Self {
    if let winit::window::Cursor::Icon(cursor_icon) = winit_cursor {
      cursor_icon.into()
    } else {
      Self::from_none()
    }
  }
}
#[cfg(feature = "egui")]
impl From<egui::CursorIcon> for OptionIcon {
  fn from(egui_cursor_icon: egui::CursorIcon) -> Self {
    // From: https://github.com/emilk/egui/blob/9f12432bcf8f8275f154cbbb8aabdb8958be9026/crates/egui-winit/src/lib.rs#L1228
    let cursor_icon = match egui_cursor_icon {
      egui::CursorIcon::None => return Self::from_none(),

      egui::CursorIcon::Alias => CursorIcon::Alias,
      egui::CursorIcon::AllScroll => CursorIcon::AllScroll,
      egui::CursorIcon::Cell => CursorIcon::Cell,
      egui::CursorIcon::ContextMenu => CursorIcon::ContextMenu,
      egui::CursorIcon::Copy => CursorIcon::Copy,
      egui::CursorIcon::Crosshair => CursorIcon::Crosshair,
      egui::CursorIcon::Default => CursorIcon::Default,
      egui::CursorIcon::Grab => CursorIcon::Grab,
      egui::CursorIcon::Grabbing => CursorIcon::Grabbing,
      egui::CursorIcon::Help => CursorIcon::Help,
      egui::CursorIcon::Move => CursorIcon::Move,
      egui::CursorIcon::NoDrop => CursorIcon::NoDrop,
      egui::CursorIcon::NotAllowed => CursorIcon::NotAllowed,
      egui::CursorIcon::PointingHand => CursorIcon::Pointer,
      egui::CursorIcon::Progress => CursorIcon::Progress,

      egui::CursorIcon::ResizeHorizontal => CursorIcon::EwResize,
      egui::CursorIcon::ResizeNeSw => CursorIcon::NeswResize,
      egui::CursorIcon::ResizeNwSe => CursorIcon::NwseResize,
      egui::CursorIcon::ResizeVertical => CursorIcon::NsResize,

      egui::CursorIcon::ResizeEast => CursorIcon::EResize,
      egui::CursorIcon::ResizeSouthEast => CursorIcon::SeResize,
      egui::CursorIcon::ResizeSouth => CursorIcon::SResize,
      egui::CursorIcon::ResizeSouthWest => CursorIcon::SwResize,
      egui::CursorIcon::ResizeWest => CursorIcon::WResize,
      egui::CursorIcon::ResizeNorthWest => CursorIcon::NwResize,
      egui::CursorIcon::ResizeNorth => CursorIcon::NResize,
      egui::CursorIcon::ResizeNorthEast => CursorIcon::NeResize,
      egui::CursorIcon::ResizeColumn => CursorIcon::ColResize,
      egui::CursorIcon::ResizeRow => CursorIcon::RowResize,

      egui::CursorIcon::Text => CursorIcon::Text,
      egui::CursorIcon::VerticalText => CursorIcon::VerticalText,
      egui::CursorIcon::Wait => CursorIcon::Wait,
      egui::CursorIcon::ZoomIn => CursorIcon::ZoomIn,
      egui::CursorIcon::ZoomOut => CursorIcon::ZoomOut,
    };
    cursor_icon.into()
  }
}

// Conversions from `OptionIcon` into others.
impl From<OptionIcon> for Option<CursorIcon> {
  #[inline]
  fn from(option_cursor_icon: OptionIcon) -> Self { option_cursor_icon.0 }
}
impl From<OptionIcon> for Option<Icon> {
  #[inline]
  fn from(option_icon: OptionIcon) -> Self {
    option_icon.into_option().map(|i| i.into())
  }
}
#[cfg(feature = "winit")]
impl From<OptionIcon> for Option<winit::window::Cursor> {
  #[inline]
  fn from(value: OptionIcon) -> Self {
    value.into_option().map(|i| winit::window::Cursor::Icon(i))
  }
}
#[cfg(feature = "egui")]
impl From<OptionIcon> for egui::CursorIcon {
  fn from(option_icon: OptionIcon) -> Self {
    match option_icon.into_option() {
      None => egui::CursorIcon::None,
      Some(cursor_icon) => cursor_icon_to_egui_cursor_icon(cursor_icon)
    }
  }
}

#[cfg(feature = "egui")]
#[inline]
fn cursor_icon_to_egui_cursor_icon(cursor_icon: CursorIcon) -> egui::CursorIcon {
  match cursor_icon {
    CursorIcon::Alias => egui::CursorIcon::Alias,
    CursorIcon::AllScroll => egui::CursorIcon::AllScroll,
    CursorIcon::Cell => egui::CursorIcon::Cell,
    CursorIcon::ContextMenu => egui::CursorIcon::ContextMenu,
    CursorIcon::Copy => egui::CursorIcon::Copy,
    CursorIcon::Crosshair => egui::CursorIcon::Crosshair,
    CursorIcon::Default => egui::CursorIcon::Default,
    CursorIcon::Grab => egui::CursorIcon::Grab,
    CursorIcon::Grabbing => egui::CursorIcon::Grabbing,
    CursorIcon::Help => egui::CursorIcon::Help,
    CursorIcon::Move => egui::CursorIcon::Move,
    CursorIcon::NoDrop => egui::CursorIcon::NoDrop,
    CursorIcon::NotAllowed => egui::CursorIcon::NotAllowed,
    CursorIcon::Pointer => egui::CursorIcon::PointingHand,
    CursorIcon::Progress => egui::CursorIcon::Progress,

    CursorIcon::EwResize => egui::CursorIcon::ResizeHorizontal,
    CursorIcon::NeswResize => egui::CursorIcon::ResizeNeSw,
    CursorIcon::NwseResize => egui::CursorIcon::ResizeNwSe,
    CursorIcon::NsResize => egui::CursorIcon::ResizeVertical,

    CursorIcon::EResize => egui::CursorIcon::ResizeEast,
    CursorIcon::SeResize => egui::CursorIcon::ResizeSouthEast,
    CursorIcon::SResize => egui::CursorIcon::ResizeSouth,
    CursorIcon::SwResize => egui::CursorIcon::ResizeSouthWest,
    CursorIcon::WResize => egui::CursorIcon::ResizeWest,
    CursorIcon::NwResize => egui::CursorIcon::ResizeNorthWest,
    CursorIcon::NResize => egui::CursorIcon::ResizeNorth,
    CursorIcon::NeResize => egui::CursorIcon::ResizeNorthEast,
    CursorIcon::ColResize => egui::CursorIcon::ResizeColumn,
    CursorIcon::RowResize => egui::CursorIcon::ResizeRow,

    CursorIcon::Text => egui::CursorIcon::Text,
    CursorIcon::VerticalText => egui::CursorIcon::VerticalText,
    CursorIcon::Wait => egui::CursorIcon::Wait,
    CursorIcon::ZoomIn => egui::CursorIcon::ZoomIn,
    CursorIcon::ZoomOut => egui::CursorIcon::ZoomOut,

    _ => egui::CursorIcon::Default,
  }
}
