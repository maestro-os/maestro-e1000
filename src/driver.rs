//! This module implements the driver structure.

use core::any::Any;
use kernel::device::bus::pci::PCIManager;
use kernel::device::driver::Driver;
use kernel::device::manager;
use kernel::device::manager::PhysicalDevice;
use nic::NIC;

/// Vendor ID for Intel.
const VENDOR_INTEL: u16 = 0x8086;
/// Device ID for emulated NICs.
const DEVICE_EMU: u16 = 0x100e;
// TODO Add real NICs

/// Structure representing the e1000 driver.
pub struct E1000Driver {}

impl E1000Driver {
    /// Creates a new instance.
    pub fn new() -> Self {
        let s = Self {};

        let manager = manager::get::<PCIManager>();
        if let Some(manager_mutex) = manager {
            let manager = manager_mutex.lock();
            let pci_manager = (&*manager as &dyn Any)
                .downcast_ref::<PCIManager>()
                .unwrap();

            for dev in pci_manager.get_devices() {
                s.on_plug(dev);
            }
        }

        s
    }
}

impl Driver for E1000Driver {
    fn get_name(&self) -> &str {
        "e1000"
    }

    fn on_plug(&self, dev: &dyn PhysicalDevice) {
        if dev.get_vendor_id() != VENDOR_INTEL {
            return;
        }

        match dev.get_device_id() {
            // TODO Add real NICs
            DEVICE_EMU => {
                match NIC::new(dev) {
                    Ok(_nic) => {
                        // TODO Insert a new device on the network manager?
                        panic!();
                    }

                    Err(e) => {
                        kernel::println!("e1000 error: {}", e);
                    }
                }
            }

            _ => {}
        }
    }

    fn on_unplug(&self, _dev: &dyn PhysicalDevice) {
        // TODO
        todo!();
    }
}
