use crate::items::{ItemPassive, ItemActive, ItemTag};

pub const DEFAULT_SCREEN_WIDTH: f32 = 1280.;
pub const DEFAULT_SCREEN_HEIGHT: f32 = 720.;
pub const DESIRED_FPS: u32 = 60;

pub const TRANSITION_SCENE_COOLDOWN: f32 = 3.;

pub const BUTTON_TEXT_FONT_SIZE: f32 = 0.1;

pub const HEALTH_BAR_SCALE: (f32, f32) = (0.15, 0.05);
pub const HEALTH_BAR_POS: (f32, f32) = (0.1, 0.05);

pub const ITEM_HOLDER_SCALE: f32 = 0.1;
pub const ITEM_HOLDER_POS: (f32, f32) = (0.05, 0.05);

pub const MINIMAP_SCALE: f32 = 0.2;
pub const MINIMAP_POS: (f32, f32) = (1. - MINIMAP_SCALE, 0.);

pub const PLAYER_SCALE: f32 = 0.6;
pub const PLAYER_SHOOT_RATE: f32 = 2.5;
pub const PLAYER_MAX_SHOOT_RATE: f32 = 10.;
pub const PLAYER_SHOOT_RANGE: f32 = 400.;
pub const PLAYER_HEALTH: f32 = 3.;
pub const PLAYER_SPEED: f32 = 5.;
pub const PLAYER_MAX_SPEED: f32 = 10.;
pub const PLAYER_DAMAGE: f32 = 1.;
pub const PLAYER_MAX_DAMAGE: f32 = 3.;
pub const PLAYER_DAMAGED_COOLDOWN: f32 = 1.;
pub const PLAYER_AFTERLOCK_COOLDOWN: f32 = 0.5;

pub const ENEMY_SCALE: f32 = 0.8;
pub const ENEMY_SHOOT_RATE: f32 = 0.5;
pub const ENEMY_SHOOT_RANGE: f32 = 500.;
pub const ENEMY_HEALTH: f32 = 3.;
pub const ENEMY_SPEED: f32 = 4.;
pub const ENEMY_DAMAGE: f32 = 0.5;
pub const ENEMY_AFTERLOCK_COOLDOWN: f32 = 2.;
pub const ENEMY_WANDERER_CHANGE_DIRECTION_COOLDOWN: f32 = 2.;

pub const BOSS_HEALTH: f32 = 50.;

pub const SHOT_SPEED: f32 = 6.;
pub const SHOT_SCALE: f32 = 0.3;

pub const COLLECTABLE_SCALE: f32 = 0.4;

pub const ITEM_COOLDOWN: f32 = 3.;
pub const ITEM_SCALE: f32 = 0.6;
pub const ITEM_POOL_PASSIVE: &[ItemTag] = &[
    ItemTag::Passive(ItemPassive::IncreaseMaxHealth(1.)),
];
pub const ITEM_POOL_ACTIVE: &[ItemTag] = &[
    ItemTag::Active(ItemActive::Heal(1.)),
];

pub const ANIMATION_COOLDOWN: f32 = 0.5;

pub const DUNGEON_GRID_ROWS: usize = 8;
pub const DUNGEON_GRID_COLS: usize = 9;

pub const WALL_SCALE: f32 = 1.;

pub const ROOM_WIDTH: usize = 15;
pub const ROOM_HEIGHT: usize = 9;

pub const ROOM_LAYOUT_START: &str = 
"
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

pub const ROOM_LAYOUTS_ITEM: &[&str] = &[
"
#######d#######
#v           v#
#      v      #
#             #
d    v p v    d
#             #
#      v      #
#v           v#
#######d#######
",
];

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
#######d#######
#             #
#   v v v v   #
#             #
d   v v v v   d
#             #
#   v v v v   #
#             #
#######d#######
",
"
#######d#######
#             #
#    ....     #
#   .....     #
d             d
#     .....   #
#     ....    #
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
"
#######d#######
#..         ..#
#.           .#
#      b      #
d     bbb     d
#      b      #
#.           .#
#..         ..#
#######d#######
",
"
#######d#######
#             #
#      .      #
#  .     .    #
d   .b m b.   d
#             #
#      .      #
#             #
#######d#######
",
"
#######d#######
#             #
#  s       s  #
#      s      #
d    s   s    d
#      s      #
#  s       s  #
#             #
#######d#######
",
"
#######d#######
#          ...#
#           m.#
#          . .#
d             d
#. .          #
#.m           #
#...          #
#######d#######
",
"
#######d#######
#.m.          #
#. .          #
#. .          #
d             d
#          . .#
#          . .#
#          .m.#
#######d#######
",
"
#######d#######
#             #
#             #
#             #
d      b      d
#             #
#             #
#             #
#######d#######
",
];

pub const ROOM_LAYOUTS_BOSS: &[&str] = &[
"
#######d#######
#             #
#      h      #
#             #
d      B      d
#             #
#             #
#             #
#######d#######
",
];
