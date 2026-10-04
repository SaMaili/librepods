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
| 2 | Opaque status; not a format-version gate |
| 3 | Case battery |
| 4 | Left earbud battery |
| 5 | Right earbud battery |
| 6–7, 12–13 | Unknown |
| 8–11, 14–15 | Zero in observed frames |

Battery bytes use bits 0–6 for percentage, bit 7 for charging, and `ff` for unknown.
Known percentages must be 0–100. Model, length, header and reserved fields are
validated. These checks are not cryptographic authentication and must not authorize
control operations. Tests use synthetic keys and packets, not accessory secrets.

## Source selection and reception

AACP and both advertisement formats share one store per paired address/component.
An unknown slot withdraws only its source; the newest sample younger than 150
seconds wins. The tray uses this store and reports charging in plain text; its
charging icon is drawn geometry. Information refreshes preserve proximity keys,
and the AACP subscription is installed before connection setup.

Existing advertisers are subscribed after startup without promoting cached bytes
to fresh readings. Monitor loss alone does not remove a D-Bus subscription.
Because BlueZ suppresses identical ManufacturerData, duplicate-report LE scans run
for eight seconds when a previously seen case is silent for at least 60 seconds.
A 30-second scheduler stops requesting refreshes after ten minutes without a case
sample. One 20-second startup window runs when stored keys exist. Live independent
case recovery after a process restart remains unverified.

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
the last earbud. The cause is unresolved. Only observed product `0x2035` is accepted;
a second pair, other models/firmware, and continuous empty-case transmission are
not validated. Raw captures, real keys, addresses and serials are excluded.
