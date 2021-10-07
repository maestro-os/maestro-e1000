//! This module implements the driver structure.

use kernel::device::bus::pci;
use kernel::device::driver::Driver;
use kernel::device::manager::PhysicalDevice;
use kernel::device::manager;
use kernel::device::network::NetworkInterface;
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
		let ptr = manager::get_by_name("pci").unwrap();
		let guard = ptr.get_mut().unwrap().lock(true);
		let pci_manager = unsafe {
			&*(guard.get() as *const _ as *const pci::PCIManager)
		};

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
				let nic = NIC::new(dev).unwrap(); // TODO Handle properly

				// TODO rm
				let mac = nic.get_mac();
				kernel::print!("MAC: {}", mac[0]);
				for i in 1..6 {
					kernel::print!(":{}", mac[i]);
				}

				// TODO Insert a new device on the network manager?
			},

			_ => {},
		}
	}

	fn on_unplug(&self, _dev: &dyn PhysicalDevice) {
		// TODO Remove the corresponding device from the network manager?
	}
}
