//! TODO doc

#![no_std]

extern crate kernel;

use kernel::module::version::Version;

kernel::module!("e1000", Version::new(1, 0, 0));

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

/// Called on module load
#[no_mangle]
pub extern "C" fn init() -> bool {
	// TODO For each ethernet devices, create a handler

	true
}

/// Called on module unload
#[no_mangle]
pub extern "C" fn fini() {}
