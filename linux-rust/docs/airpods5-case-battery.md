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
| 8–11, 15 | Zero in observed frames |
| 14 | Opaque field, observed as `0` and `1` |

Battery bytes use bits 0–6 for percentage, bit 7 for charging, and `ff` for unknown.
Known percentages must be 0–100. Model, length, header, zero fields and the
observed byte-14 values are validated. Byte 14 is not assumed to be reserved or
a charging indicator; its meaning remains unknown. These checks are not cryptographic authentication and must not authorize
control operations. Tests use synthetic keys and packets, not accessory secrets.

## Source selection and reception

AACP and both advertisement formats share one store per paired address/component.
An unknown slot withdraws only its source; the newest sample younger than 150
seconds wins. The tray uses this store and reports charging in plain text; its
charging icon is drawn geometry. Information refreshes preserve proximity keys,
and the AACP subscription is installed before connection setup.

Existing advertisers are subscribed after startup without promoting cached bytes
to fresh readings. Adapter device-added events also establish subscriptions when
BlueZ discovery creates an advertiser without a monitor DeviceFound. Device
removal clears its watcher; monitor loss alone does not remove a subscription.
Because BlueZ suppresses identical ManufacturerData, duplicate-report LE scans run
for eight seconds when a previously seen case is silent for at least 60 seconds.
A 30-second scheduler stops requesting refreshes after ten minutes without a case
sample. One 20-second startup window runs when stored keys exist. Live recovery after a process restart was verified once while an empty, closed
case was charging; ongoing advertiser discovery is covered by the adapter stream.

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
with charging set, consistent with the user's iPhone observations. The decoder
now accepts only observed byte-14 values 0 and 1; other format, percentage and
unique-key checks remain unchanged. Regression fixtures use a synthetic key.
After installation, a three-minute live test followed right 40% to 49% charging
and case 49% to 48% not charging. No component disappeared after startup; right
and case sample age stayed at or below 37.4 seconds. The user confirmed that
the widget displayed the correct state.
