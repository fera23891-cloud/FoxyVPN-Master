pub struct HardwareKillSwitch {
    pub is_armed: bool,
}

impl HardwareKillSwitch {
    pub fn arm() -> Self {
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", "Disable-NetAdapterBinding -Name * -ComponentId ms_tcpip6 -ErrorAction SilentlyContinue"])
                .output();
        }
        Self { is_armed: true }
    }

    pub fn disarm(&mut self) {
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", "Enable-NetAdapterBinding -Name * -ComponentId ms_tcpip6 -ErrorAction SilentlyContinue"])
                .output();
        }
        self.is_armed = false;
    }
}
