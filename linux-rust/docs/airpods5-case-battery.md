# AirPods 5 independent case battery reports

Tested hardware: AirPods 5 with Wireless Charging Case, earbud family
A3439/A3440/A3441 and case A3529 ([Apple model guide](https://support.apple.com/en-ie/109525)).
The observed case BLE product ID is `0x2035`, corroborated by GATT PnP ID.

## Observed format

Manufacturer company ID is `0x004c`. Its 19-byte value is `07 11 06` followed by
one AES-128-ECB block, encrypted with the AirPods EncKey already stored by LibrePods.
The case advertiser did not resolve with the earbud IRK in the capture, so the
decoder requires exactly one structurally valid match among stored AirPods keys.

| Plaintext bytes | Interpretation |
| --- | --- |
| 0–1 | Little-endian product `0x2035` |
| 2 | Lid-open counter in bits 0–2, accessory status flags in bits 3–5 |
| 3 | Case battery |
| 4 | Left earbud battery |
| 5 | Right earbud battery |
| 6–8 | Packed firmware fields |
| 9 | Multiplied by 600 into `accessoryStatusOBCTime`; OBC meaning not established |
| 10–15 | Not consumed by the recovered Apple handler |

Battery bytes use bits 0–6 for percentage, bit 7 for charging, and `ff` for unknown.
The Apple handler clamps known percentages to 100. Initial attribution still
requires exactly one key match, product 2035, plausible battery bytes and zero
bytes 10–11 and 15 from the captures. These zeros are conservative matching
heuristics rather than Apple format requirements. Once an advertiser has been
identified, firmware/time and uninterpreted tail bytes may vary without dropping
its reports. Byte 14 is not a format gate. Address rotation needs fresh initial
attribution. None of these checks authenticates a device or authorizes control
operations. Tests use synthetic keys and packets, not accessory secrets.

The record reader now supports concatenated manufacturer entries and the
`length_byte & 0x1f` rule recovered from macOS 27.0.1 build 26A434. The known
wire example above remains valid. The source-image audit, additional AirPods-5
battery paths and exact limits are documented in
[macos-26A434-advertisements.md](macos-26A434-advertisements.md).

## Source selection and reception

AACP and the supported advertisement formats share one store per paired address/component.
An unknown slot withdraws only its source; the newest sample younger than 150
seconds wins. The tray uses this store and reports charging in plain text; its
charging icon is drawn geometry. Information refreshes preserve proximity keys,
and the AACP subscription is installed before connection setup.

Existing advertisers are subscribed after startup without promoting cached bytes
to fresh readings. Adapter device-added events also establish subscriptions when
BlueZ discovery creates an advertiser without a monitor DeviceFound. Device
removal clears its watcher; monitor loss alone does not remove a subscription.
Because BlueZ suppresses identical ManufacturerData, duplicate-report LE scans run
for eight seconds when a previously seen Case is silent for at least 15 seconds,
or an earbud for at least 60 seconds. The newest source timestamp is checked
separately for left, right and case, so a fresh case cannot suppress stale earbud
refreshes. A five-second scheduler stops probing a component after ten minutes without a sample. One
20-second startup window runs when stored keys exist. Auto-connect runs separately
with a 20-second timeout on the monitored adapter, deduplicated by paired address;
an outstanding connection attempt neither blocks packet processing nor terminates
an advertiser watcher. Live recovery after a process restart was verified once
while an empty, closed case was charging; ongoing advertiser discovery is covered
by the adapter stream.

## Consumer interface

With the tray enabled and `XDG_RUNTIME_DIR` set, version-1 telemetry is published
atomically to `$XDG_RUNTIME_DIR/librepods/battery.json` (directory 0700, file 0600).
The schema maps paired addresses to left/right/case samples containing
`percentage`, `charging`, and `observedAt` (Unix milliseconds). Expired components
are omitted. Writes are limited to one per second; a five-second timer also
publishes expiry. Publication never renews the receive timestamp. Consumer-side
last-known readings must be marked stale; the separate AACP-driven application
battery window is unchanged.

## Evidence and limits

One pair was tested on Linux/BlueZ with iOS 27.0.1. A private 152-frame capture
showed case charging changes from 73% to 79% matching the iPhone. An empty case
later reported 100%, and unplugging cleared the charging bit. Earbud slot mapping
had 79 same-side matches, zero opposite-side matches, and three timing mismatches.
Closed-case tests worked with both earbuds inside and with only one inside; a
three-minute mixed-state run kept samples fresh with maximum age 88.9 seconds.

Other empty-case states supplied no independent packet, including while charging
and with iPhone Bluetooth disabled. The iPhone row also disappeared after removing
the last earbud. In a later closed, empty-case charging session, fresh independent
packets reported 95% and charging, matching the iPhone. A passive observer received
five fresh events in 45 seconds while the running service missed them; restarting
the service recovered fresh telemetry without changing the case state. A separate
35-second scan received nine case-format events. Empty-case transmission is
therefore state-dependent; the cause of silent sessions remains unresolved.
With adapter device-added subscriptions installed, a three-minute run received
fresh 96% to 97% charging updates. Apart from one startup sample before reception,
the exported case stayed present; maximum sample age was 65.7 seconds. Only observed product `0x2035` is accepted;
a second pair, other models/firmware, and continuous empty-case transmission are
not validated. Raw captures, real keys, addresses and serials are excluded.

On 2026-10-09, a right earbud charging in the case produced byte 14 = 1,
while bytes 8–11 and 15 stayed zero. The old zero-only check discarded these
packets. Independent decryption showed case 52% to 51% and right 30% to 35%
with charging set, consistent with the user's iPhone observations. At that stage the decoder
accepted only observed byte-14 values 0 and 1. The later macOS 26A434 audit
established that byte 14 is not a format gate, superseding that restriction.
Initial identity checks and synthetic-key regression fixtures remain in place.
After installation, a three-minute live test followed right 40% to 49% charging
and case 49% to 48% not charging. No component disappeared after startup; right
and case sample age stayed at or below 37.4 seconds. The user confirmed that
the widget displayed the correct state.

On 2026-10-09, both earbuds outside the case stayed cached while an independent
35-second observer received 160 fresh earbud updates (left 100%, right 99%).
The receiver awaited auto-connect while holding the connection guard, and a
duplicate attempt returned from its advertiser watcher before battery processing.
Auto-connect is now separate, bounded and deduplicated by paired address.
A fresh case also no longer suppresses scans for stale earbud components.
After installation and restart, a 184.0-second run without an independent
observer recovered both components within 2.0 seconds and followed left
100% to 99%, right 99% to 98%, both not charging. All samples from two seconds
onward contained both components; maximum sample age was 67.3 seconds. No fresh
case report was received in this state. This run does not separately validate
connection timeout/error handling.
