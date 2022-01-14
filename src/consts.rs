pub const DEFAULT_SCREEN_WIDTH: f32 = 800.;
pub const DEFAULT_SCREEN_HEIGHT: f32 = 600.;

pub const BUTTON_TEXT_FONT_SIZE: f32 = 0.1;

pub const HEALTH_BAR_SCALE: (f32, f32) = (0.15, 0.05);
pub const HEALTH_BAR_POS: (f32, f32) = (0.02, 0.05);

pub const MINIMAP_SCALE: f32 = 0.2;
pub const MINIMAP_POS: (f32, f32) = (1. - MINIMAP_SCALE, 0.);

pub const PLAYER_SCALE: f32 = 0.8;
pub const PLAYER_SHOOT_RATE: f32 = 2.5;
pub const PLAYER_SHOOT_RANGE: f32 = 400.;
pub const PLAYER_SHOOT_TIMEOUT: f32 = 0.;
pub const PLAYER_HEALTH: f32 = 5.;
pub const PLAYER_SPEED: f32 = 200.;
pub const PLAYER_DAMAGE: f32 = 1.;
pub const PLAYER_DAMAGED_COOLDOWN: f32 = 1.;

pub const ENEMY_SCALE: f32 = 0.8;
pub const ENEMY_SHOOT_RATE: f32 = 0.5;
pub const ENEMY_SHOOT_RANGE: f32 = 400.;
pub const ENEMY_SHOOT_TIMEOUT: f32 = 0.;
pub const ENEMY_HEALTH: f32 = 3.;
pub const ENEMY_SPEED: f32 = 100.;
pub const ENEMY_DAMAGE: f32 = 0.5;

pub const SHOT_SPEED: f32 = 300.;
pub const SHOT_SCALE: f32 = 0.4;

pub const ANIMATION_COOLDOWN: f32 = 0.5;

pub const DUNGEON_GRID_ROWS: usize = 8;
pub const DUNGEON_GRID_COLS: usize = 9;

pub const WALL_SCALE: f32 = 1.;

pub const ROOM_WIDTH: f32 = 15.;
pub const ROOM_HEIGHT: f32 = 9.;
pub const ROOM_LAYOUT_START: &str = "
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

pub const ROOM_LAYOUTS_EMPTY: &[&str] = &[
"
#######d#######
#             #
#   . . . .   #
#             #
d   . . . .   d
#             #
#   . . . .   #
#             #
#######d#######
",
"
#######d########
#              #
#     ....     #
#    .....     #
d              d
#     .....    #
#     ....     #
#              #
#######d########
",
"
#######d#######
#             #
#  ss sss ss  #
#             #
d  ss sss ss  d
#             #
#  ss sss ss  #
#             #
#######d#######
",
];

pub const ROOM_LAYOUTS_MOB: &[&str] = &[
"
#######d#######
#m           m#
#....     ....#
#             #
d             d
#             #
#....     ....#
#m           m#
#######d#######
",
"
#######d#######
#.           .#
#.. . ... . ..#
#.  . m.m .  .#
d   .  .  .   d
#.  . m.m .  .#
#.. . ... . ..#
#.           .#
#######d#######
",
];
