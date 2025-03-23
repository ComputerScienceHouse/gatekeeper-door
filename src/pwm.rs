use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::io::Write;

#[derive(Debug)]
pub struct Pwm {
    chip_id: usize,
    pin_id: usize,
}

impl Pwm {
    pub fn try_new(chip_id: usize, pin_id: usize) -> std::io::Result<Self> {
        let mut file = OpenOptions::new()
            .read(false)
            .write(true)
            .create(false)
            .open(format!("/sys/class/pwm/pwmchip{chip_id}/export"))?;
        match file.write_all(pin_id.to_string().as_bytes()) {
            Ok(()) => {}
            Err(err) if err.kind() == ErrorKind::ResourceBusy => {}
            Err(err) => {
                return Err(err);
            }
        }
        Ok(Self { chip_id, pin_id })
    }

    pub fn activate<'a>(&'a self) -> PwmSession<'a> {
        PwmSession {
            pwm: self,
            activated: false,
        }
    }
}

pub struct PwmSession<'a> {
    pwm: &'a Pwm,
    activated: bool,
}

impl Drop for PwmSession<'_> {
    fn drop(&mut self) {
        if self.activated {
            if let Err(err) = self.activate(false) {
                log::error!("Couldn't deactivate PWM session! {err}");
            }
        }
    }
}

impl PwmSession<'_> {
    fn activate(&mut self, state: bool) -> std::io::Result<()> {
        log::debug!("Setting activation of PWMSession to {state}!");
        if self.activated == state {
            log::debug!("Ignoring duplicate activation call... Already {state}");
            return Ok(());
        }

        // let mut file = OpenOptions::new()
        //     .read(false)
        //     .write(true)
        //     .create(false)
        //     .open(format!(
        //         "/sys/class/pwm/pwmchip{chip_id}/pwm{pin_id}/duty_cycle"
        //     ))?;

        // file.write_all()?;

        let chip_id = self.pwm.chip_id;
        let pin_id = self.pwm.pin_id;

        let mut file = OpenOptions::new()
            .read(false)
            .write(true)
            .create(false)
            .open(format!(
                "/sys/class/pwm/pwmchip{chip_id}/pwm{pin_id}/enable"
            ))?;

        file.write_all((state as usize).to_string().as_bytes())?;
        self.activated = state;
        log::debug!("Alrighty, enablement of pwm{pin_id} set to {state}!");
        Ok(())
    }

    pub fn set_period(&mut self, period: usize) -> std::io::Result<()> {
        let chip_id = self.pwm.chip_id;
        let pin_id = self.pwm.pin_id;
        let mut file = OpenOptions::new()
            .read(false)
            .write(true)
            .create(false)
            .open(format!(
                "/sys/class/pwm/pwmchip{chip_id}/pwm{pin_id}/period"
            ))?;

        file.write_all(period.to_string().as_bytes())?;
        log::debug!("Set PWM period to {period}!");
        self.activate(true)
    }
}

impl Drop for Pwm {
    fn drop(&mut self) {
        let chip_id = self.chip_id;
        let mut file = match OpenOptions::new()
            .read(false)
            .write(true)
            .create(false)
            .open(format!("/sys/class/pwm/pwmchip{chip_id}/unexport"))
        {
            Ok(file) => file,
            Err(err) => {
                log::error!("Couldn't unexport chip... {err}");
                return;
            }
        };
        match file.write_all(self.pin_id.to_string().as_bytes()) {
            Ok(()) => {}
            Err(err) => {
                log::error!("Couldn't unexport chip... {err}");
            }
        }
    }
}
