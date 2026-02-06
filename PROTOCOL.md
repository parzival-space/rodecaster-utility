# RODECaster Pro II HID Protocol
I will try to document here as much as possible while I figure out how exactly the RODECaster Pro II's USB protocol works. 

## USB Device
| Property           | Value                                    | Note                                              |
|--------------------|------------------------------------------|---------------------------------------------------|
| Vendor ID          | `0x19f7`                                 |                                                   |
| Product ID (USB 1) | `0x0037`, `0x0072`, `0x0078` or `0x0030` | Changes depending on the Multitrack configuration |
| Product ID (USB 2) | `0x0026`                                 |                                                   |

### Device Interfaces & Endpoints
| Interface | Alt | Endpoints | Type / Class | Sub (?) | Prot (?) | Note                                             |
|-----------|-----|-----------|--------------|---------|----------|--------------------------------------------------|
| 0         | 0   | 0         | 01 (audio)   | 01      | 20       |                                                  |
| 1         | 0   | 0         | 01 (audio)   | 02      | 20       |                                                  |
| 2         | 0   | 0         | 01 (audio)   | 02      | 20       |                                                  |
| 3         | 0   | 0         | 01 (audio)   | 01      | 20       |                                                  |
| 4         | 0   | 0         | 01 (audio)   | 02      | 20       |                                                  |
| 5         | 0   | 0         | 01 (audio)   | 02      | 20       |                                                  |
| 6         | 0   | 0         | 01 (audio)   | 01      | 20       |                                                  |
| 7         | 0   | 2         | 01 (audio)   | 03      | 20       | Currently unsure what the Endpoints are used for |
| 8         | 0   | 2         | 08 (storage) | 06      | 50       | Probably used during file transfers              |
| 9         | 0   | 2         | 03 (HID)     | 06      | 50       | Used to send configuration packets               |

#### Interface 7 Endpoints
| Address | Direction | Type | Note |
|---------|-----------|------|------|
| `0x03`  | OUT       | Bulk |      |
| `0x83`  | IN        | Bulk |      |

#### Interface 8 Endpoints
| Address | Direction | Type | Note |
|---------|-----------|------|------|
| `0x04`  | OUT       | Bulk |      |
| `0x84`  | IN        | Bulk |      |

#### Interface 9 Endpoints
Used by the RODE Central App to control the device

| Address | Direction | Type      | Note |
|---------|-----------|-----------|------|
| `0x05`  | OUT       | Interrupt |      |
| `0x85`  | IN        | Interrupt |      |

## Common
- All messages are **256 bytes long**, remaining bytes are padded with `0x00`.
- The device needs a initialization message to be sent before it starts sending messages.

## Initialization Packet
To tell the device to start reporting configuration changes to the host, the following packet needs to be sent to the device:

```
0x03, 0x04, 0x00, 0x00, 0x00, 0xAD, 0x10, 0xA7, 0xB0, [padding...]
```

I still have to figure out what exactly the bytes in the packet mean, but for now this is required or else the
device won't respond to any other configuration messages. After this packet is sent, the device starts sending messages
to the host whenever a configuration change is made, and also responds to configuration queries from the host.