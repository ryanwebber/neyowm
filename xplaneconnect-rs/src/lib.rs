use std::sync::Mutex;

const DEFAULT_XPLANE_AIRCRAFT: std::ffi::c_char = 0;

mod bindings {
    #[repr(C)]
    #[cfg(target_os = "windows")]
    pub struct XPCSocket {
        port: std::ffi::c_ushort,
        xp_ip: [std::ffi::c_char; 16],
        xp_port: std::ffi::c_ushort,

        // https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Networking/WinSock/struct.SOCKET.html
        socket: usize,
    }

    #[repr(C)]
    #[cfg(not(target_os = "windows"))]
    pub struct XPCSocket {
        port: std::ffi::c_ushort,
        xp_ip: [std::ffi::c_char; 16],
        xp_port: std::ffi::c_ushort,
        socket: std::ffi::c_int,
    }

    impl Clone for XPCSocket {
        fn clone(&self) -> Self {
            Self {
                port: self.port,
                xp_ip: self.xp_ip,
                xp_port: self.xp_port,
                socket: self.socket,
            }
        }
    }

    impl Copy for XPCSocket {}

    #[repr(transparent)]
    pub struct ErrorCode(std::ffi::c_int);

    impl ErrorCode {
        pub fn ok(self) -> Result<(), ()> {
            if self.0 == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    extern "C" {
        pub fn openUDP(addr: *const std::ffi::c_char) -> XPCSocket;
        pub fn closeUDP(socket: XPCSocket);

        pub fn getPOSI(
            socket: XPCSocket,
            buffer: *mut [std::ffi::c_double; 7],
            ac: std::ffi::c_char,
        ) -> ErrorCode;

        pub fn getCTRL(
            socket: XPCSocket,
            buffer: *mut [std::ffi::c_float; 7],
            ac: std::ffi::c_char,
        ) -> ErrorCode;

        pub fn getTERR(
            socket: XPCSocket,
            posi: *mut [std::ffi::c_double; 3],
            buffer: *mut [std::ffi::c_double; 11],
            ac: std::ffi::c_char,
        ) -> ErrorCode;

        pub fn sendCTRL(
            socket: XPCSocket,
            buffer: *const [std::ffi::c_float; 7],
            size: std::ffi::c_int,
            ac: std::ffi::c_char,
        ) -> ErrorCode;
    }
}

pub struct XPlaneConnection {
    socket: bindings::XPCSocket,
}

impl XPlaneConnection {
    pub fn open(addr: std::net::Ipv4Addr) -> std::sync::Mutex<Self> {
        let addr = addr.to_string();
        let addr = std::ffi::CString::new(addr).unwrap();
        let socket = unsafe { bindings::openUDP(addr.as_ptr()) };
        Mutex::new(Self { socket })
    }

    pub fn close(self) {
        unsafe { bindings::closeUDP(self.socket) };
        std::mem::drop(self);
    }

    pub fn read_position(&self) -> Result<PositionInfo, ()> {
        let mut buffer: [std::ffi::c_double; 7] = [0.0; 7];
        let result =
            unsafe { bindings::getPOSI(self.socket, &mut buffer, DEFAULT_XPLANE_AIRCRAFT) };

        result.ok().map(|_| PositionInfo {
            latitude: buffer[0] as f64,
            longitude: buffer[1] as f64,
            altitude: buffer[2] as f64,
            pitch: buffer[3] as f64,
            roll: buffer[4] as f64,
            yaw: buffer[5] as f64,
            gear: buffer[6] as f64,
        })
    }

    pub fn read_controls(&self) -> Result<ControlSurface, ()> {
        let mut buffer: [std::ffi::c_float; 7] = [0.0; 7];
        let result =
            unsafe { bindings::getCTRL(self.socket, &mut buffer, DEFAULT_XPLANE_AIRCRAFT) };

        result.ok().map(|_| ControlSurface {
            aileron: buffer[0] as f64,
            elevator: buffer[1] as f64,
            rudder: buffer[2] as f64,
            throttle: buffer[4] as f64,
            flaps: buffer[5] as f64,
            speedbrake: buffer[6] as f64,
        })
    }

    pub fn read_terrain(&self) -> Result<TerrainInfo, ()> {
        let mut posi: [std::ffi::c_double; 3] = [-998.0; 3];
        let mut buffer: [std::ffi::c_double; 11] = [0.0; 11];
        let result = unsafe {
            bindings::getTERR(self.socket, &mut posi, &mut buffer, DEFAULT_XPLANE_AIRCRAFT)
        };

        result.ok().map(|_| TerrainInfo {
            latitude: buffer[0] as f64,
            longitude: buffer[1] as f64,
            elevation: buffer[2] as f64,
            normal: (buffer[3] as f64, buffer[4] as f64, buffer[5] as f64),
            velocity: (buffer[6] as f64, buffer[7] as f64, buffer[8] as f64),
            wet: buffer[9] != 0.0,
        })
    }

    pub fn write_controls(&self, controls: SetControlSurface) -> Result<(), ()> {
        let buffer = [
            controls.elevator.unwrap_or(-998.0) as f32,
            controls.aileron.unwrap_or(-998.0) as f32,
            controls.rudder.unwrap_or(-998.0) as f32,
            -998.0f32, // gear
            controls.throttle.unwrap_or(-998.0) as f32,
            controls.flaps.unwrap_or(-998.0) as f32,
            controls.speedbrake.unwrap_or(-998.0) as f32,
        ];

        let result = unsafe {
            bindings::sendCTRL(
                self.socket,
                &buffer,
                buffer.len() as i32,
                DEFAULT_XPLANE_AIRCRAFT,
            )
        };

        result.ok()
    }
}

impl Drop for XPlaneConnection {
    fn drop(&mut self) {
        unsafe { bindings::closeUDP(self.socket) };
    }
}

#[derive(Debug, Clone)]
pub struct PositionInfo {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub pitch: f64,
    pub roll: f64,
    pub yaw: f64,
    pub gear: f64,
}

#[derive(Debug, Clone)]
pub struct ControlSurface {
    pub aileron: f64,
    pub elevator: f64,
    pub rudder: f64,
    pub throttle: f64,
    pub flaps: f64,
    pub speedbrake: f64,
}

#[derive(Debug, Clone)]
pub struct SetControlSurface {
    pub aileron: Option<f64>,
    pub elevator: Option<f64>,
    pub rudder: Option<f64>,
    pub throttle: Option<f64>,
    pub flaps: Option<f64>,
    pub speedbrake: Option<f64>,
}

impl Default for SetControlSurface {
    fn default() -> Self {
        Self {
            aileron: None,
            elevator: None,
            rudder: None,
            throttle: None,
            flaps: None,
            speedbrake: None,
        }
    }
}

// Lat, Lon, Alt, Nx, Ny, Nz, Vx, Vy, Vz, wet
#[derive(Debug, Clone)]
pub struct TerrainInfo {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
    pub normal: (f64, f64, f64),
    pub velocity: (f64, f64, f64),
    pub wet: bool,
}

#[cfg(test)]
mod test {
    use std::net::Ipv4Addr;

    #[test]
    fn test_linkage() {
        let _ = super::XPlaneConnection::open(Ipv4Addr::LOCALHOST);
    }
}
