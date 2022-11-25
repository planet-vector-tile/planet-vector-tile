use std::ops::Range;

use ahash::{AHashMap, AHashSet};
use flatdata::RawData;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::__private::de;

use crate::{manifest::Manifest, osmflat::osmflat_generated::osm::Osm};

// pub struct Rules {
//     tag_to_zoom_range: AHashMap<(usize, usize), Range<u8>>,
//     value_to_zoom_range: AHashMap<usize, Range<u8>>,
//     key_to_zoom_range: AHashMap<usize, Range<u8>>,
// }

// impl Rules {
//     pub fn new(manifest: &Manifest, archive: &Osm) -> Self {

//         let strings: RawData = archive.stringtable();

//         let mut rule_strs: AHashSet::<&str> = AHashSet::new();

//         for (_, rule) in manifest.rules {
//             for (k, v) in &rule.tags {
//                 rule_strs.insert(k);
//                 rule_strs.insert(v);
//             }
//             for v in &rule.values {
//                 rule_strs.insert(v);
//             }
//             for k in &rule.keys {
//                 rule_strs.insert(k);
//             }
//         }

//         let mut string_to_index: AHashMap<&str, usize> = AHashMap::new();

//         Rules {
//             tag_to_zoom_range,
//             value_to_zoom_range,
//             key_to_zoom_range,
//         }
//     }

// }

fn get_str_null_delimeters(strings: RawData) -> Vec<usize> {
    let bytes = strings.as_bytes();
    // Serial - 2s 135ms 674us 708ns for california
    // let mut delimeters: Vec<usize> = Vec::new();
    // for (i, byte) in bytes.iter().enumerate() {
    //     if *byte == 0 {
    //         delimeters.push(i);
    //     }
    // }

    // Parallel - 570ms 808us 375ns for california 
    let delimeters: Vec<usize> = bytes
        .par_iter()
        .enumerate()
        .filter_map(|(i, byte)| if *byte == 0 { Some(i) } else { None })
        .collect();

    delimeters
}

fn get_strs(strings: RawData) -> Vec<&str> {
    let bytes = strings.as_bytes();
    let delimeters = get_str_null_delimeters(strings);

    // Serial - 1s 77ms 806us 708ns for california
    // let mut strs = Vec::with_capacity(delimeters.len());
    // let str0 = unsafe { std::str::from_utf8_unchecked(&bytes[..delimeters[0]]) };
    // strs.push(str0);
    // for i in 1..delimeters.len() {
    //     let start = delimeters[i - 1] + 1;
    //     let end = delimeters[i];
    //     let str = unsafe { std::str::from_utf8_unchecked(&bytes[start..end]) };
    //     strs.push(str);
    // }

    // Parallel - 570ms 808us 375ns for california
    let strs: Vec<&str> = delimeters
        .par_iter()
        .enumerate()
        .map(|(i, delimeter)| {
            let start = if i == 0 { 0 } else { delimeters[i - 1] + 1 };
            let end = *delimeter;
            let slice = &bytes[start..end];
            let str = unsafe { std::str::from_utf8_unchecked(slice) };
            str
        })
        .collect();

    strs
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use flatdata::FileResourceStorage;
    use humantime::format_duration;

    use super::*;

    #[test]
    fn test_get_str_indices() {
        let strings = RawData::new(b"hello\0world\0this\0is\0a\0test\0");
        let indices = get_str_null_delimeters(strings);
        assert_eq!(indices, vec![5, 11, 16, 19, 21, 26]);
    }

    #[test]
    fn test_get_strs() {
        let strings = RawData::new(b"hello\0world\0this\0is\0a\0test\0");
        let strs = get_strs(strings);
        assert_eq!(strs, vec!["hello", "world", "this", "is", "a", "test"]);
    }

    #[test]
    fn test_get_strs_santacruz() {
        let archive = Osm::open(FileResourceStorage::new("tests/fixtures/santacruz/sort")).unwrap();
        let strings: RawData = archive.stringtable();
        let delimeters = get_str_null_delimeters(strings);
        let d1 = delimeters[0];
        assert_eq!(d1, 8);
        assert_eq!(delimeters.len(), 60755);

        let strs = get_strs(strings);
        assert_eq!(strs.len(), delimeters.len());

        assert_eq!(
            &strs[..20],
            vec![
                "osmflatc",
                "",
                "Speight",
                "stevea",
                "highway",
                "motorway_junction",
                "ref",
                "433B",
                "KindredCoda",
                "436",
                "mjn",
                "teodorab_telenav",
                "Adamant1",
                "amenity",
                "bank",
                "brand",
                "Chase",
                "brand:wikidata",
                "Q524629",
                "brand:wikipedia"
            ]
        );

        assert_eq!(
            &strs[(delimeters.len() - 10)..],
            vec![
                "Ox Fire Road",
                "Mill Pond Trail",
                "Skyline to the Sea - Saratoga Toll Road Connector Tail",
                "Auchmar Trail",
                "Ridge Top Trail",
                "site",
                "120.86",
                "124.18",
                "La Barranca Park",
                "Approximated, quite synthetic"
            ]
        );
    }

    #[test]
    #[ignore]
    fn test_california_time() {
        let archive = Osm::open(FileResourceStorage::new("/Users/n/geodata/flatdata/california")).unwrap();
        let strings: RawData = archive.stringtable();

        let time = Instant::now();
        let strs = get_strs(strings);
        assert_eq!(strs.len(), 7428597);
        // Total Time: 569ms 658us 875ns
        println!("Total Time: {}", format_duration(time.elapsed()));
    }
}
