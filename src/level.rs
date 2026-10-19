//! Level layouts. Tile coordinates have y = 0 at the bottom row.

pub const ROWS: i32 = 15;
pub const TILE: f32 = 32.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Empty,
    Ground,
    Brick,
    Question,
    Used,
    Block,
    Pipe,
    PipeTop,
}

impl Tile {
    pub fn solid(self) -> bool {
        self != Tile::Empty
    }
}

#[derive(Clone)]
pub struct LevelDef {
    pub name: &'static str,
    pub width: i32,
    pub tiles: Vec<Tile>,
    pub coins: Vec<(i32, i32)>,
    pub enemies: Vec<(i32, i32)>,
    pub start: (i32, i32),
    pub flag_x: i32,
    pub sky: [f32; 3],
}

impl LevelDef {
    pub fn get(&self, x: i32, y: i32) -> Tile {
        if x < 0 || x >= self.width {
            return Tile::Block;
        }
        if !(0..ROWS).contains(&y) {
            return Tile::Empty;
        }
        self.tiles[(y * self.width + x) as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, t: Tile) {
        if x >= 0 && x < self.width && (0..ROWS).contains(&y) {
            self.tiles[(y * self.width + x) as usize] = t;
        }
    }
}

struct Builder(LevelDef);

impl Builder {
    fn new(name: &'static str, width: i32, sky: [f32; 3]) -> Self {
        Builder(LevelDef {
            name,
            width,
            tiles: vec![Tile::Empty; (width * ROWS) as usize],
            coins: vec![],
            enemies: vec![],
            start: (2, 2),
            flag_x: width - 6,
            sky,
        })
    }
    fn set(&mut self, x: i32, y: i32, t: Tile) -> &mut Self {
        self.0.set(x, y, t);
        self
    }
    fn ground(&mut self, x0: i32, x1: i32) -> &mut Self {
        for x in x0..=x1 {
            self.set(x, 0, Tile::Ground).set(x, 1, Tile::Ground);
        }
        self
    }
    fn hole(&mut self, x0: i32, x1: i32) -> &mut Self {
        for x in x0..=x1 {
            self.set(x, 0, Tile::Empty).set(x, 1, Tile::Empty);
        }
        self
    }
    fn row(&mut self, y: i32, x0: i32, x1: i32, t: Tile) -> &mut Self {
        for x in x0..=x1 {
            self.set(x, y, t);
        }
        self
    }
    fn pipe(&mut self, x: i32, h: i32) -> &mut Self {
        for y in 2..2 + h {
            let t = if y == 1 + h { Tile::PipeTop } else { Tile::Pipe };
            self.set(x, y, t).set(x + 1, y, t);
        }
        self
    }
    fn stairs_up(&mut self, x: i32, n: i32) -> &mut Self {
        for i in 0..n {
            for y in 2..3 + i {
                self.set(x + i, y, Tile::Block);
            }
        }
        self
    }
    fn stairs_down(&mut self, x: i32, n: i32) -> &mut Self {
        for i in 0..n {
            for y in 2..2 + n - i {
                self.set(x + i, y, Tile::Block);
            }
        }
        self
    }
    fn pillar(&mut self, x: i32, w: i32, h: i32) -> &mut Self {
        for xx in x..x + w {
            for y in 0..h {
                self.set(xx, y, Tile::Block);
            }
        }
        self
    }
    fn coins(&mut self, y: i32, x0: i32, x1: i32) -> &mut Self {
        for x in x0..=x1 {
            self.0.coins.push((x, y));
        }
        self
    }
    fn enemies(&mut self, list: &[(i32, i32)]) -> &mut Self {
        self.0.enemies.extend_from_slice(list);
        self
    }
    fn flag(&mut self, x: i32) -> &mut Self {
        self.0.flag_x = x;
        self
    }
    fn done(&mut self) -> LevelDef {
        self.0.clone()
    }
}

pub const LEVEL_COUNT: usize = 3;

pub fn build(index: usize) -> LevelDef {
    use Tile::*;
    match index {
        0 => Builder::new("1-1", 150, [0.42, 0.62, 0.98])
            .ground(0, 68)
            .ground(72, 88)
            .ground(92, 149)
            .set(16, 5, Question)
            .row(5, 20, 24, Brick)
            .set(21, 5, Question)
            .set(23, 5, Question)
            .set(22, 9, Question)
            .pipe(28, 2)
            .pipe(38, 3)
            .pipe(46, 4)
            .pipe(57, 4)
            .coins(7, 69, 71)
            .row(5, 77, 79, Brick)
            .set(78, 5, Question)
            .row(9, 80, 87, Brick)
            .coins(12, 81, 86)
            .row(9, 92, 94, Brick)
            .set(95, 9, Question)
            .set(95, 5, Brick)
            .row(5, 101, 102, Brick)
            .coins(4, 100, 104)
            .set(107, 5, Question)
            .set(110, 5, Question)
            .set(110, 9, Question)
            .set(113, 5, Question)
            .set(119, 5, Brick)
            .row(9, 122, 124, Brick)
            .stairs_up(134, 8)
            .enemies(&[
                (22, 2),
                (40, 2),
                (51, 2),
                (53, 2),
                (80, 2),
                (82, 2),
                (82, 10),
                (84, 10),
                (97, 2),
                (99, 2),
                (115, 2),
                (117, 2),
                (125, 2),
                (127, 2),
            ])
            .flag(146)
            .done(),
        1 => Builder::new("1-2", 160, [0.98, 0.62, 0.40])
            .ground(0, 22)
            .set(10, 5, Question)
            .row(5, 14, 18, Brick)
            .set(16, 5, Question)
            .set(16, 9, Question)
            .row(4, 23, 26, Block)
            .ground(27, 45)
            .pipe(32, 3)
            .pipe(43, 2)
            .coins(7, 47, 49)
            .ground(51, 70)
            .stairs_up(54, 4)
            .stairs_down(58, 4)
            .row(7, 64, 70, Brick)
            .set(67, 7, Question)
            .row(11, 62, 66, Brick)
            .coins(12, 62, 66)
            .row(5, 72, 74, Block)
            .ground(76, 100)
            .pipe(80, 4)
            .pipe(88, 2)
            .row(6, 90, 98, Brick)
            .set(94, 6, Question)
            .coins(10, 90, 98)
            .ground(105, 159)
            .stairs_up(110, 5)
            .hole(115, 116)
            .stairs_down(117, 5)
            .row(5, 128, 131, Brick)
            .set(129, 5, Question)
            .stairs_up(138, 8)
            .enemies(&[
                (12, 2),
                (14, 2),
                (36, 2),
                (38, 2),
                (41, 2),
                (63, 2),
                (66, 2),
                (63, 12),
                (84, 2),
                (85, 2),
                (92, 2),
                (94, 2),
                (96, 2),
                (125, 2),
                (127, 2),
                (131, 2),
                (133, 2),
            ])
            .flag(154)
            .done(),
        _ => Builder::new("1-3", 170, [0.10, 0.12, 0.30])
            .ground(0, 15)
            .pillar(19, 2, 4)
            .pillar(24, 2, 6)
            .pillar(29, 3, 4)
            .coins(8, 24, 25)
            .ground(35, 55)
            .set(44, 5, Question)
            .set(45, 5, Brick)
            .set(46, 5, Question)
            .set(47, 5, Brick)
            .coins(9, 44, 47)
            .pipe(53, 3)
            .row(4, 57, 59, Block)
            .ground(61, 80)
            .pipe(64, 2)
            .pipe(70, 4)
            .pipe(76, 3)
            .row(5, 83, 84, Block)
            .coins(8, 82, 85)
            .ground(87, 108)
            .row(5, 90, 95, Brick)
            .row(9, 92, 93, Question)
            .stairs_up(105, 4)
            .ground(113, 128)
            .stairs_down(113, 4)
            .row(5, 118, 126, Brick)
            .set(122, 5, Question)
            .coins(9, 118, 126)
            .ground(132, 169)
            .stairs_up(148, 8)
            .enemies(&[
                (10, 2),
                (40, 2),
                (42, 2),
                (48, 2),
                (50, 2),
                (67, 2),
                (68, 2),
                (73, 2),
                (74, 2),
                (93, 6),
                (98, 2),
                (100, 2),
                (102, 2),
                (120, 2),
                (122, 2),
                (124, 2),
                (136, 2),
                (139, 2),
                (142, 2),
            ])
            .flag(164)
            .done(),
    }
}
