# gputop

A GPU monitoring TUI written in Rust, inspired by `nvidia-smi`, `jtop`, and `nvitop`, styled after `htop`.

## Features (planned)

- Real-time GPU utilization, memory usage, temperature, and power draw
- Per-process GPU usage breakdown
- Interactive process management (sort, filter, kill)
- CPU / memory / load overview alongside GPU stats
- Color-coded bars and gauges in htop style
- Mouse support for scrolling and selection
- Configurable refresh interval
- Multi-GPU support

## Getting Started

```bash
cargo run
```

## Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) — TUI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) — terminal backend
- [nvml-wrapper](https://github.com/Cldfire/nvml-wrapper) — NVIDIA GPU management library
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) — system information

## License

MIT