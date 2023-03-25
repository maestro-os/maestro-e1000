//! This module implements the driver structure.

use core::any::Any;
use kernel::device::bus::pci;
use kernel::device::driver::Driver;
use kernel::device::manager::PhysicalDevice;
use kernel::device::manager;
use kernel::net::Interface;
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
		let manager_ptr = manager::get_by_name("PCI").unwrap();
		let manager_mutex = manager_ptr.get().unwrap();
		let manager = &*manager_mutex.lock() as &dyn Any;
		let pci_manager = manager.downcast_ref::<pci::PCIManager>().unwrap();

		let s = Self {};

		for dev in pci_manager.get_devices() {
			s.on_plug(dev);
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
					Ok(nic) => {
						// TODO Insert a new device on the network manager?
						panic!();
					},

					Err(e) => {
						kernel::println!("e1000 error: {}", e);
					},
				}
			},

			_ => {},
		}
	}

	fn on_unplug(&self, _dev: &dyn PhysicalDevice) {
		// TODO
		todo!();
	}
}
