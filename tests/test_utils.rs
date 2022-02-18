use puker::{
    utils::*,
    consts::*,
};
use glam::f32::Vec2;
use ggez::{
    graphics::Rect,
};

const DELTA_TIME: f32 = 1. / DESIRED_FPS as f32; 

#[test]
fn test_circle_overlap() {
    let c1 = (Vec2::ZERO.into(), 5.);
    let c2 = (Vec2::ZERO.into(), 1.);
    let c3 = (Vec2::new(0., 2.).into(), 1.);

    assert!(circle_vs_circle(&c1, &c2));
    assert!(circle_vs_circle(&c1, &c2));
    assert!(!circle_vs_circle(&c2, &c3));
}

#[test]
fn test_ray_vs_rect() {
    let ray1 = (Vec2::ZERO, Vec2::ONE);
    let ray2 = (Vec2::ZERO, -Vec2::ONE);

    let rect1 = Rect::new(10., 10., 3., 2.);

    let (mut cp, mut cn, mut t) = (Vec2::ZERO, Vec2::ZERO, 0.);

    assert!(ray_vs_rect(&ray1.0, &ray1.1, &rect1, &mut cp, &mut cn, &mut t));
    assert!(!ray_vs_rect(&ray2.0, &ray2.1, &rect1, &mut cp, &mut cn, &mut t));
}

#[test]
fn test_dynamic_rect_vs_rect() {
    let d_rect = Rect::new(0., 1., 3., 2.);
    let d_vel = Vec2::X * 2.;
    let s_rect = Rect::new(2., 0.5, 2., 2.);
    let (mut cp, mut cn, mut t) = (Vec2::ZERO, Vec2::ZERO, 0.);

    assert!(dynamic_rect_vs_rect(&d_rect, &d_vel, &s_rect, &mut cp, &mut cn, &mut t, DELTA_TIME));

    let new_pos = Vec2::new(d_rect.x, d_rect.y) + cn * d_vel.abs() * (1. - t);
    assert!(!s_rect.overlaps(&Rect::new(new_pos.x, new_pos.y, d_rect.w, d_rect.h)));
}

#[test]
fn test_dynamic_circle_vs_rect() {
    let mut d_c = (Vec2::ZERO.into(), 2.);
    let s_rect = Rect::new(1., 1., 2., 2.);
    let (mut cp, mut cn, mut t) = (Vec2::ZERO, Vec2::ZERO, 0.);

    assert!(dynamic_circle_vs_rect(&d_c, &s_rect, &mut cp, &mut cn, &mut t, DELTA_TIME));

    d_c = ((d_c.0.0 - cn * t).into(), d_c.1);
    assert!(!dynamic_circle_vs_rect(&d_c, &s_rect, &mut cp, &mut cn, &mut t, DELTA_TIME));
}

#[test]
fn test_dynamic_circle_vs_circle() {
    let c1 = (Vec2::ZERO.into(), 2.);
    let c2 = (Vec2::X.into(), 2.);
    let mut vel1 = Vec2::X;
    let mut vel2 = -Vec2::X;
    let (mut v1, mut v2) = (Vec2::ZERO, Vec2::ZERO);

    assert!(dynamic_circle_vs_circle(&c1, &vel1, &c2, &vel2, &mut v1, &mut v2, DELTA_TIME));

    vel1 += v1;
    vel2 += v2;

    assert_eq!(vel1, Vec2::ZERO);
    assert_eq!(vel2, Vec2::ZERO);
}

#[test]
fn test_mouse_to_screen_coords() {
    let (mx1, my1) = (0., 0.);
    let (mx2, my2) = (100., 100.);
    let (sw, sh) = (DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT);
    let (ww, wh) = (DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT);

    assert_eq!(Vec2::ZERO, get_mouse_screen_coords(mx1, my1, sw, sh, ww ,wh));
    assert_eq!(Vec2::new(100., 100.), get_mouse_screen_coords(mx2, my2, sw, sh, ww ,wh));
}

#[test]
fn test_pos_to_room_coords() {
    let (sw, sh) = (DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT);
    let pos1 = Vec2::ZERO;
    let pos2 = Vec2::new(sw / 2., sh / 2.);

    assert_eq!((0, 0), pos_to_room_coords(pos1, sw, sh));
    assert_eq!((ROOM_HEIGHT / 2, ROOM_WIDTH / 2), pos_to_room_coords(pos2, sw, sh));
}

#[test]
fn test_room_coords_to_pos() {
    let (sw, sh) = (DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT);
    let (rcw, rch) = (sw / ROOM_WIDTH as f32, sh / ROOM_HEIGHT as f32);
    let coords1 = (0, 0);
    let coords2 = (4, 7);

    assert_eq!(Vec2::new(rcw / 2., rch / 2.), room_coords_to_pos(coords1.0, coords1.1, sw, sh));
    assert_eq!(Vec2::new(coords2.1 as f32 * rcw + rcw / 2., coords2.0 as f32 * rch + rch / 2.).round(), room_coords_to_pos(coords2.0, coords2.1, sw, sh));
}
