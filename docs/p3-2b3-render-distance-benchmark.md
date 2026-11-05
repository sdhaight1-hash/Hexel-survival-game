# P3.2B-3: full-resolution render-distance benchmark

This is an opt-in **full-resolution resident-radius** benchmark, not an LOD
algorithm and not a GPU timestamp benchmark.

## Run

From the repository root after pulling the merged PR:

```cmd
cargo run --release -- --torus-test-ne --render-benchmark --present-mode mailbox --fps-cap 60
```

Keep moving in a straight line at normal walking speed for the full **3m30s**.
The deterministic seven-stage sequence is **R2, R4, R3, R2, R3, R4, R2**.
Each stage lasts 30 seconds. A radius R has `5×(1+3R(R+1))` expected
full-resolution sections: R2=95, R3=185, R4=305.

Each switch reports `RENDER BENCH SWITCH` in the terminal, F2 log and
`target/benchmark-session.log`. Completion reports `RENDER BENCH COMPLETE`.
Stage summaries are written progressively to
`target/render-distance-benchmark.csv`; the CSV is flushed at each stage so
interruptions retain completed data. Existing nonbenchmark modes use radius R2.

## Interpreting the results

The first **5 seconds**, plus all periods with pending streaming sections, are
excluded from steady frame-interval percentiles. The log reports a
`VALID` stage only if at least 100 settled frame samples and 50 moving
samples were recorded. Otherwise it reports insufficient movement or settling.

`tickFPS`, p95, p99 and maximum time refer to **main-loop update intervals**,
not measured GPU frame execution. CPU generation/meshing, resident sections,
uploaded MiB and player travel distance are tracked separately. Render
distance swaps themselves contribute to generation/meshing totals and pending
peaks but are excluded from settled-frame statistics.

The order is a fixed reproducible pseudo-shuffle with repeated R2 baselines.
A seeded random generator is unnecessary: compare identical stages at
different positions while moving, but do not mistake the sequence for a
controlled identical-scene rendering benchmark. The player's movement and
terrain elevation remain variables.

## Required results

Upload `target/render-distance-benchmark.csv` and
`target/benchmark-session.log`, plus observations of stutter or visible
holes after transitions. The streamer is currently a 95-section radius-2
prototype; larger windows can stress CPU memory/meshing, so do not interpret
one GPU draw-duration number as total cost.
