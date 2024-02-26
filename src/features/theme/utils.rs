use bevy::render::color::Color;

pub struct ColorTheme {
    pub l50: Color,
    pub l100: Color,
    pub l200: Color,
    pub l300: Color,
    pub l400: Color,
    pub l500: Color,
    pub l600: Color,
    pub l700: Color,
    pub l800: Color,
    pub l900: Color,
}

impl ColorTheme {
    pub const fn from_array(value: [(f32, f32, f32); 10]) -> Self {
        Self {
            l50: Color::rgb(value[0].0, value[0].1, value[0].2),
            l100: Color::rgb(value[1].0, value[1].1, value[1].2),
            l200: Color::rgb(value[2].0, value[2].1, value[2].2),
            l300: Color::rgb(value[3].0, value[3].1, value[3].2),
            l400: Color::rgb(value[4].0, value[4].1, value[4].2),
            l500: Color::rgb(value[5].0, value[5].1, value[5].2),
            l600: Color::rgb(value[6].0, value[6].1, value[6].2),
            l700: Color::rgb(value[7].0, value[7].1, value[7].2),
            l800: Color::rgb(value[8].0, value[8].1, value[8].2),
            l900: Color::rgb(value[9].0, value[9].1, value[9].2),
        }
    }
}
pub struct ColorThemeIterator {
    last: usize,
    theme: ColorTheme,
}
