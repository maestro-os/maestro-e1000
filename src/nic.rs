//! This module implements the NIC structure, representing an e1000-compatible NIC.

use core::cmp::min;
use core::ptr::NonNull;
use kernel::device::bar::BAR;
use kernel::device::manager::PhysicalDevice;
use kernel::errno::Errno;
use kernel::net::BindAddress;
use kernel::net::MAC;
use kernel::net;
use kernel::util::container::vec::Vec;

/// Register address: EEPROM/Flash Control & Data
const REG_EECD: u16 = 0x10;
/// Register address: EEPROM Read Register
const REG_EERD: u16 = 0x14;

/// Register address: Receive Control
const REG_RCTL: u16 = 0x100;
/// Register address: Transmit Control
const REG_TCTL: u16 = 0x400;

/// Register address: Transmit Descriptor Address Low
const REG_RDBAL: u16 = 0x2800;
/// Register address: Transmit Descriptor Address High
const REG_RDBAH: u16 = 0x2804;
/// Register address: Receive Descriptor Length
const REG_RDLEN: u16 = 0x2808;
/// Register address: Receive Descriptor Head
const REG_RDH: u16 = 0x2810;
/// Register address: Receive Descriptor Tail
const REG_RDT: u16 = 0x2818;

/// Register address: Transmit Descriptor Address Low
const REG_TDBAL: u16 = 0x3800;
/// Register address: Transmit Descriptor Address High
const REG_TDBAH: u16 = 0x3804;
/// Register address: Transmit Descriptor Length
const REG_TDLEN: u16 = 0x3808;
/// Register address: Transmit Descriptor Head
const REG_TDH: u16 = 0x3810;
/// Register address: Transmit Descriptor Tail
const REG_TDT: u16 = 0x3818;

/// The number of receive descriptors.
const RX_DESC_COUNT: usize = 128; // TODO
/// The number of transmit descriptors.
const TX_DESC_COUNT: usize = 128; // TODO

/// Transmit descriptor command flag: End of Packet
const TX_CMD_EOP: u8 = 0x01;
/// Transmit descriptor command flag: Insertion of FCS
const TX_CMD_IFCS: u8 = 0x02;
/// Transmit descriptor command flag: Insert checksum
const TX_CMD_IC: u8 = 0x04;
/// Transmit descriptor command flag: Report status
const TX_CMD_RS: u8 = 0x08;
/// Transmit descriptor command flag: Report Packet Sent
const TX_CMD_RPS: u8 = 0x10;
/// Transmit descriptor command flag: VLAN Packet Enable
const TX_CMD_VLE: u8 = 0x40;
/// Transmit descriptor command flag: Interrupt Delay Enable
const TX_CMD_IDE: u8 = 0x80;

/// The receive descriptor.
#[derive(Default)]
#[repr(packed)]
struct RXDesc {
	/// The physical address of the data.
	addr: u64,
	/// The length of the data.
	length: u16,
	/// The packet's checksum.
	checksum: u16,
	/// Status flags.
	status: u8,
	/// Error flags.
	errors: u8,
	/// TODO doc
	special: u16,
}

// TODO: This is the legacy structure. Add support for the new version
/// The transmit descriptor.
#[derive(Default)]
#[repr(packed)]
struct TXDesc {
	/// The physical address of the data.
	addr: u64,
	/// The length of the data.
	length: u16,
	/// CheckSum Offset: the offset at which the checksum is to be placed in the given data.
	cso: u8,
	/// Command flags.
	cmd: u8,
	/// Status flags.
	status: u8,
	/// CheckSum Start: the offset at which computation of the checksum starts in the given data.
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
	eeprom_exists: bool,

	/// The NIC's mac address.
	mac: [u8; 6],

	/// The list of receive descriptors.
	rx_descs: Vec<NonNull<RXDesc>>, // FIXME cannot use vec since buffer must be aligned at 16
	/// The cursor in the receive ring buffer.
	rx_cur: usize,

	/// The list of transmit descriptors.
	tx_descs: Vec<NonNull<TXDesc>>, // FIXME cannot use vec since buffer must be aligned at 16
	/// The cursor in the transmit ring buffer.
	tx_cur: usize,
}

impl NIC {
	/// Creates a new instance using the given device.
	pub fn new(dev: &dyn PhysicalDevice) -> Result<Self, &str> {
		let status_reg = dev.get_status_reg().ok_or("Invalid PCI informations for NIC!")?;
		let command_reg = dev.get_command_reg().ok_or("Invalid PCI informations for NIC!")?;

		let bar0 = dev.get_bars()[0].clone().ok_or("Invalid BAR for NIC!")?;

		let mut n = Self {
			status_reg,
			command_reg,

			bar0,

			eeprom_exists: false,

			mac: [0; 6],

			rx_descs: Vec::with_capacity(RX_DESC_COUNT).unwrap(), // TODO handle error
			rx_cur: 0,

			tx_descs: Vec::with_capacity(TX_DESC_COUNT).unwrap(), // TODO handle error
			tx_cur: 0,
		};
		n.detect_eeprom();
		n.read_mac();
		n.init_desc();

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
	fn detect_eeprom(&mut self) {
		self.eeprom_exists = self.read_command(REG_EECD) & (1 << 8) != 0;
	}

	/// Reads from the EEPROM at address `addr`.
	fn eeprom_read(&self, addr: u8) -> u32 {
		// Acquire EEPROM
		self.write_command(REG_EECD, self.read_command(REG_EECD) | (1 << 6));

		// Specify read address
		self.write_command(REG_EERD, 1 | ((addr as u32) << 8));

		let data = if self.eeprom_exists {
			loop {
				let val = self.read_command(REG_EERD);
				if val & (1 << 4) != 0 {
					break (val >> 16) & 0xffff;
				}
			}
		} else {
			// TODO
			todo!();
		};

		// Release EEPROM
		self.write_command(REG_EECD, self.read_command(REG_EECD) & !(1 << 6));

		data
	}

	/// Reads the MAC address from the NIC's EEPROM.
	fn read_mac(&mut self) {
		let val = self.eeprom_read(0);
		self.mac[0] = (val & 0xff) as u8;
		self.mac[1] = ((val >> 8) & 0xff) as u8;

		let val = self.eeprom_read(1);
		self.mac[2] = (val & 0xff) as u8;
		self.mac[3] = ((val >> 8) & 0xff) as u8;

		let val = self.eeprom_read(2);
		self.mac[4] = (val & 0xff) as u8;
		self.mac[5] = ((val >> 8) & 0xff) as u8;
	}

	/// Initializes transmit and receive descriptors.
	fn init_desc(&self) {
		// TODO Init receive buffer

		// TODO Set receive buffer address

		// TODO Set receive buffer length

		// Set receive ring buffer head and tail
		self.write_command(REG_RDH, 0);
		self.write_command(REG_RDT, (RX_DESC_COUNT - 1) as _);

		// Set receive flags
		let flags = 0; // TODO
		self.write_command(REG_RCTL, flags);

		// TODO Init transmit buffer

		// TODO Set transmit buffer address

		// TODO Set transmit buffer length

		// Set transmit ring buffer head and tail
		self.write_command(REG_RDH, 0);
		self.write_command(REG_RDT, 0);

		// Set transmit flags
		let flags = 0; // TODO
		self.write_command(REG_TCTL, flags);
	}
}

impl net::Interface for NIC {
	fn get_name(&self) -> &[u8] {
		// TODO replace "TODO" with interface ID
		b"ethTODO"
	}

	fn is_up(&self) -> bool {
		// TODO
		todo!();
	}

	fn get_mac(&self) -> &MAC {
		&self.mac
	}

	fn get_addresses(&self) -> &[BindAddress] {
		// TODO
		todo!();
	}

	fn read(&mut self, _buff: &mut [u8]) -> Result<(u64, bool), Errno> {
		// TODO
		todo!();
	}

	fn write(&mut self, buff: &[u8]) -> Result<u64, Errno> {
		let mut i = 0;
		let mut desc_count = 0;

		// Fill descriptors
		while i < buff.len() && i < (TX_DESC_COUNT - 1) {
			let len = min(buff.len() - i, u16::MAX as usize);

			let desc = unsafe {
				self.tx_descs[self.tx_cur].as_mut()
			};
			desc.addr = buff.as_ptr() as _;
			desc.length = len as _;
			desc.cmd = 0; // TODO
			desc.status = 0;

			i += len;
			desc_count += 1;
		}

		// Update buffer tail
		self.tx_cur = (self.tx_cur + desc_count) % TX_DESC_COUNT;
		self.write_command(REG_TDT, self.tx_cur as _);

		// TODO sleep and wait for interrupt telling the whole queue has been processed

		Ok(i as _)
	}
}
