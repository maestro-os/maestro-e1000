//! This module implements the driver structure.

use kernel::device::driver::Driver;
use kernel::device::manager::PhysicalDevice;

/// Vendor ID for Intel.
const VENDOR_INTEL: u16 = 0x8086;
/// Device ID for emulated NICs.
const DEVICE_EMU: u16 = 0x100e;
// TODO Add real NICs

/// Structure representing the e1000 driver.
pub struct E1000Driver {}

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
				// TODO Insert a new device on the network manager?
			},

			_ => {},
		}
	}

	fn on_unplug(&self, _dev: &dyn PhysicalDevice) {
		// TODO Remove the corresponding device from the network manager?
	}
}
