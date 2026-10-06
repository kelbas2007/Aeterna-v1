use aeterna_v1::{EvoConfig, EvoPhase, RasterFieldConfig};

#[derive(Clone, Copy, Debug)]
enum Class {
    A,
    B,
}

#[derive(Clone, Debug)]
struct World {
    class: Class,
    x: usize,
    y: usize,
}

struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed ^ 0x9E37_79B9_7F4A_7C15 }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn range(&mut self, upper: usize) -> usize {
        (self.next_u64() as usize) % upper
    }

    fn bit(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }
}

fn pattern(class: Class, x: usize, y: usize) -> Vec<f32> {
    let mut raster = vec![0.0; 12 * 12];
    let points: [(usize, usize); 3] = match class {
        Class::A => [(x, y), (x + 1, y), (x + 2, y)],
        Class::B => [(x, y), (x, y + 1), (x, y + 2)],
    };
    for (px, py) in points {
        raster[py * 12 + px] = 1.0;
    }
    raster
}

fn correct_action(class: Class, swap: bool) -> usize {
    match (class, swap) {
        (Class::A, false) => 0,
        (Class::B, false) => 1,
        (Class::A, true) => 1,
        (Class::B, true) => 0,
    }
}

fn training_origins(class: Class) -> &'static [(usize, usize)] {
    match class {
        Class::A => &[(1, 1), (4, 4), (7, 7), (2, 8), (6, 2), (8, 5)],
        Class::B => &[(8, 1), (2, 6), (6, 7), (1, 2), (5, 4), (9, 6)],
    }
}

fn carrier() -> EvoPhase {
    let mut cfg = EvoConfig::default();
    cfg.sensory_cells = 12 * 12;
    cfg.motor_cells = 2;
    cfg.dormant_cells = 64;
    cfg.hdc_dim = 192;
    cfg.structural_growth_enabled = false;

    let mut evo = EvoPhase::new(cfg);
    let mut raster_cfg = RasterFieldConfig::for_raster(12, 12, 2);
    raster_cfg.match_threshold = 0.97;
    raster_cfg.min_active = 2;
    raster_cfg.max_units = 32;
    raster_cfg.min_readout_support = 2;
    raster_cfg.formation_enabled = true;
    raster_cfg.readout_enabled = false;
    raster_cfg.learning_enabled = true;
    evo.attach_raster_field(raster_cfg);
    evo
}

fn train_carrier(swap: bool) -> EvoPhase {
    let mut evo = carrier();

    for _epoch in 0..6 {
        for class in [Class::A, Class::B] {
            for &(x, y) in training_origins(class) {
                let pre = pattern(class, x, y);
                for action in 0..2 {
                    evo.observe_initial_real(&pre, false);
                    let need = action == correct_action(class, swap);
                    evo.learn_factual_transition(action, &pre, need);
                }
            }
        }
    }

    evo.set_raster_learning_enabled(false);
    evo.set_raster_readout_enabled(true);
    evo
}

fn predict_carrier(mature: &EvoPhase, raster: &[f32]) -> usize {
    let mut evo = mature.clone();
    evo.observe_initial_real(raster, false);
    evo.choose_motor()
}

fn raw_dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

#[derive(Clone)]
struct Template {
    raster: Vec<f32>,
    action: usize,
}

fn templates(swap: bool) -> Vec<Template> {
    let mut out = Vec::new();
    for class in [Class::A, Class::B] {
        for &(x, y) in training_origins(class) {
            out.push(Template {
                raster: pattern(class, x, y),
                action: correct_action(class, swap),
            });
        }
    }
    out
}

fn template_predict(bank: &[Template], raster: &[f32]) -> usize {
    bank.iter()
        .enumerate()
        .max_by(|(ia, a), (ib, b)| {
            raw_dot(&a.raster, raster)
                .partial_cmp(&raw_dot(&b.raster, raster))
                .unwrap()
                .then_with(|| ib.cmp(ia))
        })
        .map(|(_, t)| t.action)
        .unwrap_or(0)
}

struct Linear {
    w: [Vec<f32>; 2],
    b: [f32; 2],
}

impl Linear {
    fn train(swap: bool) -> Self {
        let mut model = Self {
            w: [vec![0.0; 144], vec![0.0; 144]],
            b: [0.0, 0.0],
        };

        let mut data = Vec::new();
        for class in [Class::A, Class::B] {
            for &(x, y) in training_origins(class) {
                data.push((pattern(class, x, y), correct_action(class, swap)));
            }
        }

        for _ in 0..80 {
            for (x, target) in &data {
                let pred = model.predict(x);
                if pred != *target {
                    for i in 0..x.len() {
                        model.w[*target][i] += x[i];
                        model.w[pred][i] -= x[i];
                    }
                    model.b[*target] += 1.0;
                    model.b[pred] -= 1.0;
                }
            }
        }

        model
    }

    fn predict(&self, x: &[f32]) -> usize {
        let scores = [0usize, 1usize].map(|k| {
            self.b[k]
                + self.w[k]
                    .iter()
                    .zip(x)
                    .map(|(w, v)| w * v)
                    .sum::<f32>()
        });
        if scores[1] > scores[0] { 1 } else { 0 }
    }
}

fn wilson95(success: usize, n: usize) -> (f64, f64) {
    let z = 1.959_963_984_540_054_f64;
    let n = n as f64;
    let p = success as f64 / n;
    let denom = 1.0 + z * z / n;
    let center = (p + z * z / (2.0 * n)) / denom;
    let half = z
        * ((p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt())
        / denom;
    (center - half, center + half)
}

fn fnv_mix(mut hash: u64, value: u64) -> u64 {
    const PRIME: u64 = 1_099_511_628_211;
    for byte in value.to_le_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn fresh_worlds(rng: &mut Rng) -> Vec<World> {
    let mut worlds = Vec::new();
    while worlds.len() < 8 {
        let class = if rng.bit() { Class::A } else { Class::B };
        let x = rng.range(10);
        let y = rng.range(10);
        if training_origins(class).contains(&(x, y)) {
            continue;
        }
        worlds.push(World { class, x, y });
    }
    worlds
}

fn add_distractor(mut raster: Vec<f32>, rng: &mut Rng) -> Vec<f32> {
    for _ in 0..200 {
        let idx = rng.range(144);
        if raster[idx] < 0.5 {
            raster[idx] = 1.0;
            break;
        }
    }
    raster
}

fn drop_task_pixel(mut raster: Vec<f32>, rng: &mut Rng) -> Vec<f32> {
    let active = raster
        .iter()
        .enumerate()
        .filter(|(_, v)| **v >= 0.5)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    if active.len() >= 3 {
        raster[active[rng.range(active.len())]] = 0.0;
    }
    raster
}

fn rotated(class: Class, x: usize, y: usize) -> Vec<f32> {
    // Preserve the original class label while rotating its local geometry by 90°.
    match class {
        Class::A => pattern(Class::B, x, y),
        Class::B => pattern(Class::A, x, y),
    }
}

fn doubled_spacing(class: Class, x: usize, y: usize) -> Vec<f32> {
    let mut raster = vec![0.0; 144];
    let x = x.min(7);
    let y = y.min(7);
    let points: [(usize, usize); 3] = match class {
        Class::A => [(x, y), (x + 2, y), (x + 4, y)],
        Class::B => [(x, y), (x, y + 2), (x, y + 4)],
    };
    for (px, py) in points {
        raster[py * 12 + px] = 1.0;
    }
    raster
}

#[test]
#[ignore = "requires one-use AETERNA_FRESH_SEED from CI"]
fn fresh_g1_translation_pack() {
    let authority_seed: u64 = std::env::var("AETERNA_FRESH_SEED")
        .expect("AETERNA_FRESH_SEED is mandatory")
        .parse()
        .expect("fresh seed must be u64");
    let source_sha = std::env::var("AETERNA_SOURCE_SHA").unwrap_or_else(|_| "unknown".into());
    let spec_sha = std::env::var("AETERNA_SPEC_SHA").unwrap_or_else(|_| "unknown".into());

    let mut total = 0usize;
    let mut evo_ok = 0usize;
    let mut template_ok = 0usize;
    let mut linear_ok = 0usize;
    let mut per_seed = Vec::new();

    let mut distractor_ok = 0usize;
    let mut dropout_ok = 0usize;
    let mut rotation_ok = 0usize;
    let mut scale_ok = 0usize;

    let mut digest = 14_695_981_039_346_656_037_u64;

    for sub in 0..10u64 {
        let derived = authority_seed
            ^ sub.wrapping_mul(0xD1B5_4A32_D192_ED03)
            ^ 0xA37E_2C91_5F04_771B;
        let mut rng = Rng::new(derived);
        let swap = rng.bit();

        let mature = train_carrier(swap);
        let bank = templates(swap);
        let linear = Linear::train(swap);
        let worlds = fresh_worlds(&mut rng);

        let mut sub_ok = 0usize;
        for world in worlds {
            digest = fnv_mix(digest, sub);
            digest = fnv_mix(digest, world.x as u64);
            digest = fnv_mix(digest, world.y as u64);
            digest = fnv_mix(digest, matches!(world.class, Class::B) as u64);
            digest = fnv_mix(digest, swap as u64);

            let expected = correct_action(world.class, swap);
            let clean = pattern(world.class, world.x, world.y);

            let evo_pred = predict_carrier(&mature, &clean);
            let template_pred = template_predict(&bank, &clean);
            let linear_pred = linear.predict(&clean);

            evo_ok += usize::from(evo_pred == expected);
            template_ok += usize::from(template_pred == expected);
            linear_ok += usize::from(linear_pred == expected);
            sub_ok += usize::from(evo_pred == expected);
            total += 1;

            let mut stress_rng = Rng::new(
                derived
                    ^ ((world.x as u64) << 32)
                    ^ ((world.y as u64) << 16)
                    ^ total as u64,
            );
            distractor_ok += usize::from(
                predict_carrier(&mature, &add_distractor(clean.clone(), &mut stress_rng))
                    == expected,
            );
            dropout_ok += usize::from(
                predict_carrier(&mature, &drop_task_pixel(clean.clone(), &mut stress_rng))
                    == expected,
            );
            rotation_ok += usize::from(
                predict_carrier(&mature, &rotated(world.class, world.x, world.y))
                    == expected,
            );
            scale_ok += usize::from(
                predict_carrier(&mature, &doubled_spacing(world.class, world.x, world.y))
                    == expected,
            );
        }

        per_seed.push(sub_ok);
    }

    let evo_rate = evo_ok as f64 / total as f64;
    let template_rate = template_ok as f64 / total as f64;
    let linear_rate = linear_ok as f64 / total as f64;
    let (lo, hi) = wilson95(evo_ok, total);

    println!(
        "FRESH_G1 source_sha={} spec_sha={} authority_seed={} pack_digest={:016x}",
        source_sha, spec_sha, authority_seed, digest
    );
    println!(
        "FRESH_G1 N={} evo={}/{} rate={:.4} wilson95=[{:.4},{:.4}] template={}/{} rate={:.4} linear={}/{} rate={:.4} per_seed={:?}",
        total,
        evo_ok,
        total,
        evo_rate,
        lo,
        hi,
        template_ok,
        total,
        template_rate,
        linear_ok,
        total,
        linear_rate,
        per_seed
    );
    println!(
        "FRESH_G1_STRESS distractor={}/{} dropout={}/{} rotation={}/{} scale={}/{}",
        distractor_ok,
        total,
        dropout_ok,
        total,
        rotation_ok,
        total,
        scale_ok,
        total
    );

    assert!(total >= 80);
    assert!(evo_ok >= 76, "fresh translation success must be >= 76/80");
    assert!(lo > 0.88, "Wilson 95% lower bound must exceed 0.88");
    assert!(
        evo_rate - template_rate >= 0.20,
        "EvoPhase must exceed raw-template baseline by >= 0.20"
    );
    assert!(
        evo_rate - linear_rate >= 0.15,
        "EvoPhase must exceed linear baseline by >= 0.15"
    );
    assert!(
        per_seed.iter().all(|success| *success >= 6),
        "every sub-seed must achieve at least 6/8"
    );
}
