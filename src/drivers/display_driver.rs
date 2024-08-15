use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

pub fn update_display(canvas: &mut Canvas<Window>, buffer: &[[bool; 64]; 32]) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.set_draw_color(Color::RGB(255, 255, 255));

    for y in 0..32 {
        for x in 0..64 {
            if buffer[y][x] {
                let rect = Rect::new(x as i32 * 10, y as i32 * 10, 10, 10);
                canvas.fill_rect(rect)?;
            }
        }
    }

    canvas.present();
    Ok(())
}
