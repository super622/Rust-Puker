use ggez::{
    graphics::{self, DrawParam, Rect},
    GameResult,
    Context,
    audio::SoundSource,
};
use crate::{
    assets::*,
    entities::*,
    utils::*,
    consts::*,
    traits::*,
};
use std::{
    any::Any,
    collections::VecDeque,
    f32::consts::PI,
};
use rand::{thread_rng, Rng};
use glam::f32::Vec2;

#[derive(Debug)]
pub enum RoomTag {
    Start,
    Empty,
    Mob,
    Boss,
    Item,
}

#[derive(Debug)]
pub struct Room {
    pub tag: RoomTag,
    pub width: f32,
    pub height: f32,
    pub grid_coords: (usize, usize),
    pub doors: Vec<usize>,
    pub obstacles: Vec<Box<dyn Stationary>>,
    pub enemies: Vec<Box<dyn Actor>>,
    pub shots: Vec<Shot>,
    // pub drops: Vec<Collectable>
    pub just_entered: bool,
}

impl Room {
    pub fn update(&mut self, ctx: &mut Context, assets: &mut Assets, conf: &Config, _delta_time: f32) -> GameResult {

        for shot in self.shots.iter_mut() {
            shot.update(ctx, assets, conf, _delta_time)?;
        }

        for enemy in self.enemies.iter_mut() {
            enemy.update(ctx, assets, conf, _delta_time)?;
        }

        let dead_enemies = self.enemies.iter()
            .enumerate()
            .filter(|e| e.1.get_health() <= 0.)
            .map(|e| e.0).collect::<Vec<_>>();
        for (i,d) in dead_enemies.iter().enumerate() { self.enemies.remove(d - i); }

        let dead_shots = self.shots.iter()
            .enumerate()
            .filter(|s| s.1.get_pos().distance(s.1.spawn_pos.0) >= s.1.range)
            .map(|s| s.0).collect::<Vec<_>>();
        for (i,d) in dead_shots.iter().enumerate() { 
            self.shots.remove(d - i); 
            let _ = assets.audio.get_mut("bubble_pop_sound").unwrap().play(ctx);
        }
        
        match self.tag {
            RoomTag::Mob => {
                if self.enemies.is_empty() {
                    self.tag = RoomTag::Empty;
                    let _ = assets.audio.get_mut("door_open_sound").unwrap().play(ctx);
                    for door in self.doors.iter() {
                        self.obstacles[*door].as_any_mut().downcast_mut::<Door>().unwrap().is_open = true;
                    }
                }
                else {
                    for door in self.doors.iter() {
                        let d = self.obstacles[*door].as_any_mut().downcast_mut::<Door>().unwrap();
                        if d.is_open {
                            let _ = assets.audio.get_mut("door_close_sound").unwrap().play(ctx);
                            d.is_open = false;
                        }
                    }
                }
            },
            _ => (),
        }

        Ok(())
    }
    
    pub fn draw(&self, ctx: &mut Context, assets: &mut Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let draw_params = DrawParam::default()
            .scale(Room::get_room_scale(sw, sh, assets.sprites.get("floor").unwrap().dimensions()));
        
        graphics::draw(ctx, assets.sprites.get("floor").unwrap(), draw_params)?;

        for obst in self.obstacles.iter() { obst.draw(ctx, assets, conf)?; }

        for enemy in self.enemies.iter() { enemy.draw(ctx, assets, conf)?; }

        for shot in self.shots.iter() { shot.draw(ctx, assets, conf)?; }

        Ok(())
    }

    fn get_room_scale(sw: f32, sh: f32, image: Rect) -> [f32; 2] {
        [sw / image.w, sh / image.h]
    }

    /// Helper function for determining the models/stationaries positions.
    ///
    fn get_model_pos(sw: f32, sh: f32, rw: f32, rh: f32, index: usize) -> Vec2 {
        let dims = Vec2::new(sw / rw, sh / rh);
        let coords = Vec2::new((index % (rw as usize)) as f32, (index / (rw as usize)) as f32) * dims;
        coords + dims / 2.
    }

    fn parse_layout(sw: f32, sh: f32, rw: f32, rh: f32, layout: &str, door_connects: &[Option<((usize, usize), Direction)>; 4]) -> (Vec<Box<dyn Stationary>>, Vec<Box<dyn Actor>>, Vec<usize>) {
        let mut doors: Vec<usize> = Vec::new();
        let mut obstacles: Vec<Box<dyn Stationary>> = Vec::new(); 
        let mut enemies: Vec<Box<dyn Actor>> = Vec::new();

        let mut door_index = 0_usize;

        for (i, c) in layout.chars().enumerate() {
            match c {
                '#' => {
                    obstacles.push(Box::new(Wall {
                        pos: Room::get_model_pos(sw, sh, rw, rh, i).into(),
                        scale: Vec2::splat(WALL_SCALE),
                    }));
                },
                '.' => {
                    obstacles.push(Box::new(Stone {
                        pos: Room::get_model_pos(sw, sh, rw, rh, i).into(),
                        scale: Vec2::splat(WALL_SCALE),
                    }));
                },
                'd' => {
                    if let Some((connects_to, dir)) = door_connects[door_index] {
                        doors.push(obstacles.len());
                        obstacles.push(Box::new(Door {
                            pos: Room::get_model_pos(sw, sh, rw, rh, i).into(),
                            scale: Vec2::splat(WALL_SCALE),
                            dir,
                            is_open: true,
                            connects_to,
                        }));
                    }
                    else {
                        obstacles.push(Box::new(Wall {
                            pos: Room::get_model_pos(sw, sh, rw, rh, i).into(),
                            scale: Vec2::splat(WALL_SCALE),
                        }));
                    }
                    door_index += 1;
                },
                'm' => {
                    enemies.push(Box::new(EnemyMask {
                        props: ActorProps {
                            pos: Room::get_model_pos(sw, sh, rw, rh, i).into(),
                            scale: Vec2::splat(ENEMY_SCALE),
                            translation: Vec2::ZERO,
                            forward: Vec2::ZERO,
                            velocity: Vec2::ZERO,
                        },
                        tag: EnemyTag::Shooter,
                        health: ENEMY_HEALTH,
                        state: ActorState::Base,
                        shoot_rate: ENEMY_SHOOT_RATE,
                        shoot_range: ENEMY_SHOOT_RANGE,
                        shoot_timeout: ENEMY_SHOOT_TIMEOUT,
                        animation_cooldown: 0.,
                        afterlock_cooldown: ENEMY_AFTERLOCK_COOLDOWN,
                    }));
                },
                'b' => {
                    enemies.push(Box::new(EnemyBlueGuy {
                        props: ActorProps {
                            pos: Room::get_model_pos(sw, sh, rw, rh, i).into(),
                            scale: Vec2::splat(ENEMY_SCALE),
                            translation: Vec2::ZERO,
                            forward: Vec2::ZERO,
                            velocity: Vec2::ZERO,
                        },
                        tag: EnemyTag::Chaser,
                        speed: ENEMY_SPEED,
                        health: ENEMY_HEALTH,
                        state: ActorState::Base,
                        animation_cooldown: 0.,
                        afterlock_cooldown: ENEMY_AFTERLOCK_COOLDOWN,
                    }));
                },
                _ => (),
            }
        }

        (obstacles, enemies, doors)
    }

    fn generate_room(screen: (f32, f32), grid_coords: (usize, usize), door_connects: [Option<((usize, usize), Direction)>; 4], tag: RoomTag) -> Room {
        let (sw, sh) = screen;
        
        let width = ROOM_WIDTH;
        let height = ROOM_HEIGHT;
        let shots = Vec::new();

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
            _ => ROOM_LAYOUT_START,
        });
        layout = layout.trim().split('\n').map(|l| l.trim()).collect::<String>();

        let (obstacles, enemies, doors) = Room::parse_layout(sw, sh, width, height, &layout, &door_connects);

        Room {
            tag,
            width,
            height,
            grid_coords,
            doors,
            obstacles,
            enemies,
            shots,
            just_entered: true,
        }
    }
}

#[derive(Debug)]
pub struct Dungeon {
    grid: [[usize; DUNGEON_GRID_COLS]; DUNGEON_GRID_ROWS],
    rooms: Vec<Room>,
}

impl Dungeon {
    pub fn generate_dungeon(screen: (f32, f32)) -> Self {
        let level = 1;
        let mut grid;
        let mut rooms = Vec::new();
        let mut room_grid_coords;

        loop {
            let room_count = thread_rng().gen_range(0..2) + 5 + level * 2;
            grid = [[0; DUNGEON_GRID_COLS]; DUNGEON_GRID_ROWS];
            room_grid_coords = Vec::new();

            let mut q = VecDeque::<(usize, usize)>::new();
            q.push_back(Dungeon::get_start_room_coords());

            while !q.is_empty() {
                let (i, j) = q.pop_front().unwrap();

                if room_grid_coords.len() == room_count { break }

                if thread_rng().gen_range(0..2) == 1 { continue }

                if grid[i][j] != 0 { continue }

                room_grid_coords.push((i, j));
                grid[i][j] = room_grid_coords.len();

                if i < DUNGEON_GRID_ROWS - 1 && grid[i + 1][j] == 0 && Dungeon::check_room_cardinals(&grid, (i + 1, j)) <= 1 { q.push_back((i + 1, j)); }
                if i > 0                     && grid[i - 1][j] == 0 && Dungeon::check_room_cardinals(&grid, (i - 1, j)) <= 1 { q.push_back((i - 1, j)); }
                if j < DUNGEON_GRID_COLS - 1 && grid[i][j + 1] == 0 && Dungeon::check_room_cardinals(&grid, (i, j + 1)) <= 1 { q.push_back((i, j + 1)); }
                if j > 0                     && grid[i][j - 1] == 0 && Dungeon::check_room_cardinals(&grid, (i, j - 1)) <= 1 { q.push_back((i, j - 1)); }
            }

            if room_grid_coords.len() < room_count { continue }

            if Dungeon::check_dungeon_consistency(&grid, room_count) { break }
        }

        for (i, j) in room_grid_coords.into_iter() {
            let mut doors = [None; 4];

            if i > 0                     && grid[i - 1][j] != 0 { doors[0] = Some(((i - 1, j), Direction::North)); }
            if j > 0                     && grid[i][j - 1] != 0 { doors[1] = Some(((i, j - 1), Direction::West)); }
            if j < DUNGEON_GRID_COLS - 1 && grid[i][j + 1] != 0 { doors[2] = Some(((i, j + 1), Direction::East)); }
            if i < DUNGEON_GRID_ROWS - 1 && grid[i + 1][j] != 0 { doors[3] = Some(((i + 1, j), Direction::South)); }

            const START: (usize, usize) = Dungeon::get_start_room_coords();
            let tag = match (i, j) {
                START if true => RoomTag::Start,
                _ => RoomTag::Mob,
            };
            rooms.push(Room::generate_room(screen, (i, j), doors, tag));
        }

        Dungeon {
            grid,
            rooms,
        }
    }

    pub fn get_room(&self, grid_coords: (usize, usize)) -> GameResult<&Room> {
        let index = self.get_room_index(grid_coords)?;
        if !(1..=self.rooms.len()).contains(&index) { return Err(Errors::UnknownRoomIndex(index).into()); }
        Ok(&self.rooms[index - 1])
    }

    pub fn get_room_mut(&mut self, grid_coords: (usize, usize)) -> GameResult<&mut Room> {
        let index = self.get_room_index(grid_coords)?;
        if !(1..=self.rooms.len()).contains(&index) { return Err(Errors::UnknownRoomIndex(index).into()); }
        Ok(&mut self.rooms[index - 1])
    }

    fn get_room_index(&self, grid_coords: (usize, usize)) -> GameResult<usize> {
        if !(0..DUNGEON_GRID_ROWS).contains(&grid_coords.0) { return Err(Errors::UnknownGridCoords(grid_coords).into()); }
        if !(0..DUNGEON_GRID_COLS).contains(&grid_coords.1) { return Err(Errors::UnknownGridCoords(grid_coords).into()); }
        Ok(self.grid[grid_coords.0][grid_coords.1])
    }

    pub fn get_grid(&self) -> &[[usize; DUNGEON_GRID_COLS]; DUNGEON_GRID_ROWS] { &self.grid }

    pub const fn get_start_room_coords() -> (usize, usize) { (3, 5) }

    fn check_dungeon_consistency(grid: &[[usize; DUNGEON_GRID_COLS]; DUNGEON_GRID_ROWS], rooms_len: usize) -> bool {
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

#[derive(Debug, Copy, Clone)]
pub struct Door {
    pub pos: Vec2Wrap,
    pub scale: Vec2,
    pub dir: Direction,
    // pub rotation: f32,
    pub is_open: bool,
    pub connects_to: (usize, usize),
}

impl Stationary for Door {
    fn update(&mut self, _conf: &Config, _delta_time: f32) -> GameResult { Ok(()) }

    fn draw(&self, ctx: &mut Context, assets: &mut Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let rotation = match self.dir {
            Direction::North => 0.,
            Direction::South => PI,
            Direction::West => -PI / 2.,
            Direction::East => PI / 2.,
        };
        let draw_params = DrawParam::default()
            .dest(self.pos)
            .scale(self.scale_to_screen(sw, sh, assets.sprites.get("door_closed").unwrap().dimensions()) * 1.1)
            .rotation(rotation)
            .offset([0.5, 0.5]);

        match self.is_open {
            true => graphics::draw(ctx, assets.sprites.get("door_open").unwrap(), draw_params)?,
            false => graphics::draw(ctx, assets.sprites.get("door_closed").unwrap(), draw_params)?,
        }

        if conf.draw_bbox_stationary { self.draw_bbox(ctx, (sw, sh))?; }

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.pos.0 }

    fn get_scale(&self) -> Vec2 { self.scale }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Wall {
    pub pos: Vec2Wrap,
    pub scale: Vec2,
}

impl Stationary for Wall {
    fn update(&mut self, _conf: &Config, _delta_time: f32) -> GameResult { Ok(()) }

    fn draw(&self, ctx: &mut Context, assets: &mut Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let draw_params = DrawParam::default()
            .dest(self.pos)
            .scale(self.scale_to_screen(sw, sh, assets.sprites.get("wall").unwrap().dimensions()) * 1.1)
            .offset([0.5, 0.5]);

        graphics::draw(ctx, assets.sprites.get("wall").unwrap(), draw_params)?;

        if conf.draw_bbox_stationary { self.draw_bbox(ctx, (sw, sh))?; }

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.pos.0 }

    fn get_scale(&self) -> Vec2 { self.scale }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Stone {
    pub pos: Vec2Wrap,
    pub scale: Vec2,
}

impl Stationary for Stone {
    fn update(&mut self, _conf: &Config, _delta_time: f32) -> GameResult { Ok(()) }

    fn draw(&self, ctx: &mut Context, assets: &mut Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let draw_params = DrawParam::default()
            .dest(self.pos)
            .scale(self.scale_to_screen(sw, sh, assets.sprites.get("wall").unwrap().dimensions()) * 1.1)
            .offset([0.5, 0.5]);

        graphics::draw(ctx, assets.sprites.get("stone").unwrap(), draw_params)?;

        if conf.draw_bbox_stationary { self.draw_bbox(ctx, (sw, sh))?; }

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.pos.0 }

    fn get_scale(&self) -> Vec2 { self.scale }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod unit_test_dungeon {
    use super::*;

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
}
