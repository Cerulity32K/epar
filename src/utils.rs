#![allow(dead_code)]
use std::{f32::consts::{TAU, PI}, ops::Add};

use macroquad::{prelude::{Vec2, vec2, Color}, shapes::draw_triangle, text::{draw_text, measure_text}, window::{screen_width, screen_height}, rand::gen_range};

use crate::game::GSEvent;

/// AABB:circle collision 9-patch checks
const CA_COLL: [fn(Vec2, Vec2, Vec2, f32) -> bool; 9] = [
    |posr: Vec2, _sze: Vec2, posc: Vec2, rad: f32| posr.distance_squared(posc) < rad * rad, // top left
    |posr: Vec2, _sze: Vec2, posc: Vec2, rad: f32| posr.y < posc.y + rad, // top center
    |posr: Vec2, size: Vec2, posc: Vec2, rad: f32| (posr + vec2(size.x, 0.0)).distance_squared(posc) < rad * rad, // top right
    |posr: Vec2, _sze: Vec2, posc: Vec2, rad: f32| posr.x < posc.x + rad, // center left
    |_psr: Vec2, _sze: Vec2, _psc: Vec2, _rd: f32| true, // center center
    |posr: Vec2, size: Vec2, posc: Vec2, rad: f32| posr.x + size.x > posc.x - rad, // center right
    |posr: Vec2, size: Vec2, posc: Vec2, rad: f32| (posr + vec2(0.0, size.y)).distance_squared(posc) < rad * rad, // bottom left
    |posr: Vec2, size: Vec2, posc: Vec2, rad: f32| posr.y + size.y > posc.y - rad, // bottom center
    |posr: Vec2, size: Vec2, posc: Vec2, rad: f32| (posr + size).distance_squared(posc) < rad * rad, // bottom right
];

/// Rotates a vector around (0, 0) using radians.
pub fn rotate(vec: Vec2, rot: f32) -> Vec2 {
    // literally matrix multiplication
    Vec2 {
        x: rot.cos() * vec.x + rot.sin() * vec.y,
        y: -rot.sin() * vec.x + rot.cos() * vec.y
    }
}

pub fn rotate_around(vec: Vec2, around: Vec2, rot: f32) -> Vec2 {
    rotate(vec - around, rot) + around
}

pub fn tuple(vec: Vec2) -> (f32, f32) {
    (vec.x, vec.y)
}

pub fn iter<T>(obj: T) -> impl Iterator<Item = T> {
    [obj].into_iter()
}

/// Squares a number.
#[inline]
pub fn sq(num: f32) -> f32 { num * num }
pub fn cmul(clr: Color, val: f32) -> Color { Color { r: clr.r * val, g: clr.g * val, b: clr.b * val, a: clr.a } }
pub fn acmul(clr: Color, val: f32) -> Color { Color { a: clr.a * val, ..clr } }

/// Tests if two circles collide.\
/// Fast; no division or square roots
pub fn collide_cc(pos1: Vec2, rad1: f32, pos2: Vec2, rad2: f32) -> bool {
    (pos1 - pos2).length_squared() <= sq(rad1 + rad2)
}

/// Tests if a hollow circle and a filled circle collide.\
/// Fast; no division or square roots
pub fn collide_chc(cpos: Vec2, crad: f32, hpos: Vec2, hrad: f32, hradin: f32) -> bool {
    collide_cc(cpos, crad, hpos, hrad) && !collide_cc(cpos, -crad, hpos, hradin)
}


pub fn mix(color1: Color, color2: Color, by: f32) -> Color {
    Color {
        r: lerp(color1.r, color2.r, by),
        g: lerp(color1.g, color2.g, by),
        b: lerp(color1.b, color2.b, by),
        a: lerp(color1.a, color2.a, by),
    }
}

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }

// Tests if two rotatable rectangles collide.\
// TODO: implement
// pub fn collide_rr(center1: Vec2, size1: Vec2, rot1: f32, center2: Vec2, size2: Vec2, rot2: f32) -> bool { false }

/// Tests if a circle is colliding with a rotatable rectangle.
pub fn collide_cr(rcenter: Vec2, rsize: Vec2, rot: f32, cpos: Vec2, rad: f32) -> bool {
    let around = rotate(rsize * -0.5, -rot) + rcenter;
    let new_cpos = rotate_around(cpos, around, rot);
    collide_ca(around, rsize, new_cpos, rad)
}

/// Tests if a circle is colliding with an axis-aligned rectangle.
pub fn collide_ca(rpos: Vec2, rsize: Vec2, cpos: Vec2, rad: f32) -> bool {
    let mut col_idx = 0;
    if cpos.x > rpos.x {
        col_idx += 1;
        if cpos.x >= rpos.x + rsize.x {
            col_idx += 1;
        }
    }
    if cpos.y > rpos.y {
        col_idx += 3;
        if cpos.y >= rpos.y + rsize.y {
            col_idx += 3;
        }
    }
    CA_COLL[col_idx](rpos, rsize, cpos, rad)
}

/// Tests if a circle is colliding with a capsule (point->point).\
/// This just collides a circle with a rotated rectangle of 0 width.
pub fn collide_capsule(p1: Vec2, p2: Vec2, rad: f32, other: Vec2, other_rad: f32) -> bool {
    let (c, s, r) = rectify_line(p1, p2, 0.0);
    // sneaky sneaky
    collide_cr(c, s, r, other, rad + other_rad)
}

pub fn rectify_line(start: Vec2, end: Vec2, thickness: f32) -> (Vec2, Vec2, f32) {
    let delta = end - start;
    (
        start.lerp(end, 0.5), // Position (center)
        vec2(start.distance(end), thickness), // Size
        delta.y.atan2(delta.x) // Rotation
    )
}

/// Draws a rotated rectangle.
pub fn draw_rrect(center: Vec2, size: Vec2, rot: f32, color: impl Into<Color>) {
    let clr = color.into();
    let tl = rotate(size * -0.5, rot) + center;
    let tr = rotate(size * vec2(0.5, -0.5), rot) + center;
    let bl = rotate(size * vec2(-0.5, 0.5), rot) + center;
    let br = rotate(size * 0.5, rot) + center;
    draw_triangle(tl, tr, bl, clr);
    draw_triangle(br, tr, bl, clr);
}

pub fn centered_text_draw(string: &str, pos: Vec2, font_size: f32, color: Color) {
    let text_dims = measure_text(string, None, font_size as u16, font_size / font_size.floor());
    let text_center = vec2(text_dims.width, text_dims.height) / 2.0;
    draw_text(string, pos.x + text_center.x, pos.y + text_dims.offset_y / 2.0, font_size, color);
}

pub fn gay(phase: f32) -> Color {
    Color {
        r: (phase + TAU / 3.0 * 3.0).sin() / 2.0 + 0.5,
        g: (phase + TAU / 3.0 * 2.0).sin() / 2.0 + 0.5,
        b: (phase + TAU / 3.0 * 1.0).sin() / 2.0 + 0.5,
        a: 1.0,
    }
}

pub fn screen_center() -> Vec2 {
    screen_size() / 2.0
}

pub fn screen_size() -> Vec2 {
    vec2(screen_width(), screen_height())
}

pub fn rand_vec(from: Vec2, to: Vec2) -> Vec2 {
    vec2(gen_range(from.x, to.x), gen_range(from.y, to.y))
}

pub fn floor_vec(vec: Vec2, to: Vec2) -> Vec2 {
    vec2(
        (vec.x / to.x).floor() * to.x,
        (vec.y / to.y).floor() * to.y
    )
}

pub fn screen(xfac: f32, yfac: f32) -> Vec2 {
    screen_size() * vec2(xfac, yfac)
}

/// Anonymous event repeater.\
/// This is meant to work with numeric types (like `f32`) and events that implement `Clone`.\
/// However, anything that implements `Clone + Copy + Add<O, Output = T>` can be used as `T`,\
/// and anything that implements `Copy` can be used as `O`.\
/// Usually, `T = f32` and `O = f32`.
pub fn tev_rep<T: Clone + Copy + Add<O, Output = T>, E: Clone, O: Copy>(mut ev: Vec<(T, E)>, times: usize, spacing: O) -> Vec<(T, E)> {
    let mut to_add = ev.clone();
    for _ in 0..(times - 1) {
        for (t, _) in &mut to_add {
            *t = *t + spacing;
        }
        ev.append(&mut to_add.clone());
    }
    ev
}

pub fn repeat_events(events: impl IntoIterator<Item = GSEvent>, amount: usize, offset: f32) -> Vec<GSEvent> {
    let collected = events.into_iter().collect::<Vec<GSEvent>>();
    let mut out = vec![];
    let mut total_offset = 0.0;
    for i in 0..amount {
        for i in &collected {
            out.push(i.clone().time(i.0 + total_offset));
        }
        total_offset += offset;
    }
    out
}

// Repeated sine-out easing (0-1 period)
pub fn ease_sineout_rep(x: f32) -> f32 {
    let pimodx1 = PI * (x % 1.0);
    (pimodx1.sin() + pimodx1) / PI
}
// Created for 4otf beat positional editing. Uses a circular ease to climb up y=x.
pub fn circ_climb(x: f32) -> f32 {
    ease_sineout_rep(x).sqrt() + x.floor()
}

/// Event zeroer
pub fn ez<E>(mut ev: Vec<(f32, E)>) -> Vec<(f32, E)> {
    for (f, _) in &mut ev {
        *f = 0.0;
    }
    ev
}

pub fn adjust<T: Clone>(v: &mut Vec<T>, length: usize, extension: T) {
    if v.len() > length {
        while v.len() > length {
            v.pop();
        }
    } else if v.len() < length {
        while v.len() < length {
            v.push(extension.clone());
        }
    }
}

pub fn rep_off<T: Copy + Add<Output = T>>(iter: impl IntoIterator<Item = T>, count: usize, offset: T) -> Vec<T> {
    let mut src = iter.into_iter().collect::<Vec<T>>();
    let mut dst = vec![];
    for _ in 0..count {
        for v in &mut src {
            dst.push(*v);
            *v = *v + offset;
        }
    }
    dst
}

/// Approaches 1 as x -> inf.\
/// 1-1/(x+1)
pub fn recip_ease(t: f32) -> f32 {
    1.0 - 1.0 / (t + 1.0)
}

pub fn collide_circ_arc(cpos: Vec2, crad: f32, apos: Vec2, aradout: f32, aradin: f32, ang1: f32, ang2: f32) -> bool {
    collide_chc(cpos, crad, apos, aradout, aradin) &&
    rotate_around(cpos, apos, ang1).y >= apos.y - crad &&
    rotate_around(cpos, apos, ang2).y <= apos.y - crad
}

pub fn draw_arc(center: Vec2, inner_rad: f32, outer_rad: f32, ang1: f32, ang2: f32, segments: usize, color: impl Into<Color>) {
    let color = color.into();
    let angdiff = ang2 - ang1;
    let seglen = angdiff / segments as f32;

    for i in 0..segments {
        let p1 = i as f32 * seglen + ang1;
        let p2 = (i + 1) as f32 * seglen + ang1;
        
        let tl = vec2(p1.sin(), p1.cos()) * outer_rad + center;
        let tr = vec2(p2.sin(), p2.cos()) * outer_rad + center;
        let bl = vec2(p1.sin(), p1.cos()) * inner_rad + center;
        let br = vec2(p2.sin(), p2.cos()) * inner_rad + center;

        draw_triangle(tl, tr, br, color);
        draw_triangle(tl, bl, br, color);
    }
}

pub fn gen_sign() -> f32 {
    gen_range(0, 2) as f32 * 2.0 - 1.0
}
