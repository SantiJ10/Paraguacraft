//! Pre-genera texturas y resource packs de branding en compile time.
//! En runtime solo se copia el artefacto correcto según la versión de MC.

use std::env;
use std::fs::{self, File};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use image::imageops::FilterType;
use image::{GenericImageView, Rgba, RgbaImage};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let res = manifest.join("resources/branding");
    let packs_out = out.join("packs");
    fs::create_dir_all(&packs_out).expect("create OUT_DIR/packs");

    let main_menu = fs::read(res.join("paraguacraft_main_menu.png")).expect("main_menu png");
    let legacy_src = fs::read(res.join("paraguacraft_legacy.png")).expect("legacy png");
    let startup = fs::read(res.join("paraguacraft_startup.png")).expect("startup png");

    let split256 = bake_split256(&main_menu);
    let legacy256 = bake_legacy256(&legacy_src);
    let wide1024 = bake_wide1024(&main_menu);
    let bedrock = bake_bedrock_title(&main_menu);

    split256
        .save(out.join("minecraft_split256.png"))
        .expect("save split256");
    legacy256
        .save(out.join("minecraft_legacy256.png"))
        .expect("save legacy256");
    wide1024
        .save(out.join("minecraft_wide1024.png"))
        .expect("save wide1024");
    bedrock
        .save(out.join("bedrock_title.png"))
        .expect("save bedrock");
    fs::write(out.join("mojangstudios.png"), &startup).expect("save startup");

    let edition = RgbaImage::from_pixel(128, 16, Rgba([0, 0, 0, 0]));
    edition.save(out.join("edition.png")).expect("save edition");

    // < 1.6 — texture pack clásico
    write_texturepack_zip(
        &packs_out.join("classic.zip"),
        &legacy256,
        None,
    );

    // 1.6 – 1.15
    write_java_pack_zip(
        &packs_out.join("legacy.zip"),
        &legacy256,
        &startup,
        &edition,
        r#"{"pack":{"pack_format":4,"description":"Marca Oficial Paraguacraft"}}"#,
    );

    // 1.16 – 1.20.1
    write_java_pack_zip(
        &packs_out.join("standard.zip"),
        &split256,
        &startup,
        &edition,
        r#"{"pack":{"pack_format":6,"description":"Marca Oficial Paraguacraft"}}"#,
    );

    // 1.20.2 – 1.21.3
    write_java_pack_zip(
        &packs_out.join("standard_range.zip"),
        &split256,
        &startup,
        &edition,
        r#"{"pack":{"pack_format":15,"description":"Marca Oficial Paraguacraft","supported_formats":[15,9999]}}"#,
    );

    // 1.21.4 – 1.21.4 / snapshots amplios con textura wide
    write_java_pack_zip(
        &packs_out.join("wide.zip"),
        &wide1024,
        &startup,
        &edition,
        r#"{"pack":{"pack_format":46,"description":"Marca Oficial Paraguacraft","supported_formats":[46,9999]}}"#,
    );

    // 1.21.5+ / 26.x — schema nuevo (zip en resourcepacks/)
    write_java_pack_zip(
        &packs_out.join("modern.zip"),
        &wide1024,
        &startup,
        &edition,
        r#"{"pack":{"description":"Marca Oficial Paraguacraft","min_format":70,"max_format":99999}}"#,
    );

    println!("cargo:rerun-if-changed=resources/branding");

    // JARs PvP embebidos en el instalador (offline / sin depender del release remoto).
    let repo_bundled = manifest.join("../../bundled/pvp");
    let res_pvp = manifest.join("resources/bundled/pvp");
    if repo_bundled.is_dir() {
        let _ = fs::create_dir_all(&res_pvp);
        for entry in fs::read_dir(&repo_bundled).into_iter().flatten().flatten() {
            let p = entry.path();
            let name = entry.file_name();
            if p.is_dir() {
                if name == "defaults" {
                    let _ = copy_dir_all(&p, &res_pvp.join("defaults"));
                }
                continue;
            }
            let is_jar = p.extension().and_then(|e| e.to_str()) == Some("jar");
            let is_manifest = name == "manifest.json";
            if is_jar || is_manifest {
                let _ = fs::copy(&p, res_pvp.join(name));
            }
        }
        println!("cargo:rerun-if-changed=../../bundled/pvp");
    }

    tauri_build::build();
}

fn alpha_bbox(img: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let (w, h) = img.dimensions();
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    let mut found = false;
    for y in 0..h {
        for x in 0..w {
            if img.get_pixel(x, y)[3] > 10 {
                found = true;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x + 1);
                max_y = max_y.max(y + 1);
            }
        }
    }
    found.then_some((min_x, min_y, max_x, max_y))
}

fn crop_bbox(img: &RgbaImage, bbox: (u32, u32, u32, u32)) -> RgbaImage {
    img.view(
        bbox.0,
        bbox.1,
        bbox.2 - bbox.0,
        bbox.3 - bbox.1,
    )
    .to_image()
}

fn load_rgba(bytes: &[u8]) -> RgbaImage {
    let mut im = image::load_from_memory(bytes)
        .expect("decode png")
        .to_rgba8();
    if let Some(b) = alpha_bbox(&im) {
        im = crop_bbox(&im, b);
    }
    im
}

fn bake_split256(main_menu: &[u8]) -> RgbaImage {
    const HALF_W: u32 = 155;
    const RENDER_H: u32 = 44;
    const ROW2_Y: u32 = 45;
    const TW: u32 = 256;
    const TH: u32 = 256;

    let im = load_rgba(main_menu);
    let (w, h) = im.dimensions();
    let scale = ((HALF_W * 2) as f64 / w as f64).min(RENDER_H as f64 / h as f64);
    let new_w = ((w as f64 * scale).round() as u32).max(1);
    let new_h = ((h as f64 * scale).round() as u32).max(1);
    let logo_scaled = image::imageops::resize(&im, new_w, new_h, FilterType::Lanczos3);
    let mut canvas = RgbaImage::from_pixel(HALF_W * 2, RENDER_H, Rgba([0, 0, 0, 0]));
    let px = (HALF_W * 2 - new_w) / 2;
    let py = (RENDER_H - new_h) / 2;
    image::imageops::overlay(&mut canvas, &logo_scaled, px.into(), py.into());
    let mut out = RgbaImage::from_pixel(TW, TH, Rgba([0, 0, 0, 0]));
    let left = canvas.view(0, 0, HALF_W, RENDER_H).to_image();
    let right = canvas.view(HALF_W, 0, HALF_W, RENDER_H).to_image();
    image::imageops::overlay(&mut out, &left, 0, 0);
    image::imageops::overlay(&mut out, &right, 0, ROW2_Y.into());
    out
}

fn bake_legacy256(legacy: &[u8]) -> RgbaImage {
    const HALF_W: u32 = 155;
    const RENDER_H: u32 = 44;
    const ROW2_Y: u32 = 45;
    const GAP: u32 = 2;
    const TW: u32 = 256;
    const TH: u32 = 256;

    let im = load_rgba(legacy);
    let (w, h) = im.dimensions();
    let row_alpha: Vec<u32> = (0..h)
        .map(|y| {
            (0..w)
                .filter(|&x| im.get_pixel(x, y)[3] > 10)
                .count() as u32
        })
        .collect();
    let s0 = (h / 4) as usize;
    let s1 = (3 * h / 4) as usize;
    let min_val = row_alpha[s0..s1].iter().copied().min().unwrap_or(0);
    let gap_rows: Vec<u32> = (s0..s1)
        .filter(|&i| row_alpha[i] == min_val)
        .map(|i| i as u32)
        .collect();
    let split_y = gap_rows
        .get(gap_rows.len() / 2)
        .copied()
        .unwrap_or(h / 2);

    let mut top_half = im.view(0, 0, w, split_y).to_image();
    let mut bot_half = im.view(0, split_y, w, h - split_y).to_image();
    if let Some(b) = alpha_bbox(&top_half) {
        top_half = crop_bbox(&top_half, b);
    }
    if let Some(b) = alpha_bbox(&bot_half) {
        bot_half = crop_bbox(&bot_half, b);
    }

    let (tw, th) = top_half.dimensions();
    let (bw, bh) = bot_half.dimensions();
    let mut new_tw = ((tw as f64 * RENDER_H as f64 / th as f64).round() as u32).max(1);
    let mut new_bw = ((bw as f64 * RENDER_H as f64 / bh as f64).round() as u32).max(1);
    let (new_th, new_bh) = if new_tw + GAP + new_bw > HALF_W * 2 {
        let fit = (HALF_W * 2 - GAP) as f64 / (new_tw + new_bw) as f64;
        new_tw = ((new_tw as f64 * fit).round() as u32).max(1);
        new_bw = ((new_bw as f64 * fit).round() as u32).max(1);
        (
            ((RENDER_H as f64 * fit).round() as u32).max(1),
            ((RENDER_H as f64 * fit).round() as u32).max(1),
        )
    } else {
        (RENDER_H, RENDER_H)
    };

    let top_scaled = image::imageops::resize(&top_half, new_tw, new_th, FilterType::Lanczos3);
    let bot_scaled = image::imageops::resize(&bot_half, new_bw, new_bh, FilterType::Lanczos3);
    let combined_w = new_tw + GAP + new_bw;
    let mut canvas = RgbaImage::from_pixel(HALF_W * 2, RENDER_H, Rgba([0, 0, 0, 0]));
    let start_x = (HALF_W * 2 - combined_w) / 2;
    image::imageops::overlay(
        &mut canvas,
        &top_scaled,
        start_x.into(),
        ((RENDER_H - new_th) / 2).into(),
    );
    image::imageops::overlay(
        &mut canvas,
        &bot_scaled,
        (start_x + new_tw + GAP).into(),
        ((RENDER_H - new_bh) / 2).into(),
    );
    let mut out = RgbaImage::from_pixel(TW, TH, Rgba([0, 0, 0, 0]));
    let left = canvas.view(0, 0, HALF_W, RENDER_H).to_image();
    let right = canvas.view(HALF_W, 0, HALF_W, RENDER_H).to_image();
    image::imageops::overlay(&mut out, &left, 0, 0);
    image::imageops::overlay(&mut out, &right, 0, ROW2_Y.into());
    out
}

fn bake_wide1024(main_menu: &[u8]) -> RgbaImage {
    const TW: u32 = 1024;
    const TH: u32 = 176;
    let im = load_rgba(main_menu);
    let (w, h) = im.dimensions();
    let scale = (TW as f64 / w as f64).min(TH as f64 / h as f64);
    let new_w = ((w as f64 * scale).round() as u32).max(1);
    let new_h = ((h as f64 * scale).round() as u32).max(1);
    let logo_scaled = image::imageops::resize(&im, new_w, new_h, FilterType::Lanczos3);
    let mut out = RgbaImage::from_pixel(TW, TH, Rgba([0, 0, 0, 0]));
    image::imageops::overlay(
        &mut out,
        &logo_scaled,
        ((TW - new_w) / 2).into(),
        ((TH - new_h) / 2).into(),
    );
    out
}

fn bake_bedrock_title(main_menu: &[u8]) -> RgbaImage {
    const TW: u32 = 512;
    const TH: u32 = 128;
    let im = load_rgba(main_menu);
    let (w, h) = im.dimensions();
    let scale = (TW as f64 / w as f64).min(TH as f64 / h as f64);
    let new_w = ((w as f64 * scale).round() as u32).max(1);
    let new_h = ((h as f64 * scale).round() as u32).max(1);
    let logo_scaled = image::imageops::resize(&im, new_w, new_h, FilterType::Lanczos3);
    let mut out = RgbaImage::from_pixel(TW, TH, Rgba([0, 0, 0, 0]));
    image::imageops::overlay(
        &mut out,
        &logo_scaled,
        ((TW - new_w) / 2).into(),
        ((TH - new_h) / 2).into(),
    );
    out
}

fn png_bytes(img: &RgbaImage) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png)
        .expect("encode png");
    buf.into_inner()
}

fn write_texturepack_zip(path: &Path, minecraft: &RgbaImage, _mcmeta: Option<&str>) {
    let file = File::create(path).expect("create classic zip");
    let mut zip = ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let buf = png_bytes(minecraft);
    zip.start_file("gui/title/minecraft.png", opts)
        .expect("zip entry");
    zip.write_all(&buf).expect("zip write");
    zip.finish().expect("zip finish");
}

fn write_java_pack_zip(
    path: &Path,
    minecraft: &RgbaImage,
    startup: &[u8],
    edition: &RgbaImage,
    mcmeta: &str,
) {
    let file = File::create(path).expect("create pack zip");
    let mut zip = ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let base = "assets/minecraft/textures/gui/title";

    zip.start_file("pack.mcmeta", opts).expect("mcmeta");
    zip.write_all(mcmeta.as_bytes()).expect("mcmeta bytes");

    let mc_buf = png_bytes(minecraft);
    zip.start_file(format!("{base}/minecraft.png"), opts)
        .expect("minecraft entry");
    zip.write_all(&mc_buf).expect("minecraft bytes");

    zip.start_file(format!("{base}/mojangstudios.png"), opts)
        .expect("startup entry");
    zip.write_all(startup).expect("startup bytes");

    let ed_buf = png_bytes(edition);
    zip.start_file(format!("{base}/edition.png"), opts)
        .expect("edition entry");
    zip.write_all(&ed_buf).expect("edition bytes");

    zip.finish().expect("zip finish");
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &to)?;
        } else {
            fs::copy(entry.path(), to)?;
        }
    }
    Ok(())
}
