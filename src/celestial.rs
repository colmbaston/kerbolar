use std::ops::{ Add, Sub };

pub struct V3<T>
{
    pub x : T,
    pub y : T,
    pub z : T
}

impl V3<f64>
{
    const ZERO : V3<f64> = V3 { x: 0.0, y: 0.0, z: 0.0 };

    fn pythagoras(&self, other : &V3<f64>) -> f64
    {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;

        (dx*dx + dy*dy + dz*dz).sqrt()
    }

    fn scale_by(&self, scalar : f64) -> V3<f64>
    {
        V3
        {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar
        }
    }

    fn scale_to(&self, magnitude : f64) -> V3<f64>
    {
        self.scale_by(magnitude / self.pythagoras(&V3::ZERO))
    }
}

impl<T : Copy + Add<Output = T>> Add for &V3<T>
{
    type Output = V3<T>;

    fn add(self, other : &V3<T>) -> V3<T>
    {
        V3
        {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z
        }
    }
}

impl<T : Copy + Sub<Output = T>> Sub for &V3<T>
{
    type Output = V3<T>;

    fn sub(self, other : &V3<T>) -> V3<T>
    {
        V3
        {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z
        }
    }
}

const G : f64 = 6.674e-11;

pub struct Celestial
{
    pub name:     String,
    pub colour:   V3<f32>,
    pub mass:     f64,
    pub radius:   f64,
    pub orbit:    StateVectors
}

pub struct StateVectors
{
    pub position: V3<f64>,
    pub velocity: V3<f64>
}

pub fn nbody_step(bodies : &mut [Celestial], delta_t : f64)
{
    for i in 0 .. bodies.len()
    {
        for j in i+1 .. bodies.len()
        {
            let distance     = bodies[i].orbit.position.pythagoras(&bodies[j].orbit.position);
            let force_scalar = delta_t * (G * bodies[i].mass * bodies[j].mass) / (distance * distance);
            let force_vector = &bodies[i].orbit.position - &bodies[j].orbit.position;

            bodies[i].orbit.velocity = &bodies[i].orbit.velocity - &force_vector.scale_to(force_scalar / bodies[i].mass);
            bodies[j].orbit.velocity = &bodies[j].orbit.velocity + &force_vector.scale_to(force_scalar / bodies[j].mass);
        }
        bodies[i].orbit.position = &bodies[i].orbit.position + &bodies[i].orbit.velocity.scale_by(delta_t);
    }
}

fn mean_anomaly_to_true(ma : f64, ecc : f64, tolerance : f64) -> f64
{
    let mut ea = if ecc > 0.8 { std::f64::consts::PI } else { ma };

    loop
    {
        let f = ea - ecc * ea.sin() - ma;
        ea = ea - f / (1.0 - ecc * ea.cos());

        if f < tolerance { return 2.0 * (((1.0+ecc)/(1.0-ecc)).sqrt() * (ea/2.0).tan()).atan() }
    }
}

impl StateVectors
{
    fn from_keplerian(gm : f64, sma : f64, ecc : f64, inc : f64, aop : f64, lan : f64, ma : f64) -> StateVectors
    {
        let p      = sma * (1.0 - ecc.powi(2));
        let ta     = mean_anomaly_to_true(ma, ecc, 1.0e-30);
        let radius = p / (1.0 + ecc * ta.cos());

        let inc = inc.to_radians();
        let aop = aop.to_radians();
        let lan = lan.to_radians();

        let (sin_aop_ta, cos_aop_ta) = (aop + ta).sin_cos();
        let (sin_inc,    cos_inc)    = inc.sin_cos();
        let (sin_lan,    cos_lan)    = lan.sin_cos();
        let (sin_aop,    cos_aop)    = aop.sin_cos();

        let x = radius * (cos_aop_ta * cos_lan - cos_inc * sin_aop_ta * sin_lan);
        let y = radius * (cos_aop_ta * sin_lan + cos_inc * sin_aop_ta * cos_lan);
        let z = radius *  sin_aop_ta * sin_inc;
        let position = V3 { x, y, z };

        let sqrt_gm_p  = (gm / p).sqrt();
        let cos_ta_ecc = ta.cos() + ecc;
        let sin_ta     = ta.sin();

        let x = sqrt_gm_p *  cos_ta_ecc * (-sin_aop * cos_lan - cos_inc * sin_lan * cos_aop) - sqrt_gm_p * sin_ta * (cos_aop * cos_lan - cos_inc * sin_lan * sin_aop);
        let y = sqrt_gm_p *  cos_ta_ecc * (-sin_aop * sin_lan + cos_inc * cos_lan * cos_aop) - sqrt_gm_p * sin_ta * (cos_aop * sin_lan + cos_inc * cos_lan * sin_aop);
        let z = sqrt_gm_p * (cos_ta_ecc *   sin_inc * cos_aop - sin_ta  * sin_inc * sin_aop);
        let velocity = V3 { x, y, z };

        StateVectors { position, velocity }
    }

    fn relative_to(self, other : &StateVectors) -> StateVectors
    {
        StateVectors
        {
            position: &self.position + &other.position,
            velocity: &self.velocity + &other.velocity
        }
    }
}

pub fn kerbolar_system() -> Vec<Celestial>
{
    let kerbol = Celestial
    {
        name:   String::from("Kerbol"),
        colour: V3 { x: 1.0,  y: 0.9, z: 0.5 },
        mass:   1.756_545_9e28,
        radius: 261_600_000.0,
        orbit:  StateVectors { position: V3::ZERO, velocity: V3::ZERO }
    };

    let moho = Celestial
    {
        name:   String::from("Moho"),
        colour: V3 { x: 0.8,  y: 0.4, z: 0.0 },
        mass:   2.526_331_4e21,
        radius: 250_000.0,
        orbit:  StateVectors::from_keplerian(G * kerbol.mass,
                                             5_263_138_304.0,
                                              0.2,
                                              7.0,
                                             15.0,
                                             70.0,
                                              3.14).relative_to(&kerbol.orbit)
    };

    let eve = Celestial
    {
        name:   String::from("Eve"),
        colour: V3 { x: 0.4,  y: 0.3, z: 0.4 },
        mass:   1.224_398_0e23,
        radius: 700_000.0,
        orbit:  StateVectors::from_keplerian(G * kerbol.mass,
                                             9_832_684_544.0,
                                              0.01,
                                              2.1,
                                              0.0,
                                             15.0,
                                              3.14).relative_to(&kerbol.orbit)
    };

    let gilly = Celestial
    {
        name:   String::from("Gilly"),
        colour: V3 { x: 0.8,  y: 0.4, z: 0.0 },
        mass:   1.242_036_3e17,
        radius: 13_000.0,
        orbit:  StateVectors::from_keplerian(G * eve.mass,
                                             31_500_000.0,
                                              0.55,
                                             12.0,
                                             10.0,
                                             80.0,
                                              0.9).relative_to(&eve.orbit)
    };

    let kerbin = Celestial
    {
        name:   String::from("Kerbin"),
        colour: V3 { x: 0.0,  y: 0.5, z: 0.0 },
        mass:   5.291_515_8e22,
        radius: 600_000.0,
        orbit:  StateVectors::from_keplerian(G * kerbol.mass,
                                             13_599_840_256.0,
                                              0.0,
                                              0.0,
                                              0.0,
                                              0.0,
                                             3.14).relative_to(&kerbol.orbit)
    };

    let mun = Celestial
    {
        name:   String::from("Mun"),
        colour: V3 { x: 0.4,  y: 0.4,   z: 0.4 },
        mass:   9.759_906_6e20,
        radius: 200_000.0,
        orbit:  StateVectors::from_keplerian(G * kerbin.mass,
                                             12_000_000.0,
                                             0.0,
                                             0.0,
                                             0.0,
                                             0.0,
                                             1.7).relative_to(&kerbin.orbit)
    };

    let minmus = Celestial
    {
        name:   String::from("Minmus"),
        colour: V3 { x: 0.69, y: 0.882, z: 0.808 },
        mass:   2.645_758_0e19,
        radius: 60_000.0,
        orbit:  StateVectors::from_keplerian(G * kerbin.mass,
                                             47_000_000.0,
                                              0.0,
                                              6.0,
                                             38.0,
                                             78.0,
                                              0.9).relative_to(&kerbin.orbit)
    };

    vec![kerbol, moho, eve, gilly, kerbin, mun, minmus]
}
