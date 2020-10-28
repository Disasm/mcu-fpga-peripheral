use svd_parser::{Device, Peripheral, RegisterProperties, RegisterCluster, Register, RegisterInfo};
use crate::types::*;


pub fn convert(device: &Device) -> ModelDevice {
    let mut peripherals = Vec::new();
    let mut instances = Vec::new();

    for peripheral in &device.peripherals {
        let default_register_properties = peripheral.default_register_properties.merge(&device.default_register_properties);

        let (p, i) = convert_peripheral(peripheral, &default_register_properties);
        peripherals.push(p);
        instances.push(i);
    }

    ModelDevice {
        peripherals,
        instances,
    }
}

pub fn convert_peripheral(peripheral: &Peripheral, default_register_properties: &RegisterProperties) -> (ModelPeripheral, ModelPeripheralInstance) {
    assert!(peripheral.derived_from.is_none(), "Derived peripherals are not supported");

    let doc = if let Some(description) = peripheral.description.as_ref() {
        description
    } else {
        &peripheral.name
    };

    let module_name = peripheral.name.to_ascii_lowercase();

    let mut registers = Vec::new();
    let mut reset_values = Vec::new();

    if let Some(regs) = peripheral.registers.as_ref() {
        for register_or_cluster in regs {
            let register = match register_or_cluster {
                RegisterCluster::Register(register) => register,
                RegisterCluster::Cluster(_) => unimplemented!("register clusters are not supported"),
            };
    
            let info = match register {
                Register::Single(info) => info,
                Register::Array(_, _) => unimplemented!("register arrays are not supported"),
            };

            let mut info = info.clone();
            info.update_properties(default_register_properties);

            let register = convert_register(&info);

            let reset_value = ResetValue {
                register: register.name.clone(),
                value: register.properties.reset_value,
            };

            registers.push(register);
            reset_values.push(reset_value);
        }
    }
    

    let p = ModelPeripheral {
        name: peripheral.name.clone(),
        description: doc.to_string(),
        module_name: module_name.clone(),
        registers,
    };
    let i = ModelPeripheralInstance {
        name: peripheral.name.clone(),
        description: doc.to_string(),
        module_name: module_name.clone(),
        peripheral_module: module_name,
        base_address: peripheral.base_address,
        reset_values,
    };
    (p, i)
}

fn convert_register(register: &RegisterInfo) -> FinalRegisterInfo {
    let mut final_fields = Vec::new();
    if let Some(fields) = register.fields.as_ref() {
        for field in fields {
            let final_field = FinalFieldInfo {
                name: field.name.clone(),
                description: field.description.clone(),
                bit_range: field.bit_range,
                access: field.access.or(register.access).unwrap(),
            };
            final_fields.push(final_field);
        }
    }

    let properties = FinalRegisterProperties {
        size: register.size.unwrap(),
        reset_value: register.reset_value.unwrap() as u64,
        reset_mask: register.reset_mask.unwrap() as u64,
        access: register.access.unwrap(),
    };

    let final_info = FinalRegisterInfo {
        name: register.name.clone(),
        description: register.description.clone(),
        address_offset: register.address_offset,
        properties,
        fields: final_fields,
    };

    final_info
}

trait RegisterPropertiesExt {
    fn merge(&self, parent: &RegisterProperties) -> RegisterProperties;
}

impl RegisterPropertiesExt for RegisterProperties {
    fn merge(&self, parent: &RegisterProperties) -> RegisterProperties {
        let mut merged = parent.clone();

        merged.size = self.size.or(parent.size);
        merged.reset_value = self.reset_value.or(parent.reset_value);
        merged.reset_mask = self.reset_mask.or(parent.reset_mask);
        merged.access = self.access.or(parent.access);
        merged
    }
}

trait RegisterInfoExt {
    fn merge_properties(&self, parent: &RegisterProperties) -> RegisterProperties;

    fn update_properties(&mut self, parent: &RegisterProperties);
}

impl RegisterInfoExt for RegisterInfo {
    fn merge_properties(&self, parent: &RegisterProperties) -> RegisterProperties {
        let mut merged = parent.clone();

        merged.size = self.size.or(parent.size);
        merged.reset_value = self.reset_value.or(parent.reset_value);
        merged.reset_mask = self.reset_mask.or(parent.reset_mask);
        merged.access = self.access.or(parent.access);
        merged
    }

    fn update_properties(&mut self, parent: &RegisterProperties) {
        self.size = self.size.or(parent.size);
        self.reset_value = self.reset_value.or(parent.reset_value);
        self.reset_mask = self.reset_mask.or(parent.reset_mask);
        self.access = self.access.or(parent.access);
    }
}