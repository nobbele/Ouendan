use ogfx::{GraphicsContext, RenderContext, Renderable};

use crate::{game::chart, math};

use super::atlas::Atlas;

pub struct Slider {
    track: ogfx::ArcTexture,
    vertex: ogfx::Buffer,
    index: ogfx::Buffer,
    // Temp
    pub instance: ogfx::Buffer,
    #[allow(dead_code)]
    view: ogfx::Buffer,
    view_binding: wgpu::BindGroup,
}

impl Slider {
    pub fn new(
        gfx: &GraphicsContext,
        curve_type: chart::CurveType,
        initial_position: cgmath::Vector2<f32>,
        control_points: &[cgmath::Vector2<f32>],
        atlas: &Atlas<String>,
        entry: &str,
    ) -> Self {
        let spline = osu_utils::Spline::from_control(
            match curve_type {
                chart::CurveType::Perfect => osu_types::CurveType::Perfect,
                chart::CurveType::Bezier => osu_types::CurveType::Bezier,
                chart::CurveType::Linear => osu_types::CurveType::Linear,
            },
            std::iter::once(initial_position)
                .chain(control_points.iter().copied())
                .map(|p| osu_types::osu_point(p.x as _, p.y as _))
                .collect::<Vec<_>>()
                .as_slice(),
            None,
        );

        let mut builder = lyon::path::Path::builder();
        builder.begin(lyon::math::point(
            spline.spline_points[0].x,
            spline.spline_points[0].y,
        ));
        for spline_point in &spline.spline_points[1..] {
            builder.line_to(lyon::math::point(spline_point.x, spline_point.y));
        }
        builder.end(false);
        let path = builder.build();

        let mut geometry: lyon::lyon_tessellation::VertexBuffers<ogfx::Vertex, u16> =
            lyon::lyon_tessellation::VertexBuffers::new();
        let mut tessellator = lyon::lyon_tessellation::StrokeTessellator::new();
        {
            let length = lyon::algorithms::length::approximate_length(
                &path,
                lyon::lyon_tessellation::StrokeOptions::DEFAULT_TOLERANCE,
            );
            tessellator
                .tessellate_path(
                    &path,
                    &lyon::lyon_tessellation::StrokeOptions::default()
                        .with_line_width(60.0)
                        .with_start_cap(lyon::lyon_tessellation::LineCap::Round)
                        .with_end_cap(lyon::lyon_tessellation::LineCap::Round),
                    &mut lyon::lyon_tessellation::BuffersBuilder::new(
                        &mut geometry,
                        |vertex: lyon::lyon_tessellation::StrokeVertex| ogfx::Vertex {
                            position: cgmath::vec2(vertex.position().x, vertex.position().y),
                            uv: cgmath::vec2(
                                match vertex.side() {
                                    lyon::lyon_tessellation::Side::Left => 0.0,
                                    lyon::lyon_tessellation::Side::Right => 1.0,
                                },
                                math::remap(0.0, length, 0.0, 1.0, vertex.advancement()),
                            ),
                        },
                    ),
                )
                .unwrap();
        }

        let vertex_buffer =
            ogfx::Buffer::new_with_data(gfx, &geometry.vertices, wgpu::BufferUsages::VERTEX);
        let index_buffer =
            ogfx::Buffer::new_with_data(gfx, &geometry.indices, wgpu::BufferUsages::INDEX);

        let instance_buffer = ogfx::Buffer::new_with_alignable_data(
            gfx,
            &[ogfx::Transform::default().as_matrix()],
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        let view = ogfx::Transform {
            source: atlas.sub_textures[entry].cast(),
            ..Default::default()
        };
        let view_buffer = ogfx::Buffer::new_with_alignable_data(
            gfx,
            &[view.as_matrix()],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        let view_binding = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &gfx.view_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_buffer.buffer.as_entire_binding(),
            }],
            label: None,
        });

        Slider {
            track: atlas.texture.clone(),
            vertex: vertex_buffer,
            index: index_buffer,
            instance: instance_buffer,
            view: view_buffer,
            view_binding,
        }
    }
}

impl Renderable for Slider {
    fn render<'data>(
        &'data self,
        _rctx: &RenderContext<'data>,
        pass: &mut wgpu::RenderPass<'data>,
    ) {
        pass.set_bind_group(1, &self.view_binding, &[]);
        pass.set_bind_group(2, &self.track.raw.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex.buffer.slice(..));
        pass.set_vertex_buffer(1, self.instance.buffer.slice(..));
        pass.set_index_buffer(self.index.buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(
            0..self.index.element_count as _,
            0,
            0..self.instance.element_count as _,
        );
    }
}
