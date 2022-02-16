use puker::{
    dungeon::*,
    consts::*,
};
use glam::f32::Vec2;

const SCREEN: (f32, f32) = (DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT);

#[test]
fn test_dungeon_consistency_checker() {
    let grid_bad = [[0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [0, 0, 0, 0, 3, 1, 2, 0, 0],
                    [0, 0, 0, 0, 4, 0, 0, 0, 0],
                    [0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [0, 0, 0, 0, 0, 0, 0, 0, 0],];

    let grid_good = [[0, 0, 0, 0, 0, 0, 0, 0, 0],
                     [0, 0, 0, 0, 0, 0, 0, 0, 0],
                     [0, 0, 0, 0, 0, 5, 0, 0, 0],
                     [0, 0, 0, 0, 3, 1, 2, 0, 0],
                     [0, 0, 0, 0, 4, 0, 6, 7, 0],
                     [0, 0, 0, 0, 0, 0, 0, 0, 0],
                     [0, 0, 0, 0, 0, 0, 0, 0, 0],
                     [0, 0, 0, 0, 0, 0, 0, 0, 0],];

    assert!(!Dungeon::check_dungeon_consistency(&grid_bad, 7));
    assert!(Dungeon::check_dungeon_consistency(&grid_good, 7));
}

#[test]
fn test_dungeon_generation_test() {
    for i in 1..5 {
        let dungeon = Dungeon::generate_dungeon(SCREEN, i);
        let mut rooms_count: i32 = 0;
        let mut specials_count: i32 = 0;

        for r in dungeon.get_grid() {
            for c in r {
                match c {
                    Some(room) => {
                        rooms_count += 1;
                        match room.tag {
                            RoomTag::Boss | RoomTag::Item => specials_count += 1,
                            _ => (),
                        }
                    }
                    None => (),
                }
            }
        }

        assert!(rooms_count - 5 + i as i32 * 2 >= 0);
        assert_eq!(2, specials_count);
    }
}

#[test]
fn test_dungeon_room_retrieval() {
    let mut dungeon = Dungeon::generate_dungeon(SCREEN, 1);
    let start_room = Dungeon::get_start_room_coords();

    assert!(matches!(dungeon.get_room(start_room).unwrap(), Some(_)));
    assert!(matches!(dungeon.get_room_mut(start_room).unwrap(), Some(_)));
    assert!(matches!(dungeon.get_room((0, 0)).unwrap(), None));
}

#[test]
fn test_dungeon_state_update() {
    let mut dungeon = Dungeon::generate_dungeon(SCREEN, 1);
    let (i, j) = Dungeon::get_start_room_coords();

    dungeon.update_rooms_state((i, j)).unwrap();

    if let Some(r) = dungeon.get_room((i - 1, j)).unwrap() { assert_eq!(RoomState::Discovered, r.state); }
    if let Some(r) = dungeon.get_room((i + 1, j)).unwrap() { assert_eq!(RoomState::Discovered, r.state); }
    if let Some(r) = dungeon.get_room((i, j - 1)).unwrap() { assert_eq!(RoomState::Discovered, r.state); }
    if let Some(r) = dungeon.get_room((i, j + 1)).unwrap() { assert_eq!(RoomState::Discovered, r.state); }
}

#[test]
fn test_room_target_distance_grid() {
    let dungeon = Dungeon::generate_dungeon(SCREEN, 1);
    let grid = dungeon.get_room((3, 5)).unwrap().unwrap().get_target_distance_grid(Vec2::new(SCREEN.0 / 2., SCREEN.1 / 2.), SCREEN.0, SCREEN.1);

    let (mut i, mut j) = (1, 1);

    loop {
        if grid[i + 1][j] > grid[i][j] { i += 1; continue }
        if grid[i - 1][j] > grid[i][j] { i -= 1; continue }
        if grid[i][j + 1] > grid[i][j] { j += 1; continue }
        if grid[i][j - 1] > grid[i][j] { j -= 1; continue }
        break
    }

    assert_eq!((4, 7), (i, j));
}
