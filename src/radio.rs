use rppal::gpio::{Gpio, OutputPin};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use std::error::Error;
use std::thread;
use std::time::Duration;

const W_REGISTER: u8 = 0x20;
const W_TX_PAYLOAD: u8 = 0xA0;
const FLUSH_TX: u8 = 0xE1;

pub struct Nrf24l01 {
    spi: Spi,
    ce: OutputPin,
}

impl Nrf24l01 {
    pub fn new(ce_pin: u8) -> Result<Self, Box<dyn Error>> {
        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 8_000_000, Mode::Mode0)?;
        let ce = Gpio::new()?.get(ce_pin)?.into_output();
        let mut radio = Self { spi, ce };
        radio.init()?;
        Ok(radio)
    }

    fn write_reg(&mut self, reg: u8, value: u8) -> Result<(), Box<dyn Error>> {
        self.spi.write(&[W_REGISTER | reg, value])?;
        Ok(())
    }

    fn write_cmd(&mut self, cmd: u8, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let mut buf = Vec::with_capacity(1 + data.len());
        buf.push(cmd);
        buf.extend_from_slice(data);
        self.spi.write(&buf)?;
        Ok(())
    }

    fn status(&mut self) -> Result<u8, Box<dyn Error>> {
        let mut buf = [0u8; 1];
        self.spi.transfer(&mut buf, &[0xFF])?; // NOP — STATUS is the first byte clocked out
        Ok(buf[0])
    }

    fn init(&mut self) -> Result<(), Box<dyn Error>> {
        self.ce.set_low();
        thread::sleep(Duration::from_millis(5));

        self.write_reg(0x00, 0x0E)?; // CONFIG: PWR_UP, TX mode, 2-byte CRC
        self.write_reg(0x01, 0x01)?; // EN_AA: enable auto-ack on pipe 0
        self.write_reg(0x03, 0x03)?; // SETUP_AW: 5-byte addresses
        self.write_reg(0x04, 0x1F)?; // SETUP_RETR: 500 µs delay, 15 retries
        self.write_reg(0x05, 2)?;    // RF_CH: 2402 MHz
        self.write_reg(0x06, 0x06)?; // RF_SETUP: 1 Mbps, 0 dBm
        self.write_reg(0x11, 32)?;   // RX_PW_P0: 32-byte payload

        let addr = [0xE7u8; 5];
        self.write_cmd(W_REGISTER | 0x10, &addr)?; // TX_ADDR
        self.write_cmd(W_REGISTER | 0x0A, &addr)?; // RX_ADDR_P0 must match TX_ADDR for ACK

        thread::sleep(Duration::from_millis(2));
        Ok(())
    }

    pub fn send(&mut self, payload: &[u8; 32]) -> Result<(), Box<dyn Error>> {
        // clear TX_DS and MAX_RT flags from any previous transmission
        self.write_reg(0x07, 0x30)?;
        self.spi.write(&[FLUSH_TX])?;
        self.write_cmd(W_TX_PAYLOAD, payload)?;

        self.ce.set_high();
        thread::sleep(Duration::from_micros(15));
        self.ce.set_low();

        // poll STATUS until TX_DS (ack received) or MAX_RT (gave up)
        for _ in 0..200 {
            let status = self.status()?;
            if status & 0x20 != 0 {
                return Ok(()); // TX_DS: ACK received
            }
            if status & 0x10 != 0 {
                // MAX_RT: flush so the stale payload doesn't block the FIFO
                self.spi.write(&[FLUSH_TX])?;
                return Err("max retransmit reached — receiver not responding".into());
            }
            thread::sleep(Duration::from_micros(100));
        }
        Err("send timed out".into())
    }
}
