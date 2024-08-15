use crate::config::Config;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub fn update_display(
    canvas: &mut Canvas<Window>,
    buffer: &[[bool; 64]; 32],
    config: &Config,
) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.set_draw_color(Color::RGB(255, 255, 255));

    let scale = config.scale_factor;
    for y in 0..32 {
        for x in 0..64 {
            if buffer[y][x] {
                let rect = Rect::new(
                    x as i32 * scale as i32,
                    y as i32 * scale as i32,
                    scale,
                    scale,
                );
                canvas.fill_rect(rect)?;
            }
        }
    }

    canvas.present();
    Ok(())
}
