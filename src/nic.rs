//! This module implements the NIC structure, representing an e1000-compatible NIC.

use core::cmp::min;
use core::mem::size_of;
use core::slice;
use kernel::device::bar::BAR;
use kernel::device::manager::PhysicalDevice;
use kernel::errno::Errno;
use kernel::memory::buddy;
use kernel::memory;
use kernel::net::BindAddress;
use kernel::net::MAC;
use kernel::net;

/// The number of receive descriptors.
const RX_DESC_COUNT: usize = 128;
/// The size of a receive descriptor's buffer.
const RX_BUFF_SIZE: usize = 16384;
/// The number of transmit descriptors.
const TX_DESC_COUNT: usize = 128;
/// The size of a transmit descriptor's buffer.
const TX_BUFF_SIZE: usize = 16384;

/// Register address: EEPROM/Flash Control & Data
const REG_EECD: u16 = 0x10;
/// Register address: EEPROM Read Register
const REG_EERD: u16 = 0x14;

/// Register address: Receive Control
const REG_RCTL: u16 = 0x100;
/// Register address: Transmit Control
const REG_TCTL: u16 = 0x400;
/// Register address: Transmit IPG
const REG_TIPG: u16 = 0x410;

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

/// RCTL flag: Receiver Enable
const RCTL_EN: u32 = 1 << 1;
/// RCTL flag: Store Bad Packets
const RCTL_SBP: u32 = 1 << 2;
/// RCTL flag: Unicast Promiscuous Enabled
const RCTL_UPE: u32 = 1 << 3;
/// RCTL flag: Multicast Promiscuous Enabled
const RCTL_MPE: u32 = 1 << 4;
/// RCTL flag: Long Packet Reception Enable
const RCTL_LPE: u32 = 1 << 5;
/// RCTL flag: Broadcast Accept Mode
const RCTL_BAM: u32 = 1 << 15;
/// RCTL flag: VLAN Filter Enable
const RCTL_VFE: u32 = 1 << 18;
/// RCTL flag: Canonical Form Indicator Enable
const RCTL_CFIEN: u32 = 1 << 19;
/// RCTL flag: Canonical Form Indicator bit value
const RCTL_CFI: u32 = 1 << 20;
/// RCTL flag: Discard Pause Frames
const RCTL_DPF: u32 = 1 << 22;
/// RCTL flag: Pass MAC Control Frames
const RCTL_PMCF: u32 = 1 << 23;
/// RCTL flag: Buffer Size Extension
const RCTL_BSEX: u32 = 1 << 25;
/// RCTL flag: Strip Ethernet CRC from incoming packet
const RCTL_SECRC: u32 = 1 << 26;

/// TCTL flag: Transmit Enable
const TCTL_EN: u32 = 1 << 1;
/// TCTL flag: Pad Short Packets
const TCTL_PSP: u32 = 1 << 3;
/// TCTL flag: Software XOFF Transmission
const TCTL_SWXOFF: u32 = 1 << 22;
/// TCTL flag: Re-transmit on Late Collission
const TCTL_RTLC: u32 = 1 << 24;
/// TCTL flag: No Re-transmit on underrun
const TCTL_NRTU: u32 = 1 << 25;

/// Receive descriptor status flag: Descriptor Done
const RX_STA_DD: u8 = 1 << 0;
/// Receive descriptor status flag: End of Packet
const RX_STA_EOP: u8 = 1 << 1;
/// Receive descriptor status flag: Ignore Checksum Indication
const RX_STA_IXSM: u8 = 1 << 2;
/// Receive descriptor status flag: Packet is 802.1Q
const RX_STA_VP: u8 = 1 << 3;
/// Receive descriptor status flag: TCP Checksum Calculated on Packet
const RX_STA_TCPCS: u8 = 1 << 5;
/// Receive descriptor status flag: IP Checksum Calculated on Packet
const RX_STA_IPCS: u8 = 1 << 6;
/// Receive descriptor status flag: Passed in-exact filter
const RX_STA_PIF: u8 = 1 << 7;

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

/// Transmit descriptor status flag: Descriptor Done
const TX_STA_DD: u8 = 1 << 0;
/// Transmit descriptor status flag: Excess Collisions
const TX_STA_EC: u8 = 1 << 1;
/// Transmit descriptor status flag: Late Collision
const TX_STA_LC: u8 = 1 << 2;
/// Transmit descriptor status flag: Transmit Underrun
const TX_STA_TU: u8 = 1 << 3;

// TODO caches need to be flushed before reading/writing from/to receive/transmit buffers

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
	rx_descs: *mut RXDesc,
	/// The cursor in the receive ring buffer.
	rx_cur: usize,

	/// The list of transmit descriptors.
	tx_descs: *mut TXDesc,
	/// The cursor in the transmit ring buffer.
	tx_cur: usize,
}

impl NIC {
	/// Creates a new instance using the given device.
	pub fn new(dev: &dyn PhysicalDevice) -> Result<Self, &str> {
		let status_reg = dev.get_status_reg().ok_or("Invalid PCI informations for NIC")?;
		let command_reg = dev.get_command_reg().ok_or("Invalid PCI informations for NIC")?;

		let bar0 = dev.get_bars()[0].clone().ok_or("Invalid BAR for NIC")?;

		let rx_order = buddy::get_order(RX_DESC_COUNT * size_of::<RXDesc>());
		let rx_descs = buddy::alloc_kernel(rx_order)
			.map_err(|_| "Memory allocation failed")? as *mut RXDesc;

		let tx_order = buddy::get_order(TX_DESC_COUNT * size_of::<TXDesc>());
		let tx_descs = buddy::alloc_kernel(tx_order)
			.map_err(|_| "Memory allocation failed")? as *mut TXDesc;

		let mut n = Self {
			status_reg,
			command_reg,

			bar0,

			eeprom_exists: false,

			mac: [0; 6],

			rx_descs,
			rx_cur: 0,

			tx_descs,
			tx_cur: 0,
		};
		n.detect_eeprom();
		n.read_mac();
		n.init_desc().map_err(|_| "Memory allocation failed")?;

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
	fn init_desc(&self) -> Result<(), Errno> {
		// Init receive ring buffer
		let rx_buffs_order = buddy::get_order(RX_DESC_COUNT * RX_BUFF_SIZE);
		let rx_buffs = buddy::alloc(rx_buffs_order, buddy::FLAG_ZONE_TYPE_KERNEL)?;
		for i in 0..RX_DESC_COUNT {
			let desc = unsafe {
				&mut *self.rx_descs.add(i)
			};

			*desc = RXDesc::default();
			desc.addr = unsafe {
				rx_buffs.add(i * RX_BUFF_SIZE)
			} as _;
		}

		// Set receive ring buffer address
		let phys_ptr = memory::kern_to_phys(self.rx_descs);
		self.write_command(REG_RDBAL, ((phys_ptr as u64) & 0xffffffff) as _);
		self.write_command(REG_RDBAH, ((phys_ptr as u64) >> 32) as _);

		// Set receive ring buffer length
		self.write_command(REG_RDLEN, (RX_DESC_COUNT * size_of::<RXDesc>()) as u32);

		// Set receive ring buffer head and tail
		self.write_command(REG_RDH, 0);
		self.write_command(REG_RDT, (RX_DESC_COUNT - 1) as _);

		// Set receive flags
		let mut flags = RCTL_EN | RCTL_UPE | RCTL_MPE | RCTL_BAM;
		flags |= RCTL_BSEX | (0b01 << 16); // 16K buffer
		self.write_command(REG_RCTL, flags);

		// Init transmit ring buffer
		for i in 0..TX_DESC_COUNT {
			let desc = unsafe {
				&mut *self.tx_descs.add(i)
			};

			*desc = TXDesc::default();
			desc.status = TX_STA_DD;
		}

		// Set transmit ring buffer address
		let phys_ptr = memory::kern_to_phys(self.tx_descs);
		self.write_command(REG_TDBAL, ((phys_ptr as u64) & 0xffffffff) as _);
		self.write_command(REG_TDBAH, ((phys_ptr as u64) >> 32) as _);

		// Set transmit ring buffer length
		self.write_command(REG_TDLEN, (TX_DESC_COUNT * size_of::<TXDesc>()) as u32);

		// Set transmit ring buffer head and tail
		self.write_command(REG_TDH, 0);
		self.write_command(REG_TDT, 0);

		// Set transmit flags
		let retry_count = 0xf;
		let collision_dist = 0x200;
		let flags = TCTL_EN | (retry_count << 4) | (collision_dist << 12);
		self.write_command(REG_TCTL, flags);
		let flags = 0; // TODO
		self.write_command(REG_TIPG, flags);

		Ok(())
	}
}

impl net::Interface for NIC {
	fn get_name(&self) -> &[u8] {
		b"eth"
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

	fn read(&mut self, buff: &mut [u8]) -> Result<u64, Errno> {
		let mut i = 0;
		let mut prev_cursor = None;

		while i < buff.len() {
			let desc = unsafe {
				&mut *self.tx_descs.add(self.rx_cur)
			};
			if desc.status & RX_STA_DD == 0 {
				break;
			}

			let addr = memory::kern_to_virt(desc.addr as *const u8);
			let len = min(buff.len() - i, desc.length as usize);
			let slice = unsafe {
				slice::from_raw_parts(addr, len)
			};
			buff[i..(i + len)].copy_from_slice(slice);

			desc.status = 0;

			i += len;

			prev_cursor = Some(self.rx_cur);
			self.rx_cur = (self.rx_cur + 1) % RX_DESC_COUNT;
		}

		if let Some(prev_cursor) = prev_cursor {
			self.write_command(REG_RDT, prev_cursor as _);
		}

		Ok(i as _)
	}

	fn write(&mut self, buff: &[u8]) -> Result<u64, Errno> {
		let mut i = 0;

		// Fill descriptors
		while i < buff.len() && i < (TX_DESC_COUNT - 1) {
			let len = min(buff.len() - i, TX_BUFF_SIZE);

			let desc = unsafe {
				&mut *self.tx_descs.add(self.tx_cur)
			};
			desc.addr = buff.as_ptr() as _;
			desc.length = len as _;
			desc.cmd = TX_CMD_RS;
			desc.status = 0;

			if i + len >= buff.len() {
				desc.cmd |= TX_CMD_EOP | TX_CMD_IFCS;
			}

			i += len;

			self.tx_cur = (self.tx_cur + 1) % TX_DESC_COUNT;
		}

		// Update buffer tail
		self.write_command(REG_TDT, self.tx_cur as _);

		// TODO sleep and wait for interrupt telling the whole queue has been processed

		Ok(i as _)
	}
}

impl Drop for NIC {
	fn drop(&mut self) {
		let rx_buffs = unsafe {
			(*self.rx_descs).addr
		} as _;
		let rx_buffs_order = buddy::get_order(RX_DESC_COUNT * RX_BUFF_SIZE);
		buddy::free(rx_buffs, rx_buffs_order);

		let rx_order = buddy::get_order(RX_DESC_COUNT * size_of::<RXDesc>());
		buddy::free_kernel(self.rx_descs as _, rx_order);

		let tx_order = buddy::get_order(TX_DESC_COUNT * size_of::<TXDesc>());
		buddy::free_kernel(self.tx_descs as _, tx_order);
	}
}
