use svd_parser::{BitRange, Access};

pub struct ModelDevice {
    pub peripherals: Vec<ModelPeripheral>,
    pub instances: Vec<ModelPeripheralInstance>,
}

pub struct ModelPeripheral {
    pub name: String,
    pub description: String,
    pub module_name: String,
    pub registers: Vec<FinalRegisterInfo>,
}

pub struct ModelPeripheralInstance {
    pub name: String,
    pub description: String,
    pub module_name: String,
    pub peripheral_module: String,
    pub base_address: u32, // Limitation of the parser
    pub reset_values: Vec<ResetValue>,
}

pub struct ResetValue {
    pub register: String,
    pub value: u64,
}

pub struct FinalFieldInfo {
    pub name: String,
    pub description: Option<String>,
    pub bit_range: BitRange,
    pub access: Access,
}

pub struct FinalRegisterInfo {
    pub name: String,
    pub description: Option<String>,
    pub address_offset: u32,
    pub properties: FinalRegisterProperties,
    pub fields: Vec<FinalFieldInfo>,
}

pub struct FinalRegisterProperties {
    pub size: u32,
    pub reset_value: u64,
    pub reset_mask: u64,
    pub access: Access,
}

impl FinalRegisterProperties {
    pub fn access_type_name(&self) -> &'static str {
        match self.access {
            Access::ReadOnly => "RORegister",
            Access::ReadWrite => "RWRegister",
            Access::ReadWriteOnce => "RWRegister",
            Access::WriteOnce => "WORegister",
            Access::WriteOnly => "WORegister",
        }
    }

    pub fn size_type_name(&self) -> &'static str {
        match self.size {
            8 => "u8",
            16 => "u16",
            32 => "u32",
            64 => "u64",
            other => unimplemented!("Unsupported size: {}", other),
        }
    }
}
