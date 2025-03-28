mod helpers;
use helpers::*;
use vizia::prelude::*;
#[derive(Lens)]
pub struct AppData {
    text: String,
    value: f32,
    checked: bool,
}
#[derive(Debug)]
pub enum AppEvent {
    Toggle,
}
impl Model for AppData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            AppEvent::Toggle => {
                self.checked ^= true;
            }
        });
    }
}
fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        AppData {
            text: String::from("As well as model data which implements ToString:"),
            value: std::f32::consts::PI,
            checked: false,
        }
        .build(cx);

        ExamplePage::vertical(cx, |cx| {
            Label::new(cx, "A label can display a static string of unicode 😂")
                .background_color(Color::gray())
                .padding(Pixels(20.0));

            Label::new(cx, AppData::text);

            Label::new(cx, AppData::value);

            Label::new(cx, "Text which is too long for the label will be wrapped.")
                .text_wrap(true)
                .width(Pixels(200.0));

            Label::new(cx, "Unless text wrapping is disabled.")
                .width(Pixels(200.0))
                .text_wrap(false)
                .font_slant(FontSlant::Italic);

            HStack::new(cx, |cx| {
                Checkbox::new(cx, AppData::checked)
                    .on_toggle(|cx| cx.emit(AppEvent::Toggle))
                    .id("checkbox_1")
                    .top(Units::Pixels(2.0))
                    .bottom(Units::Pixels(2.0));

                Label::new(cx, "A label that is describing a form element also acts as a trigger")
                    .describing("checkbox_1");
            })
            .width(Auto)
            .height(Auto)
            .alignment(Alignment::Center)
            .horizontal_gap(Pixels(8.0));
        });
    })
    .title("Label")
    .run()
}
