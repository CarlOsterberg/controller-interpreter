use controller_radio_interface::{ControllerState, PACKET_SIZE};
use evdev::InputEventKind;

mod packet;
mod radio;

const TARGET: &str = "Microsoft X-Box 360 pad";
const RADIO_PAYLOAD_SIZE: usize = 32;

fn main() {
    let mut device = evdev::enumerate()
        .find(|(_, d)| d.name().is_some_and(|n| n == TARGET))
        .map(|(_, d)| d)
        .unwrap_or_else(|| panic!("no device named {TARGET:?} found"));
    println!("Opened: {}", device.name().unwrap_or("unknown"));

    // CE on GPIO 25; SPI0 CS0 is used automatically via SlaveSelect::Ss0
    let mut radio = radio::Nrf24l01::new(25)
        .expect("failed to initialize NRF24L01 — is SPI enabled? (raspi-config → Interface Options → SPI)");

    let mut state = ControllerState::default();

    loop {
        for ev in device.fetch_events().expect("failed to fetch events") {
            match ev.kind() {
                InputEventKind::Key(key) => {
                    if let Some(button) = packet::button_from_evdev(key) {
                        state.set_button(button, ev.value() != 0);
                    }
                }
                InputEventKind::AbsAxis(axis) => {
                    if let Some(axis) = packet::axis_from_evdev(axis) {
                        state.set_axis(axis, ev.value() as i16);
                    }
                }
                _ => {}
            }

            let mut payload = [0u8; RADIO_PAYLOAD_SIZE];
            payload[..PACKET_SIZE].copy_from_slice(&state.serialize());
            if let Err(e) = radio.send(&payload) {
                eprintln!("radio send failed: {e}");
            }
        }
    }
}
