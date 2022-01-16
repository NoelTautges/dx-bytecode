use anyhow::{Result, Context};
use find_winsdk::{SdkInfo, SdkVersion};
use glob::glob;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug)]
enum ShaderType {
    Vertex,
    Pixel,
}

fn get_fxc_path() -> Result<PathBuf> {
    let sdk = SdkInfo::find(SdkVersion::Any)
        .with_context(|| "Error while finding Windows SDK!")?
        .with_context(|| "Couldn't find Windows SDK!")?;
    let sdk_path = sdk.installation_folder();
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        s => s,
    };
    let fxc_glob = format!("{}/bin/*/{}/fxc.exe", sdk_path.display(), arch);
    let fxc = glob(&fxc_glob)?
        .last()
        .with_context(|| format!("Failed to find fxc.exe! Looked at: {}", fxc_glob))??;
    Ok(fxc)
}

fn get_compiled_path(path: &PathBuf, ty: &ShaderType) -> PathBuf {
    let mut extended = path.file_name().unwrap().to_owned();
    extended.push("_");
    extended.push(match ty {
        ShaderType::Vertex => "v",
        ShaderType::Pixel => "p",
    });
    path.with_file_name(extended).with_extension("dxbc")
}

#[cfg(target_os = "windows")]
fn main() -> Result<()> {
    let fxc = get_fxc_path()?;

    let shaders_dir = fs::canonicalize(std::env::current_exe()?.join("../../../shaders"))?;
    let mut shaders: Vec<(PathBuf, ShaderType)> = vec![];
    println!("{}", shaders_dir.display());

    for entry in WalkDir::new(&shaders_dir).into_iter().filter_map(|e| e.ok()) {
        let path = match fs::canonicalize(entry.into_path()) {
            Ok(p) => p,
            Err(_) => continue,
        };
        match path.extension() {
            Some(ext) => if ext.to_string_lossy() == "hlsl" {
                let text = match fs::read_to_string(&path) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                if text.contains("VSMain") && !get_compiled_path(&path, &ShaderType::Vertex).exists() {
                    shaders.push((path.clone(), ShaderType::Vertex));
                }
                if text.contains("PSMain") && !get_compiled_path(&path, &ShaderType::Pixel).exists() {
                    shaders.push((path.clone(), ShaderType::Pixel));
                }
            },
            None => continue,
        };
    }

    println!("Shaders to compile: {}", shaders.len());

    shaders.par_iter().for_each(|(path, ty)| {
        match path.strip_prefix(&shaders_dir) {
            Ok(relative) => println!("Compiling {}", relative.display()),
            Err(_) => return,
        }
        let (profile, entry_point) = match ty {
            &ShaderType::Vertex => ("vs_5_1", "VSMain"),
            &ShaderType::Pixel => ("ps_5_1", "PSMain"),
        };
        let compiled_path = get_compiled_path(path, ty);
        
        let output = match Command::new(&fxc)
            .args([
                "/T",
                profile,
                "/E",
                entry_point,
                "/Fo",
                &compiled_path.to_string_lossy(),
                &path.to_string_lossy(),
            ])
            .output() {
                Ok(o) => o,
                Err(_) => return,
            };
        if !output.status.success() {
            println!("Compilation of {} failed (status code {})", path.display(), output.status);
            match std::str::from_utf8(&output.stderr) {
                Ok(s) => println!("{}", s),
                Err(_) => println!("UTF-8 error!"),
            }
        }
    });

    Ok(())
}
