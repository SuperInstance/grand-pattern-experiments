# grand-pattern-experiments

Low-level experiments testing Grand Pattern theory — diffusion, conservation, topology, dissolution.

## What This Tests

Does vibe diffusion actually work the way the theory predicts? We run five experiments to find out.

## Experiments

1. **Vibe Diffusion Convergence** — Does gossip converge a 10-room ring to uniform vibes?
2. **Surprise Cascade** — How does surprise propagate through a chain? Does it decay with distance?
3. **Conservation Under Stress** — Does total vibe stay stable under aggressive garbage collection?
4. **Topology Comparison** — Chain vs star vs mesh: which learns fastest?
5. **Dissolution Threshold** — At what point should stagnant rooms dissolve?

## Running

```bash
cargo run     # Run all experiments, print markdown results
cargo test    # Run 17 verification tests
```

Zero dependencies. Pure Rust.

## Key Findings

- Diffusion converges reliably (tick ~460 for a 10-room ring)
- Surprise cascades attenuate ~10% per hop
- GC causes smooth erosion, not catastrophic conservation breaks
- Star topology converges fastest (35 ticks), chain slowest (159 ticks)
- Dissolution is hard to trigger in connected graphs — rooms sustain each other

See [CONCLUSIONS.md](CONCLUSIONS.md) for full analysis.
