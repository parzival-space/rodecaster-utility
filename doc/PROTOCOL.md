# HID Device Protocol
This documentation is incomplete. I will try to update it as I figure out how the RODECaster Pro II's USB protocol works.

## Frame Structure
Message _can_ be variable in length, but the official App **always** sends 256 byte messages, with the remaining bytes being padded with `0x00`.
A message has the following structure:

| Offset | Length (bytes) | Example               | Description / Note                                                                                    |
|--------|----------------|-----------------------|-------------------------------------------------------------------------------------------------------|
| 0      | 1              | `0x03`                | MAGIC Number                                                                                          |
| 1      | 1              | `0x17`                | Message length (excluding magic, length and seperator bytes)                                          |
| 2      | 3              | `0x00 0x00 0x00`      | Seperator, always seems to be `0x00`                                                                  |
| 5      | 4              | `0x01 0x01 0x01 0x01` | Unknown, always seems to be `0x01`                                                                    |
| 9      | 1              | `0x05`                | Unknown, seems to be data type of configuration value. `0x0d` used for boolean? `0x05` used for enum? |
| 10     | n              | `metering`            | Command name, ASCII encoded.                                                                          |
| 10 + n | 1              | `0x00`                | Null terminator for command name.                                                                     |
| 11 + n | m              | `0x01`                | Unsure, Configuration value, format seems to depend on the data type byte at offset 9.                |

### Initialization Packet
The initialization packet differs from the usual frame structure, and seems to be required to be sent to the device before it starts sending messages to the host. 
It has the following structure:

| Offset | Length (bytes) | Example               | Description / Note                               |
|--------|----------------|-----------------------|--------------------------------------------------|
| 0      | 1              | `0x03`                | MAGIC Number                                     |
| 1      | 1              | `0x04`                | Message length (excluding magic and length byte) |
| 2      | 3              | `0x00 0x00 0x00`      | Seperator, always seems to be `0x00`             |
| 5      | 4              | `0xAD 0x10 0xA7 0xB0` | Unknown.                                         |