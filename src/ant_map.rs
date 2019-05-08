use std::iter;
use std::slice::Iter;

#[derive(Clone , Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

use Direction as Go;

#[derive(Clone, Copy)]
pub enum Rotation {
    Left,  // counter-clockwise rotation
    Right, // clockwise rotation
}

use Rotation as Turn;

impl Rotation {
    pub fn vec_from_string(s: &str) -> Vec<Rotation> {
        s.chars()
            .flat_map(|c| match c {
                'R' => Some(Turn::Right),
                'L' => Some(Turn::Left),
                _ => None,
            })
            .collect()
    }
}

pub struct AntMap {
    ant: (usize, usize, Direction),
    map: Vec<Vec<u32>>,
    width: usize,
    height: usize,

    rots: Vec<Rotation>,
}

impl AntMap {
    pub fn new(width: usize, height: usize, looking: Direction, stages: Vec<Rotation>) -> AntMap {
        AntMap {
            ant: (width / 2, height / 2, looking),
            map: iter::repeat(iter::repeat(0).take(height).collect()).take(width).collect(),
            width,
            height,

            rots: stages,
        }
    }

    pub fn scale(&mut self, amount: usize) {

        self.map = iter::repeat(iter::repeat(0).take(amount * 2 + self.height).collect()).take(amount)
        .chain(self.map.iter().map(|v| iter::repeat(0).take(amount)
            .chain(v.iter().map(|v| *v)
                .chain(iter::repeat(0).take(amount))
            ).collect())
        ).chain(iter::repeat(iter::repeat(0).take(amount * 2 + self.height).collect()).take(amount)).collect();
        
        self.height += amount * 2;
        self.width += amount * 2;
        
        self.ant.0 += amount;
        self.ant.1 += amount;
    }

    pub fn step_ahead(&mut self) -> bool {

        match self.ant.2 {
            Go::Up => if self.ant.1 > 0 {
                    self.ant.1 -= 1;
                } else { return false },
            
            Go::Down => if self.ant.1 < self.height - 1 {
                    self.ant.1 += 1;
                } else { return false },
            
            Go::Left => if self.ant.0 > 0 {
                    self.ant.0 -= 1;
                } else { return false },
            
            Go::Right => if self.ant.0 < self.width - 1 {
                    self.ant.0 += 1;
                } else { return false },
        };

        let pos = &mut self.map[self.ant.0][self.ant.1];

        if *pos == self.rots.len() as u32 { *pos = 1; }
        else { *pos += 1; }

        self.ant.2 = match (&self.ant.2, &self.rots[*pos as usize - 1]) {
            (Go::Up,    Turn::Left ) => Go::Left,
            (Go::Down,  Turn::Left ) => Go::Right,
            (Go::Left,  Turn::Left ) => Go::Down,
            (Go::Right, Turn::Left ) => Go::Up,

            (Go::Up,    Turn::Right) => Go::Right,
            (Go::Down,  Turn::Right) => Go::Left,
            (Go::Left,  Turn::Right) => Go::Up,
            (Go::Right, Turn::Right) => Go::Down,
        };
        
        true
    }

    pub fn invert_rotation(&mut self, index: usize) {
        self.rots[index] = match self.rots[index] {
            Turn::Left => Turn::Right,
            Turn::Right => Turn::Left,
        }
    }

    pub fn add_stage(&mut self, turn: Rotation) { self.rots.push(turn); }
    pub fn remove_stage(&mut self, index: usize) { self.rots.remove(index); }

    pub fn iter(&self) -> Iter<'_, Vec<u32>> { self.map.iter() }
    pub fn ant(&self) -> &(usize, usize, Direction) { &self.ant }
    pub fn stages(&self) -> &Vec<Rotation> { &self.rots }
    
    pub fn width (&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }

    pub fn reset(&mut self) {
        self.map.iter_mut().for_each(|v| v.iter_mut().for_each(|v| *v = 0 ));
        self.ant = (self.width / 2, self.height / 2, Go::Up);
    }

    pub fn shrink(&mut self) {
        let mut up = self.height / 2;
        let mut down = self.height / 2 + 1;
        let mut left = self.width / 2;
        let mut right = self.width / 2 + 1;

        self.map.iter_mut().enumerate().for_each(|(x, v)| {
            v.iter_mut().enumerate().for_each(|(y, v)| {
                if *v != 0 {
                    if y < up { up = y; }
                    else if y > down { down = y; }
                    
                    if x < left { left = x; }
                    else if x > right { right = x; }
                }
            })
        });

        self.map = self.map.iter().skip(left).take(right - left + 1).map(|v| {
            v.iter().skip(up).take(down - up + 1).map(|v| *v).collect()
        }).collect();

        self.width = right - left;
        self.height = down - up;
        self.ant.0 -= left;
        self.ant.1 -= up;
    }
}

impl std::ops::Index<usize> for AntMap {
    type Output = Vec<u32>;

    fn index(&self, index: usize) -> &Vec<u32> {
        &self.map[index]
    }
}
