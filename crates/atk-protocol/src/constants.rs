/* Known constants for ATK / VXE devices. */

/* Vendor ID for ATK / VXE devices */
pub const VENDOR_ID_ATK: u16 = 0x3554;
pub const VENDOR_ID_ATK_ALT: u16 = 0x373B;

/* Product ID for ATK VXE R1 Pro Max in wired USB-C mode */
pub const PRODUCT_ID_R1_PRO_MAX_WIRED: u16 = 0xF58C;

/* Product ID for ATK VXE R1 Pro Max via 2.4 GHz receiver/dongle */
pub const PRODUCT_ID_R1_PRO_MAX_WIRELESS: u16 = 0xF58A;

/* Standard HID report size used for ATK vendor command packets (17 bytes wire) */
pub const ATK_WIRE_REPORT_SIZE: usize = 17;
pub const ATK_PAYLOAD_SIZE: usize = 16;

/* Target vendor collection */
pub const ATK_USAGE_PAGE_CONFIG: u16 = 0xFF02;
pub const ATK_USAGE_CONFIG: u16 = 0x0002;

/* Standard vendor Report ID for control packets */
pub const ATK_REPORT_ID_VENDOR: u8 = 0x08;

/* Protocol command opcodes */
pub const CMD_READ_EEPROM: u8 = 0x08;
pub const CMD_WRITE_EEPROM: u8 = 0x07;
pub const CMD_BATTERY: u8 = 0x04;
pub const CMD_VERSION: u8 = 0x12;
pub const CMD_CID_MID: u8 = 0x10;
pub const CMD_CURRENT_PROFILE: u8 = 0x0E;
pub const CMD_SET_PROFILE: u8 = 0x0F;
pub const CMD_RECEIVER_ONLINE: u8 = 0x03;

/* EEPROM Register addresses */
pub const ADDR_SYSTEM: u16 = 0x0000;
pub const ADDR_LIFT_OFF: u16 = 0x000A;
pub const ADDR_DPI_BASE: u16 = 0x000C;
pub const ADDR_STAGE_COLOR_BASE: u16 = 0x002C;
pub const ADDR_DPI_LIGHT: u16 = 0x004C;
pub const ADDR_ADVANCED: u16 = 0x00A9;
pub const ADDR_SENSOR_MODE: u16 = 0x00B5;
pub const ADDR_BUTTON_BASE: u16 = 0x0060;
