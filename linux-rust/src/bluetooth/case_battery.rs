//! Experimental independent case advertisements, observed on product 0x2035.
//! Structural checks are not cryptographic authentication. Never use this for
//! control commands or relax the format checks without another capture.
use aes::{
    Aes128,
    cipher::{Array, BlockCipherDecrypt, KeyInit},
};
use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

pub const MAX_AGE: Duration = Duration::from_secs(150);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaseBattery {
    pub percent: Option<u8>,
    pub charging: bool,
}

pub fn decode_components(data: &[u8], key: &[u8; 16]) -> Option<[u8; 3]> {
    if data.len() != 19 || data[..3] != [0x07, 0x11, 0x06] {
        return None;
    }
    let mut block = Array::from(<[u8; 16]>::try_from(&data[3..]).ok()?);
    Aes128::new(&Array::from(*key)).decrypt_block(&mut block);
    // Byte 2 is opaque, not a proven format version: observed 01, 03, 05.
    // Rejecting a new value here can retain an obsolete charging indication.
    if block[..2] != [0x35, 0x20]
        || block[8..12] != [0; 4]
        || block[14..16] != [0; 2]
        || block[3..6].iter().any(|v| *v != 0xff && v & 0x7f > 100)
    {
        return None;
    }
    Some([block[4], block[5], block[3]]) // left, right, case (canonical order)
}
pub fn decode(data: &[u8], key: &[u8; 16]) -> Option<CaseBattery> {
    let case = decode_components(data, key)?[2];
    Some(CaseBattery {
        percent: (case != 255).then_some(case & 127),
        charging: case != 255 && case & 128 != 0,
    })
}

pub fn match_components<'a>(
    data: &[u8],
    keys: &'a [(String, [u8; 16])],
) -> Option<(&'a str, [u8; 3])> {
    let mut matches = keys
        .iter()
        .filter_map(|(id, key)| decode_components(data, key).map(|v| (id.as_str(), v)));
    let result = matches.next()?;
    if matches.next().is_some() {
        None
    } else {
        Some(result)
    }
}

pub fn match_device<'a>(
    data: &[u8],
    keys: &'a [(String, [u8; 16])],
) -> Option<(&'a str, CaseBattery)> {
    let mut matches = keys
        .iter()
        .filter_map(|(id, key)| decode(data, key).map(|v| (id.as_str(), v)));
    let result = matches.next()?;
    if matches.next().is_some() {
        None
    } else {
        Some(result)
    }
}

#[derive(Debug, Default)]
pub struct CaseReadings(HashMap<String, (CaseBattery, SystemTime)>);

impl CaseReadings {
    pub fn update(&mut self, device: &str, value: CaseBattery, now: SystemTime) {
        if value.percent.is_some() {
            self.0.insert(device.to_string(), (value, now));
        } else {
            self.0.remove(device);
        }
    }
    pub fn get(&self, device: Option<&str>, now: SystemTime) -> Option<CaseBattery> {
        let (value, at) = self.0.get(device?)?;
        (now.duration_since(*at).unwrap_or(MAX_AGE) < MAX_AGE).then_some(*value)
    }
    pub fn expire(&mut self, now: SystemTime) -> bool {
        let before = self.0.len();
        self.0
            .retain(|_, (_, at)| now.duration_since(*at).unwrap_or(MAX_AGE) < MAX_AGE);
        before != self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aes::cipher::BlockCipherEncrypt;
    const KEY: [u8; 16] = [0x42; 16]; // Synthetic; not an accessory key.
    fn packet(plain: [u8; 16]) -> Vec<u8> {
        let mut block = Array::from(plain);
        Aes128::new(&Array::from(KEY)).encrypt_block(&mut block);
        let mut data = vec![7, 17, 6];
        data.extend_from_slice(&block);
        data
    }
    fn plain(level: u8) -> [u8; 16] {
        [
            0x35, 0x20, 1, level, 255, 255, 90, 134, 0, 0, 0, 0, 1, 2, 0, 0,
        ]
    }
    #[test]
    fn empty_case_charges_independently() {
        for level in 73..=79 {
            for charging in [false, true] {
                assert_eq!(
                    decode(&packet(plain(level | if charging { 128 } else { 0 })), &KEY),
                    Some(CaseBattery {
                        percent: Some(level),
                        charging
                    })
                );
            }
        }
    }
    #[test]
    fn live_full_empty_case_status_three() {
        let mut p = plain(100);
        p[2] = 3;
        assert_eq!(
            decode(&packet(p), &KEY),
            Some(CaseBattery {
                percent: Some(100),
                charging: false
            })
        );
    }
    #[test]
    fn opaque_status_change_clears_old_charge_state() {
        let now = SystemTime::now();
        let mut old = plain(228);
        old[2] = 3;
        let mut new = plain(100);
        new[2] = 5;
        let mut readings = CaseReadings::default();
        readings.update("A", decode(&packet(old), &KEY).unwrap(), now);
        assert!(readings.get(Some("A"), now).unwrap().charging);
        readings.update("A", decode(&packet(new), &KEY).unwrap(), now);
        let expected = CaseBattery {
            percent: Some(100),
            charging: false,
        };
        assert_eq!(readings.get(Some("A"), now), Some(expected));
        for opaque in 0..=u8::MAX {
            new[2] = opaque;
            assert_eq!(decode(&packet(new), &KEY), Some(expected));
        }
    }
    #[test]
    fn independent_case_exposes_both_earbud_slots_and_unknowns() {
        let mut p = plain(81);
        p[4] = 128 + 65;
        p[5] = 255;
        assert_eq!(decode_components(&packet(p), &KEY), Some([193, 255, 81]));
        p[4] = 255;
        p[5] = 128 + 70;
        assert_eq!(decode_components(&packet(p), &KEY), Some([255, 198, 81]));
    }
    #[test]
    fn boundaries_unknown_and_invalid() {
        for level in [0, 100, 128, 228, 255] {
            assert!(decode(&packet(plain(level)), &KEY).is_some());
        }
        assert_eq!(decode(&packet(plain(255)), &KEY).unwrap().percent, None);
        for level in [101, 127, 229, 254] {
            assert!(decode(&packet(plain(level)), &KEY).is_none());
        }
        for (offset, value) in [
            (0, 0),
            (1, 0),
            (4, 101),
            (5, 254),
            (8, 1),
            (11, 1),
            (14, 1),
            (15, 1),
        ] {
            let mut p = plain(76);
            p[offset] = value;
            assert!(decode(&packet(p), &KEY).is_none());
        }
        let p = packet(plain(76));
        for len in 0..19 {
            assert!(decode(&p[..len], &KEY).is_none());
        }
        let mut extended = p.clone();
        extended.push(0);
        assert!(decode(&extended, &KEY).is_none());
        let mut wrong = p.clone();
        wrong[2] = 1;
        assert!(decode(&wrong, &KEY).is_none());
        assert!(decode(&p, &[0; 16]).is_none());
    }
    #[test]
    fn unique_identity_required() {
        let p = packet(plain(76));
        assert!(match_device(&p, &[]).is_none());
        let one = vec![("pair-A".into(), KEY), ("pair-B".into(), [0; 16])];
        assert_eq!(match_device(&p, &one).unwrap().0, "pair-A");
        let ambiguous = vec![("pair-A".into(), KEY), ("pair-B".into(), KEY)];
        assert!(match_device(&p, &ambiguous).is_none());
    }
    #[test]
    fn readings_are_per_device_and_expire() {
        let now = SystemTime::now();
        let mut values = CaseReadings::default();
        let value = CaseBattery {
            percent: Some(76),
            charging: true,
        };
        values.update("A", value, now);
        assert_eq!(values.get(Some("A"), now), Some(value));
        assert_eq!(values.get(Some("B"), now), None);
        assert_eq!(values.get(None, now), None);
        assert!(values.get(Some("A"), now + MAX_AGE).is_none());
        assert!(values.expire(now + MAX_AGE));
        assert!(!values.expire(now + MAX_AGE));
        values.update("A", value, now);
        values.update(
            "A",
            CaseBattery {
                percent: None,
                charging: false,
            },
            now,
        );
        assert!(values.get(Some("A"), now).is_none());
    }
}
