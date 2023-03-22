//! This module implements the NIC structure, representing a e1000-compatible NIC.

use kernel::device::bar::BAR;
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
	/// TODO doc
	status_reg: u16,
	/// TODO doc
	command_reg: u16,

	/// The BAR0 of the device.
	bar0: BAR,

	/// Tells whether the EEPROM exist.
	eeprom_exist: bool,

	/// The NIC's mac address.
	mac: [u8; 6],
}

impl NIC {
	/// Creates a new instance using the given device.
	pub fn new(dev: &dyn PhysicalDevice) -> Result<Self, &str> {
		let status_reg = dev.get_status_reg().ok_or("Invalid PCI informations for NIC!")?;
		let command_reg = dev.get_command_reg().ok_or("Invalid PCI informations for NIC!")?;

		let bar0 = dev.get_bar(0).ok_or("Invalid BAR for NIC!")?;

		let mut n = Self {
			status_reg,
			command_reg,

			bar0,

			eeprom_exist: false,

			mac: [0; 6],
		};
		n.detect_eeprom();
		n.read_mac();

		Ok(n)
	}

	/// Sends a command to read at address `addr` in the NIC memory.
	fn read_command(&self, addr: u16) -> u32 {
		self.bar0.read::<u32>(addr as _) as _
	}

	/// Sends a command to write the value `val` at address `addr` in the NIC memory.
	fn write_command(&self, addr: u16, val: u32) {
		self.bar0.write::<u32>(addr as _, val as _);
	}

	/// Detects whether the EEPROM exists.
	fn detect_eeprom(&mut self) -> bool {
		// TODO
		todo!();
	}

	/// Reads from the EEPROM at address `addr`.
	fn eeprom_read(&self, _addr: u8) -> u32 {
		// TODO
		todo!();
	}

	/// Reads the MAC address from the NIC's EEPROM.
	fn read_mac(&mut self) {
		// TODO If the EEPROM exists, read from it. Else, read from memory
		todo!();
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
		self.mac
	}

	// TODO Reading (use interrupts)

	fn write(&self, _buff: &[u8]) -> usize {
		// TODO
		todo!();
	}
}
