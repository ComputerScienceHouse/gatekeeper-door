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
        let pwm = Pwm::new(PWM_CHIP, PWM_NUMBER);

        if pwm.is_ok() {
            Some(Beeper {
                pwm: pwm.unwrap(),
            })
        } else {
            None
        }
    }

    fn setup(&self) {
        println!("Set period PWM");
        self.pwm.set_period_ns(BEEPER_PERIOD).unwrap();
        println!("Set duty cycle PWM");
        self.pwm.set_duty_cycle_ns(BEEPER_DUTY_CYCLE).unwrap();
    }

    pub fn access_denied(&self) {
        println!("Attempt export PWM");
        self.pwm.with_exported(|| {
            self.setup();

            for _ in 0..3 {
                println!("Enable PWM");
                self.pwm.enable(true).unwrap();
                sleep(Duration::from_millis(80));
                println!("Disable PWM");
                self.pwm.enable(false).unwrap();
                sleep(Duration::from_millis(80));
            }

            Ok(())
        }).unwrap();
    }

    pub fn access_granted(&self) {
        self.pwm.with_exported(|| {
            self.setup();

            println!("Enable PWM");
            self.pwm.enable(true).unwrap();
            sleep(Duration::from_millis(80));
            println!("Disable PWM");
            self.pwm.enable(false).unwrap();

            Ok(())
        }).unwrap();
    }
}
