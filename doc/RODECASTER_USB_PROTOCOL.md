# RODECaster USB Protocol

This document describes the USB/HID communication protocol as reverse-engineered from
live captures and reflected in the Rust implementation under `usb/src/protocol/`.

It is very likely that all RODECaster devices implement this protocol, but the information in this document is based
on captures from a single RODECaster Pro II unit.

> **Note:** The information in this document might be incomplete or inaccurate in places.
> I will update it as I learn more about the protocol.
> If you have any insights or corrections, please open an issue or submit a pull request!

---

## 1. HID Transport Layer

The device exposes a HID interface (Interface 9, for the RODECaster Pro II).
Communication uses **256-byte HID reports**:

| Direction     | Report ID | HID type | Payload size |
|---------------|-----------|----------|--------------|
| Host → Device | `0x03`    | Output   | 255 bytes    |
| Device → Host | `0x04`    | Feature  | 255 bytes    |

The Report ID byte occupies byte 0 on the wire; the 255 remaining bytes carry the
protocol payload described in Message Framing.

> Implementation reference: `usb/src/rodecaster_pro_ii/device.rs`
> (`HID_REPORT_ID_SEND = 0x03`, `HID_REPORT_ID_RECEIVE = 0x04`)

---

## 2. Message Framing

A logical *message* may be larger than one HID report. The framing layer handles
reassembly transparently.

### 2.1 First frame

```
┌────────────────────┬──────────────────────────────────────────────────┐
│  u32 LE  (4 bytes) │  Packet data  (up to 251 bytes in this frame)    │
│  Packet length N   │  First N bytes of the packet                     │
└────────────────────┴──────────────────────────────────────────────────┘
```

- The **packet length** field gives the *total* byte count of the packet data.
- If N ≤ 251, the full packet fits in a single frame.
- Bytes after the packet data inside the last frame are `0x00` padding and are discarded.

### 2.2 Continuation frames

If the packet data is longer than 251 bytes, further HID reports are read and their
payloads (all 255 bytes) are concatenated directly after the first frame's data until
N bytes have been collected.

```
Frame 1:  [4-byte length] [first 251 bytes of packet data]
Frame 2:  [next 255 bytes of packet data]
Frame N:  [remaining bytes] [0x00 padding …]
```

> Implementation reference: `usb/src/protocol/framing.rs` – `read_framed_message` /
> `write_framed_message`

---

## 3. Initialization

Before the device sends any state, the host must send a single *handshake* output report:

```
Byte offset  Value   Note
──────────── ─────── ───────────────────────────────────────
0            0x03    HID Report ID (host → device)
1–4          0x04 0x00 0x00 0x00   Packet length = 4 (u32 LE)
5            0xAD    │
6            0x10    │ 4-byte handshake payload (purpose unknown)
7            0xA7    │
8            0xB0    │
```

After receiving this packet the device streams one **DeviceReport** packets
containing its complete current state, followed by **PropertyUpdate** packets
for every subsequent state change.

---

## 4. Packet Identification

The first byte of every reassembled packet is a **type discriminant**:

| Byte   | Type             | Description                   |
|--------|------------------|-------------------------------|
| `0x01` | `PropertyUpdate` | Incremental property change   |
| `0x02` | `DeviceReport`   | Full state snapshot as a tree |
| other  | Unknown          | Ignored; logged as a warning  |

---

## 5. Packet Types

### 5.1 PropertyUpdatePacket (`0x01`)

Sent whenever a single property changes on the device (button press, fader move, etc.)
as well as in response to configuration writes from the host.

```
Offset  Size      Value / Note
──────  ────────  ──────────────────────────────────────────────────────────
0       1 byte    0x01  – packet type discriminant
1       1 byte    0x01  – indices-count length tag (always 0x01 in practice)
2       u8        indices_count  – number of tree-navigation indices that follow
3 …     variable  indices_count × Index   
?       variable  Property name  (null-terminated ASCII)
?       variable  Value   
```

The indices form a *path* into the device's state tree: navigate
`children[indices[0]].children[indices[1]]. … .properties[name]`.

#### 5.1.1 Index encoding

Each index is encoded as a length-tagged integer:

```
Byte 0 (tag)  Meaning
────────────  ─────────────────────────────────────────────────
0x00          Index value is 0; no additional byte follows
0x01          Index follows as 1-byte unsigned integer (u8)
0x02          Index follows as 2-byte little-endian unsigned integer (u16)
```

#### 5.1.2 Example – encoded `PropertyUpdatePacket`

```
Packet bytes (hex):
  01 01 02 01 01 01 02 74 65 73 74 00 01 05 2A 00 00 00 00

Breakdown:
  01          packet type (PropertyUpdate)
  01          indices_count_length (always 0x01)
  02          indices_count = 2
  01 01       index[0]: tag=0x01 (u8),  value=1
  01 02       index[1]: tag=0x01 (u8),  value=2
  74 65 73 74 00   property name = "test\0"
  01 05       Value: len_type=u8, total_len=5 (1 type byte + 4 data bytes)
  01          Value type = U32 (0x01)
  2A 00 00 00 value = 42 (u32 LE)
```

Complete framed form (with 10-byte frame size):

```
Frame 1: 12 00 00 00  01 01 02 01 01 01    ← [length=18] [first 6 bytes]
Frame 2: 02 74 65 73  74 00 01 05 2A 00    ← continuation
Frame 3: 00 00 00 00  00 00 00 00 00 00    ← padding
```

---

### 5.2 DeviceReportPacket (`0x02`)

Sent by the device on initialization or when a bulk state refresh is requested.
Contains the complete device state as a recursive **Structured** tree.

```
Offset  Size      Value / Note
──────  ────────  ──────────────────────────────────────────────────────────
0       1 byte    0x02  – packet type discriminant
1 …     variable  Root Structured node
```

---

## 6. Serialization Primitives

### 6.1 C-String

A null-terminated ASCII string: bytes up to (and including) the first `0x00` byte.
The null terminator is consumed but not included in the resulting string value.

### 6.2 Value

A length-prefixed, type-tagged value used for both property values and Value fields
inside Structured nodes.

```
Field           Size    Description
──────────────  ──────  ─────────────────────────────────────────────────────
len_type        1 byte  0x01 = length fits in u8; 0x02 = length fits in u16 LE
total_len       1 or 2  Total byte count of (type byte + data bytes)
data_type       1 byte  See table below
data            varies  total_len − 1 bytes of payload
```

**Data type discriminants:**

| `data_type` | Rust variant  | Payload                                                              |
|-------------|---------------|----------------------------------------------------------------------|
| `0x01`      | `U32`         | 4 bytes, little-endian unsigned 32-bit int                           |
| `0x02`      | `Bool(true)`  | 0 bytes (type byte encodes the value)                                |
| `0x03`      | `Bool(false)` | 0 bytes (type byte encodes the value)                                |
| `0x04`      | `F64`         | 8 bytes, little-endian IEEE 754 double                               |
| `0x05`      | `String`      | Null-terminated ASCII (C-String)                                     |
| `0x06`      | `Double`      | 8 bytes, little-endian IEEE 754 double (semantic variant of `0x04`)  |
| `0x08`      | `Combined`    | `total_len − 1` bytes containing multiple concatenated Value records |
| other       | `Unknown`     | Raw bytes; logged as a warning                                       |

**Serialisation sizes for common types:**

| Type       | `len_type`              | `total_len` | Total wire bytes |
|------------|-------------------------|-------------|------------------|
| `U32`      | `0x01`                  | `0x05`      | 6                |
| `Bool`     | `0x01`                  | `0x01`      | 3                |
| `F64`      | `0x01`                  | `0x09`      | 10               |
| `Double`   | `0x01`                  | `0x09`      | 10               |
| `String n` | `0x01` (if ≤ 253 chars) | `n+2`       | `n+4`            |

### 6.3 Structured

The `Structured` type represents a node in the device's hierarchical state tree.
Each node has a name, optional properties (key→value map), and optional children.

```
Field            Size    Description
───────────────  ──────  ────────────────────────────────────────────────────
name             varies  C-String
component_kind   1 byte  Node kind: 0x00 = Collection, 0x01 = Object
…                        Remainder depends on kind (see below)
```

#### Kind `0x00` – Collection

Contains only child nodes. Collections may have more than 255 children.

```
Field           Size      Description
──────────────  ────────  ─────────────────────────────────────────────────
count_len       1 byte    0x00 = empty; 0x01 = count is u8; 0x02 = count is u16 LE
child_count     0/1/2     Number of children
children        varies    child_count × Structured nodes (recursive)
```

#### Kind `0x01` – Object

Contains properties and optionally child nodes. Objects have at most 255 children.

```
Field           Size      Description
──────────────  ────────  ──────────────────────────────────────────────────
prop_count      1 byte    Number of properties
properties      varies    prop_count × (C-String name + Value)
close_tag       1 byte    0x00 = no children follow; 0x01 = children follow
[child_count]   1 byte    Only present when close_tag == 0x01
[children]      varies    child_count × Structured nodes (recursive)
```

#### Addressing properties

The `indices` field of a `PropertyUpdatePacket` forms a path through the tree:

```
root.children[indices[0]].children[indices[1]]. … .properties[name]
```

An empty `indices` list means the property lives directly on the root node.

---

