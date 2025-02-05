use iced::{
    widget::{self},
    Alignment, Color, Element, Length,
};

pub type Layers = Vec<Vec<f64>>;

pub fn view<Message: 'static>(layers: &Layers) -> Element<Message> {
    let columns = layers.iter().map(|layer| {
        let values = layer.iter().map(|value| {
            let text = widget::text({
                let mut s = value.to_string();
                s.truncate(10);
                s
            });

            let color = Color::from_rgb(
                value.max(0.0).min(1.0) as f32,
                0.0,
                value.min(0.0).max(-1.0).abs() as f32,
            );

            widget::container(text)
                .style(move |_| widget::container::background(color))
                .width(100)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into()
        });

        widget::column(values).height(Length::Fill).into()
    });

    widget::container(
        widget::row(columns)
            .spacing(15)
            .align_y(Alignment::Center)
            .height(Length::Fill),
    )
    .style(widget::container::rounded_box)
    .into()
}
