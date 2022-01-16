use anyhow::{Context, Result};
use find_winsdk::{SdkInfo, SdkVersion};
use glob::glob;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
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

fn get_compiled_path(path: &Path, output_dir: &Path, ty: &ShaderType) -> (PathBuf, PathBuf) {
    let mut file_name = path.file_name().unwrap().to_owned();
    file_name.push("_");
    file_name.push(match ty {
        ShaderType::Vertex => "v",
        ShaderType::Pixel => "p",
    });
    let relative_path = path
        .strip_prefix(output_dir.parent().unwrap())
        .unwrap()
        .with_file_name(file_name)
        .with_extension("dxbc");
    let mut absolute_path = output_dir.to_path_buf();
    absolute_path.push(&relative_path);
    (absolute_path, relative_path)
}

#[cfg(target_os = "windows")]
fn main() -> Result<()> {
    let fxc = get_fxc_path()?;

    let shader_dir = fs::canonicalize(std::env::current_exe()?.join("../../../shaders"))?;
    let mut output_dir = shader_dir.clone();
    output_dir.push("output");
    let mut shaders: Vec<(PathBuf, ShaderType)> = vec![];
    println!("{}", shader_dir.display());

    for entry in WalkDir::new(&shader_dir).into_iter().filter_map(|e| e.ok()) {
        let path = match fs::canonicalize(entry.into_path()) {
            Ok(p) => p,
            Err(_) => continue,
        };
        match path.extension() {
            Some(ext) => {
                if ext.to_string_lossy() == "hlsl" {
                    let text = match fs::read_to_string(&path) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };
                    if text.contains("VSMain")
                        && !get_compiled_path(&path, &output_dir, &ShaderType::Vertex)
                            .0
                            .exists()
                    {
                        shaders.push((path.clone(), ShaderType::Vertex));
                    }
                    if text.contains("PSMain")
                        && !get_compiled_path(&path, &output_dir, &ShaderType::Pixel)
                            .0
                            .exists()
                    {
                        shaders.push((path.clone(), ShaderType::Pixel));
                    }
                }
            }
            None => continue,
        };
    }

    println!("Shaders to compile: {}", shaders.len());

    shaders
        .par_iter()
        .progress_count(shaders.len() as u64)
        .for_each(|(path, ty)| {
            let (profile, entry_point) = match ty {
                ShaderType::Vertex => ("vs_5_1", "VSMain"),
                ShaderType::Pixel => ("ps_5_1", "PSMain"),
            };
            let (compiled_path, relative_path) = get_compiled_path(path, &output_dir, ty);
            if let Some(parent) = compiled_path.parent() {
                match fs::create_dir_all(parent) {
                    Ok(_) => (),
                    Err(_) => {
                        println!(
                            "Error creating output directory {}!",
                            relative_path.display()
                        );
                        return;
                    }
                }
            }

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
                .output()
            {
                Ok(o) => o,
                Err(_) => return,
            };
            if !output.status.success() {
                println!(
                    "Compilation of {} failed (status code {})",
                    path.display(),
                    output.status
                );
                match std::str::from_utf8(&output.stderr) {
                    Ok(s) => println!("{}", s),
                    Err(_) => println!("UTF-8 error!"),
                }
            }
        });

    Ok(())
}
