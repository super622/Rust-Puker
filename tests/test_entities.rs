use puker::{
    player::*,
    enemies::*,
    consts::*,
    traits::*,
    utils::*,
    shots::*,
    dungeon::*,
};
use glam::f32::Vec2;

const SCREEN: (f32, f32) = (DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT);
const DELTA_TIME: f32 = 1. / 60.;

#[test]
fn test_player_shoot() {
    let mut player = Player::default();
    let mut shots = Vec::new();

    player.shoot(&mut shots);

    assert_eq!(player.shoot_timeout, 1. / player.shoot_rate);
    assert_eq!(player.animation_cooldown, ANIMATION_COOLDOWN / player.shoot_rate);
    assert!(!shots.is_empty());
    assert_eq!(shots[0].tag, ShotTag::Player);

    player.shoot(&mut shots);

    assert_eq!(shots.len(), 1);
}

#[test]
fn test_enemy_shooter() {
    let player = Player::default();
    let mut enemy = EnemyMask {
        props: ActorProps {
            pos: Vec2::new(player.props.pos.0.x - ENEMY_SHOOT_RANGE / 2., player.props.pos.0.y).into(),
            ..Default::default()
        },
        afterlock_cooldown: 0.,
        ..Default::default()
    };
    let mut shots = Vec::new();

    enemy.shoot(SCREEN.0, SCREEN.1, &Vec::new(), &mut shots, &player);

    assert_eq!(enemy.shoot_timeout, 1. / enemy.shoot_rate);
    assert_eq!(enemy.animation_cooldown, ANIMATION_COOLDOWN / enemy.shoot_rate);
    assert!(!shots.is_empty());
    assert_eq!(shots[0].tag, ShotTag::Enemy);

    enemy.shoot(SCREEN.0, SCREEN.1, &Vec::new(), &mut shots, &player);

    assert_eq!(shots.len(), 1);
    
    enemy.shoot_timeout = 0.;
    let obst: Vec<Box<dyn Stationary>> = vec![
        Box::new(Block {
            pos: (enemy.props.pos.0 + Vec2::X * 100.).into(),
            scale: Vec2::splat(WALL_SCALE),
            tag: BlockTag::Wall,
        }),
    ];

    enemy.shoot(SCREEN.0, SCREEN.1, &obst, &mut shots, &player);

    assert_eq!(shots.len(), 1);
}

#[test]
fn test_velocity_lerp() {
    let mut player = Player::default();
    player.set_translation(Vec2::X);

    while player.get_velocity().length() < player.speed {
        player.velocity_lerp(DELTA_TIME, player.speed, DELTA_TIME * 2., 400.);
    }

    assert_eq!(player.get_velocity().length(), player.speed);

    player.set_translation(Vec2::ZERO);

    while player.get_velocity().length() > 0. {
        player.velocity_lerp(DELTA_TIME, player.speed, DELTA_TIME * 2., 400.);
    }

    assert_eq!(player.get_velocity().length(), 0.);
}

// #[test]
// fn test_player_pick_up_passive() {
//     let player = Player::default();
//     let passive = Item {
//     }
// }
