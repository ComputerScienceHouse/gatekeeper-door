use sysfs_pwm::{Pwm};
use std::thread::sleep;
use std::time::Duration;

// PIN: EHRPWM0A (P1_36)
const PWM_CHIP: u32 = 0;
const PWM_NUMBER: u32 = 0;

// Beeper config
const BEEPER_PERIOD: u32 = 250_000;
const BEEPER_DUTY_CYCLE: u32 = 125_000;

pub struct Beeper {
    pwm: Pwm,
}

impl Beeper {
    pub fn new() -> Option<Self> {
        let pwm = Pwm::new(PWM_CHIP, PWM_NUMBER).unwrap();

        Some(Beeper {
            pwm
        })
    }

    fn setup(&self) {
        self.pwm.enable(true).unwrap();
        self.pwm.set_period_ns(BEEPER_PERIOD).unwrap();
    }

    pub fn access_denied(&self) {
        self.pwm.with_exported(|| {
            self.setup();

            for _ in 0..3 {
                self.pwm.set_duty_cycle_ns(BEEPER_DUTY_CYCLE).unwrap();
                sleep(Duration::from_millis(80));
                self.pwm.set_duty_cycle_ns(0).unwrap();
                sleep(Duration::from_millis(80));
            }

            self.pwm.set_duty_cycle_ns(0)
        }).unwrap();
    }

    pub fn access_granted(&self) {
        self.pwm.with_exported(|| {
            self.setup();

            self.pwm.set_duty_cycle_ns(BEEPER_DUTY_CYCLE).unwrap();
            sleep(Duration::from_millis(200));
            self.pwm.set_duty_cycle_ns(0)
        }).unwrap();
    }
}
