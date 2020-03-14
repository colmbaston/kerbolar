mod celestial;
use celestial::*;

use std::cmp::Ordering;
use nannou::prelude::*;
use winit::event::{ DeviceEvent, VirtualKeyCode, ElementState };

const STEPS_PER_FRAME  : u64 = 60;
const SECONDS_PER_STEP : f64 = 1.0;
const DAYS_PER_FRAME   : f64 = STEPS_PER_FRAME as f64 * SECONDS_PER_STEP / 86_400.0;
const ZOOM             : f64 = 1.1;

fn main()
{
    nannou::app(model).event(event).simple_window(view).run();
}

struct Model
{
    frame:      usize,
    scale:      f64,
    celestials: Vec<Celestial>,
    focus:      usize,
    paused:     bool,
    highlight:  bool,
    window:     WindowState
}

struct WindowState
{
    focused: bool,
    mouse:   bool
}

fn model(app : &App) -> Model
{
    app.main_window().set_title("Kerbolar");

    Model
    {
        frame:      0,
        scale:      3.0e-9,
        celestials: kerbolar_system(),
        focus:      0,
        paused:     true,
        highlight:  false,
        window:     WindowState
        {
            focused: true,
            mouse:   false
        }
    }
}

fn event(_ : &App, model : &mut Model, e : Event)
{
    match e
    {
        Event::Update(_) =>
        {
            if !model.paused
            {
                for _ in 0 .. STEPS_PER_FRAME
                {
                    nbody_step_euler(&mut model.celestials, SECONDS_PER_STEP);
                }
                model.frame += 1;
            }
        },

        Event::DeviceEvent(_, e) =>
        {
            if let DeviceEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, v) } = e
            {
                if model.window.mouse
                {
                    model.scale *= match v.partial_cmp(&0.0)
                    {
                        Some(Ordering::Less)    => ZOOM,
                        Some(Ordering::Greater) => ZOOM.recip(),
                        _                       => 1.0
                    };

                    model.scale = model.scale.max(2.0e-12).min(1.0e-4);
                }
            }
            else if model.window.focused
            {
                if let DeviceEvent::Key(k) = e
                {
                    if let ElementState::Pressed = k.state
                    {
                        match k.virtual_keycode
                        {
                            Some(VirtualKeyCode::P)     => model.paused    = !model.paused,
                            Some(VirtualKeyCode::H)     => model.highlight = !model.highlight,
                            Some(VirtualKeyCode::Left)  => model.focus     = (model.focus - 1 + model.celestials.len()) % model.celestials.len(),
                            Some(VirtualKeyCode::Right) => model.focus     = (model.focus + 1)                          % model.celestials.len(),
                            _                           => ()
                        }
                    }
                }
            }
        },

        Event::WindowEvent { simple: Some(e), .. } =>
        {
            match e
            {
                WindowEvent::Focused      => model.window.focused = true,
                WindowEvent::Unfocused    => model.window.focused = false,
                WindowEvent::MouseEntered => model.window.mouse   = true,
                WindowEvent::MouseExited  => model.window.mouse   = false,
                _                      => ()

            }
        },

        _ => ()
    }
}

fn view(app : &App, model : &Model, frame : Frame)
{
    let draw = app.draw();
    draw.background().rgb(0.0, 0.0, 0.0);

    let focus     = &model.celestials[model.focus];
    let focus_pos = &focus.orbit.position;

    for celestial in model.celestials.iter().rev()
    {
        let diameter = (2.0 * celestial.radius * model.scale) as f32;

        draw.ellipse()
            .rgb(celestial.colour.x, celestial.colour.y, celestial.colour.z)
            .w(diameter.max(0.1))
            .h(diameter.max(0.1))
            .x_y(((celestial.orbit.position.x - focus_pos.x) * model.scale) as f32, ((celestial.orbit.position.y - focus_pos.y) * model.scale) as f32);

        if model.highlight && diameter < 1.0
        {
            draw.ellipse()
                .w(12.0)
                .h(12.0)
                .x_y(((celestial.orbit.position.x - focus_pos.x) * model.scale) as f32, ((celestial.orbit.position.y - focus_pos.y) * model.scale) as f32)
                .no_fill()
                .stroke_weight(0.4)
                .stroke(Rgb::new(celestial.colour.x, celestial.colour.y, celestial.colour.z));
        }
    }


    draw.text(&focus.name)
        .rgb(focus.colour.x, focus.colour.y, focus.colour.z)
        .x_y(0.0, -(focus.radius * model.scale) as f32 - 15.0);

    let top = app.window_rect().top();
    let day = model.frame as f64 * DAYS_PER_FRAME;
    draw.text(&format!("Day {}, Hour {}", day as u64, (day.fract() * 24.0) as u64))
        .color(WHITE)
        .x_y(0.0, top - 20.0);

    if model.paused
    {
        draw.text("PAUSED")
            .color(WHITE)
            .x_y(0.0, top - 40.0);
    }

    draw.to_frame(app, &frame).unwrap();

}
