# RODECaster Pro II

## USB Device

| Property           | Value                                    | Note                                              |
|--------------------|------------------------------------------|---------------------------------------------------|
| Vendor ID          | `0x19f7`                                 |                                                   |
| Product ID (USB 1) | `0x0037`, `0x0072`, `0x0078` or `0x0030` | Changes depending on the Multitrack configuration |
| Product ID (USB 2) | `0x0026`                                 |                                                   |

### Device Interfaces & Endpoints

These are the device endpoints, when using full Multitrack mode (Device PID `0072`).

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