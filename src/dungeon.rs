use ggez::{
    graphics::{self, DrawParam, Rect},
    GameResult,
    Context,
    audio::SoundSource,
};
use crate::{
    enemies::*,
    utils::*,
    consts::*,
    traits::*,
    items::*,
    shots::*,
    player::*,
};
use std::{
    any::Any,
    collections::VecDeque,
    f32::consts::PI,
};
use rand::{thread_rng, Rng};
use glam::f32::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoomTag {
    Start,
    Empty,
    Mob,
    Boss,
    Item,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoomState {
    Undiscovered,
    Discovered,
    Cleared,
}

#[derive(Debug)]
pub struct Room {
    pub tag: RoomTag,
    pub state: RoomState,
    pub width: f32,
    pub height: f32,
    pub grid: [[i32; ROOM_WIDTH]; ROOM_HEIGHT],
    pub dungeon_coords: (usize, usize),
    pub doors: Vec<usize>,
    pub obstacles: Vec<Box<dyn Stationary>>,
    pub enemies: Vec<Box<dyn Actor>>,
    pub shots: Vec<Shot>,
    pub drops: Vec<Collectable>,
}

impl Room {
    pub fn update(&mut self, ctx: &mut Context, conf: &mut Config, _player: &mut Player, _delta_time: f32) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let target_grid = &self.get_target_distance_grid(_player.get_pos(), sw, sh);

        for shot in self.shots.iter_mut() {
            shot.update(ctx, conf, _delta_time)?;
        }

        for enemy in self.enemies.iter_mut() {
            enemy.act(sw, sh, target_grid, &self.obstacles, &mut self.shots, _player)?;
            enemy.update(ctx, conf, _delta_time)?;
        }

        for drop in self.drops.iter_mut() {
            drop.update(ctx, conf, _delta_time)?;
        }

        let dead_enemies = self.enemies.iter()
            .enumerate()
            .filter(|e| e.1.get_state() == ActorState::Dead)
            .map(|e| e.0).collect::<Vec<_>>();
        for (i,d) in dead_enemies.iter().enumerate() { self.enemies.remove(d - i); }

        let dead_shots = self.shots.iter()
            .enumerate()
            .filter(|s| s.1.get_pos().distance(s.1.spawn_pos.0) >= s.1.range)
            .map(|s| s.0).collect::<Vec<_>>();
        for (i,d) in dead_shots.iter().enumerate() { 
            self.shots.remove(d - i); 
            let _ = conf.assets.audio.get_mut("bubble_pop_sound").unwrap().play(ctx);
        }

        let dead_drops = self.drops.iter()
            .enumerate()
            .filter(|d| d.1.state == CollectableState::Consumed)
            .map(|d| d.0).collect::<Vec<_>>();
        for (i,d) in dead_drops.iter().enumerate() { self.drops.remove(d - i); }
        
        if self.enemies.is_empty() {
            self.state = RoomState::Cleared;

            match self.tag {
                RoomTag::Mob | RoomTag::Boss => {
                    self.generate_collectable(sw, sh);
                    self.tag = RoomTag::Empty;
                    let _ = conf.assets.audio.get_mut("door_open_sound").unwrap().play(ctx);
                    for door in self.doors.iter() {
                        let block = self.obstacles[*door].as_any_mut().downcast_mut::<Block>().unwrap();
                        block.tag = match block.tag {
                            BlockTag::Door { dir, connects_to, .. } => BlockTag::Door { dir, connects_to, is_open: true },
                            BlockTag::Hatch(_) => BlockTag::Hatch(true),
                            _ => unreachable!(),
                        }
                    }
                    if let Some(i) = &mut _player.item {
                        i.cooldown = f32::max(i.cooldown - 1., 0.);
                    }
                },
                _ => (),
            }
        }
        else {
            for door in self.doors.iter() {
                let block = self.obstacles[*door].as_any_mut().downcast_mut::<Block>().unwrap();
                block.tag = match block.tag {
                    BlockTag::Door { dir, connects_to, is_open } => {
                        if is_open {
                            let _ = conf.assets.audio.get_mut("door_close_sound").unwrap().play(ctx);
                            BlockTag::Door { dir, connects_to, is_open: !is_open }
                        }
                        else { BlockTag::Door { dir, connects_to, is_open } }
                    },
                    BlockTag::Hatch(_) => BlockTag::Hatch(false),
                    _ => unreachable!(),
                }
            }
        }

        Ok(())
    }
    
    pub fn draw(&self, ctx: &mut Context, conf: &mut Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let scale = Vec2::from(Room::get_room_scale(sw, sh, conf.assets.sprites.get("floor").unwrap().dimensions()));
        let draw_params = DrawParam::default()
            .scale(scale);
        
        graphics::draw(ctx, conf.assets.sprites.get("floor").unwrap(), draw_params)?;

        if self.tag == RoomTag::Start {
            graphics::draw(ctx, conf.assets.sprites.get("instructions").unwrap(), draw_params.scale(scale * 0.7).offset([-0.2, -0.2]))?;
        }

        for obst in self.obstacles.iter() { obst.draw(ctx, conf)?; }

        for drop in self.drops.iter() { drop.draw(ctx, conf)?; }

        for shot in self.shots.iter() { shot.draw(ctx, conf)?; }

        for enemy in self.enemies.iter() { enemy.draw(ctx, conf)?; }

        Ok(())
    }

    fn get_room_scale(sw: f32, sh: f32, image: Rect) -> [f32; 2] {
        [sw / image.w, sh / image.h]
    }

    /// Helper function for determining the models/stationaries positions.
    ///
    fn get_entity_pos(sw: f32, sh: f32, rw: f32, rh: f32, index: usize) -> Vec2 {
        let dims = Vec2::new(sw / rw, sh / rh);
        let coords = Vec2::new((index % (rw as usize)) as f32, (index / (rw as usize)) as f32) * dims;
        coords + dims / 2.
    }

    fn parse_layout(sw: f32, sh: f32, rw: f32, rh: f32, layout: &str, door_connects: &[Option<((usize, usize), Direction)>; 4], level: u32) -> (Vec<Box<dyn Stationary>>, Vec<Box<dyn Actor>>, Vec<usize>, [[i32; ROOM_WIDTH]; ROOM_HEIGHT]) {
        let mut doors: Vec<usize> = Vec::new();
        let mut obstacles: Vec<Box<dyn Stationary>> = Vec::new(); 
        let mut enemies: Vec<Box<dyn Actor>> = Vec::new();
        let mut grid = [[0; ROOM_WIDTH]; ROOM_HEIGHT];

        let mut door_index = 0_usize;

        let enemy_damage_amplifier = (level as f32 / ENEMY_DAMAGE).clamp(0.5, 3.);

        for (i, c) in layout.chars().enumerate() {
            match c {
                '#'|'.'|'v'|'d'|'h'|'p' => {
                    if c != 'v' { grid[i / ROOM_WIDTH as usize][i % ROOM_WIDTH as usize] = i32::MIN; }

                    obstacles.push(Box::new(Block {
                        pos: Room::get_entity_pos(sw, sh, rw, rh, i).into(),
                        scale: Vec2::splat(WALL_SCALE),
                        tag: match c {
                            'd' => {
                                door_index += 1;
                                if let Some((connects_to, dir)) = door_connects[door_index - 1] {
                                    doors.push(obstacles.len());
                                    BlockTag::Door {
                                        dir,
                                        is_open: true,
                                        connects_to,
                                    }
                                }
                                else { BlockTag::Wall }
                            }
                            '#' => BlockTag::Wall,
                            '.' => BlockTag::Stone,
                            'v' => BlockTag::Spikes,
                            'h' => {
                                doors.push(obstacles.len());
                                BlockTag::Hatch(false)
                            },
                            'p' => BlockTag::Pedestal(Some(Room::generate_item())),
                            _ => unreachable!(),
                        },
                    }));
                },
                'm' => {
                    enemies.push(Box::new(EnemyMask {
                        props: ActorProps {
                            pos: Room::get_entity_pos(sw, sh, rw, rh, i).into(),
                            scale: Vec2::splat(ENEMY_SCALE),
                            ..Default::default()
                        },
                        damage: ENEMY_DAMAGE * enemy_damage_amplifier,
                        ..Default::default()
                    }));
                },
                'b' => {
                    enemies.push(Box::new(EnemyBlueGuy {
                        props: ActorProps {
                            pos: Room::get_entity_pos(sw, sh, rw, rh, i).into(),
                            scale: Vec2::splat(ENEMY_SCALE),
                            ..Default::default()
                        },
                        damage: ENEMY_DAMAGE * enemy_damage_amplifier,
                        ..Default::default()
                    }));
                },
                's' => {
                    enemies.push(Box::new(EnemySlime {
                        props: ActorProps {
                            pos: Room::get_entity_pos(sw, sh, rw, rh, i).into(),
                            scale: Vec2::new(ENEMY_SCALE, ENEMY_SCALE * 0.5),
                            ..Default::default()
                        },
                        damage: ENEMY_DAMAGE * enemy_damage_amplifier,
                        ..Default::default()
                    }));
                },
                'B' => {
                    enemies.push(Box::new(BossWeirdBall {
                        props: ActorProps {
                            pos: Room::get_entity_pos(sw, sh, rw, rh, i).into(),
                            scale: Vec2::splat(ENEMY_SCALE * 2.),
                            ..Default::default()
                        },
                        damage: ENEMY_DAMAGE * 2. * enemy_damage_amplifier,
                        ..Default::default()
                    }));
                },
                _ => (),
            }
        }

        (obstacles, enemies, doors, grid)
    }

    fn generate_room(screen: (f32, f32), dungeon_coords: (usize, usize), door_connects: [Option<((usize, usize), Direction)>; 4], tag: RoomTag, level: u32) -> Room {
        let (sw, sh) = screen;
        
        let state = RoomState::Undiscovered;
        let width = ROOM_WIDTH as f32;
        let height = ROOM_HEIGHT as f32;
        let shots = Vec::<Shot>::new();
        let drops = Vec::<Collectable>::new();

        let mut layout = String::from(match tag {
            RoomTag::Start => ROOM_LAYOUT_START,
            RoomTag::Mob => {
                let layout_index = thread_rng().gen_range(0..ROOM_LAYOUTS_MOB.len()) as usize;
                ROOM_LAYOUTS_MOB[layout_index]
            },
            RoomTag::Empty => {
                let layout_index = thread_rng().gen_range(0..ROOM_LAYOUTS_EMPTY.len()) as usize;
                ROOM_LAYOUTS_EMPTY[layout_index]
            },
            RoomTag::Item => {
                let layout_index = thread_rng().gen_range(0..ROOM_LAYOUTS_ITEM.len()) as usize;
                ROOM_LAYOUTS_ITEM[layout_index]
            }
            RoomTag::Boss => {
                let layout_index = thread_rng().gen_range(0..ROOM_LAYOUTS_BOSS.len()) as usize;
                ROOM_LAYOUTS_BOSS[layout_index]
            }
        });
        layout = layout.trim().split('\n').map(|l| l.trim()).collect::<String>();

        let (obstacles, enemies, doors, grid) = Room::parse_layout(sw, sh, width, height, &layout, &door_connects, level);

        Room {
            tag,
            state,
            width,
            height,
            grid,
            dungeon_coords,
            doors,
            obstacles,
            enemies,
            shots,
            drops,
        }
    }

    pub fn get_target_distance_grid(&self, target: Vec2, sw: f32, sh: f32) -> [[i32; ROOM_WIDTH]; ROOM_HEIGHT] {
        let mut grid = self.grid;
        let (ti, tj) = pos_to_room_coords(target, sw, sh);
        if !(0..ROOM_HEIGHT).contains(&ti) || !(0..ROOM_WIDTH).contains(&tj) { return grid; }

        grid[ti][tj] = i32::MAX;
        let mut q = VecDeque::<(usize, usize)>::new();
        q.push_back((ti, tj));

        while !q.is_empty() {
            let (i, j) = q.pop_front().unwrap();

            if i > 0               && grid[i - 1][j] == 0 { 
                grid[i - 1][j] = grid[i][j] - 1;
                q.push_back((i - 1, j));
            }
            if j > 0               && grid[i][j - 1] == 0 {
                grid[i][j - 1] = grid[i][j] - 1;
                q.push_back((i, j - 1));
            }
            if j < ROOM_WIDTH - 1  && grid[i][j + 1] == 0 {
                grid[i][j + 1] = grid[i][j] - 1;
                q.push_back((i, j + 1));
            }
            if i < ROOM_HEIGHT - 1 && grid[i + 1][j] == 0 {
                grid[i + 1][j] = grid[i][j] - 1;
                q.push_back((i + 1, j));
            }
        }

        grid
    }

    fn generate_collectable(&mut self, sw: f32, sh: f32) {
        if thread_rng().gen_bool(0.8) {
            let (mut r, mut c) = (ROOM_HEIGHT / 2, ROOM_WIDTH / 2);
            let (bw, bh) = (sw / self.width, sh / self.height);
            let mut visited = [[false; ROOM_WIDTH]; ROOM_HEIGHT];
            let mut q = VecDeque::new();
            q.push_back((r, c));

            while !q.is_empty() {
                let (i, j) = q.pop_front().unwrap();
                visited[i][j] = true;

                if self.grid[i][j] == 0 { r = i; c = j; break }

                if i > 0           && !visited[i - 1][j] { q.push_back((i - 1, j)) }
                if j > 0           && !visited[i][j - 1] { q.push_back((i, j - 1)) }
                if i < ROOM_HEIGHT && !visited[i + 1][j] { q.push_back((i + 1, j)) }
                if j < ROOM_WIDTH  && !visited[i][j + 1] { q.push_back((i, j + 1)) }
            }

            let p = thread_rng().gen_range(0..101);

            self.drops.push(Collectable {
                props: ActorProps {
                    pos: Vec2::new(bw * (c as f32) + bw / 2., bh * (r as f32) + bh / 2.).into(),
                    scale: Vec2::splat(COLLECTABLE_SCALE),
                    translation: Vec2::ZERO,
                    forward: Vec2::ZERO,
                    velocity: Vec2::ZERO,
                },
                tag: match p {
                    0..=50 => CollectableTag::RedHeart((thread_rng().gen::<f32>() + 1.).round() / 2.),
                    51..=65 => CollectableTag::SpeedBoost(1.3),
                    66..=80 => CollectableTag::DamageBoost(1.5),
                    _ => CollectableTag::ShootRateBoost(1.3),
                },
                state: CollectableState::Base,
            });
        }
    }

    fn generate_item() -> Item {
        Item {
            tag: match thread_rng().gen_bool(0.5) {
                true => ITEM_POOL_ACTIVE[thread_rng().gen_range(0..ITEM_POOL_ACTIVE.len())],
                false => ITEM_POOL_PASSIVE[thread_rng().gen_range(0..ITEM_POOL_PASSIVE.len())],
            },
            cooldown: 0.,
        }
    }
}

#[derive(Debug)]
pub struct Dungeon {
    grid: [[Option<Room>; DUNGEON_GRID_COLS]; DUNGEON_GRID_ROWS],
    level: u32,
}

impl Dungeon {
    pub fn generate_dungeon(screen: (f32, f32), level: u32) -> Self {
        const INIT: Option<Room> = None;
        const INIT_ROW: [Option<Room>; DUNGEON_GRID_COLS] = [INIT; DUNGEON_GRID_COLS];
        let mut grid_rooms: [[Option<Room>; DUNGEON_GRID_COLS]; DUNGEON_GRID_ROWS] = [INIT_ROW; DUNGEON_GRID_ROWS];
        let mut grid;
        let mut room_dungeon_coords;

        loop {
            let room_count = (thread_rng().gen_range(0..2) + 5 + level * 2) as usize;
            grid = [[0_usize; DUNGEON_GRID_COLS]; DUNGEON_GRID_ROWS];
            room_dungeon_coords = Vec::new();
            let start_room = Dungeon::get_start_room_coords();

            let mut q = VecDeque::<((usize, usize), usize)>::new();
            q.push_back((start_room, 0));
            room_dungeon_coords.push((start_room, 0));
            grid[start_room.0][start_room.1] = room_dungeon_coords.len();

            while !q.is_empty() {
                let ((i, j), c) = q.pop_front().unwrap();

                if thread_rng().gen_bool(0.5) && room_dungeon_coords.len() < room_count && i < DUNGEON_GRID_ROWS - 1 && grid[i + 1][j] == 0 && Dungeon::check_room_cardinals(&grid, (i + 1, j)) <= 1 { 
                    room_dungeon_coords.push(((i + 1, j), c + 1));
                    grid[i + 1][j] = room_dungeon_coords.len();
                    q.push_back(((i + 1, j), c + 1)); 
                }
                if thread_rng().gen_bool(0.5) && room_dungeon_coords.len() < room_count && i > 0                     && grid[i - 1][j] == 0 && Dungeon::check_room_cardinals(&grid, (i - 1, j)) <= 1 {
                    room_dungeon_coords.push(((i - 1, j), c + 1));
                    grid[i - 1][j] = room_dungeon_coords.len();
                    q.push_back(((i - 1, j), c + 1));
                }
                if thread_rng().gen_bool(0.5) && room_dungeon_coords.len() < room_count && j < DUNGEON_GRID_COLS - 1 && grid[i][j + 1] == 0 && Dungeon::check_room_cardinals(&grid, (i, j + 1)) <= 1 {
                    room_dungeon_coords.push(((i, j + 1), c + 1));
                    grid[i][j + 1] = room_dungeon_coords.len();
                    q.push_back(((i, j + 1), c + 1));
                }
                if thread_rng().gen_bool(0.5) && room_dungeon_coords.len() < room_count && j > 0                     && grid[i][j - 1] == 0 && Dungeon::check_room_cardinals(&grid, (i, j - 1)) <= 1 {
                    room_dungeon_coords.push(((i, j - 1), c + 1));
                    grid[i][j - 1] = room_dungeon_coords.len();
                    q.push_back(((i, j - 1), c + 1));
                }
            }

            if room_dungeon_coords.len() < room_count { continue }

            if Dungeon::check_dungeon_consistency(&grid, room_count) { break }
        }

        room_dungeon_coords.sort_by_key(|k| (k.1, Dungeon::check_room_cardinals(&grid, k.0)));

        let mut special_rooms = vec![
            RoomTag::Item,
            RoomTag::Boss,
        ];

        for ((i, j), _) in room_dungeon_coords.into_iter() {
            let mut doors = [None; 4];

            if i > 0                     && grid[i - 1][j] != 0 { doors[0] = Some(((i - 1, j), Direction::North)); }
            if j > 0                     && grid[i][j - 1] != 0 { doors[1] = Some(((i, j - 1), Direction::West)); }
            if j < DUNGEON_GRID_COLS - 1 && grid[i][j + 1] != 0 { doors[2] = Some(((i, j + 1), Direction::East)); }
            if i < DUNGEON_GRID_ROWS - 1 && grid[i + 1][j] != 0 { doors[3] = Some(((i + 1, j), Direction::South)); }

            let tag;
            if (i, j) == Dungeon::get_start_room_coords() { tag = RoomTag::Start; }
            else if let Some(s) = special_rooms.pop() { tag = s; }
            else { 
                tag = match thread_rng().gen_bool(0.8) {
                    true => RoomTag::Mob, 
                    false => RoomTag::Empty, 
                };
            }

            grid_rooms[i][j] = Some(Room::generate_room(screen, (i, j), doors, tag, level));
        }

        Dungeon {
            grid: grid_rooms,
            level,
        }
    }

    pub fn get_room(&self, dungeon_coords: (usize, usize)) -> GameResult<Option<&Room>> {
        if !(0..DUNGEON_GRID_ROWS).contains(&dungeon_coords.0) { return Err(Errors::UnknownGridCoords(dungeon_coords).into()); }
        if !(0..DUNGEON_GRID_COLS).contains(&dungeon_coords.1) { return Err(Errors::UnknownGridCoords(dungeon_coords).into()); }
        Ok(self.grid[dungeon_coords.0][dungeon_coords.1].as_ref())
    }

    pub fn get_room_mut(&mut self, dungeon_coords: (usize, usize)) -> GameResult<Option<&mut Room>> {
        if !(0..DUNGEON_GRID_ROWS).contains(&dungeon_coords.0) { return Err(Errors::UnknownGridCoords(dungeon_coords).into()); }
        if !(0..DUNGEON_GRID_COLS).contains(&dungeon_coords.1) { return Err(Errors::UnknownGridCoords(dungeon_coords).into()); }
        Ok(self.grid[dungeon_coords.0][dungeon_coords.1].as_mut())
    }

    pub fn get_grid(&self) -> &[[Option<Room>; DUNGEON_GRID_COLS]; DUNGEON_GRID_ROWS] { &self.grid }

    pub fn get_level(&self) -> u32 { self.level }

    pub const fn get_start_room_coords() -> (usize, usize) { (3, 5) }

    pub fn update_rooms_state(&mut self, (i, j): (usize, usize)) -> GameResult {
        let mut rooms = vec![(i, j)];

        if i > 0                     { rooms.push((i - 1, j)); }
        if j > 0                     { rooms.push((i, j - 1)); }
        if j < DUNGEON_GRID_COLS - 1 { rooms.push((i, j + 1)); }
        if i < DUNGEON_GRID_ROWS - 1 { rooms.push((i + 1, j)); }

        for (ri, rj) in rooms {
            match self.grid[ri][rj].as_mut() {
                Some(r) => {
                    r.state = match r.state {
                        RoomState::Undiscovered => RoomState::Discovered,
                        _ => r.state,
                    };
                },
                None => (),
            };
        }

        Ok(())
    }

    pub fn check_dungeon_consistency(grid: &[[usize; DUNGEON_GRID_COLS]; DUNGEON_GRID_ROWS], rooms_len: usize) -> bool {
        let mut checked = vec![false; rooms_len];
        let mut q = VecDeque::<(usize, usize)>::new();
        q.push_back(Dungeon::get_start_room_coords());

        while !q.is_empty() {
            let (i, j) = q.pop_front().unwrap();

            if checked[grid[i][j] - 1] { continue; }

            checked[grid[i][j] - 1] = true;

            if i < DUNGEON_GRID_ROWS - 1 && grid[i + 1][j] != 0 { q.push_back((i + 1, j)); }
            if i > 0_usize               && grid[i - 1][j] != 0 { q.push_back((i - 1, j)); }
            if j < DUNGEON_GRID_COLS - 1 && grid[i][j + 1] != 0 { q.push_back((i, j + 1)); }
            if j > 0_usize               && grid[i][j - 1] != 0 { q.push_back((i, j - 1)); }
        }

        !checked.contains(&false)
    }

    fn check_room_cardinals(grid: &[[usize; DUNGEON_GRID_COLS]; DUNGEON_GRID_ROWS], room: (usize, usize)) -> usize {
        let mut result = 0;
        let (i, j) = room;

        if i < DUNGEON_GRID_ROWS - 1 && grid[i + 1][j] != 0 { result += 1; }
        if i > 0                     && grid[i - 1][j] != 0 { result += 1; }
        if j < DUNGEON_GRID_COLS - 1 && grid[i][j + 1] != 0 { result += 1; }
        if j > 0                     && grid[i][j - 1] != 0 { result += 1; }

        result
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub pos: Vec2Wrap,
    pub scale: Vec2,
    pub tag: BlockTag,
}

#[derive(Debug, Copy, Clone)]
pub enum BlockTag {
    Door {
        dir: Direction,
        is_open: bool,
        connects_to: (usize, usize),
    },
    Wall,
    Stone,
    Spikes,
    Hatch(bool),
    Pedestal(Option<Item>),
}

impl Stationary for Block {
    fn update(&mut self, _conf: &mut Config, _delta_time: f32) -> GameResult { Ok(()) }

    fn draw(&self, ctx: &mut Context, conf: &mut Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);

        let mut rotation = 0.;
        let sprite = match self.tag {
            BlockTag::Door { dir, is_open, .. } => {
                rotation = match dir {
                    Direction::North => 0.,
                    Direction::South => PI,
                    Direction::West => -PI / 2.,
                    Direction::East => PI / 2.,
                };

                match is_open {
                    true => conf.assets.sprites.get("door_open").unwrap(),
                    false => conf.assets.sprites.get("door_closed").unwrap(),    
                }
            },
            BlockTag::Wall => conf.assets.sprites.get("wall").unwrap(),
            BlockTag::Stone => conf.assets.sprites.get("stone").unwrap(),
            BlockTag::Spikes => conf.assets.sprites.get("spikes").unwrap(),
            BlockTag::Hatch(is_open) => {
                match is_open {
                    true => conf.assets.sprites.get("hatch_open").unwrap(),
                    false => conf.assets.sprites.get("hatch_closed").unwrap(),
                }
            },
            BlockTag::Pedestal(_) => conf.assets.sprites.get("item_pedestal").unwrap(),
        };

        let draw_params = DrawParam::default()
            .dest(self.pos)
            .rotation(rotation)
            .scale(self.scale_to_screen(sw, sh, sprite.dimensions()) * 1.1)
            .offset([0.5, 0.5]);

        graphics::draw(ctx, sprite, draw_params)?;
        if let BlockTag::Pedestal(Some(i)) = self.tag {
            let item_sprite = match i.tag {
                ItemTag::Passive(p) => match p {
                    ItemPassive::IncreaseMaxHealth(_) => conf.assets.sprites.get("poop_item").unwrap(),
                },
                ItemTag::Active(a) => match a {
                    ItemActive::Heal(_) => conf.assets.sprites.get("heart_item").unwrap(),
                },
            };

            graphics::draw(ctx, item_sprite, draw_params.scale(self.scale_to_screen(sw, sh, item_sprite.dimensions()) * ITEM_SCALE))?;
        };

        if conf.draw_bbox_stationary { self.draw_bbox(ctx, (sw, sh))?; }

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.pos.0 }

    fn get_scale(&self) -> Vec2 { self.scale }

    fn get_tag(&self) -> BlockTag { self.tag }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
