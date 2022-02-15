use puker::dungeon::*;

#[test]
fn test_room_layout_generation() {
    let screen = (DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT);
    let room = Room::generate_room(screen, (1, 1), [None; 4], RoomTag::Start, )
}
