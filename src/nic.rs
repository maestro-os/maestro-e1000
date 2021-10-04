//! This module implements the NIC structure, representing a e1000-compatible NIC.

use kernel::device::manager::PhysicalDevice;
use kernel::device::network::NetworkInterface;

/// The receive descriptor.
#[repr(packed)]
struct RXDesc {
	/// The physical address of the data.
	addr: u64,
	/// The length of the data.
	length: u16,
	/// TODO doc
	checksum: u16,
	/// TODO doc
	status: u8,
	/// TODO doc
	errors: u8,
	/// TODO doc
	special: u16,
}

/// The transmit descriptor.
#[repr(packed)]
struct TXDesc {
	/// The physical address of the data.
	addr: u64,
	/// The length of the data.
	length: u16,
	/// TODO doc
	cso: u8,
	/// TODO doc
	cmd: u8,
	/// TODO doc
	status: u8,
	/// TODO doc
	css: u8,
	/// TODO doc
	special: u16,
}

/// Structure representing a Network Interface Card.
pub struct NIC {
	// TODO BAR addresses

	mac: [u8; 6],
}

impl NIC {
	/// Creates a new instance using the given device.
	pub fn new(_dev: &dyn PhysicalDevice) -> Self {
		let mut n = Self {
			// TODO Read BARs from the device

			mac: [0; 6],
		};
		n.read_mac();
		n
	}

	/// Reads from the EEPROM at address `addr`.
	fn eeprom_read(_addr: u8) -> u32 {
		// TODO
		todo!();
	}

	/// Reads the MAC address from the NIC's EEPROM and saves it in the structure.
	fn read_mac(&mut self) {
		// TODO If the EEPROM exists, read from it. Else, read from memory
	}

	/// Receives data using the given descriptor.
	fn receive(&self, _rx_desc: &mut RXDesc) {
		// TODO
		todo!();
	}

	/// Transmits the data of the given descriptor.
	fn transmit(&self, _tx_desc: &mut TXDesc) {
		// TODO
		todo!();
	}
}

impl NetworkInterface for NIC {
	fn get_mac(&self) -> [u8; 6] {
		// TODO
		todo!();
	}

	// TODO Reading (use interrupts)

	fn write(&self, _buff: &[u8]) -> usize {
		// TODO
		todo!();
	}
}
