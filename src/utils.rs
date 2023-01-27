use chrono::{self, Datelike, Timelike};
use flate2::read::ZlibDecoder;
use std::io::prelude::*;

use crate::client::DataPack;

pub fn fill_datapack_header(type_pack: DataPack, data_pack: &mut [u8], sequence: u32) {
    let pack_len = data_pack.len() as u32;
    for (i, byte) in pack_len.to_be_bytes().iter().enumerate() {
        data_pack[i] = *byte;
    }
    let head_len: u16 = 16;
    let mut ver: u16 = 0;
    let mut op_code: u32 = 0;
    let mut offset = 4;

    match type_pack {
        DataPack::Auth => {
            (ver, op_code) = (1, 7);
        }
        DataPack::HeartBeat => {
            (ver, op_code) = (1, 2);
        }
        _ => {}
    }

    let bs1 = head_len.to_be_bytes();
    let bs2 = ver.to_be_bytes();
    let bs3 = op_code.to_be_bytes();
    let bs4 = sequence.to_be_bytes();

    for byte in bs1 {
        data_pack[offset] = byte;
        offset += 1;
    }
    for byte in bs2 {
        data_pack[offset] = byte;
        offset += 1;
    }
    for byte in bs3 {
        data_pack[offset] = byte;
        offset += 1;
    }
    for byte in bs4 {
        data_pack[offset] = byte;
        offset += 1;
    }
}

pub fn zlib_dec(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut d = ZlibDecoder::new(data);
    let mut buf = Vec::new();
    d.read_to_end(&mut buf)?;

    Ok(buf)
}

pub fn split_packs(data: &[u8]) -> Vec<Vec<u8>> {
    let total_len = data.len();
    let mut packs: Vec<Vec<u8>> = vec![];
    // start point of per pack
    let mut start: usize = 0;
    // length of per pack
    let mut len = 0;
    while start + len < total_len {
        let mut pack: Vec<u8> = vec![];
        len = u32::from_be_bytes(data[(start)..(start + 4)].try_into().unwrap()) as usize;

        pack.extend_from_slice(&data[start + 16..start + len]);
        packs.push(pack);

        start += len;
    }

    packs
}

pub fn timestamp_to_datetime(ts: u64) -> chrono::DateTime<chrono::Utc> {
    let offset = chrono::FixedOffset::east_opt(8 * 3600).unwrap();

    // UTC+8
    chrono::DateTime::<chrono::Utc>::from(
        std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts),
    ) + offset
}

pub fn timestamp_to_time_minutes(ts: u64) -> chrono::Duration {
    chrono::Duration::minutes(ts as i64 / 60)
}

pub fn duration(start_ts: u64) -> chrono::Duration {
    let now_ts = chrono::Utc::now().timestamp() as u64;
    let duration_secs = now_ts - start_ts;

    timestamp_to_time_minutes(duration_secs)
}

pub fn display_duration(duration: chrono::Duration) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() - hours * 60;

    format!("{}:{}", hours, minutes)
}

#[test]
fn test_fill_datapack_header() {
    let mut data_pack: Vec<u8> = vec![0; 32];
    let expected: [u16; 16] = [0, 32, 16, 1, 0, 7, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0];
    fill_datapack_header(DataPack::Auth, data_pack.as_mut_slice(), 1);
    unsafe {
        let raw_expected = &expected as *const u16 as *const u8;
        let new_expected = std::slice::from_raw_parts(raw_expected.wrapping_sub(1), 32);
        assert_eq!(new_expected, data_pack.as_slice());
    }
}
