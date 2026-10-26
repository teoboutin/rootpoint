//! Throwaway prototype: draw the editor with no window and fill egui's triangles on the CPU.

use std::collections::HashMap;
use std::time::Instant;

use eframe::egui::{
    Color32, Context, ImageData, Pos2, RawInput, Rect, TextureId, Vec2, epaint::Primitive,
};
use rootpoint_app::App;

struct Tex {
    size: [usize; 2],
    pixels: Vec<Color32>,
}

fn frame(ctx: &Context, app: &mut App, size: Vec2, ppp: f32, textures: &mut HashMap<TextureId, Tex>, t: f64) -> eframe::egui::FullOutput {
    let mut input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
        max_texture_side: Some(8192),
        time: Some(t),
        ..Default::default()
    };
    input.viewports.entry(input.viewport_id).or_default().native_pixels_per_point = Some(ppp);
    let mut out = ctx.run_ui(input, |ui| app.frame(ui));
    for (id, deltas) in &out.textures_delta.set {
      for delta in deltas.iter() {
        let ImageData::Color(img) = &delta.image;
        match delta.pos {
            None => {
                textures.insert(*id, Tex { size: img.size, pixels: img.pixels.clone() });
            }
            Some([x0, y0]) => {
                let t = textures.get_mut(id).unwrap();
                for y in 0..img.size[1] {
                    for x in 0..img.size[0] {
                        t.pixels[(y0 + y) * t.size[0] + x0 + x] = img.pixels[y * img.size[0] + x];
                    }
                }
            }
        }
      }
    }
    for id in &out.textures_delta.free {
        textures.remove(id);
    }
    out.textures_delta.clear();
    out
}

fn sample(t: &Tex, u: f32, v: f32) -> [f32; 4] {
    let x = (u * t.size[0] as f32 - 0.5).clamp(0.0, (t.size[0] - 1) as f32);
    let y = (v * t.size[1] as f32 - 0.5).clamp(0.0, (t.size[1] - 1) as f32);
    let (x0, y0) = (x.floor() as usize, y.floor() as usize);
    let (x1, y1) = ((x0 + 1).min(t.size[0] - 1), (y0 + 1).min(t.size[1] - 1));
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let p = |x: usize, y: usize| t.pixels[y * t.size[0] + x].to_array().map(|c| c as f32);
    let (a, b, c, d) = (p(x0, y0), p(x1, y0), p(x0, y1), p(x1, y1));
    std::array::from_fn(|i| {
        (a[i] * (1.0 - fx) + b[i] * fx) * (1.0 - fy) + (c[i] * (1.0 - fx) + d[i] * fx) * fy
    })
}

fn rasterize(prims: &[eframe::egui::ClippedPrimitive], textures: &HashMap<TextureId, Tex>, w: usize, h: usize, ppp: f32) -> Vec<[f32; 4]> {
    let mut buf = vec![[0.0f32; 4]; w * h];
    for prim in prims {
        let Primitive::Mesh(mesh) = &prim.primitive else { continue };
        let tex = &textures[&mesh.texture_id];
        let clip = prim.clip_rect;
        let (cx0, cy0) = ((clip.min.x * ppp).max(0.0) as usize, (clip.min.y * ppp).max(0.0) as usize);
        let (cx1, cy1) = (((clip.max.x * ppp).ceil() as usize).min(w), ((clip.max.y * ppp).ceil() as usize).min(h));
        for tri in mesh.indices.chunks_exact(3) {
            let v = [&mesh.vertices[tri[0] as usize], &mesh.vertices[tri[1] as usize], &mesh.vertices[tri[2] as usize]];
            let p: [Pos2; 3] = std::array::from_fn(|i| (v[i].pos.to_vec2() * ppp).to_pos2());
            let area = (p[1].x - p[0].x) * (p[2].y - p[0].y) - (p[2].x - p[0].x) * (p[1].y - p[0].y);
            if area.abs() < 1e-6 {
                continue;
            }
            let minx = p.iter().map(|q| q.x).fold(f32::MAX, f32::min).floor().max(cx0 as f32) as usize;
            let maxx = (p.iter().map(|q| q.x).fold(f32::MIN, f32::max).ceil() as usize).min(cx1);
            let miny = p.iter().map(|q| q.y).fold(f32::MAX, f32::min).floor().max(cy0 as f32) as usize;
            let maxy = (p.iter().map(|q| q.y).fold(f32::MIN, f32::max).ceil() as usize).min(cy1);
            for y in miny..maxy {
                for x in minx..maxx {
                    let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                    let e = |a: Pos2, b: Pos2| (b.x - a.x) * (py - a.y) - (px - a.x) * (b.y - a.y);
                    let w0 = e(p[1], p[2]) / area;
                    let w1 = e(p[2], p[0]) / area;
                    let w2 = e(p[0], p[1]) / area;
                    if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                        continue;
                    }
                    let col: [f32; 4] = std::array::from_fn(|i| {
                        let c = |k: usize| v[k].color.to_array()[i] as f32;
                        c(0) * w0 + c(1) * w1 + c(2) * w2
                    });
                    let u = v[0].uv.x * w0 + v[1].uv.x * w1 + v[2].uv.x * w2;
                    let vv = v[0].uv.y * w0 + v[1].uv.y * w1 + v[2].uv.y * w2;
                    let t = sample(tex, u, vv);
                    let src: [f32; 4] = std::array::from_fn(|i| col[i] * t[i] / 255.0);
                    let d = &mut buf[y * w + x];
                    let a = src[3] / 255.0;
                    for i in 0..4 {
                        d[i] = src[i] + d[i] * (1.0 - a);
                    }
                }
            }
        }
    }
    buf
}

#[test]
#[ignore]
fn snapshot_of_the_editor() {
    let out_dir = std::env::var("PROTO_OUT").unwrap();
    let ppp: f32 = std::env::var("PROTO_PPP").ok().and_then(|s| s.parse().ok()).unwrap_or(1.0);
    let mut app = App::in_memory();
    app.open_sample();
    let ctx = Context::default();
    let size = Vec2::new(1600.0, 900.0);
    let mut textures = HashMap::new();
    let mut out = None;
    for i in 0..4 {
        out = Some(frame(&ctx, &mut app, size, ppp, &mut textures, i as f64 / 60.0));
    }
    let out = out.unwrap();
    let start = Instant::now();
    let prims = ctx.tessellate(out.shapes, out.pixels_per_point);
    let tess = start.elapsed();
    let (w, h) = ((size.x * ppp) as usize, (size.y * ppp) as usize);
    let buf = rasterize(&prims, &textures, w, h, ppp);
    let total = start.elapsed();
    let triangles: usize = prims.iter().map(|p| match &p.primitive { Primitive::Mesh(m) => m.indices.len() / 3, _ => 0 }).sum();
    println!("ppp {ppp}: {} primitives, {triangles} triangles, tessellate {tess:?}, total {total:?}", prims.len());
    let bytes: Vec<u8> = buf.iter().flat_map(|p| p.map(|c| c.round().clamp(0.0, 255.0) as u8)).collect();
    image::save_buffer(format!("{out_dir}/editor-{ppp}.png"), &bytes, w as u32, h as u32, image::ExtendedColorType::Rgba8).unwrap();
    if let Some(r) = app.button_rect("Preview") {
        println!("Preview button at {r:?}");
    }
}
