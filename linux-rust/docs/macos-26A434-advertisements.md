# Apple advertisement parsing recovered from macOS 26A434

The reference image is `UniversalMac_27.0.1_26A434_Restore.ipsw`, with manifest
version 27.0.1 and build 26A434, selected identity j274ap. CoreBluetooth arm64e
UUID: `70635a3a-c674-335c-b557-31566cf46c32`. bluetoothd arm64e UUID:
`26c91e94-8e53-39a4-a683-266c0a825dfc`.

This investigation traces the known `07 19 01` and `07 11 06` packets into
Apple's receive code. No Apple binaries, disassembly, private captures, addresses,
serials or accessory keys are included in this contribution.

## Framing and routing

After company ID 004c, manufacturer data is a sequence of type/length/payload
records. The actual payload length is `length_byte & 0x1f`. Truncated records
stop parsing; other complete records can precede or follow a pairing record.
CoreBluetooth's manufacturer dispatcher is at `0x19c253fa0`; its proximity
subtype dispatcher is at `0x19c2253e4`.

The proximity dispatcher handles subtypes 0, 1, 2, 4, 5, 6, 7, 8 and 9. Their
handlers are WxSetup, WxStatus, AppleTVRemote, AirPodsMismatched, ObjectSetup,
AccessoryStatus, ProximityPairingV2, FindMyAccessoryStatus and ObjectSetupV2.
Recognition does not mean each subtype is emitted by AirPods 5.

## Linux integration

- `apple_advertisement.rs` reads concatenated records and upper length flags.
- Existing legacy subtype-1 decoding receives a normalized record, preserving
  its established encryption and component offsets.
- Subtypes 4/7 provide Case/Left/Right bytes after subtype, two-byte product ID
  and Status1. This path is enabled only after IRK attribution to a paired device,
  for image-confirmed AirPods-5 IDs 2030, 2032, 2036 and 2037.
- Standalone Battery1/2/3 helpers target Case/Left/Right, preserve previous fields
  on ff, and clamp the seven-bit level to 100. Unknown fields do not renew Linux
  receive timestamps.
- Independent Case subtype 6 continues to require a unique initial key match.
  The identified advertiser then retains its paired key for the lifetime of
  that watcher and uses Apple's recovered field interpretation.
- New recognized shapes on an attributed advertiser are logged by type,
  subtype and payload length, without dumping payloads or keys.

## Independent Case fields

bluetoothd at `0x1007f4fcc` iterates manufacturer records and accepts subtypes
6 and 8 with at least 17 payload bytes. Subtype 6 uses EncKey; subtype 8 uses
an IRK. The former calls a one-block AES-128-ECB helper at `0x100254fd8`.

For subtype 6 the decrypted product ID is at 0..1, lid-open counter and flags
at 2, primary/left/right battery at 3/4/5, packed firmware at 6..8, and an OBC
time byte at 9. The handler multiplies that byte by 600 and stores
`accessoryStatusOBCTime`; the expansion of OBC is not established. Offsets 10..15
are not consumed by the handler, so byte 14 is not a format-version gate.
The primary battery targets Case for product 2035, with explicit Main exceptions
for products 200a, 201f and 202d. Here ff clears the source's battery field,
unlike the standalone helpers used by subtypes 4/7.

## Existing public work and contribution scope

A GitHub comparison on 2026-10-21 found earlier public descriptions. These
formats and method names are not claimed as newly discovered here:

- [iOS 18.2 CBDevice decompilation](https://github.com/EthanArbuckle/iPhone17-1_18.2_22C152_Restore/blob/e26ed4563f78871c59d2d96856756a65d62517e5/System/Library/Frameworks/CoreBluetooth.framework/CBDevice.m)
  already contains the low-five-bit manufacturer length rule, subtype dispatch,
  AccessoryStatus Case/Left/Right fields, charging bits, clamping and OBC scaling.
  It also contains the subtype-4/7 battery layouts. Its older inline handling of
  ff differs from the standalone helpers examined in build 26A434; behavior
  must be attributed to a particular image rather than assumed universal.
- [macOS 26.4 runtime headers](https://github.com/thatmarcel/macOS-26.4-headers/blob/4d5d4f5eba9020ff6bf2879071d565dcce0f4db1/headers/CoreBluetooth/CBDevice.h)
  already expose the parser and battery-helper names, without implementations.
- LibrePods [#138](https://github.com/librepods-org/librepods/pull/138) implements
  standard BLE battery reception. [#766](https://github.com/librepods-org/librepods/pull/766)
  forwards existing Rust BLE readings to the application window and includes
  overlapping startup error handling. Neither reviewed patch adds the independent
  Case parser or the subtype-4/7 Rust paths implemented here.
- LibrePods [#103](https://github.com/librepods-org/librepods/pull/103) discusses
  coarse Find My battery data. It is not evidence for a precise subtype-8 AirPods
  percentage. [#836](https://github.com/librepods-org/librepods/pull/836) addresses
  Android battery merging and popup reception; [#822](https://github.com/librepods-org/librepods/pull/822)
  exports Qt-client status. Those concerns overlap with consumer integration,
  rather than providing this Rust decoder.

The checked upstream `linux/rust` head was
`672e65ad36eebf21ff1c1a508066f9197ee56d17`. It still required a single 27-byte
legacy record. The contribution is an independently performed audit of the
newer image, original Rust integration, image-specific corrections and
validation. Search results cannot establish that no equivalent work exists
anywhere on GitHub.

## Validation

Original ARM64 instructions were executed with Unicorn. Pointer-authentication
prologues/returns were adapted and ObjC dependencies explicitly mocked.

| Scope | Comparisons |
| --- | ---: |
| Battery helpers, all byte values and four prior field states | 3,072 |
| Manufacturer routing, all types, length flags, truncation and concatenation | 18,433 |
| Proximity subtype routing and short payloads | 1,792 |
| AccessoryStatus fields and all byte values at offsets 2..15 | 14,336 |
| FindMyAccessoryStatus raw fields | 1,280 |
| Combined subtype 4/7 caller and battery helpers | 1,562 |
| **Total** | **40,475** |

The Rust suite passes 24 tests, including first-attribution ambiguity, bound
firmware/tail changes, percentage/charging/unknown handling, and framing.
Tests use synthetic keys. Native comparisons validate the listed parser
semantics, not a complete macOS runtime or actual radio reception.

## Remaining limits

The initial unbound Case association still uses conservative capture heuristics:
product 2035, plausible battery bytes, zero bytes 10..11 and 15, and exactly one
key match. Those zeros are identity-matching heuristics, not Apple format rules.
After a successful attribution, firmware/time and tail bytes may vary without
invalidating a report. A new RPA requires fresh attribution. Removing all initial
checks while trying arbitrary keys would not reproduce Apple's address/serial
key lookup, and AES-ECB supplies no authentication tag.

Subtype 8 is independently decoded by Apple and has a different raw main-field
conversion. It is not implemented as a precise AirPods percentage in Linux.
Object Setup subtypes 5/9 have coarse battery state/performance, not a validated
precise AirPods percentage. Full AACP decoding, new subtype-4/7 live captures,
and future firmware are not claimed as covered. No decoder can establish fresh
battery state without an actual received report.
