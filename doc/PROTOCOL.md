# HID Device Protocol
This documentation is incomplete. I will try to update it as I figure out how the RODECaster Pro II's USB protocol works.

## Frame Structure
Message _can_ be variable in length, but the official App **always** sends 256 byte messages, with the remaining bytes being padded with `0x00`.
A message has the following structure:

| Offset | Length (bytes) | Example          | Description / Note                                                                                 |
|--------|----------------|------------------|----------------------------------------------------------------------------------------------------|
| 0      | 1              | `0x03` or `0x04` | MAGIC Number. Host-to-device `0x03` or device-to-host `0x04`.                                      |
| 1      | 1              | `0x17`           | Message length (excluding magic, length and seperator bytes)                                       |
| 2      | 3              | `0x00 0x00 0x00` | Seperator, always seems to be 3x `0x00`                                                            |
| 6      | 1              | `0x01`           | Message Metadata.<br> Always `0x01` in case of control message, differs for initialization packet. |
| 7      | 1              | `0x01`           | Message Metadata.<br> Always `0x01` in case of control message, differs for initialization packet. |
| 8      | 1              | `0x01`           | Message Metadata.<br> Always `0x01` in case of control message, differs for initialization packet. |
| 9      | 1              | `0x01`           | Message Metadata.<br> Always `0x01` in case of control message, differs for initialization packet. |
| 10     | 1              | `0x01`           | Message Metadata.<br>                                                                              |
| 11     | n              | `0x01...`        | ASCII encoded command. Init-Command does not use this.                                             |
| 11+n   | 1              | `0x00`           | Null-Terminator.                                                                                   |
| 12+n   | m              | `0x01 0x01 0x02` | Command data.                                                                                      |

## Initialization Packet
Before the RODECaster Pro II accepts control messages, it first needs to be initialized.
After that the device will respond with multiple messages containing the current state of the device.
It will also send messages every time an interaction with the device is made, e.g. a button is pressed or a fader is moved.

The initialization packet has the following structure:

| Offset | Length (bytes) | Value            | Description / Note |
|--------|----------------|------------------|--------------------|
| 0      | 1              | `0x03`           | MAGIC Number       |
| 1      | 1              | `0x04`           | Message length     |
| 2      | 3              | `0x00 0x00 0x00` | Seperator          |
| 5      | 1              | `0xAD`           | Unknown            |
| 6      | 1              | `0x10`           | Unknown            |
| 7      | 1              | `0xA7`           | Unknown            |
| 8      | 1              | `0xB0`           | Unknown            |

## Control Messages
The messages are sorted based on where they are used in the official App.

### Display

#### Automatic Brightness Control
This command enables or disables the automatic brightness control of the display.

| Offset | Length (bytes) | Value                       | Description / Note    |
|--------|----------------|-----------------------------|-----------------------|
| 0      | 1              | `0x03`                      | MAGIC Number          |
| 1      | 1              | `0x17`                      | Message length        |
| 2      | 3              | `0x00 0x00 0x00`            | Seperator.            |
| 6      | 1              | `0x01`                      |                       |
| 7      | 1              | `0x01`                      |                       |
| 8      | 1              | `0x01`                      |                       |
| 9      | 1              | `0x01`                      |                       |
| 10     | 1              | `0x05`                      | Unknown               |
| 11     | 14             | `autoBrightness`            | ASCII Command         |
| 25     | 1              | `0x00`                      | Null-Terminator       |
| 26     | 1              | `0x01`                      | Unknown               |
| 27     | 1              | `0x01`                      | Unknown               |
| 28     | 1              | `0x02` (on) or `0x03` (off) | Auto Brightness State |

#### Display Metering
This command changes the display metering mode.
The RODECaster has 2 modes: "Default" and "Broadcast".

| Offset | Length (bytes) | Value                                  | Description / Note |
|--------|----------------|----------------------------------------|--------------------|
| 0      | 1              | `0x03`                                 | MAGIC Number       |
| 1      | 1              | `0x15`                                 | Message length     |
| 2      | 3              | `0x00 0x00 0x00`                       | Seperator.         |
| 6      | 1              | `0x01`                                 |                    |
| 7      | 1              | `0x01`                                 |                    |
| 8      | 1              | `0x01`                                 |                    |
| 9      | 1              | `0x01`                                 |                    |
| 10     | 1              | `0x05`                                 | Unknown            |
| 11     | 8              | `metering`                             | ASCII Command      |
| 19     | 1              | `0x00`                                 | Null-Terminator    |
| 20     | 1              | `0x01`                                 | Unknown            |
| 21     | 1              | `0x05`                                 | Unknown            |
| 22     | 1              | `0x01`                                 | Unknown            |
| 22     | 1              | `0x00` (default) or `0x01` (broadcast) | Metering Mode      |
| 23     | 3              | `0x00 0x00 0x00`                       | Unknown            |

#### Rec-Button Mode
This command changes the mode of the Rec-Button.
The RODECaster has 2 modes: "Pause" and "Stop".

| Offset | Length (bytes) | Value                           | Description / Note |
|--------|----------------|---------------------------------|--------------------|
| 0      | 1              | `0x03`                          | MAGIC Number       |
| 1      | 1              | `0x23`                          | Message length     |
| 2      | 3              | `0x00 0x00 0x00`                | Seperator.         |
| 6      | 1              | `0x01`                          |                    |
| 7      | 1              | `0x01`                          |                    |
| 8      | 1              | `0x01`                          |                    |
| 9      | 1              | `0x01`                          |                    |
| 10     | 1              | `0x0D`                          | Unknown            |
| 11     | 22             | `systemRecButtonSetting`        | ASCII Command      |
| 33     | 1              | `0x00`                          | Null-Terminator    |
| 34     | 1              | `0x01`                          | Unknown            |
| 35     | 1              | `0x05`                          | Unknown            |
| 36     | 1              | `0x01`                          | Unknown            |
| 37     | 1              | `0x00` (pause) or `0x01` (stop) | Rec-Button Mode    |
| 38     | 3              | `0x00 0x00 0x00`                | Unknown            |

#### Haptics Mode
The RODECaster Pro II has a haptic motor that can be used to provide feedback to the user.
There are 3 modes: "Off", "Hold" and "Always".

| Offset | Length (bytes) | Value                                            | Description / Note |
|--------|----------------|--------------------------------------------------|--------------------|
| 0      | 1              | `0x03`                                           | MAGIC Number       |
| 1      | 1              | `0x20`                                           | Message length     |
| 2      | 3              | `0x00 0x00 0x00`                                 | Seperator.         |
| 6      | 1              | `0x01`                                           |                    |
| 7      | 1              | `0x01`                                           |                    |
| 8      | 1              | `0x01`                                           |                    |
| 9      | 1              | `0x01`                                           |                    |
| 10     | 1              | `0x0D`                                           | Unknown            |
| 11     | 19             | `systemHapticsSetting`                           | ASCII Command      |
| 30     | 1              | `0x00`                                           | Null-Terminator    |
| 31     | 1              | `0x01`                                           | Unknown            |
| 32     | 1              | `0x05`                                           | Unknown            |
| 33     | 1              | `0x01`                                           | Unknown            |
| 34     | 1              | `0x00` (off) or `0x01` (hold) or `0x02` (always) | Haptics Mode       |
| 35     | 3              | `0x00 0x00 0x00`                                 | Unknown            |

### System

#### Midi Control
The RODECaster Pro II can be controlled using MIDI messages.
This command enables or disables MIDI control.

| Offset | Length (bytes) | Value                       | Description / Note |
|--------|----------------|-----------------------------|--------------------|
| 0      | 1              | `0x03`                      | MAGIC Number       |
| 1      | 1              | `0x1A`                      | Message length     |
| 2      | 3              | `0x00 0x00 0x00`            | Seperator.         |
| 6      | 1              | `0x01`                      |                    |
| 7      | 1              | `0x01`                      |                    |
| 8      | 1              | `0x01`                      |                    |
| 9      | 1              | `0x01`                      |                    |
| 10     | 1              | `0x0D`                      | Unknown            |
| 11     | 17             | `systemMidiControl`         | ASCII Command      |
| 28     | 1              | `0x00`                      | Null-Terminator    |
| 29     | 1              | `0x01`                      | Unknown            |
| 30     | 1              | `0x01`                      | Unknown            |
| 31     | 1              | `0x02` (on) or `0x03` (off) | Midi Control-State |

### Other

#### Trigger 'Transfer Mode'
The RODECaster Pro II has to be put into a special mode called "Transfer Mode" to begin file transfers for the SMART Pads.
This command triggers the device to enter this mode, but this is probably not enough to actually start a file transfer, 
as the device also needs to be informed about which pad should be updates.

| Offset | Length (bytes) | Value                       | Description / Note  |
|--------|----------------|-----------------------------|---------------------|
| 0      | 1              | `0x03`                      | MAGIC Number        |
| 1      | 1              | `0x1D`                      | Message length      |
| 2      | 3              | `0x00 0x00 0x00`            | Seperator.          |
| 6      | 1              | `0x01`                      |                     |
| 7      | 1              | `0x01`                      |                     |
| 8      | 1              | `0x01`                      |                     |
| 9      | 1              | `0x01`                      |                     |
| 10     | 1              | `0x0D`                      | Unknown             |
| 11     | 16             | `transferModeType`          | ASCII Command       |
| 27     | 1              | `0x00`                      | Null-Terminator     |
| 28     | 1              | `0x01`                      | Unknown             |
| 29     | 1              | `0x05`                      | Unknown             |
| 30     | 1              | `0x01`                      | Unknown             |
| 31     | 1              | `0x01` (on) or `0x00` (off) | Transfer Mode-State |
| 32     | 3              | `0x00 0x00 0x00`            | Unknown             |