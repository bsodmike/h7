use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Dimensions, Point},
    mono_font::{MonoFont, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
    text::TextStyle,
    transform::Transform,
    Pixel,
};

#[allow(dead_code)]
pub enum XPos {
    Absolute(i32),
    Left(i32),
    Center(i32),
    Right(i32),
}

#[allow(dead_code)]
pub enum YPos {
    Absolute(i32),
    Top(i32),
    Center(i32),
    Bottom(i32),
}

#[allow(dead_code)]
impl XPos {
    pub fn to_absolute(&self, display_width: u32) -> i32 {
        match self {
            Self::Absolute(n) => *n,
            Self::Left(n) => *n,
            Self::Center(n) => (display_width as i32 / 2) + n,
            Self::Right(n) => display_width as i32 - n,
        }
    }

    pub fn to_absolute_with_width(&self, display_width: u32, width: u32) -> i32 {
        match self {
            Self::Absolute(n) => *n,
            Self::Left(n) => *n,
            Self::Center(n) => (display_width as i32 / 2) - (width as i32 / 2) + n,
            Self::Right(n) => display_width as i32 - width as i32 - *n,
        }
    }
}

#[allow(dead_code)]
impl YPos {
    pub fn to_absolute(&self, display_height: u32) -> i32 {
        match self {
            Self::Absolute(n) => *n,
            Self::Top(n) => *n,
            Self::Center(n) => (display_height as i32 / 2) + n,
            Self::Bottom(n) => display_height as i32 - n,
        }
    }

    pub fn to_absolute_with_height(&self, display_height: u32, height: u32) -> i32 {
        match self {
            Self::Absolute(n) => *n,
            Self::Top(n) => *n,
            Self::Center(n) => (display_height as i32 / 2) - (height as i32 / 2) + n,
            Self::Bottom(n) => display_height as i32 - height as i32 - n,
        }
    }
}

pub struct Display<D> {
    pub display: D,
}

impl<D: DrawTarget + Dimensions> Display<D> {
    pub fn new(display: D) -> Self {
        Self { display }
    }

    pub fn clear(
        &mut self,
        color: <D as embedded_graphics::draw_target::DrawTarget>::Color,
    ) -> Result<(), <D as DrawTarget>::Error> {
        self.display.clear(color)
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        font: &MonoFont,
        color: <D as embedded_graphics::draw_target::DrawTarget>::Color,
        x: XPos,
        y: YPos,
    ) -> Result<Size, <D as DrawTarget>::Error> {
        let rt = Text::new(text, Point::new(0, 0), MonoTextStyle::new(font, color));

        let rt_size = rt.bounding_box().size;
        let display = self.display.bounding_box().size;
        let (lx, ly) = (
            x.to_absolute_with_width(display.width, rt_size.width),
            y.to_absolute_with_height(display.height, rt_size.height),
        );
        rt.translate(Point::new(lx, ly))
            .draw(&mut self.display)
            .map(|_| rt_size)
    }

    pub fn draw_text_fontdue(
        &mut self,
        text: &[(&str, &fontdue::Font)],
        size: f32,
        color: <D as embedded_graphics::draw_target::DrawTarget>::Color,
        threshold: u8,
        x: XPos,
        y: YPos,
    ) -> Result<Size, <D as DrawTarget>::Error> {
        let mut layout =
            fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown);
        for i in 0..text.len() {
            layout.append(
                text.iter()
                    .map(|v| v.1)
                    .collect::<Vec<&fontdue::Font>>()
                    .as_ref(),
                &fontdue::layout::TextStyle::new(text[i].0, size, i),
            );
        }

        let display_size = self.display.bounding_box().size;
        let mut lsize = Size::new(0, layout.height() as u32);
        let glyphs = layout.glyphs();
        let rasterized_glyphs = glyphs
            .iter()
            .map(|g| text[g.font_index].1.rasterize(g.parent, size))
            .collect::<Vec<_>>();
        lsize.width = rasterized_glyphs
            .iter()
            .fold(0, |acc, rg| acc + rg.0.advance_width as u32);

        for (n, (metrics, bitmap)) in rasterized_glyphs.iter().enumerate() {
            for i in 0..metrics.width {
                for j in 0..metrics.height {
                    if bitmap[(j * metrics.width) + i] > threshold {
                        self.draw_pixel(
                            color,
                            XPos::Absolute(
                                glyphs[n].x as i32
                                    + i as i32
                                    + x.to_absolute_with_width(display_size.width, lsize.width),
                            ),
                            YPos::Absolute(
                                glyphs[n].y as i32
                                    + j as i32
                                    + y.to_absolute_with_height(display_size.height, lsize.height),
                            ),
                        )?;
                    }
                }
            }
        }

        Ok(lsize)
    }

    pub fn draw_pixel(
        &mut self,
        color: <D as embedded_graphics::draw_target::DrawTarget>::Color,
        x: XPos,
        y: YPos,
    ) -> Result<Size, <D as DrawTarget>::Error> {
        let display_size = self.display.bounding_box().size;
        let (lx, ly) = (
            x.to_absolute_with_width(display_size.width, 1),
            y.to_absolute_with_height(display_size.height, 1),
        );
        Pixel(Point::new(lx, ly), color)
            .draw(&mut self.display)
            .map(|_| Size::new(1, 1))
    }

    pub fn draw_glyph(
        &mut self,
        font: &fontdue::Font,
        glyph: char,
        size: f32,
        color: <D as embedded_graphics::draw_target::DrawTarget>::Color,
        threshold: u8,
        x: XPos,
        y: YPos,
    ) -> Result<fontdue::Metrics, <D as DrawTarget>::Error> {
        let display_size = self.display.bounding_box().size;
        let (metrics, bitmap) = font.rasterize(glyph, size);
        for i in 0..metrics.width {
            for j in 0..metrics.height {
                if bitmap[(j * metrics.width) + i] > 127 {
                    self.draw_pixel(
                        color,
                        XPos::Absolute(
                            i as i32
                                + x.to_absolute_with_width(
                                    display_size.width,
                                    metrics.advance_width as u32,
                                )
                                - metrics.xmin,
                        ),
                        YPos::Absolute(
                            j as i32
                                + y.to_absolute_with_height(
                                    display_size.height,
                                    metrics.advance_height as u32,
                                )
                                - metrics.ymin,
                        ),
                    )?;
                }
            }
        }
        Ok(metrics)
    }
}
