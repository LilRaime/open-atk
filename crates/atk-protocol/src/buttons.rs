/* Mouse buttons, mappings, and standard button actions. */

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Forward,
    Back,
    DpiSwitch,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ButtonAction {
    Default,
    Disabled,
    ClickLeft,
    ClickRight,
    ClickMiddle,
    BrowserForward,
    BrowserBack,
    DpiCycle,
    DpiStage(usize),
    KeyboardKey(u16),
    MacroId(String),
}

