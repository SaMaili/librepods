# AirPods 5 independent case battery reports

AirPods 5 with Wireless Charging Case (earbud model family A3439/A3440/A3441,
case model A3529) can send battery information from an independent BLE advertiser.
The model family is listed in [Apple's identification guide](https://support.apple.com/en-ie/109525).
The BLE product ID observed for this case is `0x2035`; GATT PnP ID corroborated it.

The case advertiser did not resolve with the earbud IRK in the tested capture.
The decoder instead tries encryption keys of locally stored AirPods pairs and
requires exactly one structurally valid result. No user's address, real key,
serial number, or captured encrypted frame is part of the tests or this document.

## Observed payload

BlueZ manufacturer company ID: `0x004c`. The value excludes the company ID and is
19 bytes long: `07 11 06` followed by a 16-byte AES-128-ECB encrypted block. It uses
the existing AirPods EncKey obtained by LibrePods.

| Plaintext bytes | Observed interpretation |
| --- | --- |
| 0–1 | Little-endian product ID `0x2035` |
| 2 | Opaque changing value; not treated as a format version |
| 3 | Case battery: low seven bits percentage, high bit charging |
| 4 | Left earbud battery in the same format |
| 5 | Right earbud battery in the same format |
| 6–7 | Unknown |
| 8–11 | Zero in the observed capture |
| 12–13 | Unknown |
| 14–15 | Zero in the observed capture |

A component byte `ff` means unknown/absent. Valid known percentages are 0–100.
The case's left/right slots were compared with standard earbud reports within a
one-second window: 79 exact same-side matches, zero opposite-side matches, and
three mismatches attributed to timing. The decoder does not broaden acceptance
to other product IDs or nonzero reserved fields without additional evidence.
Structural checks are not cryptographic authentication: results are for battery
display, never for pairing authorization or control commands.

## Receiving and merging reports

AACP, standard earbud advertisements, and independent case advertisements are
tracked per paired address and per component. An unknown component withdraws only
that source; it does not erase another source's valid reading. The newest fresh
sample wins. Samples expire after 150 seconds, including after suspend.

An early AACP subscription queues battery replies before connection setup.
Information refreshes preserve proximity keys already obtained from pairing.
Normal INFO logs no longer dump these keys.

The monitor subscribes to existing Apple advertiser objects after process startup
as well as newly found devices. Existing cached manufacturer data is not treated
as a freshly received sample. Monitor DeviceLost does not by itself remove a
still-existing D-Bus device's property subscription.

BlueZ suppresses repeated identical ManufacturerData by default. A bounded LE
scan with duplicate reports is requested for eight seconds if a previously seen
independent case has been silent for at least 60 seconds. A 30-second scheduler
stops those refreshes after ten minutes without a case sample. With stored
AirPods keys, a single 20-second startup window requests fresh reports. The
startup recovery path has unit/build coverage but has not been confirmed by a
successful independent-case live restart test.

## Optional consumer interface

When the tray is enabled and `XDG_RUNTIME_DIR` exists, battery telemetry is
exported atomically to `$XDG_RUNTIME_DIR/librepods/battery.json`. The directory is
mode 0700 and the file mode 0600. Writes are limited to at most one per second,
with a five-second expiry/publication timer. The schema is version 1:

```json
{
  "version": 1,
  "devices": {
    "00:11:22:33:44:55": {
      "left": {"percentage": 81, "charging": false, "observedAt": 1234567890000},
      "right": {"percentage": 84, "charging": true, "observedAt": 1234567890000},
      "case": {"percentage": 76, "charging": false, "observedAt": 1234567890000}
    }
  }
}
```

The address and values above are synthetic. `observedAt` is Unix milliseconds;
publication never renews the receive timestamp. Expired components are omitted.
This interface allows desktop consumers to display data independently of the
audio connection. Consumer-side last-known caches must explicitly mark stale
readings and must not infer a current charging flag. The separate application's
AACP-driven battery window is not changed by this patch.

## Validation and limits

Validation used one AirPods 5 Wireless Charging Case pair on Linux/BlueZ and an
iPhone running iOS 27.0.1. A private capture contained 152 independent case frames,
including charging changes from 73% to 79% matching the iPhone. Later tests decoded
an empty, unplugged case at 100%, also matching the iPhone, and charging-status
transitions without retaining an obsolete charging bit.

Live tests also received left/right/case data with the lid closed, both earbuds
inside, and with only one earbud inside. A three-minute mixed-state test kept all
components fresh after bounded duplicate-report refreshes; the maximum observed
sample age was 88.9 seconds.

This is not a guarantee of continuous transmission by an empty case. In a later
state, opening or charging the empty case did not produce an independent report,
and the iPhone case row disappeared shortly after removing the last earbud.
Temporarily inserting one earbud restored a 66% value on the iPhone. A 45-second
comparison with iPhone Bluetooth disabled, including 35 seconds of duplicate
scanning, received 103 standard earbud reports and zero case-format packets.
A separate empty/closed/charging test received 119 standard earbud reports and
zero case-format packets. The cause of this accessory state is unresolved.

Synthetic Rust tests cover malformed packets, unknown component slots, opaque
byte-2 transitions, wrong and ambiguous keys, pair isolation, source merging,
receive-time expiry, refresh policy, and tray selection of the newest case value.
Only the observed Wireless Charging Case product `0x2035` is supported by this
new decoder. Other models, a second pair, and other firmware remain untested.
