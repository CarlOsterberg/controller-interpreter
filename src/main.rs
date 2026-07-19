use controller_radio_interface::{ControllerState, PACKET_SIZE};
use evdev::InputEventKind;

mod packet;
mod radio;

const TARGET: &str = "Microsoft X-Box 360 pad";
const RADIO_PAYLOAD_SIZE: usize = 32;

fn main() {
    // CE on GPIO 25; SPI0 CS0 is used automatically via SlaveSelect::Ss0
    let mut radio = radio::Nrf24l01::new(25).expect(
        "failed to initialize NRF24L01 — is SPI enabled? (raspi-config → Interface Options → SPI)",
    );

    loop {
        let device_option = evdev::enumerate()
            .find(|(_, d)| d.name().is_some_and(|n| n == TARGET))
            .map(|(_, d)| d);
        match device_option {
            Some(mut device) => {
                loop {
                    println!("Opened: {}", device.name().unwrap_or("unknown"));
                    let mut state = ControllerState::default();
                    let events = device.fetch_events();
                    if events.is_err() {
                        println!("Xbox controller disconnected");
                        break;
                    };
                    for ev in events.unwrap() {
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
            None => {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
}
