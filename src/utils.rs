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
