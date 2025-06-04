use rppal::gpio::OutputPin;

pub struct Motor {
    pub a: OutputPin,
    pub b: OutputPin,
}

impl Motor {
    pub fn open(&mut self) {
        self.a.set_low();
        self.b.set_high();
    }

    pub fn close(&mut self) {
        self.b.set_low();
        self.a.set_high();
    }

    pub fn stop(&mut self) {
        self.a.set_low();
        self.b.set_low();
    }

    pub fn flip_polarity(&mut self) {
        std::mem::swap(&mut self.a, &mut self.b);
    }
}
