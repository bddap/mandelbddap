use std::iter::repeat_with;

use macroquad::prelude::*;
use num::{complex::Complex32, Complex};

#[macroquad::main("Mandelbddap")]
async fn main() {
    let mut hw = (screen_height(), screen_width());
    // let mut image = get_screen_data();
    let mut image = canvas_of_apropriate_size();
    let mut gradid: usize = 0;
    let mut mb = MandelBrot {
        colorgrad: GRADIENTS[gradid % GRADIENTS.len()](),
        iter_max: 100,
        mouse_pos: Complex::new(0.0, 0.0),
        zoom: 0.5 / 2.0,
    };

    loop {
        if is_key_pressed(KeyCode::Q) {
            return;
        }
        if is_key_pressed(KeyCode::W) {
            mb.zoom *= 0.9;
        }
        if is_key_pressed(KeyCode::E) {
            mb.zoom *= 1.1;
        }
        if is_key_pressed(KeyCode::R) {
            gradid += 1;
            mb.colorgrad = GRADIENTS[gradid % GRADIENTS.len()]();
        }
        if is_key_pressed(KeyCode::A) {
            mb.iter_max += 1;
        }
        if is_key_pressed(KeyCode::S) {
            mb.iter_max = mb.iter_max.saturating_sub(1);
        }

        let nhw = (screen_height(), screen_width());
        if hw != nhw {
            image = canvas_of_apropriate_size();
            hw = nhw;
        }

        let aspect = screen_width() / screen_height();
        let (mut mx, mut my) = mouse_position();
        mx = mx / screen_width() * 2.0 - 1.0;
        my = (my / screen_height() * 2.0 - 1.0) / aspect;
        mb.mouse_pos = Complex::new(mx, my);

        let iw = image.width();
        let iwf = iw as f32;
        let ih = image.height();
        let ihf = ih as f32;
        for x in 0..iw {
            for y in 0..ih {
                let c = mb.get(
                    x as f32 / iwf * 2.0 - 1.0,
                    (y as f32 / ihf * 2.0 - 1.0) / aspect,
                );
                image.set_pixel(x as u32, y as u32, c);
            }
        }
        let tex = Texture2D::from_image(&image);

        draw_texture_ex(
            tex,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(screen_width(), screen_height())),
                ..DrawTextureParams::default()
            },
        );

        next_frame().await
    }
}

struct MandelBrot {
    colorgrad: colorgrad::Gradient,
    iter_max: usize,
    mouse_pos: Complex32,
    zoom: f32,
}

impl MandelBrot {
    fn get(&self, x: f32, y: f32) -> Color {
        let ogy = y;
        let x = x / self.zoom;
        let y = y / self.zoom;
        let mouse_pos = self.mouse_pos / self.zoom;
        if (mouse_pos - Complex::new(x, y)).norm_sqr() < 0.01 {
            let s = ogy.abs() / 2.;
            return ctoc(self.colorgrad.at(s as f64));
        }

        let mindist = 0.5;

        let count = mandel_walk(Complex::new(x, y))
            .skip(5)
            .take(self.iter_max)
            .take_while(|c| (c - self.mouse_pos).norm_sqr() >= mindist)
            .count();

        let c = self.colorgrad.at(count as f64 / self.iter_max as f64);
        ctoc(c)
    }
}

fn mandel_walk(z: Complex<f32>) -> impl Iterator<Item = Complex<f32>> {
    let mut z1 = Complex::new(0.0, 0.0);
    repeat_with(move || {
        z1 = z1 * z1 + z;
        z1
    })
}

fn ctoc(c: colorgrad::Color) -> macroquad::color::Color {
    Color {
        r: c.r as f32,
        g: c.g as f32,
        b: c.b as f32,
        a: c.a as f32,
    }
}

const GRADIENTS: &[fn() -> colorgrad::Gradient] = &[
    colorgrad::br_bg,
    colorgrad::pr_gn,
    colorgrad::pi_yg,
    colorgrad::pu_or,
    colorgrad::rd_bu,
    colorgrad::rd_gy,
    colorgrad::rd_yl_bu,
    colorgrad::rd_yl_gn,
    colorgrad::spectral,
    colorgrad::blues,
    colorgrad::greens,
    colorgrad::greys,
    colorgrad::oranges,
    colorgrad::purples,
    colorgrad::reds,
    colorgrad::turbo,
    colorgrad::viridis,
    colorgrad::inferno,
    colorgrad::magma,
    colorgrad::plasma,
    colorgrad::cividis,
    colorgrad::warm,
    colorgrad::cool,
    colorgrad::cubehelix_default,
    colorgrad::bu_gn,
    colorgrad::bu_pu,
    colorgrad::gn_bu,
    colorgrad::or_rd,
    colorgrad::pu_bu_gn,
    colorgrad::pu_bu,
    colorgrad::pu_rd,
    colorgrad::rd_pu,
    colorgrad::yl_gn_bu,
    colorgrad::yl_gn,
    colorgrad::yl_or_br,
    colorgrad::yl_or_rd,
    colorgrad::rainbow,
    colorgrad::sinebow,
];

fn canvas_of_apropriate_size() -> Image {
    let mut w = screen_width();
    let mut h = screen_height();
    let max_pix = 660680.0;

    // desired property: w * h <= max_pix;
    // lazy math:
    while w * h > max_pix {
        w *= 0.9;
        h *= 0.9;
    }
    Image::gen_image_color(w as u16, h as u16, BLACK)
}

const SKY_SHADER_VERTEX: &str =
    include_str!(concat!(env!("OUT_DIR"), "/shaders/sky-shader-main_vs.glsl"));
const SKY_SHADER_FRAGMENT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/shaders/sky-shader-main_fs.glsl"));
