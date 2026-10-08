use windows::Win32::{
    Foundation::HMODULE,
    Graphics::{
        Direct3D::{
            D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL, D3D_FEATURE_LEVEL_11_0,
            D3D_FEATURE_LEVEL_11_1,
        },
        Direct3D11::{
            D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext,
            D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
        },
    },
};

pub struct GpuContext {
    _device: ID3D11Device,
    _context: ID3D11DeviceContext,
    feature_level: D3D_FEATURE_LEVEL,
}

impl GpuContext {
    pub fn try_create() -> Option<Self> {
        let feature_levels = [D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_11_0];
        let mut device = None;
        let mut context = None;
        let mut feature_level = D3D_FEATURE_LEVEL::default();

        unsafe {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                Some(&feature_levels),
                D3D11_SDK_VERSION,
                Some(&mut device),
                Some(&mut feature_level),
                Some(&mut context),
            )
            .ok()?;
        }

        Some(Self {
            _device: device?,
            _context: context?,
            feature_level,
        })
    }

    #[allow(dead_code)]
    pub fn feature_level(&self) -> D3D_FEATURE_LEVEL {
        self.feature_level
    }
}
