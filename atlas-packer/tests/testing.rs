use atlas_packer::PackSolver;

fn verify_rects(rects: &[mint::Vector2<u32>]) {
    let solver = PackSolver::new(&rects);
    let output = solver.solve().output;

    println!(
        "{:#?}",
        output
            .iter()
            .map(|(pos, idx)| (pos, rects[*idx]))
            .collect::<Vec<_>>()
    );

    assert_eq!(output.len(), rects.len());
    for &(pos, idx) in &output {
        let rect = rects[idx];
        println!(
            "Checking collisions for ({}x{}) at ({},{})",
            rect.x, rect.y, pos.x, pos.y
        );
        for &(other_pos, other_idx) in &output {
            if other_idx == idx {
                continue;
            }

            let other_rect = rects[other_idx];

            println!(
                "Checking collisions with ({}x{}) at ({},{})",
                other_rect.x, other_rect.y, other_pos.x, other_pos.y
            );

            assert!(
                !(pos.x >= other_pos.x
                    && pos.x <= other_pos.x + other_rect.x - 1
                    && pos.y >= other_pos.y
                    && pos.y <= other_pos.y + other_rect.y - 1),
                "first corner ({:?}) is inside rectangle of size ({:?}) at ({:?})",
                pos,
                other_rect,
                other_pos
            );
            assert!(
                !(pos.x + rect.x - 1 >= other_pos.x
                    && pos.x + rect.x - 1 <= other_pos.x + other_rect.x - 1
                    && pos.y + rect.y - 1 >= other_pos.y
                    && pos.y + rect.y - 1 <= other_pos.y + other_rect.y - 1),
                "second corner ({:?}) is inside rectangle of size ({:?}) at ({:?})",
                mint::Point2 {
                    x: pos.x + rect.x,
                    y: pos.y + rect.y,
                },
                other_rect,
                other_pos
            );
        }
    }
}

#[test]
fn basic() {
    let mut rects = Vec::new();
    rects.push(mint::Vector2 { x: 10, y: 10 });
    rects.push(mint::Vector2 { x: 12, y: 10 });
    rects.push(mint::Vector2 { x: 10, y: 15 });
    rects.push(mint::Vector2 { x: 20, y: 5 });

    verify_rects(&rects);
}

#[test]
fn fuzzer() {
    use rand::RngCore;
    const MAX_RECT_COUNT: u32 = 100;

    let seed: u64 = rand::random();
    println!("Using seed = {}", seed);
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);

    let mut rects = Vec::new();
    for _ in 0..MAX_RECT_COUNT {
        rects.push(mint::Vector2 {
            x: rng.next_u32() / MAX_RECT_COUNT,
            y: rng.next_u32() / MAX_RECT_COUNT,
        });
        verify_rects(&rects);
    }
}
