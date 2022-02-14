use puker::utils::*;
use glam::f32::Vec2;
use ggez::{
    graphics::{Rect},
};

const DELTA_TIME: f32 = 1. / 60.;

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
    let d_vel = Vec2::X;
    let s_rect = Rect::new(1., 1., 2., 2.);
    let (mut cp, mut cn, mut t) = (Vec2::ZERO, Vec2::ZERO, 0.);

    assert!(dynamic_circle_vs_rect(&d_c, &d_vel, &s_rect, &mut cp, &mut cn, &mut t, DELTA_TIME));

    d_c = ((d_c.0.0 - cn * t).into(), d_c.1);
    assert!(!dynamic_circle_vs_rect(&d_c, &d_vel, &s_rect, &mut cp, &mut cn, &mut t, DELTA_TIME));
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
