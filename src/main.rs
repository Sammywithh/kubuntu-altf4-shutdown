mod actions;
mod style;

fn main() {
    let _palette = style::ThemePalette::detect();
    let _count = actions::ACTIONS.len();
}
