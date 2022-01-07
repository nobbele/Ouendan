#[derive(Debug)]
pub struct Pack {
    pub output: Vec<(mint::Point2<u32>, usize)>,
    pub dimensions: mint::Vector2<u32>,
}

pub struct PackSolver<'a, T> {
    rects: &'a [mint::Vector2<T>],
    shortest: Vec<usize>,
    thinnest: Vec<usize>,
    //squarest: Vec<usize>,
    pack: Pack,
}

impl<'a> PackSolver<'a, u32> {
    pub fn new(rects: &'a [mint::Vector2<u32>]) -> Self {
        let mut shortest = rects.iter().enumerate().collect::<Vec<_>>();
        shortest.sort_unstable_by_key(|tup| tup.1.y);
        let shortest = shortest.into_iter().map(|tup| tup.0).collect::<Vec<_>>();

        let mut thinnest = rects.iter().enumerate().collect::<Vec<_>>();
        thinnest.sort_unstable_by_key(|tup| tup.1.x);
        let thinnest = thinnest.into_iter().map(|tup| tup.0).collect::<Vec<_>>();

        /*let mut squarest = rects.iter().enumerate().collect::<Vec<_>>();
        squarest.sort_unstable_by_key(|tup| tup.1.x.max(tup.1.y) / tup.1.x.min(tup.1.y));
        let squarest = squarest.into_iter().map(|tup| tup.0).collect::<Vec<_>>();*/

        PackSolver {
            rects,
            shortest,
            thinnest,
            //squarest,
            pack: Pack {
                output: Vec::new(),
                dimensions: mint::Vector2 { x: 0, y: 0 },
            },
        }
    }

    fn move_to_output(&mut self, position: mint::Point2<u32>, index: usize) {
        self.shortest
            .remove(self.shortest.iter().position(|el| *el == index).unwrap());
        self.thinnest
            .remove(self.thinnest.iter().position(|el| *el == index).unwrap());
        /*self.squarest
        .remove(self.squarest.iter().position(|el| *el == index).unwrap());*/
        self.pack.output.push((position, index));
        self.pack.dimensions.x = self.pack.dimensions.x.max(position.x + self.rects[index].x);
        self.pack.dimensions.y = self.pack.dimensions.y.max(position.y + self.rects[index].y);
    }

    fn fully_solved(&mut self) -> bool {
        self.shortest.is_empty() && self.thinnest.is_empty()
    }

    fn find_tallest(&mut self) -> usize {
        *self.shortest.last().unwrap()
        //*self.squarest.first().unwrap()
    }

    pub fn solve(mut self) -> Pack {
        let mut position = mint::Point2 { x: 0, y: 0 };

        let mut tallest_in_row = 0;
        while !self.fully_solved() {
            assert_eq!(self.shortest.len(), self.thinnest.len());

            let tallest = self.find_tallest();
            self.move_to_output(position, tallest);
            tallest_in_row = tallest_in_row.max(self.rects[tallest].y);

            /*println!(
                "Added initial rectangle {:?} at {:?}",
                self.rects[tallest], position
            );*/

            position.x += self.rects[tallest].x;

            let mut current_h = 0;
            let extended = self
                .shortest
                .iter()
                .copied()
                .take_while(|&el| {
                    let rect = self.rects[el];
                    current_h += rect.y;
                    current_h <= self.rects[tallest].y
                })
                .collect::<Vec<_>>();
            //dbg!(&extended);
            let mut current_y = position.y;
            let mut width = 0;
            for &extend_rect in &extended {
                /*println!(
                    "pos.x = {}, current_y = {}, ({:?})",
                    position.x, current_y, self.rects[extend_rect]
                );*/

                self.move_to_output(
                    mint::Point2 {
                        x: position.x,
                        y: current_y,
                    },
                    extend_rect,
                );

                let mut current_w = 0;
                let rowed = self
                    .thinnest
                    .iter()
                    .copied()
                    .filter(|el| !extended.contains(el))
                    .take_while(|&el| {
                        let rect = self.rects[el];
                        current_w += rect.x;
                        current_w <= self.rects[extend_rect].x && rect.y < self.rects[extend_rect].y
                    })
                    .collect::<Vec<_>>();

                let mut current_x = position.x + self.rects[extend_rect].x;
                for rowed_rect in rowed {
                    /*println!(
                        "current_x = {}, current_y = {}, ({:?})",
                        current_x, current_y, self.rects[rowed_rect]
                    );*/

                    self.move_to_output(
                        mint::Point2 {
                            x: current_x,
                            y: current_y,
                        },
                        rowed_rect,
                    );

                    current_x += self.rects[rowed_rect].x
                }
                width = width.max(current_x);

                current_y += self.rects[extend_rect].y;
            }

            //dbg!(self.pack.dimensions);

            if self.pack.dimensions.y >= self.pack.dimensions.x {
                let next_x = width;
                //println!("Setting next position.x to {}", next_x);
                position.x = next_x;
            } else {
                let next_y = position.y + tallest_in_row;
                //println!("Setting next position.y to {}", next_y);
                position.y = next_y;
                position.x = 0;
                tallest_in_row = 0;
            }
        }

        self.pack
    }
}
