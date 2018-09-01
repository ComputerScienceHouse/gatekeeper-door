use sysfs_pwm::{Pwm, Result, Error};
use std::thread::sleep;
use std::time::Duration;

// PIN: EHRPWM0A (P1_36)
const PWM_CHIP: u32 = 0;
const PWM_NUMBER: u32 = 0;

// Buzzer config
const BUZZER_PERIOD: u32 = 250_000;
const BUZZER_DUTY_CYCLE: u32 = 125_000;

pub struct Beeper {
    pwm: Pwm,
}

impl Beeper {
    pub fn new() -> Result<Beeper> {
        let pwm = Pwm::new(PWM_CHIP, PWM_NUMBER)?;
        pwm.enable(true)?;
        pwm.set_period_ns(BUZZER_PERIOD)?;

        Ok(Beeper {
            pwm
        })
    }

    pub fn access_denied(&self) -> Result<()> {
        self.pwm.with_exported(|| {
            for _ in 0..3 {
                self.pwm.set_duty_cycle_ns(BUZZER_DUTY_CYCLE).unwrap();
                sleep(Duration::from_millis(80));
                self.pwm.set_duty_cycle_ns(0).unwrap();
                sleep(Duration::from_millis(80));
            }

            self.pwm.set_duty_cycle_ns(0)
        })
    }

    pub fn access_granted(&self) -> Result<()> {
        self.pwm.with_exported(|| {
            self.pwm.set_duty_cycle_ns(BUZZER_DUTY_CYCLE).unwrap();
            sleep(Duration::from_millis(200));
            self.pwm.set_duty_cycle_ns(0)
        })
    }
}
