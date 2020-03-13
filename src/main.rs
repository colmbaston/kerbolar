mod celestial;
use celestial::*;

use std::cmp::Ordering;
use nannou::prelude::*;
use winit::event::{ DeviceEvent, VirtualKeyCode, ElementState };

const INITIAL_SCALE    : f64 = 3.0e-9;
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
    active:     bool
}

fn model(app : &App) -> Model
{
    app.main_window().set_title("Kerbolar");

    println!("Kerbolar");
    println!("--------");
    println!("Cycle through the celestial bodies using the left and right arrow keys; zoom using the scroll wheel.");
    println!("Press H to toggle highlighting of celestial bodies when they become too small to see.");
    println!("The simulation is currently paused; press P to toggle the pause state.");
    println!();

    Model { frame: 0, scale: INITIAL_SCALE, celestials: kerbolar_system(), focus: 0, paused: true, highlight: false, active: true }
}

fn event(_ : &App, model : &mut Model, e : Event)
{
    match e
    {
        Event::Update(_) =>
        {
            if !model.paused
            {
                let day = (model.frame as f64 * DAYS_PER_FRAME).floor() as u64;

                if day != ((model.frame.wrapping_sub(1)) as f64 * DAYS_PER_FRAME).floor() as u64
                {
                    println!("Beginning day {}.", day);
                }

                for _ in 0 .. STEPS_PER_FRAME
                {
                    nbody_step_euler(&mut model.celestials, SECONDS_PER_STEP);
                }

                model.frame += 1;
            }
        },

        Event::DeviceEvent(_, e) =>
        {
            if model.active
            {
                if let DeviceEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, v) } = e
                {
                    model.scale *= match v.partial_cmp(&0.0)
                    {
                        Some(Ordering::Less)    => ZOOM,
                        Some(Ordering::Greater) => ZOOM.recip(),
                        _                       => 1.0
                    };
                }
                else if let DeviceEvent::Key(k) = e
                {
                    if let ElementState::Pressed = k.state
                    {
                        match k.virtual_keycode
                        {
                            Some(VirtualKeyCode::P)     => { model.paused    = !model.paused;    println!("Simulation {}.",   if model.paused    { "paused" } else { "unpaused" }) },
                            Some(VirtualKeyCode::H)     => { model.highlight = !model.highlight; println!("Highlighting {}.", if model.highlight { "on"     } else { "off"      }) },
                            Some(VirtualKeyCode::Left)  => { model.focus = if model.focus == 0 { model.celestials.len() - 1 } else { model.focus - 1 }; println!("Now focusing {}.", model.celestials[model.focus].name) },
                            Some(VirtualKeyCode::Right) => { model.focus =   (model.focus + 1) % model.celestials.len();                                println!("Now focusing {}.", model.celestials[model.focus].name) },
                            _                           => ()
                        }
                    }
                }
            }
        },

        Event::WindowEvent { id: _, simple: Some(e) } =>
        {
            match e
            {
                WindowEvent::Focused   => model.active = true,
                WindowEvent::Unfocused => model.active = false,
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

    let focus_pos = &model.celestials[model.focus].orbit.position;

    for celestial in model.celestials.iter().rev()
    {
        let diameter = (2.0 * celestial.radius * model.scale) as f32;

        draw.ellipse().rgb(celestial.colour.x, celestial.colour.y, celestial.colour.z)
                      .w(diameter.max(0.1))
                      .h(diameter.max(0.1))
                      .x_y(((celestial.orbit.position.x - focus_pos.x) * model.scale) as f32, ((celestial.orbit.position.y - focus_pos.y) * model.scale) as f32);

        if model.highlight && diameter < 1.0
        {
            draw.ellipse().w(12.0)
                          .h(12.0)
                          .x_y(((celestial.orbit.position.x - focus_pos.x) * model.scale) as f32, ((celestial.orbit.position.y - focus_pos.y) * model.scale) as f32)
                          .no_fill()
                          .stroke_weight(0.4)
                          .stroke(Rgb::new(celestial.colour.x, celestial.colour.y, celestial.colour.z));
        }
    }

    draw.to_frame(app, &frame).unwrap();
}
