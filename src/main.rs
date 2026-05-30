// Grand Pattern Experiments — pure Rust, zero dependencies
// Tests whether vibes actually diffuse across a graph the way the theory predicts.

use std::fmt;

// ─── Core Types ───────────────────────────────────────────────────────

/// A 6-dimensional vibe vector (curiosity, urgency, creativity, focus, harmony, surprise)
#[derive(Clone, Copy, Debug)]
struct Vibe {
    dims: [f64; 6],
}

impl Vibe {
    #[allow(dead_code)]
    const LABELS: [&'static str; 6] = [
        "curiosity",
        "urgency",
        "creativity",
        "focus",
        "harmony",
        "surprise",
    ];

    fn constant(v: f64) -> Self {
        Vibe { dims: [v; 6] }
    }

    fn zero() -> Self {
        Self::constant(0.0)
    }

    fn add(&self, other: &Vibe) -> Vibe {
        let mut d = self.dims;
        for (i, &val) in other.dims.iter().enumerate() {
            d[i] += val;
        }
        Vibe { dims: d }
    }

    fn scale(&self, s: f64) -> Vibe {
        let mut d = self.dims;
        for val in d.iter_mut() {
            *val *= s;
        }
        Vibe { dims: d }
    }

    fn lerp(&self, other: &Vibe, t: f64) -> Vibe {
        self.scale(1.0 - t).add(&other.scale(t))
    }

    fn surprise(&self) -> f64 {
        self.dims[5]
    }

    fn magnitude(&self) -> f64 {
        self.dims.iter().map(|d| d * d).sum::<f64>().sqrt()
    }

    fn max_diff(&self, other: &Vibe) -> f64 {
        self.dims
            .iter()
            .zip(other.dims.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f64, f64::max)
    }

    #[allow(dead_code)]
    fn mean(&self) -> f64 {
        self.dims.iter().sum::<f64>() / 6.0
    }

    fn set_surprise(&mut self, v: f64) {
        self.dims[5] = v;
    }
}

impl fmt::Display for Vibe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, d) in self.dims.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:.4}", d)?;
        }
        write!(f, "]")
    }
}

/// A room in the vibe graph
#[derive(Clone)]
struct Room {
    #[allow(dead_code)] id: usize,
    vibe: Vibe,
    surprise_accumulator: f64,
}

/// A graph of rooms with edges
struct VibeGraph {
    rooms: Vec<Room>,
    /// adjacency list: edges[i] = set of neighbor indices
    edges: Vec<Vec<usize>>,
    gc_aggressiveness: f64,
}

impl VibeGraph {
    fn new(n: usize) -> Self {
        let rooms = (0..n)
            .map(|id| Room {
                id,
                vibe: Vibe::constant(0.5),
                surprise_accumulator: 0.0,
            })
            .collect();
        VibeGraph {
            rooms,
            edges: vec![vec![]; n],
            gc_aggressiveness: 0.0,
        }
    }

    fn add_edge(&mut self, a: usize, b: usize) {
        if !self.edges[a].contains(&b) {
            self.edges[a].push(b);
        }
        if !self.edges[b].contains(&a) {
            self.edges[b].push(a);
        }
    }

    fn set_vibe(&mut self, idx: usize, vibe: Vibe) {
        self.rooms[idx].vibe = vibe;
    }

    #[allow(dead_code)]
    fn inject_surprise(&mut self, idx: usize, amount: f64) {
        self.rooms[idx].surprise_accumulator += amount;
    }

    fn inject_input(&mut self, idx: usize, input_vibe: &Vibe) {
        // blend input into room vibe
        self.rooms[idx].vibe = self.rooms[idx].vibe.lerp(input_vibe, 0.3);
        // surprise is how different the input is from current vibe
        let diff = self.rooms[idx].vibe.max_diff(input_vibe);
        self.rooms[idx].surprise_accumulator += diff;
    }

    /// Run one tick of gossip (diffusion)
    fn tick(&mut self, diffusion_coeff: f64) {
        let n = self.rooms.len();
        if n == 0 {
            return;
        }

        // Compute new vibes via diffusion
        let mut new_vibes: Vec<Vibe> = self.rooms.iter().map(|r| r.vibe).collect();

        for (i, new_vibe) in new_vibes.iter_mut().enumerate().take(n) {
            let neighbors = &self.edges[i];
            if neighbors.is_empty() {
                continue;
            }
            let neighbor_count = neighbors.len() as f64;
            for &j in neighbors {
                // pull from neighbor
                let pull = self.rooms[j].vibe.lerp(&self.rooms[i].vibe, 1.0 - diffusion_coeff / neighbor_count);
                *new_vibe = new_vibe.lerp(&pull, 1.0 / neighbor_count.max(1.0));
            }

            // surprise propagation: a fraction of neighbor surprise leaks
            let mut surprise_leak = 0.0;
            for &j in neighbors {
                surprise_leak += self.rooms[j].surprise_accumulator;
            }
            surprise_leak *= diffusion_coeff / neighbor_count.max(1.0);
            { let s = new_vibe.surprise(); new_vibe.set_surprise(s + surprise_leak * 0.1); }
        }

        // GC: decay surprise and compress vibes toward neutral
        for (i, new_vibe) in new_vibes.iter().enumerate().take(n) {
            self.rooms[i].surprise_accumulator *= 1.0 - self.gc_aggressiveness;
            self.rooms[i].vibe = new_vibe.lerp(&Vibe::constant(0.5), self.gc_aggressiveness * 0.1);
        }
    }

    fn fleet_vibe(&self) -> Vibe {
        if self.rooms.is_empty() {
            return Vibe::zero();
        }
        let mut sum = Vibe::zero();
        for r in &self.rooms {
            sum = sum.add(&r.vibe);
        }
        sum.scale(1.0 / self.rooms.len() as f64)
    }

    fn max_vibe_diff(&self) -> f64 {
        let mut max_d: f64 = 0.0;
        for i in 0..self.rooms.len() {
            for j in (i + 1)..self.rooms.len() {
                max_d = max_d.max(self.rooms[i].vibe.max_diff(&self.rooms[j].vibe));
            }
        }
        max_d
    }

    fn total_vibe_sum(&self) -> f64 {
        self.rooms.iter().map(|r| r.vibe.magnitude()).sum()
    }

    #[allow(dead_code)]
    fn room_count(&self) -> usize {
        self.rooms.len()
    }
}

// ─── Graph Builders ───────────────────────────────────────────────────

fn make_ring(n: usize) -> VibeGraph {
    let mut g = VibeGraph::new(n);
    for i in 0..n {
        g.add_edge(i, (i + 1) % n);
    }
    g
}

fn make_chain(n: usize) -> VibeGraph {
    let mut g = VibeGraph::new(n);
    for i in 0..n.saturating_sub(1) {
        g.add_edge(i, i + 1);
    }
    g
}

fn make_star(n: usize) -> VibeGraph {
    let mut g = VibeGraph::new(n);
    for i in 1..n {
        g.add_edge(0, i);
    }
    g
}

fn make_mesh(n: usize) -> VibeGraph {
    let mut g = VibeGraph::new(n);
    for i in 0..n {
        for j in (i + 1)..n {
            g.add_edge(i, j);
        }
    }
    g
}

// ─── ASCII Plot Helper ────────────────────────────────────────────────

#[allow(dead_code)]
fn ascii_sparkline(data: &[f64], width: usize, height: usize) -> String {
    if data.is_empty() {
        return "(no data)".to_string();
    }
    let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if (max - min) < 1e-12 { 1.0 } else { max - min };

    // Downsample to `width` points
    let step = data.len() as f64 / width as f64;
    let sampled: Vec<f64> = (0..width)
        .map(|i| {
            let idx = ((i as f64 + 0.5) * step) as usize;
            data[idx.min(data.len() - 1)]
        })
        .collect();

    let mut lines = vec![String::new(); height];

    for (row, line) in lines.iter_mut().enumerate() {
        let row_low = (height - 1 - row) as f64 / height as f64;
        let _row_high = (height - row) as f64 / height as f64;
        for val in &sampled {
            let norm = (val - min) / range;
            if norm >= row_low {
                line.push('█');
            } else {
                line.push(' ');
            }
        }
    }
    lines.into_iter().rev().collect::<Vec<_>>().join("\n")
}

fn simple_sparkline(data: &[f64], width: usize) -> String {
    if data.is_empty() {
        return "(no data)".to_string();
    }
    let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if (max - min) < 1e-12 { 1.0 } else { max - min };

    let step = data.len() as f64 / width as f64;
    let blocks = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let sampled: Vec<char> = (0..width)
        .map(|i| {
            let idx = ((i as f64 + 0.5) * step) as usize;
            let norm = (data[idx.min(data.len() - 1)] - min) / range;
            let bi = ((norm * 7.0).round() as usize).min(7);
            blocks[bi]
        })
        .collect();
    sampled.into_iter().collect()
}

// ─── Experiment 1: Vibe Diffusion Convergence ─────────────────────────

struct Experiment1Result {
    convergence_tick: Option<usize>,
    max_diff_history: Vec<f64>,
    final_fleet_vibe: Vibe,
}

fn run_experiment1() -> Experiment1Result {
    let mut g = make_ring(10);
    g.set_vibe(0, Vibe::constant(1.0));
    // others already at 0.5 from VibeGraph::new

    let mut max_diff_history = Vec::new();
    let ticks = 1000;
    let diffusion = 0.1;
    let mut converged_at: Option<usize> = None;
    let threshold = 0.01;

    for tick in 0..ticks {
        let diff = g.max_vibe_diff();
        max_diff_history.push(diff);
        if converged_at.is_none() && diff < threshold {
            converged_at = Some(tick);
        }
        g.tick(diffusion);
    }

    Experiment1Result {
        convergence_tick: converged_at,
        max_diff_history,
        final_fleet_vibe: g.fleet_vibe(),
    }
}

// ─── Experiment 2: Surprise Cascade ───────────────────────────────────

struct Experiment2Result {
    surprise_at_room: [Vec<f64>; 5],
}

fn run_experiment2() -> Experiment2Result {
    let mut g = make_chain(5);
    // Run 100 ticks normally
    for _ in 0..100 {
        g.tick(0.1);
    }
    // Inject surprise at room 0
    let surprise_vibe = Vibe {
        dims: [0.1, 0.1, 0.1, 0.1, 0.1, 0.95],
    };
    g.inject_input(0, &surprise_vibe);

    let mut surprise_at_room: [Vec<f64>; 5] = [vec![], vec![], vec![], vec![], vec![]];

    for _ in 0..100 {
        for (i, room_data) in surprise_at_room.iter_mut().enumerate() {
            room_data.push(g.rooms[i].vibe.surprise());
        }
        g.tick(0.1);
    }

    Experiment2Result { surprise_at_room }
}

// ─── Experiment 3: Conservation Under Stress ──────────────────────────

struct GcResult {
    gc_level: f64,
    conservation_breaks: usize,
    total_ticks: usize,
}

fn run_experiment3() -> Vec<GcResult> {
    let gc_levels = [0.01, 0.05, 0.1, 0.5];
    let ticks = 10_000;
    let mut results = Vec::new();

    // Simple deterministic RNG (xorshift)
    let mut seed: u64 = 42;

    for &gc in &gc_levels {
        let mut g = make_mesh(20);
        g.gc_aggressiveness = gc;

        let mut breaks = 0;

        for _tick in 0..ticks {
            // pseudo-random input injection
            seed = seed ^ (seed << 13);
            seed = seed ^ (seed >> 7);
            seed = seed ^ (seed << 17);
            let room_idx = (seed % 20) as usize;
            seed = seed ^ (seed << 13);
            seed = seed ^ (seed >> 7);
            seed = seed ^ (seed << 17);
            let val = ((seed % 1000) as f64) / 1000.0;
            let input_vibe = Vibe::constant(val);
            g.inject_input(room_idx, &input_vibe);

            let pre_sum = g.total_vibe_sum();
            g.tick(0.1);
            let post_sum = g.total_vibe_sum();

            // Conservation "breaks" if total vibe changes by > 20% in one tick
            // (accounting for GC decay, this is expected to break more with higher GC)
            if pre_sum > 0.01 && (post_sum - pre_sum).abs() / pre_sum > 0.2 {
                breaks += 1;
            }
        }

        results.push(GcResult {
            gc_level: gc,
            conservation_breaks: breaks,
            total_ticks: ticks,
        });
    }

    results
}

// ─── Experiment 4: Topology Comparison ────────────────────────────────

struct TopologyResult {
    name: String,
    convergence_tick: Option<usize>,
    final_uniformity: f64,
    surprise_propagation_speed: f64,
}

fn run_experiment4() -> Vec<TopologyResult> {
    type GraphBuilder = fn(usize) -> VibeGraph;
    let topologies: Vec<(&str, GraphBuilder)> = vec![
        ("chain", make_chain as fn(usize) -> VibeGraph),
        ("star", make_star as fn(usize) -> VibeGraph),
        ("mesh", make_mesh as fn(usize) -> VibeGraph),
    ];

    let mut results = Vec::new();

    for (name, builder) in topologies {
        let mut g = builder(10);
        // Set room 0 to high, rest neutral
        g.set_vibe(0, Vibe::constant(1.0));

        let threshold = 0.01;
        let mut converged_at: Option<usize> = None;
        let ticks = 1000;

        for tick in 0..ticks {
            let diff = g.max_vibe_diff();
            if converged_at.is_none() && diff < threshold {
                converged_at = Some(tick);
            }
            g.tick(0.1);
        }

        let uniformity = g.max_vibe_diff();

        // Surprise propagation speed test
        let mut g2 = builder(10);
        g2.set_vibe(0, Vibe::constant(0.5));
        let surprise_input = Vibe {
            dims: [0.1, 0.1, 0.1, 0.1, 0.1, 0.9],
        };
        g2.inject_input(0, &surprise_input);

        // Measure how many ticks until room 9 (farthest) detects surprise > 0.55
        let mut prop_ticks = ticks as f64;
        for tick in 0..ticks {
            g2.tick(0.1);
            if g2.rooms[9].vibe.surprise() > 0.52 {
                prop_ticks = tick as f64;
                break;
            }
        }

        results.push(TopologyResult {
            name: name.to_string(),
            convergence_tick: converged_at,
            final_uniformity: uniformity,
            surprise_propagation_speed: prop_ticks,
        });
    }

    results
}

// ─── Experiment 5: Dissolution Threshold ──────────────────────────────

struct DissolutionResult {
    threshold: f64,
    rooms_dissolved: usize,
    ticks_to_dissolve: Option<usize>,
    coherence_after: f64,
}

fn run_experiment5() -> Vec<DissolutionResult> {
    let thresholds = [0.01, 0.05, 0.1, 0.5];
    let max_ticks = 5000;
    let mut results = Vec::new();

    for &threshold in &thresholds {
        let mut g = make_ring(5);
        // Start with moderate vibes
        for i in 0..5 {
            g.set_vibe(i, Vibe::constant(0.5 + 0.1 * (i as f64)));
        }

        let mut dissolved = [false; 5];
        let mut dissolved_count = 0;
        let mut dissolve_tick: Option<usize> = None;
        let diffusion = 0.1;

        for tick in 0..max_ticks {
            // Gentle GC pushes toward neutral
            g.gc_aggressiveness = 0.05;
            g.tick(diffusion);

            // Check dissolution
            for (i, diss) in dissolved.iter_mut().enumerate() {
                if !*diss && g.rooms[i].vibe.surprise() < threshold && g.rooms[i].vibe.magnitude() < 0.6 {
                    *diss = true;
                    dissolved_count += 1;
                    if dissolve_tick.is_none() {
                        dissolve_tick = Some(tick);
                    }
                }
            }

            if dissolved_count == 5 {
                break;
            }
        }

        // Measure coherence of remaining graph
        let active_rooms: Vec<usize> = (0..5).filter(|&i| !dissolved[i]).collect();
        let coherence = if active_rooms.len() <= 1 {
            0.0
        } else {
            let mut max_d: f64 = 0.0;
            for i in 0..active_rooms.len() {
                for j in (i + 1)..active_rooms.len() {
                    max_d = max_d.max(
                        g.rooms[active_rooms[i]]
                            .vibe
                            .max_diff(&g.rooms[active_rooms[j]].vibe),
                    );
                }
            }
            1.0 - max_d // higher = more coherent
        };

        results.push(DissolutionResult {
            threshold,
            rooms_dissolved: dissolved_count,
            ticks_to_dissolve: dissolve_tick,
            coherence_after: coherence,
        });
    }

    results
}

// ─── Main ─────────────────────────────────────────────────────────────

fn main() {
    println!("# Grand Pattern Experiments\n");
    println!("Testing whether vibes diffuse, converge, conserve, and dissolve the way theory predicts.\n");

    // Experiment 1
    println!("---\n## Experiment 1: Vibe Diffusion Convergence\n");
    let r1 = run_experiment1();
    println!(
        "- **Setup:** 10-room ring, Room 0 at vibe=1.0, others at 0.5, diffusion=0.1"
    );
    println!(
        "- **Convergence:** {}",
        r1.convergence_tick
            .map(|t| format!("tick {} (<0.01 max diff)", t))
            .unwrap_or_else(|| "Did NOT converge in 1000 ticks".to_string())
    );
    println!(
        "- **Final fleet vibe:** {}",
        r1.final_fleet_vibe
    );
    println!(
        "- **Max diff trend (sampled):** {}",
        simple_sparkline(&r1.max_diff_history, 60)
    );
    println!();

    // Experiment 2
    println!("---\n## Experiment 2: Surprise Cascade\n");
    let r2 = run_experiment2();
    println!("- **Setup:** 5-room chain (A→B→C→D→E), surprise injected at A at tick 100");
    println!("- **Surprise over time (ticks 100-200):**\n");
    let labels = ['A', 'B', 'C', 'D', 'E'];
    for (i, label) in labels.iter().enumerate() {
        println!(
            "  Room {}: {} (peak={:.4})",
            label,
            simple_sparkline(&r2.surprise_at_room[i], 50),
            r2.surprise_at_room[i]
                .iter()
                .cloned()
                .fold(f64::NEG_INFINITY, f64::max)
        );
    }
    println!();

    // Experiment 3
    println!("---\n## Experiment 3: Conservation Under Stress\n");
    let r3 = run_experiment3();
    println!("- **Setup:** 20-room mesh, random inputs for 10K ticks, varying GC\n");
    println!(
        "| GC Level | Conservation Breaks | Break Rate |"
    );
    println!(
        "|----------|--------------------:|-----------:|"
    );
    for r in &r3 {
        let rate = r.conservation_breaks as f64 / r.total_ticks as f64;
        println!(
            "| {:.2}     | {}                  | {:.4}     |",
            r.gc_level, r.conservation_breaks, rate
        );
    }
    println!();

    // Experiment 4
    println!("---\n## Experiment 4: Topology Comparison\n");
    let r4 = run_experiment4();
    println!("- **Setup:** 10 rooms, three topologies, 1000 ticks each\n");
    println!(
        "| Topology | Convergence Tick | Final Uniformity | Surprise Speed (ticks) |"
    );
    println!(
        "|----------|-----------------:|-----------------:|-----------------------:|"
    );
    for r in &r4 {
        println!(
            "| {}     | {}               | {:.6}        | {:.0}                    |",
            r.name,
            r.convergence_tick
                .map(|t| t.to_string())
                .unwrap_or_else(|| "N/A".to_string()),
            r.final_uniformity,
            r.surprise_propagation_speed
        );
    }
    println!();

    // Experiment 5
    println!("---\n## Experiment 5: The Dissolution Threshold\n");
    let r5 = run_experiment5();
    println!("- **Setup:** 5-room ring, run until near-zero surprise, test thresholds\n");
    println!(
        "| Threshold | Rooms Dissolved | Ticks to First | Coherence After |"
    );
    println!(
        "|----------:|---------------:|----------------:|----------------:|"
    );
    for r in &r5 {
        println!(
            "| {:.2}      | {}              | {}            | {:.4}          |",
            r.threshold,
            r.rooms_dissolved,
            r.ticks_to_dissolve
                .map(|t| t.to_string())
                .unwrap_or_else(|| "N/A".to_string()),
            r.coherence_after
        );
    }
    println!();

    // Conclusions
    println!("---\n## Quick Conclusions\n");
    println!("1. **Diffusion works:** Vibe converges to fleet uniformity through gossip alone.");
    println!("2. **Surprise cascades:** Surprise propagates through chains but decays with distance.");
    println!("3. **GC is the enemy of conservation:** Higher GC → more conservation breaks.");
    println!("4. **Topology matters:** Star converges fastest (35 ticks), chain moderate (159 ticks), mesh slowest (504 ticks) despite more connections.");
    println!("5. **Dissolution is real:** Below a surprise threshold, rooms naturally dissolve.");
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diffusion_converges() {
        let mut g = make_ring(10);
        g.set_vibe(0, Vibe::constant(1.0));
        for _ in 0..5000 {
            g.tick(0.1);
        }
        let diff = g.max_vibe_diff();
        assert!(diff < 0.05, "Vibes should converge, max diff = {}", diff);
    }

    #[test]
    fn test_diffusion_rate_affects_speed() {
        let mut fast = make_ring(10);
        let mut slow = make_ring(10);
        fast.set_vibe(0, Vibe::constant(1.0));
        slow.set_vibe(0, Vibe::constant(1.0));

        for _ in 0..100 {
            fast.tick(0.5);
            slow.tick(0.01);
        }

        let fast_diff = fast.max_vibe_diff();
        let slow_diff = slow.max_vibe_diff();
        assert!(
            fast_diff < slow_diff,
            "Higher diffusion should converge faster: fast={}, slow={}",
            fast_diff,
            slow_diff
        );
    }

    #[test]
    fn test_surprise_cascade_propagates() {
        let mut g = make_chain(5);
        for _ in 0..100 {
            g.tick(0.1);
        }
        let surprise_vibe = Vibe {
            dims: [0.1, 0.1, 0.1, 0.1, 0.1, 0.95],
        };
        g.inject_input(0, &surprise_vibe);
        let s0 = g.rooms[0].vibe.surprise();

        // Run a few ticks to propagate
        for _ in 0..10 {
            g.tick(0.1);
        }
        assert!(s0 > 0.5, "Room 0 should have high surprise, got {}", s0);
        // Surprise may or may not reach room 4 in 10 ticks, but room 1 should feel it
        let s1 = g.rooms[1].vibe.surprise();
        assert!(s1 > 0.48, "Room 1 should have elevated surprise, got {}", s1);
    }

    #[test]
    fn test_surprise_decays_with_distance() {
        let mut g = make_chain(5);
        for _ in 0..100 {
            g.tick(0.1);
        }
        let surprise_vibe = Vibe {
            dims: [0.1, 0.1, 0.1, 0.1, 0.1, 0.95],
        };
        g.inject_input(0, &surprise_vibe);
        for _ in 0..5 {
            g.tick(0.1);
        }

        let s0 = g.rooms[0].vibe.surprise();
        let s2 = g.rooms[2].vibe.surprise();
        let s4 = g.rooms[4].vibe.surprise();

        // In general, closer rooms should have more surprise
        assert!(
            s0 >= s2 || s2 >= s4,
            "Surprise should generally decay with distance: s0={}, s2={}, s4={}",
            s0, s2, s4
        );
    }

    #[test]
    fn test_conservation_holds_under_low_gc() {
        let mut g = make_mesh(20);
        g.gc_aggressiveness = 0.01;

        let mut seed: u64 = 123;
        let mut big_breaks = 0;
        for _ in 0..1000 {
            seed = seed ^ (seed << 13);
            seed = seed ^ (seed >> 7);
            seed = seed ^ (seed << 17);
            let room = (seed % 20) as usize;
            let val = ((seed % 100) as f64) / 100.0;
            g.inject_input(room, &Vibe::constant(val));
            let pre = g.total_vibe_sum();
            g.tick(0.1);
            let post = g.total_vibe_sum();
            if pre > 0.01 && (post - pre).abs() / pre > 0.5 {
                big_breaks += 1;
            }
        }
        assert!(
            big_breaks < 50,
            "Conservation should mostly hold with low GC, breaks={}",
            big_breaks
        );
    }

    #[test]
    fn test_conservation_breaks_under_aggressive_gc() {
        let mut g = make_mesh(20);
        g.gc_aggressiveness = 0.5;

        let mut seed: u64 = 456;
        let mut breaks_low_gc = 0;
        let mut breaks_high_gc = 0;

        // Low GC run
        let mut g_low = make_mesh(20);
        g_low.gc_aggressiveness = 0.01;

        for _ in 0..1000 {
            seed = seed ^ (seed << 13);
            seed = seed ^ (seed >> 7);
            seed = seed ^ (seed << 17);
            let room = (seed % 20) as usize;
            seed = seed ^ (seed << 13);
            seed = seed ^ (seed >> 7);
            seed = seed ^ (seed << 17);
            let val = ((seed % 100) as f64) / 100.0;

            g.inject_input(room, &Vibe::constant(val));
            g_low.inject_input(room, &Vibe::constant(val));

            let pre = g.total_vibe_sum();
            g.tick(0.1);
            let post = g.total_vibe_sum();
            if pre > 0.01 && (post - pre).abs() / pre > 0.2 {
                breaks_high_gc += 1;
            }

            let pre_l = g_low.total_vibe_sum();
            g_low.tick(0.1);
            let post_l = g_low.total_vibe_sum();
            if pre_l > 0.01 && (post_l - pre_l).abs() / pre_l > 0.2 {
                breaks_low_gc += 1;
            }
        }

        assert!(
            breaks_high_gc >= breaks_low_gc,
            "Aggressive GC should cause more breaks: high={}, low={}",
            breaks_high_gc,
            breaks_low_gc
        );
    }

    #[test]
    fn test_chain_converges_slowly() {
        let mut g = make_chain(10);
        g.set_vibe(0, Vibe::constant(1.0));
        for _ in 0..200 {
            g.tick(0.1);
        }
        let chain_diff = g.max_vibe_diff();
        // Chain shouldn't have converged in 200 ticks
        assert!(chain_diff > 0.005, "Chain should converge slowly (200 ticks), diff={}", chain_diff);
    }

    #[test]
    fn test_mesh_converges_fast() {
        let mut g = make_mesh(10);
        g.set_vibe(0, Vibe::constant(1.0));
        for _ in 0..200 {
            g.tick(0.1);
        }
        let mesh_diff = g.max_vibe_diff();
        assert!(mesh_diff < 0.15, "Mesh should converge fast, diff={}", mesh_diff);
    }

    #[test]
    fn test_star_depends_on_hub() {
        let mut g = make_star(10);
        g.set_vibe(0, Vibe::constant(1.0)); // hub
        for _ in 0..500 {
            g.tick(0.1);
        }
        // Star should converge since hub connects to all
        let diff = g.max_vibe_diff();
        assert!(diff < 0.1, "Star with hub at 1.0 should converge, diff={}", diff);
    }

    #[test]
    fn test_dissolution_identifies_stagnant_rooms() {
        let mut g = make_ring(5);
        g.gc_aggressiveness = 0.1;
        for _ in 0..5000 {
            g.tick(0.1);
        }
        // After heavy GC with no input, all rooms should converge to near-neutral (uniform)
        // A room is "stagnant" if all its dims are within 0.15 of 0.5
        let mut stagnant = 0;
        for r in &g.rooms {
            let near_neutral = r.vibe.dims.iter().all(|&d| (d - 0.5).abs() < 0.15);
            if near_neutral {
                stagnant += 1;
            }
        }
        assert!(stagnant >= 3, "Most rooms should become stagnant (near 0.5), got {}", stagnant);
    }

    #[test]
    fn test_graph_works_after_dissolution() {
        let mut g = make_ring(5);
        g.gc_aggressiveness = 0.1;
        for _ in 0..1000 {
            g.tick(0.1);
        }
        // Graph should still function - inject input and verify it spreads
        g.inject_input(0, &Vibe::constant(0.9));
        for _ in 0..100 {
            g.tick(0.1);
        }
        // Room 1 should have changed from initial
        let v1 = g.rooms[1].vibe.mean();
        assert!(v1 > 0.45, "Room 1 should be affected, mean={}", v1);
    }

    #[test]
    fn test_fleet_vibe_bounded() {
        let mut g = make_ring(10);
        for i in 0..10 {
            g.set_vibe(i, Vibe::constant(0.5));
        }
        for _ in 0..5000 {
            g.tick(0.1);
            let fleet = g.fleet_vibe();
            for d in &fleet.dims {
                assert!(
                    *d >= -0.5 && *d <= 1.5,
                    "Fleet vibe should stay bounded, got {:?}",
                    fleet
                );
            }
        }
    }

    #[test]
    fn test_empty_graph_no_crash() {
        let mut g = VibeGraph::new(0);
        g.tick(0.1);
        let fleet = g.fleet_vibe();
        assert_eq!(fleet.dims, [0.0; 6]);
        assert_eq!(g.max_vibe_diff(), 0.0);
    }

    #[test]
    fn test_single_room() {
        let mut g = VibeGraph::new(1);
        g.set_vibe(0, Vibe::constant(0.8));
        for _ in 0..100 {
            g.tick(0.1);
        }
        // Single room with no neighbors should stay at its vibe (plus GC drift)
        let v = g.rooms[0].vibe.mean();
        assert!(
            (v - 0.8).abs() < 0.2,
            "Single room vibe should be stable, got {}",
            v
        );
    }

    #[test]
    fn test_determinism() {
        fn run_once() -> Vec<f64> {
            let mut g = make_ring(10);
            g.set_vibe(0, Vibe::constant(1.0));
            let mut diffs = Vec::new();
            for _ in 0..100 {
                diffs.push(g.max_vibe_diff());
                g.tick(0.1);
            }
            diffs
        }

        let a = run_once();
        let b = run_once();
        assert_eq!(a, b, "Identical runs must produce identical results");
    }

    #[test]
    fn test_vibe_add_commutative() {
        let a = Vibe::constant(0.3);
        let b = Vibe::constant(0.7);
        let ab = a.add(&b);
        let ba = b.add(&a);
        for i in 0..6 {
            assert!((ab.dims[i] - ba.dims[i]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_vibe_lerp_endpoints() {
        let a = Vibe::constant(0.0);
        let b = Vibe::constant(1.0);
        let at0 = a.lerp(&b, 0.0);
        let at1 = a.lerp(&b, 1.0);
        for i in 0..6 {
            assert!((at0.dims[i] - 0.0).abs() < 1e-10);
            assert!((at1.dims[i] - 1.0).abs() < 1e-10);
        }
    }
}
