mod celestial;
use celestial::*;

use std::cmp::Ordering;
use nannou::prelude::*;
use winit::event::{ DeviceEvent, VirtualKeyCode, ElementState };

fn main()
{
    nannou::app(model).event(event).simple_window(view).run();
}

struct Model
{
    time:       f64,
    steps:      u32,
    speed:      Scale,
    zoom:       Scale,
    celestials: Vec<Celestial>,
    focus:      usize,
    paused:     bool,
    highlight:  bool,
    window:     WindowState
}

struct Scale
{
    current : f64,
    min     : f64,
    max     : f64
}

impl Scale
{
    fn multiply(&mut self, factor : f64)
    {
        self.current = (self.current * factor).max(self.min).min(self.max)
    }
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
        time:  0.0,
        steps: 360,
        speed: Scale
        {
            current: 1.0,
            min:     360.0.recip(),
            max:     360.0
        },
        zoom: Scale
        {
            current: 3.0e-9,
            min:     2.0e-12,
            max:     1.0e-4,
        },
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

fn event(app : &App, model : &mut Model, e : Event)
{
    match e
    {
        Event::Update(_) =>
        {
            if !model.paused
            {
                for _ in 0 .. model.steps
                {
                    nbody_step_euler(&mut model.celestials, model.speed.current);
                    model.time += model.speed.current;
                }
            }
        },

        Event::DeviceEvent(_, e) =>
        {
            if let DeviceEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, v) } = e
            {
                if model.window.mouse
                {
                    model.zoom.multiply(match v.partial_cmp(&0.0)
                    {
                        Some(Ordering::Less)    => 1.1,
                        Some(Ordering::Greater) => 1.1.recip(),
                        _                       => 1.0
                    })
                }
            }
            else if model.window.focused
            {
                if let DeviceEvent::Key(k) = e
                {
                    if let ElementState::Pressed = k.state
                    {
                        let modifiers = app.keys.mods.bits();

                        match k.virtual_keycode
                        {
                            Some(VirtualKeyCode::P)        => if modifiers <= 8 { model.paused    = !model.paused    },
                            Some(VirtualKeyCode::H)        => if modifiers <= 8 { model.highlight = !model.highlight },
                            Some(VirtualKeyCode::Left)     => if modifiers == 0 { model.focus     = (model.focus - 1 + model.celestials.len()) % model.celestials.len() },
                            Some(VirtualKeyCode::Right)    => if modifiers == 0 { model.focus     = (model.focus + 1)                          % model.celestials.len() },
                            Some(VirtualKeyCode::Up)       => if modifiers == 0 { model.speed.multiply(1.1)         },
                            Some(VirtualKeyCode::Down)     => if modifiers == 0 { model.speed.multiply(1.1.recip()) },
                            Some(VirtualKeyCode::LBracket) => if modifiers <= 8 { model.steps = (model.steps - 10).max(100) },
                            Some(VirtualKeyCode::RBracket) => if modifiers <= 8 { model.steps =  model.steps + 10           },
                            _                              => ()
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
                _                         => ()

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
        let diameter = (2.0 * celestial.radius * model.zoom.current) as f32;

        draw.ellipse()
            .rgb(celestial.colour.x, celestial.colour.y, celestial.colour.z)
            .w(diameter.max(0.1))
            .h(diameter.max(0.1))
            .x_y(((celestial.orbit.position.x - focus_pos.x) * model.zoom.current) as f32, ((celestial.orbit.position.y - focus_pos.y) * model.zoom.current) as f32);

        if model.highlight && diameter < 1.0
        {
            draw.ellipse()
                .w(12.0)
                .h(12.0)
                .x_y(((celestial.orbit.position.x - focus_pos.x) * model.zoom.current) as f32, ((celestial.orbit.position.y - focus_pos.y) * model.zoom.current) as f32)
                .no_fill()
                .stroke_weight(0.4)
                .stroke(Rgb::new(celestial.colour.x, celestial.colour.y, celestial.colour.z));
        }
    }

    draw.text(&focus.name)
        .rgb(focus.colour.x, focus.colour.y, focus.colour.z)
        .x_y(0.0, -(focus.radius * model.zoom.current) as f32 - 15.0);

    let top  = app.window_rect().top();
    let day  = model.time as u64 / 86_400;
    let hour = model.time as u64 % 86_400 / 3_600;

    draw.text(&format!("Day {}, Hour {:02}", day, hour))
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
