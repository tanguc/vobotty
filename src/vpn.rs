pub struct VpnCredentials {
    email: String,
    password: String,
    preferred_zone: String,
}

impl VpnCredentials {
    pub fn new(email: String, password: String) -> Self {
        Self {
            email,
            password,
            preferred_zone: "FR".to_string()
        }
    }
}

///
/// VPN engine
pub struct Vpn {
    credentials: VpnCredentials,
}

impl Vpn {
    pub fn new(credentials: VpnCredentials) -> Result<Self, String> {
        Ok(Self { credentials })
    }

    pub fn connect(&self) -> Result<String, String> {
        println!("Start connecting to VPN");

        Ok("127.0.0.1".to_string())
    }
}