//! Maps evdev's controller identifiers onto the shared wire-format enums
//! from `controller-radio-interface`.

use controller_radio_interface::{Axis, Button};
use evdev::{AbsoluteAxisType, Key};

pub fn axis_from_evdev(axis: AbsoluteAxisType) -> Option<Axis> {
    match axis {
        AbsoluteAxisType::ABS_X => Some(Axis::LeftStickX),
        AbsoluteAxisType::ABS_Y => Some(Axis::LeftStickY),
        AbsoluteAxisType::ABS_RX => Some(Axis::RightStickX),
        AbsoluteAxisType::ABS_RY => Some(Axis::RightStickY),
        AbsoluteAxisType::ABS_Z => Some(Axis::LeftTrigger),
        AbsoluteAxisType::ABS_RZ => Some(Axis::RightTrigger),
        AbsoluteAxisType::ABS_HAT0X => Some(Axis::DPadX),
        AbsoluteAxisType::ABS_HAT0Y => Some(Axis::DPadY),
        _ => None,
    }
}

pub fn button_from_evdev(key: Key) -> Option<Button> {
    match key {
        Key::BTN_SOUTH => Some(Button::South),
        Key::BTN_EAST => Some(Button::East),
        Key::BTN_WEST => Some(Button::North), // West and North seem flipped
        Key::BTN_NORTH => Some(Button::West), // West and North seem flipped
        Key::BTN_TL => Some(Button::LeftBumper),
        Key::BTN_TR => Some(Button::RightBumper),
        Key::BTN_START => Some(Button::Start),
        Key::BTN_SELECT => Some(Button::Select),
        Key::BTN_THUMBL => Some(Button::LeftThumb),
        Key::BTN_THUMBR => Some(Button::RightThumb),
        _ => None,
    }
}