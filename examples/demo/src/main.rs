use lvgl;
use lvgl::Object;
use lvgl_sys;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use std::panic;
use std::time::Duration;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let mut framebuffer = [[Color::from((0, 0, 0)); lvgl_sys::LV_VER_RES_MAX as usize];
        lvgl_sys::LV_HOR_RES_MAX as usize];

    let window = video_subsystem
        .window(
            "TFT Display: Demo",
            lvgl_sys::LV_HOR_RES_MAX,
            lvgl_sys::LV_VER_RES_MAX,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    unsafe {
        lvgl_sys::lv_init();
    }

    // Implement and register a function which can copy a pixel array to an area of your display:
    let mut display_driver = DisplayDriver::new(move |points, colors| {
        for (i, point) in points.into_iter().enumerate() {
            let color = &mut framebuffer[point.x() as usize][point.y() as usize];
            *color = colors[i].clone();
        }
        canvas.clear();
        for (x, line) in framebuffer.iter().enumerate() {
            for (y, color) in line.iter().enumerate() {
                canvas.set_draw_color(color.clone());
                canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
            }
        }
        canvas.present();
    });

    // Create screen and widgets
    let mut screen = display_driver.get_active_screen();

    // let mut button = lvgl::Button::new(&mut screen);
    // button.set_pos(50, 50);
    // button.set_size(100, 50);
    //
    // let mut label = lvgl::Label::new(&mut button);
    // label.set_text("Hello Mundo!\0");

    let font_roboto_28 = unsafe { &lvgl_sys::lv_font_roboto_28 };
    let font_noto_sans_numeric_28 = unsafe { &noto_sans_numeric_80 };

    let mut screen_style = lvgl::Style::new();
    screen_style.set_body_main_color(lvgl::Color::from_rgb((0, 0, 0)));
    screen_style.set_body_grad_color(lvgl::Color::from_rgb((0, 0, 0)));
    screen.set_style(&mut screen_style);

    let mut time = lvgl::Label::new(&mut screen);
    let mut style_time = lvgl::Style::new();
    style_time.set_text_font(font_noto_sans_numeric_28);
    style_time.set_text_color(lvgl::Color::from_rgb((255, 255, 255)));
    time.set_style(&mut style_time);
    time.set_align(&mut screen, lvgl::Align::InLeftMid, 20, 0);
    time.set_text("20:46\0");
    time.set_width(240);
    time.set_height(240);

    let mut bt = lvgl::Label::new(&mut screen);
    let mut style_bt = lvgl::Style::new();
    style_bt.set_text_font(font_roboto_28);
    let mut style_power = style_bt.clone();
    bt.set_style(&mut style_bt);
    bt.set_width(50);
    bt.set_height(80);
    bt.set_recolor(true);
    bt.set_text("#5794f2 \u{F293}#\0");
    bt.set_label_align(lvgl::LabelAlign::Left);
    bt.set_align(&mut screen, lvgl::Align::InTopLeft, 0, 0);

    let mut power = lvgl::Label::new(&mut screen);
    power.set_style(&mut style_power);
    power.set_recolor(true);
    power.set_width(80);
    power.set_height(20);
    power.set_text("#fade2a 20%#\0");
    power.set_label_align(lvgl::LabelAlign::Right);
    power.set_align(&mut screen, lvgl::Align::InTopRight, 0, 0);

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                _ => {}
            }
        }

        ::std::thread::sleep(Duration::from_millis(
            lvgl_sys::LV_DISP_DEF_REFR_PERIOD as u64,
        ));

        unsafe {
            lvgl_sys::lv_task_handler();
        }
    }

    Ok(())
}

// Reference to native font for LittlevGL, defined in the file: "fonts_noto_sans_numeric_80.c"
extern "C" {
    pub static mut noto_sans_numeric_80: lvgl_sys::lv_font_t;
}

#[allow(dead_code)]
struct DisplayDriver<F>
where
    F: FnMut(Vec<Point>, Vec<Color>),
{
    pub raw: lvgl_sys::lv_disp_drv_t,
    callback: F,
    display_buffer: MaybeUninit<lvgl_sys::lv_disp_buf_t>,
    refresh_buffer: [MaybeUninit<lvgl_sys::lv_color_t>; lvgl_sys::LV_HOR_RES_MAX as usize * 10],
}

impl<F> DisplayDriver<F>
where
    F: FnMut(Vec<Point>, Vec<Color>),
{
    fn new(mut callback: F) -> Self {
        // Create a display buffer for LittlevGL
        let mut display_buffer = MaybeUninit::<lvgl_sys::lv_disp_buf_t>::uninit();
        let mut refresh_buffer: [MaybeUninit<lvgl_sys::lv_color_t>;
            lvgl_sys::LV_HOR_RES_MAX as usize * 10] =
            unsafe { MaybeUninit::uninit().assume_init() }; /*Declare a buffer for 10 lines*/
        unsafe {
            // Initialize the display buffer
            lvgl_sys::lv_disp_buf_init(
                display_buffer.as_mut_ptr(),
                refresh_buffer.as_mut_ptr() as *mut c_void,
                std::ptr::null_mut(),
                (lvgl_sys::LV_HOR_RES_MAX * 10) as u32,
            );
        }
        let mut disp_drv = unsafe {
            let mut disp_drv = MaybeUninit::<lvgl_sys::lv_disp_drv_t>::uninit().assume_init(); /*Descriptor of a display driver*/
            lvgl_sys::lv_disp_drv_init(&mut disp_drv); // Basic initialization
            disp_drv.flush_cb = Some(display_callback_wrapper::<F>); // Set your driver function
            disp_drv.user_data = &mut callback as *mut _ as *mut c_void;
            disp_drv
        };
        disp_drv.buffer = display_buffer.as_mut_ptr(); // Assign the buffer to the display
        unsafe {
            lvgl_sys::lv_disp_drv_register(&mut disp_drv); // Finally register the driver
        }
        Self {
            raw: disp_drv,
            callback,
            display_buffer,
            refresh_buffer,
        }
    }

    fn get_active_screen(&mut self) -> lvgl::ObjectX<'static> {
        lvgl::display::get_active_screen()
    }
}

unsafe extern "C" fn display_callback_wrapper<F>(
    disp_drv: *mut lvgl_sys::lv_disp_drv_t,
    area: *const lvgl_sys::lv_area_t,
    color_p: *mut lvgl_sys::lv_color_t,
) where
    F: FnMut(Vec<Point>, Vec<Color>),
{
    // We need to make sure panics can't escape across the FFI boundary.
    let _ = panic::catch_unwind(|| {
        let mut i = 0;
        let disp = *disp_drv;

        // Rust code closure reference
        let closure = &mut *(disp.user_data as *mut F);

        let mut points = vec![];
        let mut colors = vec![];

        for y in (*area).y1..=(*area).y2 {
            for x in (*area).x1..=(*area).x2 {
                // Convert point to paint to a high-level Rust repr
                points.push(Point::new(x as i32, y as i32));

                // Convert C color representation to high-level Rust
                let raw_color = *color_p.add(i);
                let color = Color::from((
                    raw_color.ch.red,
                    raw_color.ch.green,
                    raw_color.ch.blue,
                    raw_color.ch.alpha,
                ));
                colors.push(color);

                i = i + 1;
            }
        }

        // Callback the Rust closure to flush the new points to the screen
        closure(points, colors);

        // Indicate to LittlevGL that you are ready with the flushing
        lvgl_sys::lv_disp_flush_ready(disp_drv);
    });
}
