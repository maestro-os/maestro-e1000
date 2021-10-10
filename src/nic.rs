//! This module implements the NIC structure, representing a e1000-compatible NIC.

use core::ptr;
use kernel::device::manager::PhysicalDevice;
use kernel::device::network::NetworkInterface;
use kernel::io;

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

	/// The type of BAR0, indicating the way to communicate with the NIC.
	bar0_type: u8,
	/// The registers' base address.
	mem_base_addr: *mut (),
	/// The flash memory's base address.
	flash_base_addr: *mut (),
	/// The IO space's base address.
	io_base_addr: *mut (),

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
		let bar0_type = bar0.get_type();
		let mem_base_addr = bar0.get_physical_address() as *mut ();

		let flash_base_addr = dev.get_bar(1).ok_or("Invalid BAR for NIC!")?
			.get_physical_address() as *mut ();
		let io_base_addr = dev.get_bar(2).ok_or("Invalid BAR for NIC!")?
			.get_physical_address() as *mut ();

		kernel::println!("{:p} {:p} {:p}", mem_base_addr, flash_base_addr, io_base_addr); // TODO rm
		let mut n = Self {
			status_reg,
			command_reg,

			bar0_type,
			mem_base_addr,
			flash_base_addr,
			io_base_addr,

			eeprom_exist: false,

			mac: [0; 6],
		};
		n.detect_eeprom();
		n.read_mac();
		Ok(n)
	}

	/// Sends a command to read at address `addr` in the NIC memory.
	fn read_command(&self, addr: u16) -> u32 {
		if self.bar0_type == 0 {
			// TODO Handle paging issues
			unsafe {
				let ptr = self.mem_base_addr.add(addr as _) as *mut u32;
				ptr::read_volatile(ptr)
			}
		} else {
			unsafe {
				io::outl(self.io_base_addr as _, addr as _);
				io::inl(self.io_base_addr as u16 + 4)
			}
		}
	}

	/// Sends a command to write the value `val` at address `addr` in the NIC memory.
	fn write_command(&self, addr: u16, val: u32) {
		if self.bar0_type == 0 {
			// TODO Handle paging issues
			unsafe {
				let ptr = self.mem_base_addr.add(addr as _) as *mut u32;
				ptr::write_volatile(ptr, val);
			}
		} else {
			unsafe {
				io::outl(self.io_base_addr as _, addr as _);
				io::outl(self.io_base_addr as u16 + 4, val)
			}
		}
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
