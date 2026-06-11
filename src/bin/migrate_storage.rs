use std::env;
use std::path::PathBuf;

use khaslana::{
    AppStorage, default_database_path, default_legacy_storage_paths, legacy_storage_paths,
};

fn main() {
    if let Err(err) = run() {
        eprintln!("迁移失败：{err}");
        std::process::exit(1);
    }
}

fn run() -> khaslana::Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let force = args.iter().any(|arg| arg == "--force");
    let legacy_dir = arg_value(&args, "--legacy-dir").map(PathBuf::from);
    let db_path = arg_value(&args, "--db").map(PathBuf::from);

    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(());
    }

    let db_path = db_path
        .or_else(default_database_path)
        .ok_or_else(|| khaslana::GitError::Message("无法定位默认 SQLite 数据库路径".into()))?;
    // 迁移工具是唯一读取旧 JSON 的入口，主程序本身不保留旧格式兼容路径。
    let legacy_paths = if let Some(dir) = legacy_dir {
        legacy_storage_paths(&dir)
    } else {
        default_legacy_storage_paths()
            .ok_or_else(|| khaslana::GitError::Message("无法定位旧 JSON 配置目录".into()))?
    };

    if db_path.exists() && !force {
        let storage = AppStorage::open(&db_path)?;
        let summary = storage.import_legacy_json(&legacy_paths, false)?;
        print_summary(&db_path, &summary, false);
        return Ok(());
    }

    if db_path.exists() && force {
        std::fs::remove_file(&db_path)?;
    }

    let storage = AppStorage::open(&db_path)?;
    let summary = storage.import_legacy_json(&legacy_paths, force)?;
    print_summary(&db_path, &summary, force);
    Ok(())
}

fn arg_value(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == name)
        .map(|window| window[1].clone())
}

fn print_summary(db_path: &std::path::Path, summary: &khaslana::LegacyImportSummary, force: bool) {
    println!("SQLite 数据库：{}", db_path.display());
    println!(
        "导入模式：{}",
        if force {
            "强制重建"
        } else {
            "保守导入"
        }
    );
    println!(
        "导入结果：session={} diff_encodings={} remote_credentials={} network_proxy={} credentials={}",
        imported(summary.session),
        imported(summary.diff_encodings),
        imported(summary.remote_credentials),
        imported(summary.network_proxy),
        imported(summary.credentials),
    );
    println!("旧 JSON 文件未删除，可按需手动备份或清理。");
}

fn imported(value: bool) -> &'static str {
    if value { "已导入" } else { "跳过" }
}

fn print_help() {
    println!(
        "Khaslana SQLite 迁移工具\n\
         \n\
         用法：\n\
           cargo run --bin migrate_storage\n\
           cargo run --bin migrate_storage -- --force\n\
           cargo run --bin migrate_storage -- --legacy-dir <旧配置目录> --db <数据库路径>\n\
         \n\
         说明：\n\
           默认读取 Khaslana 配置目录中的旧 JSON 文件，写入 khaslana.sqlite3。\n\
           密钥仍保留在系统 Keyring，本工具只迁移 credentials.json 中的非密索引。"
    );
}
