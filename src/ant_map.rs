use std::slice::Iter;

#[derive(Clone , Copy)]
pub enum Direction {
    Up = 0,
    Right,
    Down,
    Left,
}

use Direction as Go;

impl Direction {
    pub fn rotated(self, rot: Rotation) -> Self {
        use Direction::*;
        
        [Up, Right, Down, Left][(self as usize + rot as usize) % 4]
    }
    
    pub fn rotate(&mut self, rot: Rotation) {
        *self = self.rotated(rot);
    }
}

#[derive(Clone, Copy)]
pub enum Rotation {
    Left = 3,  // counter-clockwise rotation
    Right = 1, // clockwise rotation
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
    
    pub fn invert(&mut self) {
        use Rotation::*;
        
        *self = match *self {
            Left => Right,
            Right => Left,
        }
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
    pub fn new(width: usize, height: usize, looking: Direction, rots: Vec<Rotation>) -> AntMap {
        AntMap {
            ant: (width / 2, height / 2, looking),
            map: vec![vec![0; height]; width],
            width,
            height,
            rots,
        }
    }

    pub fn scale(&mut self, amount: usize) {
        for col in &mut self.map {
            col.extend((0..amount * 2).map(|_| 0));
            col.rotate_right(amount);
        }
        
        let new_width = self.width + amount * 2;
        self.map.extend((0..amount * 2).map(|_| vec![0; new_width]));
        self.map.rotate_right(amount);
        
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
      
        *pos += 1;
        if *pos == self.rots.len() as u32 {
            *pos = 1;
        }

        let turn = self.rots[*pos as usize - 1];
        self.ant.2.rotate(turn);
        
        true
    }

    pub fn invert_rotation(&mut self, index: usize) {
        self.rots[index].invert()
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
