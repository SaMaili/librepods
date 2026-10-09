//! Experimental independent case advertisements, observed on product 0x2035.
//! Structural checks are not cryptographic authentication. Never use this for
//! control commands or relax the format checks without another capture.
use aes::{
    Aes128,
    cipher::{Array, BlockCipherDecrypt, KeyInit},
};
pub fn decode_components(data: &[u8], key: &[u8; 16]) -> Option<[u8; 3]> {
    if data.len() != 19 || data[..3] != [0x07, 0x11, 0x06] {
        return None;
    }
    let mut block = Array::from(<[u8; 16]>::try_from(&data[3..]).ok()?);
    Aes128::new(&Array::from(*key)).decrypt_block(&mut block);
    // Bytes 2 and 14 are opaque; byte 14 has been observed as 0 or 1.
    if block[..2] != [0x35, 0x20]
        || block[8..12] != [0; 4]
        || block[14] > 1
        || block[15] != 0
        || block[3..6].iter().any(|v| *v != 0xff && v & 0x7f > 100)
    {
        return None;
    }
    Some([block[4], block[5], block[3]]) // left, right, case (canonical order)
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
    fn empty_case_reports_percentage_and_charging_bit() {
        for level in [0, 73, 79, 100] {
            for charging in [false, true] {
                let value = level | if charging { 128 } else { 0 };
                assert_eq!(
                    decode_components(&packet(plain(value)), &KEY),
                    Some([255, 255, value])
                );
            }
        }
    }

    #[test]
    fn opaque_status_change_does_not_retain_old_charging_bit() {
        for opaque in 0..=u8::MAX {
            let mut p = plain(128 + 100);
            p[2] = opaque;
            assert_eq!(decode_components(&packet(p), &KEY), Some([255, 255, 228]));
            p[3] = 100;
            assert_eq!(decode_components(&packet(p), &KEY), Some([255, 255, 100]));
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
    fn observed_flag_one_preserves_charging_right_bud() {
        let mut p = plain(51);
        p[2] = 0x0b;
        p[14] = 1;
        for level in [30, 31, 32, 33, 40] {
            p[5] = 128 + level;
            assert_eq!(
                decode_components(&packet(p), &KEY),
                Some([255, 128 + level, 51])
            );
        }
        p[5] = 40;
        assert_eq!(decode_components(&packet(p), &KEY), Some([255, 40, 51]));
        for flag in 2..=u8::MAX {
            p[14] = flag;
            assert!(decode_components(&packet(p), &KEY).is_none());
        }
    }

    #[test]
    fn boundaries_unknown_and_invalid() {
        for level in [0, 100, 128, 228, 255] {
            assert!(decode_components(&packet(plain(level)), &KEY).is_some());
        }
        assert_eq!(
            decode_components(&packet(plain(255)), &KEY),
            Some([255, 255, 255])
        );
        for level in [101, 127, 229, 254] {
            assert!(decode_components(&packet(plain(level)), &KEY).is_none());
        }
        for (offset, value) in [
            (0, 0),
            (1, 0),
            (4, 101),
            (5, 254),
            (8, 1),
            (11, 1),
            (14, 2),
            (15, 1),
        ] {
            let mut p = plain(76);
            p[offset] = value;
            assert!(decode_components(&packet(p), &KEY).is_none());
        }
        let p = packet(plain(76));
        for len in 0..19 {
            assert!(decode_components(&p[..len], &KEY).is_none());
        }
        let mut extended = p.clone();
        extended.push(0);
        assert!(decode_components(&extended, &KEY).is_none());
        let mut wrong = p.clone();
        wrong[2] = 1;
        assert!(decode_components(&wrong, &KEY).is_none());
        assert!(decode_components(&p, &[0; 16]).is_none());
    }
    #[test]
    fn unique_identity_required() {
        let p = packet(plain(76));
        assert!(match_components(&p, &[]).is_none());
        let one = vec![("pair-A".into(), KEY), ("pair-B".into(), [0; 16])];
        assert_eq!(match_components(&p, &one).unwrap().0, "pair-A");
        let ambiguous = vec![("pair-A".into(), KEY), ("pair-B".into(), KEY)];
        assert!(match_components(&p, &ambiguous).is_none());
    }
}
