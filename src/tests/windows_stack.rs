use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

#[test]
fn windows_gui_reserves_stack_without_eager_commit() {
    // 验证链接后的 PE，而非检查构建脚本文本，保证链接参数确实作用于 GUI 目标。
    let mut executable = File::open(std::env::current_exe().unwrap()).unwrap();
    executable.seek(SeekFrom::Start(60)).unwrap();
    let mut offset = [0; 4];
    executable.read_exact(&mut offset).unwrap();
    let pe = u32::from_le_bytes(offset) as u64;
    executable.seek(SeekFrom::Start(pe)).unwrap();
    let mut signature = [0; 4];
    executable.read_exact(&mut signature).unwrap();
    assert_eq!(&signature, b"PE\0\0");
    executable.seek(SeekFrom::Start(pe + 24)).unwrap();
    let mut magic = [0; 2];
    executable.read_exact(&mut magic).unwrap();
    executable.seek(SeekFrom::Start(pe + 24 + 72)).unwrap();
    let (reserve, commit) = match u16::from_le_bytes(magic) {
        0x20b => {
            let mut values = [0; 16];
            executable.read_exact(&mut values).unwrap();
            (u64::from_le_bytes(values[..8].try_into().unwrap()), u64::from_le_bytes(values[8..].try_into().unwrap()))
        }
        0x10b => {
            let mut values = [0; 8];
            executable.read_exact(&mut values).unwrap();
            (u32::from_le_bytes(values[..4].try_into().unwrap()) as u64, u32::from_le_bytes(values[4..].try_into().unwrap()) as u64)
        }
        other => panic!("未知 PE 格式：{other:x}"),
    };
    assert!(reserve >= 8 * 1024 * 1024, "GUI 主线程栈预留不足：{reserve}");
    assert_eq!(commit, 4096, "增加栈预留不能同时增加初始提交量");
}
