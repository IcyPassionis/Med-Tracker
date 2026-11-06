macro_rules! button_with_icon_text{
($label:expr, $icon_path:expr) => {
        container(
            row![
                Image::new($icon_path)
                    .content_fit(ContentFit::Cover)
                    .width(30)
                    .height(30),
                text($label).size($crate::ui::scale::text_px(14.0)).align_y(alignment::Vertical::Bottom)
            ].spacing(10).align_y(alignment::Vertical::Center)
        ).align_y(alignment::Vertical::Center)
    };
}
macro_rules! button_with_icon{
($icon_path:expr, $size:expr, $spacing:expr) => {
        container(
            row![
                Image::new($icon_path)
                    .content_fit(ContentFit::Cover)
                    .width($size)
                    .height($size),
            ].spacing($spacing).align_y(alignment::Vertical::Center)
        ).align_y(alignment::Vertical::Center)
    };
}
pub(crate) use button_with_icon;
pub(crate) use button_with_icon_text;
