use std::sync::Arc;

use crate::game::GameContext;

pub struct GameUI {
    ctx: Arc<GameContext>,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl GameUI {
    pub fn new(ctx: Arc<GameContext>) -> GameUI {
        GameUI { ctx }
    }
}

impl iced_winit::Program for GameUI {
    type Renderer = iced_wgpu::Renderer;
    type Message = Message;

    fn update(&mut self, _message: Self::Message) -> iced::Command<Self::Message> {
        //match message {}

        iced::Command::none()
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        let chart_progress = self.ctx.chart_progress();
        iced::Column::new()
            .height(iced::Length::Fill)
            .push(
                iced::Row::new()
                    .width(iced::Length::Fill)
                    .push(iced::Text::new("A"))
                    .push(iced::Space::with_width(iced::Length::Fill))
                    .push(iced::Text::new("B")),
            )
            .push(iced::Space::with_height(iced::Length::Fill))
            .push(
                iced::Row::new()
                    .width(iced::Length::Fill)
                    .align_items(iced::Alignment::End)
                    .push(
                        iced::Text::new(format!(
                            "{}x",
                            chart_progress.map(|p| p.combo).unwrap_or(0)
                        ))
                        .size(48),
                    )
                    .push(iced::Space::with_width(iced::Length::Fill))
                    .push(
                        iced::ProgressBar::new(
                            0.0..=1.0,
                            chart_progress.map(|p| p.progress).unwrap_or(0.0),
                        )
                        .width(iced::Length::FillPortion(2)),
                    )
                    .push(iced::Space::with_width(iced::Length::Fill))
                    .push(iced::Text::new("D")),
            )
            .into()
    }
}
