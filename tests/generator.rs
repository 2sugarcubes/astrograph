use astrolabe::{
    dynamic::keplerian::Keplerian,
    generator::{artifexian::Artifexian, Generator},
    Float,
};

#[test]
fn inclinations() {
    let mut rng = rand::thread_rng();
    let root = Artifexian::generate(&mut rng);

    for star in root.read().unwrap().get_children() {
        let star = star.read().unwrap();

        let mut inclinations = Vec::with_capacity(8);

        for planet in star.get_children() {
            let planet = planet.read().unwrap();
            if let Some(planet_dynamic) = planet.get_dynamic().as_any().downcast_ref::<Keplerian>()
            {
                let planet_inclination = planet_dynamic.get_inclination().to_owned();
                inclinations.push(planet_inclination.clone());

                for moon in planet.get_children() {
                    let moon = moon.read().unwrap();
                    if let Some(moon_dynamic) =
                        moon.get_dynamic().as_any().downcast_ref::<Keplerian>()
                    {
                        let moon_inclination = moon_dynamic.get_inclination().to_owned();
                        let qd =
                            quaternion::mul(quaternion::conj(planet_inclination), moon_inclination);
                        let deflection = 2.0 * (3.0 as Float).atan2(qd.0);
                        println!("moon deflection: {deflection}");
                    }
                }
            }
        }

        let mut sum = (0.0, [0.0, 0.0, 0.0]);
        for q in &inclinations {
            sum = quaternion::add(sum, q.to_owned());
        }
        sum = quaternion::scale(sum, inclinations.len() as Float);

        for q in &inclinations {
            let qd = quaternion::mul(sum, q.to_owned());
            let deflection = 2.0 * (3.0 as Float).atan2(qd.0);
            println!("planet deflection: {deflection}");
        }
    }
}
