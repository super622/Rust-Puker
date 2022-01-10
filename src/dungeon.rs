use ggez::{
    graphics::{self, DrawParam, Rect, Color},
    GameResult,
    Context
};
use crate::{
    assets::*,
    entities::*,
    utils::*,
    consts::*,
};
use std::{
    collections::{VecDeque},
    fmt::{Formatter, Display},
    f32::consts::PI,
};
use rand::{thread_rng, Rng};
use glam::f32::Vec2;

#[derive(Clone, Copy, Hash, Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Direction::North => write!(f, "North"),
            Direction::South => write!(f, "South"),
            Direction::East => write!(f, "East"),
            Direction::West => write!(f, "West"),
        }
    }
}

impl PartialEq for Direction {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}
impl Eq for Direction {}

pub trait ModelActor: Model + Actor {}

impl<T: Model + Actor> ModelActor for T {}

#[derive(Debug)]
pub struct Room {
    pub width: f32,
    pub height: f32,
    pub grid_num: usize,
    pub doors: [Option<Door>; 4],
    pub obstacles: Vec<Box<dyn Stationary>>,
    pub enemies: Vec<Box<dyn ModelActor>>,
}

impl Room {
    pub fn update(&mut self, _delta_time: f32) -> GameResult {
        let mut dead_enemies = Vec::<usize>::new();

        for (i, enemy) in self.enemies.iter_mut().enumerate() {
            enemy.update(_delta_time)?;
            if enemy.get_health() <= 0. { dead_enemies.push(i); }
        }
        
        for dead in dead_enemies {
            self.enemies.remove(dead);
        }

        if self.enemies.is_empty() {
            for door in self.doors.iter_mut() {
                match door {
                    Some(d) => d.is_open = true,
                    _ => ()
                }
            }
        }

        Ok(())
    }
    
    pub fn draw(&self, ctx: &mut Context, assets: &Assets, world_coords: (f32, f32)) -> GameResult {
        let (sw, sh) = world_coords;
        let draw_params = DrawParam::default()
            .dest([sw / 2., sh / 2.])
            .scale(Room::get_room_scale(sw, sh, assets.floor.dimensions()))
            .offset([0.5, 0.5]);
        
        graphics::draw(ctx, &assets.floor, draw_params)?;

        for obst in self.obstacles.iter() {
            obst.draw(ctx, assets, world_coords)?;
        }

        // for (i, v) in self.doors.iter().enumerate() {
        //     match i {
        //         0 => { graphics::draw(ctx, &assets.door_base, draw_params)?; },
        //         1 => { graphics::draw(ctx, &assets.door_east, draw_params)?; },
        //         2 => { graphics::draw(ctx, &assets.door_south, draw_params)?; },
        //         _ => { graphics::draw(ctx, &assets.door_west, draw_params)?; },
        //     }
        // }
        //
        for enemy in self.enemies.iter() {
            enemy.draw(ctx, assets, world_coords)?;
        }

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
        screen_to_world_space(sw, sh, coords + dims / 2.)
    }

    fn generate_room(grid_num: usize, screen: (f32, f32)) -> Room {
        let (sw, sh) = screen;
        
        let width = ROOM_WIDTH;
        let height = ROOM_HEIGHT;
        let doors = [None; 4];
        let mut obstacles: Vec<Box<dyn Stationary>> = Vec::new(); 
        let mut enemies: Vec<Box<dyn ModelActor>> = Vec::new();

        let mut layout = ROOM_LAYOUTS_MOB[0].trim().split('\n').map(|l| l.trim()).collect::<String>();

        for (i, c) in layout.chars().enumerate() {
            match c {
                '#' => {
                    obstacles.push(Box::new(Wall {
                        pos: Room::get_model_pos(sw, sh, width, height, i).into(),
                        scale: Vec2::splat(WALL_SCALE),
                    }));
                },
                'd' => {
                    obstacles.push(Box::new(Wall {
                        pos: Room::get_model_pos(sw, sh, width, height, i).into(),
                        scale: Vec2::splat(WALL_SCALE),
                    }));
                },
                'e' => {
                    enemies.push(Box::new(EnemyMask {
                            props: ActorProps {
                                pos: Room::get_model_pos(sw, sh, width, height, i).into(),
                                scale: Vec2::splat(ENEMY_SCALE),
                                translation: Vec2::ZERO,
                                forward: Vec2::ZERO,
                            },
                            speed: ENEMY_SPEED,
                            health: ENEMY_HEALTH,
                            state: ActorState::BASE,
                            shoot_rate: ENEMY_SHOOT_RATE,
                            shoot_range: ENEMY_SHOOT_RANGE,
                            shoot_timeout: ENEMY_SHOOT_TIMEOUT,
                            shots: Vec::new(),
                            color: Color::WHITE,
                        })
                    );
                },
                _ => (),
            }
        }

        Room {
            width,
            height,
            grid_num,
            doors,
            obstacles,
            enemies,
        }
    }
}

#[derive(Debug)]
pub struct Dungeon {
    grid: [usize; DUNGEON_GRID_ROWS * DUNGEON_GRID_COLS],
    rooms: Vec<Room>,
}

impl Dungeon {
    pub fn generate_dungeon(screen: (f32, f32)) -> Self {
        let level = 1;
        let room_count = thread_rng().gen_range(0..2) + 5 + level * 2;
        let mut grid = [0; DUNGEON_GRID_ROWS * DUNGEON_GRID_COLS];
        let mut rooms = Vec::new();

        let mut q = VecDeque::<usize>::new();
        q.push_back(35);

        while !q.is_empty() && room_count > rooms.len() {
            let cur = q.pop_front().unwrap();

            if cur < 1 || cur > 79 { continue; }

            if grid[cur] != 0 { continue; }

            rooms.push(Room::generate_room(cur, screen));
            grid[cur] = rooms.len();

            q.push_back(cur + 10);
            q.push_back(cur - 10);
            q.push_back(cur + 1);
            q.push_back(cur - 1);
        }

        for r in rooms.iter_mut() {
            let n = r.grid_num - 10;
            let s = r.grid_num + 10;
            let w = r.grid_num - 1;
            let e = r.grid_num + 1;

            // if n > 0          && grid[n] != 0 { r.doors[0] = Some(Door { pos: is_open: false, connects_to: grid[n] }); }
            // if s > grid.len() && grid[s] != 0 { r.doors[2] = Some(Door { is_open: false, connects_to: grid[s] }); }
            // if w > 0          && grid[w] != 0 { r.doors[1] = Some(Door { is_open: false, connects_to: grid[w] }); }
            // if e > grid.len() && grid[e] != 0 { r.doors[3] = Some(Door { is_open: false, connects_to: grid[e] }); }
        }

        Dungeon {
            grid,
            rooms,
        }
    }

    pub fn get_room(&self, grid_num: usize) -> GameResult<&Room> {
        let index = self.get_room_index(grid_num)?;
        if !(0..self.rooms.len()).contains(&index) { return Err(Errors::UnknownRoomIndex(index).into()); }
        Ok(&self.rooms[index])
    }

    pub fn get_room_mut(&mut self, grid_num: usize) -> GameResult<&mut Room> {
        let index = self.get_room_index(grid_num)?;
        if !(0..self.rooms.len()).contains(&index) { return Err(Errors::UnknownRoomIndex(index).into()); }
        Ok(&mut self.rooms[index])
    }

    fn get_room_index(&self, grid_num: usize) -> GameResult<usize> {
        if !(0..self.grid.len()).contains(&grid_num) { return Err(Errors::UnknownGridNum(grid_num).into()); }
        Ok(self.grid[grid_num] - 1)
    }

    pub fn get_start_room_grid_num() -> usize { DUNGEON_GRID_ROWS * DUNGEON_GRID_COLS / 2 }
}

#[derive(Debug, Copy, Clone)]
pub struct Door {
    pub pos: Vec2Wrap,
    pub scale: Vec2,
    pub is_open: bool,
    pub connects_to: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct Wall {
    pub pos: Vec2Wrap,
    pub scale: Vec2,
}

impl Stationary for Wall {
    fn draw(&self, ctx: &mut Context, assets: &Assets, screen: (f32, f32)) -> GameResult {
        let (sw, sh) = screen;
        let pos: Vec2Wrap = world_to_screen_space(sw, sh, self.pos.into()).into();
        let draw_params = DrawParam::default()
            .dest(pos)
            .scale(self.scale_to_screen(sw, sh, assets.wall.dimensions()))
            .offset([0.5, 0.5]);

        graphics::draw(ctx, &assets.wall, draw_params)?;

        self.draw_bbox(ctx, screen)?;

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.pos.0 }

    fn get_scale(&self) -> Vec2 { self.scale }
}
