pub const DEFAULT_SCREEN_WIDTH: f32 = 800.;
pub const DEFAULT_SCREEN_HEIGHT: f32 = 600.;

pub const PLAYER_SCALE: f32 = 0.8;
pub const PLAYER_SHOOT_RATE: f32 = 2.5;
pub const PLAYER_SHOOT_RANGE: f32 = 400.;
pub const PLAYER_SHOOT_TIMEOUT: f32 = 0.;
pub const PLAYER_HEALTH: f32 = 5.;
pub const PLAYER_SPEED: f32 = 150.;
pub const PLAYER_DAMAGE: f32 = 1.;

pub const ENEMY_SCALE: f32 = 0.8;
pub const ENEMY_SHOOT_RATE: f32 = 0.5;
pub const ENEMY_SHOOT_RANGE: f32 = 400.;
pub const ENEMY_SHOOT_TIMEOUT: f32 = 0.;
pub const ENEMY_HEALTH: f32 = 3.;
pub const ENEMY_SPEED: f32 = 100.;
pub const ENEMY_DAMAGE: f32 = 0.5;

pub const SHOT_SPEED: f32 = 300.;
pub const SHOT_SCALE: f32 = 0.4;

pub const DUNGEON_GRID_ROWS: usize = 8;
pub const DUNGEON_GRID_COLS: usize = 9;

pub const WALL_SCALE: f32 = 1.;
pub const WALL_TEST: &str = "#";
pub const WALL_TEST_ROW: &str = "#######d#######";

pub const ROOM_WIDTH: f32 = 15.;
pub const ROOM_HEIGHT: f32 = 9.;
pub const ROOM_LAYOUT_EMPTY: &str = "
#######d#######
#             #
#             #
#             #
d             d
#             #
#             #
#             #
#######d#######
";

pub const ROOM_LAYOUTS_MOB: &[&str] = &[
"
#######d#######
#             #
#   e     e   #
#             #
d             d
#             #
#   e     e   #
#             #
#######d#######
",
"
#######d#######
#             #
#   #  #  #   #
#             #
d   #  #  #   d
#             #
#   #  #  #   #
#             #
#######d#######
"
];
