# CONCLUSIONS.md — Grand Pattern Experiment Findings

## Experiment 1: Vibe Diffusion Convergence
- **Result:** ✅ Vibes converge to fleet uniformity via gossip alone.
- A 10-room ring with one room at max vibe (1.0) and the rest at neutral (0.5) converged to within 0.01 max difference by tick ~460.
- The final fleet vibe settled at ~0.572 across all dimensions — slightly above the neutral midpoint because the high-vibe room contributed more mass than the initial average.
- The diffusion curve is exponential decay: fast initial leveling, then slow asymptotic approach.

## Experiment 2: Surprise Cascade
- **Result:** ✅ Surprise propagates through chains and decays with distance.
- Room A (injection point) peaked at 0.635 surprise.
- Room B peaked at 0.573 — a measurable but attenuated propagation.
- Rooms C, D, E showed progressively lower peaks (0.524, 0.509, 0.508).
- The propagation wave is visible in the sparklines: a time-delayed bell curve that flattens with distance.
- **Key insight:** Surprise does cascade, but each hop attenuates the signal. This matches the theory's prediction that information degrades as it diffuses through intermediate rooms.

## Experiment 3: Conservation Under Stress
- **Result:** ⚠️ Conservation holds well across all tested GC levels.
- Even at GC aggressiveness of 0.5, no conservation breaks (>20% total vibe change in one tick) were observed.
- This suggests that the GC mechanism in our implementation smoothly decays vibes rather than causing abrupt discontinuities.
- **Caveat:** Our "conservation" metric measures tick-to-tick stability, not strict conservation of total vibe mass. GC inherently reduces total mass — the question is whether it does so smoothly or chaotically. Answer: smoothly.
- **Key insight:** There may not be a sharp threshold where conservation "breaks" — instead, GC causes gradual erosion. The practical question becomes: how much erosion is acceptable?

## Experiment 4: Topology Comparison
- **Result:** ✅ Topology dramatically affects convergence speed and surprise propagation.
- **Star topology:** Converged fastest (tick 35). The hub room acts as a super-spreader, quickly averaging all vibes. Surprise propagation is essentially instant (1 tick to reach the farthest room).
- **Chain topology:** Slow convergence (tick 159). Information has to hop through every intermediate room. Surprise propagation was so slow it didn't reach the end in 1000 ticks.
- **Mesh topology:** Surprisingly slow convergence (tick 504) despite full connectivity. This is because each room diffuses with ALL others simultaneously, creating a damping effect — too many neighbors pulling in different directions.
- **Key insight:** Star topology learns fastest but is most fragile (hub failure kills the graph). Chain is robust but slow. Mesh is robust but noisy. The sweet spot is likely a small-world graph.

## Experiment 5: The Dissolution Threshold
- **Result:** ⚠️ Dissolution didn't trigger under our test conditions.
- Even at threshold 0.50, no rooms dissolved. The GC mechanism pushes vibes toward neutral (0.5), but they never dip below the thresholds because diffusion keeps pulling them back toward the fleet average.
- This suggests dissolution requires more extreme conditions: either much higher GC, isolation (no neighbors), or explicit room death mechanisms.
- **Key insight:** In a connected graph with moderate GC, dissolution is hard to trigger naturally. Rooms sustain each other through gossip. Dissolution likely requires structural isolation (edges removed) or intentional shutdown, not just low surprise.

## Meta-Conclusions

1. **Diffusion is real and works as predicted.** Gossip-based vibe sharing converges to fleet uniformity given enough time.

2. **Surprise cascades attenuate with distance.** This is both a feature (prevents runaway cascades) and a limitation (distant rooms learn slowly).

3. **GC is smooth, not catastrophic.** There's no sharp conservation-breaking threshold — GC causes gradual erosion.

4. **Topology is destiny.** The graph structure determines learning speed, surprise propagation, and robustness more than any parameter tuning.

5. **Dissolution requires more than just low vibes.** In a connected graph, rooms sustain each other. True dissolution needs structural changes (edge removal, isolation).

6. **Determinism confirmed.** Identical runs produce identical results — the system is fully deterministic, which is essential for reproducible experiments.

## What's Next?
- Test small-world topologies (ring + random shortcuts) for the "sweet spot"
- Test dissolution under edge removal (what happens when a room loses all neighbors?)
- Test non-uniform diffusion coefficients (some edges are "wider" than others)
- Test adversarial inputs (can a single room disrupt fleet convergence?)
