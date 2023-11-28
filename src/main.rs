use std::path::Path;
use std::time::Instant;
use nannou::wgpu::{Texture, ToTextureView, WithDeviceQueuePair};
use nannou::prelude::*;
use nannou::winit::event::VirtualKeyCode;

#[derive(Debug)]
struct Model {
    angle: f32,
    speed:  (f32, f32),
    position: (f32, f32),
    bullets: Vec<Bullet>,
    ship: Texture,
    last_fired: Instant,
}

#[derive(Debug)]
struct Bullet {
    speed:  (f32, f32),
    position: (f32, f32),
    angle: f32,
}

fn main() {
    nannou::app(model).update(update).run();
}

// in per-seconds
const ROTATE_SPEED: f32 = 0.5 * 2.0 * PI;
const THRUST: f32 = 10.0;
const DRAG: f32 = 1.0;

const FIRE_COOLDOWN: u128 = 100;




#[derive(Debug)]
enum RotationControl {
    CW, CCW, None
}

fn model(app: &App) -> Model {
    let _window: WindowId = app.new_window().view(view).build().unwrap();
    app.window(_window).unwrap().set_cursor_visible(false);
    let ship = Texture::from_path(app, Path::new("ship.png")).unwrap();


    Model {
        angle: 0.0,
        speed: (0.0, 0.0),
        position: (0.0, 0.0),
        bullets: Vec::new(),
        ship,
        last_fired: Instant::now()
    }
}

fn add_at_an_angle(component: &mut (f32, f32), scalar: f32, angle: f32) {
    component.0 = component.0 + scalar * (angle).cos();
    component.1 = component.1 + scalar * (angle).sin();
}

fn scalarize(c: (f32, f32)) -> f32 {
    ((c.0.pow(2) + c.1.pow(2)) as f32).sqrt()
}

fn update(app: &App, model: &mut Model, update: Update) {
    let secs_elapsed: f32 = update.since_last.as_millis() as f32 / 1000.0;

    let rot = if app.keys.down.contains(&VirtualKeyCode::A) {
        RotationControl::CCW
    } else if app.keys.down.contains(&VirtualKeyCode::D) {
        RotationControl::CW
    } else {
        RotationControl::None
    };


    let rotate_speed = ROTATE_SPEED * secs_elapsed;

    let angle_dir = match rot {
        RotationControl::CCW => 1,
        RotationControl::CW => -1,
        RotationControl::None => 0
    };

    if angle_dir != 0 {
        model.angle = angle_dir as f32 * rotate_speed + model.angle
    }

    if app.keys.down.contains(&VirtualKeyCode::W) {
        //go
        add_at_an_angle(&mut model.speed, THRUST * secs_elapsed, model.angle)
    } else if app.keys.down.contains(&VirtualKeyCode::S) {
        // stop
        add_at_an_angle(&mut model.speed, -1.0 * THRUST * secs_elapsed, model.angle)
    } else if scalarize(model.speed).abs() < 0.001 {
        model.speed.0 = 0.0;
        model.speed.1 = 0.0;
    } else {
        // draggg
        model.speed.0 *= 1.0 - (DRAG * secs_elapsed);
        model.speed.1 *= 1.0 - (DRAG * secs_elapsed);
    }

    let now = Instant::now();
    let last_fired = now-model.last_fired;
    if app.keys.down.contains(&VirtualKeyCode::Space) && last_fired.as_millis() > FIRE_COOLDOWN {
        let mut speed = model.speed.clone();
        add_at_an_angle(&mut speed, 20.0, model.angle);
        model.last_fired = Instant::now();
        model.bullets.push(Bullet {
            speed,
            position: model.position.clone(),
            angle: model.angle
        })
    }

    // println!("angle: {:?} cos: {:?}, sin: {:?}", model.angle, model.angle.cos(), model.angle.sin());
    if scalarize(model.speed) != 0.0 {
        model.position.0 += model.speed.0;
        model.position.1 += model.speed.1;
    }

    let (width, height) = app.main_window().inner_size_pixels();
    // retina
    let width = width/2;
    let height = height/2;

    let half_width = width as f32 / 2.0;
    let half_height = height as f32 / 2.0;

    if model.position.0 < -half_width {
        model.position.0 += width as f32;
    } else if model.position.0 > half_width {
        model.position.0 -= width as f32;
    }

    if model.position.1 < -half_height {
        model.position.1 += height as f32;
    } else if model.position.1 > half_height {
        model.position.1 -= height as f32;
    }

    let mut i = 0;
    loop {
        if i == model.bullets.len() {
            break;
        }
        let b = model.bullets.get(i).unwrap();
        if b.position.0 < -half_width ||
            model.position.0 > half_width ||
            model.position.1 < -half_height ||
            model.position.1 > half_height
        {
            // we out of bounds
            model.bullets.remove(i);
        } else {
            i += 1;
        }
    }

    model.bullets.iter_mut().for_each(|b| {
        b.position.0 += b.speed.0;
        b.position.1 += b.speed.1;
    })



    // println!("{:?} on a {:?}", model.position, (width, height));

}

fn draw_ship(draw: &Draw, model: &Model) {
    let img_size = model.ship.size();

    // draw.polygon().points(z).color(WHITE)
    draw.texture(&model.ship)
        .roll(model.angle - PI/2.0)
        .x(model.position.0)
        .y(model.position.1)
        .width(img_size[0] as f32 / 2.0)
        .height(img_size[1] as f32 / 2.0);

}

fn draw_bullets (draw: &Draw, model: &Model) {
    model.bullets.iter().for_each(|b| {
        let mut  left_pos = b.position.clone();
        let mut right_pos = b.position.clone();
        add_at_an_angle(&mut left_pos, 10.0, b.angle + PI/2.0);
        add_at_an_angle(&mut right_pos, 10.0, b.angle - PI/2.0);

        draw.rect()
            .width(20.0)
            .height(1.0)
            .x(left_pos.0)
            .y(left_pos.1)
            .rotate(b.angle)
            .color(GREENYELLOW);

        draw.rect()
            .width(20.0)
            .height(1.0)
            .x(right_pos.0)
            .y(right_pos.1)
            .rotate(b.angle)
            .color(GREENYELLOW);
    });

}


fn draw_stars (draw: &Draw, model: &Model) {

}

fn view(app: &App, model: &Model, frame: Frame) {
    // println!("{:?}", model);
    let draw = app.draw();


    draw.background().color(BLACK);
    draw_stars(&draw, model);

    draw_bullets(&draw, model);
    draw_ship(&draw, model);


    draw.to_frame(app, &frame).unwrap();
}