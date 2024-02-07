use gpio_cdev::{errors::Error as GpioError, Chip, LineHandle, LineRequestFlags};
use std::path::Path;
use std::time::Duration;

#[derive(Default, Debug)]
pub struct FakeDoor;

impl Door for FakeDoor {
    fn access_denied(&self) {
        println!("Simulating access denied.");
    }

    fn access_granted(&self) {
        println!("Simulating access granted.");
    }

    fn unlock(&self) {
        println!("Simulating unlocking door...");
    }

    fn lock(&self) {
        println!("Simulating locking door...");
    }
}

#[derive(Debug)]
pub struct ZuulDoor {
    motor_f: LineHandle,
    motor_r: LineHandle,
    led: LineHandle,
}

pub trait Door {
    /// Indicate to the user that they're forbidden
    fn access_denied(&self);
    /// Let the user in, lock the door behind them
    fn access_granted(&self);
    /// Unlock the door
    fn unlock(&self);
    /// Lock the door
    fn lock(&self);
}

impl ZuulDoor {
    pub fn new<P: AsRef<Path>>(
        gpio_dev_path: &P,
        motor_r_pin: u32,
        motor_f_pin: u32,
        led_pin: u32,
    ) -> Self {
        let mut chip = Chip::new(gpio_dev_path).expect("Bad GPIO path");
        let motor_f = chip
            .get_line(motor_f_pin)
            .and_then(|line| line.request(LineRequestFlags::OUTPUT, 0, "motor-forward"))
            .expect("Bad motor forward pin");
        let motor_r = chip
            .get_line(motor_r_pin)
            .and_then(|line| line.request(LineRequestFlags::OUTPUT, 0, "motor-reverse"))
            .expect("Bad motor reverse pin");
        let led = chip
            .get_line(led_pin)
            .and_then(|line| line.request(LineRequestFlags::OUTPUT, 0, "blinkenlight"))
            .expect("Bad led pin");
        Self {
            motor_f,
            motor_r,
            led,
        }
    }

    /// Wink out the LED once
    fn blink(&self) {
        self.led.set_value(0).expect("Couldn't write to LED");
        std::thread::sleep(Duration::from_millis(500));
        self.led.set_value(1).expect("Couldn't write to LED");
    }

    /// Move the motor in a particular direction until it reaches its limit
    fn drive(&self, primary: &LineHandle, secondary: &LineHandle) -> Result<(), GpioError> {
        // Drive the bolt:
        primary.set_value(1)?;
        secondary.set_value(0)?;
        // TODO: How long?
        std::thread::sleep(Duration::from_millis(50));
        // Park it:
        primary.set_value(0)?;
        secondary.set_value(0)?;
        Ok(())
    }
}

impl Door for ZuulDoor {
    fn unlock(&self) {
        self.drive(&self.motor_f, &self.motor_r)
            .expect("Couldn't write to motor");
        self.blink();
    }
    fn lock(&self) {
        self.drive(&self.motor_r, &self.motor_f)
            .expect("Couldn't write to motor");
        self.blink();
    }
    fn access_denied(&self) {
        self.blink();
        std::thread::sleep(Duration::from_millis(500));
        self.blink();
    }
    fn access_granted(&self) {
        self.unlock();
        log::info!("Opened the door!");
        std::thread::sleep(Duration::from_secs(5));
        log::info!("Closing the door!");
        self.lock();
    }
}
