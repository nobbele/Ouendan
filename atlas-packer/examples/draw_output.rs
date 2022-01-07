use atlas_packer::PackSolver;
use draw::{render, Canvas, Drawing, Shape, Style, SvgRenderer};

pub fn main() {
    let rects = [
        (mint::Vector2 { x: 10, y: 10 }, draw::Color::random()),
        (mint::Vector2 { x: 15, y: 10 }, draw::Color::random()),
        (mint::Vector2 { x: 15, y: 5 }, draw::Color::random()),
        (mint::Vector2 { x: 5, y: 15 }, draw::Color::random()),
        (mint::Vector2 { x: 15, y: 12 }, draw::Color::random()),
        (mint::Vector2 { x: 5, y: 3 }, draw::Color::random()),
        (mint::Vector2 { x: 5, y: 5 }, draw::Color::random()),
        (mint::Vector2 { x: 10, y: 7 }, draw::Color::random()),
        (mint::Vector2 { x: 10, y: 19 }, draw::Color::random()),
        (mint::Vector2 { x: 26, y: 4 }, draw::Color::random()),
        (mint::Vector2 { x: 5, y: 4 }, draw::Color::random()),
        (mint::Vector2 { x: 7, y: 4 }, draw::Color::random()),
        (mint::Vector2 { x: 19, y: 19 }, draw::Color::random()),
        (mint::Vector2 { x: 4, y: 1 }, draw::Color::random()),
        (mint::Vector2 { x: 1, y: 2 }, draw::Color::random()),
        (mint::Vector2 { x: 3, y: 2 }, draw::Color::random()),
        (mint::Vector2 { x: 5, y: 4 }, draw::Color::random()),
    ];
    let solve_rects = rects.iter().map(|r| r.0).collect::<Vec<_>>();
    let solver = PackSolver::new(&solve_rects);
    let pack = solver.solve();

    let mut canvas = Canvas::new(pack.dimensions.x, pack.dimensions.y);
    for (pos, idx) in pack.output {
        let (rect, color) = rects[idx];
        canvas.display_list.add(
            Drawing::new()
                .with_shape(Shape::Rectangle {
                    width: rect.x,
                    height: rect.y,
                })
                .with_xy(pos.x as f32, pos.y as f32)
                .with_style(Style::filled(color)),
        );
    }

    render::save(&canvas, "output.svg", SvgRenderer::new()).expect("Failed to save")
}
