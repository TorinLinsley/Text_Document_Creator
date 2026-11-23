/// 功能：编译期调用 rc.exe 嵌入图标、版本信息与清单
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 功能：构建入口
fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=assets/icon.ico");

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let res = out_dir.join("app.res");
    let rc = find_rc().expect("未找到 rc.exe，请设置环境变量 RC_EXE 指向 Windows SDK 的 rc.exe");
    let status = Command::new(&rc)
        .arg("/nologo")
        .arg("/fo")
        .arg(&res)
        .arg("app.rc")
        .current_dir(&manifest)
        .status()
        .expect("启动 rc.exe 失败");
    assert!(status.success(), "rc.exe 编译资源失败");
    println!("cargo:rustc-link-arg={}", res.display());
}

/// 功能：定位 Windows SDK 资源编译器
fn find_rc() -> Option<PathBuf> {
    if let Ok(p) = env::var("RC_EXE") {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Some(p);
        }
    }
    for root in sdk_roots() {
        if let Some(found) = scan_bin(&root) {
            return Some(found);
        }
    }
    None
}

/// 功能：枚举可能的 SDK bin 根目录
fn sdk_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for key in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Ok(base) = env::var(key) {
            roots.push(Path::new(&base).join("Windows Kits").join("10").join("bin"));
        }
    }
    for drive in ["C", "D", "E", "F"] {
        roots.push(PathBuf::from(format!("{}:\\Windows Kits\\10\\bin", drive)));
    }
    roots
}

/// 功能：在 SDK bin 下挑选版本号最大的 x64/rc.exe
fn scan_bin(root: &Path) -> Option<PathBuf> {
    let mut versions: Vec<(u64, PathBuf)> = Vec::new();
    for entry in std::fs::read_dir(root).ok()? {
        let entry = entry.ok()?;
        let name = entry.file_name().to_string_lossy().to_string();
        let numeric = name.chars().all(|c| c.is_ascii_digit() || c == '.');
        if !numeric {
            continue;
        }
        let exe = entry.path().join("x64").join("rc.exe");
        if !exe.is_file() {
            continue;
        }
        let weight = name
            .split('.')
            .filter_map(|p| p.parse::<u64>().ok())
            .fold(0u64, |acc, v| acc * 1000 + v);
        versions.push((weight, exe));
    }
    versions.sort_by(|a, b| b.0.cmp(&a.0));
    versions.into_iter().next().map(|(_, p)| p)
}
