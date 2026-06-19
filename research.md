# UI Layout Research: htop, nvitop, nvidia-smi, jtop

## 1. htop — Classic Linux Process Viewer

### Overall Screen Layout (3 zones)

```
┌──────────────────────────────────────────────────────────────┐
│  ┌─ CPU[###########        ] ┌─ Mem[######   ########]     │  ← HEADER (meters)
│  │ CPU1[####      ]  48.2%   │ Task: 245, 987 thr; 3 running │
│  │ CPU2[######    ]  62.1%   │ Load avg: 2.14 1.98 1.76     │
│  │ CPU3[###       ]  32.8%   │ Uptime: 12:45:32              │
│  └───────────────────────────┴───────────────────────────────┘
│  PID  USER  PRI  NI  VIRT   RES    SHR  S CPU% MEM%  TIME+  │  ← PROCESS TABLE
│  1234 jlu    20   0  1.2G  240M   80M  S  5.2  3.1  0:12.34 │     (sortable columns)
│  5678 root   20   0  512M   64M   32M  S  0.0  0.8  2:34.56 │
│  ...                                                         │
├──────────────────────────────────────────────────────────────┤
│ Help Setup Search Filter Tree Sort Nice Nice Kill  Quit     │  ← FOOTER (F1-F10)
│ F1   F2    F3     F4    F5   F6   F7  F8   F9   F10        │
└──────────────────────────────────────────────────────────────┘
```

### Header Meters

- **CPU meters** — per-core bars or overall average. Color-coded segments:
  - Green = user processes
  - Blue = low-priority (nice) threads
  - Red = kernel time / IRQs
  - Yellow/Orange = steal + guest time (virtualization)
- **Memory meter** — bar with segments:
  - Green = used memory
  - Blue = buffers
  - Orange/Yellow = cache
  - Grey = free/unused
  - Magenta = shared memory (optional view)
- **Swap meter** — blue = used, orange = cache, grey = frontswap
- **Tasks** — total processes, threads, running count
- **Load average** — 1/5/15 minute averages
- **Uptime** — system uptime

### Process Table (default columns)

| Column | Meaning |
|--------|---------|
| PID | Process ID |
| USER | Owner username (magenta=root/elevated, bold gray=other users, white=current user) |
| PRI | Kernel priority (nice+20) |
| NI | Nice value (19=low, -20=high priority) |
| VIRT | Virtual memory size |
| RES | Resident set size (physical RAM) |
| SHR | Shared pages size |
| S | State: S(sleep), I(idle), R(run), D(disk sleep), Z(zombie), T(traced) |
| CPU% | Per-core CPU percentage |
| MEM% | Percentage of physical RAM |
| TIME+ | Accumulated CPU time |
| COMMAND | Command line (magenta=normal, bold blue=userland thread, bold gray=kernel thread) |

### Interactive Features

| Key | Action |
|-----|--------|
| F1 / h / ? | Help screen |
| F2 / S | Setup: configure meters, display options, colors, columns |
| F3 / / | Incremental command-line search |
| F4 / \ | Filter by command line substring (pipe `|` for OR) |
| F5 / t | Toggle tree view (parent-child hierarchy; `+`/`-` collapse/expand, `*` expand all) |
| F6 / < / > | Select sort column from a picker |
| F7 / ] | Increase priority (niceness) |
| F8 / [ | Decrease priority |
| F9 / k | Kill: send any signal to tagged or selected process |
| F10 / q | Quit |
| Space | Tag/untag a process |
| c | Tag process + its children |
| U | Untag all |
| I | Invert sort order |

### Color Scheme Highlights

- **CPU bars**: Green / Blue / Red / Yellow segments
- **Memory bars**: Green / Blue / Orange / Grey segments
- **Command**: Magenta (regular), Bold Blue (userland thread), Bold Gray (kernel thread)
- **USER**: Magenta (elevated), Bold Gray (other), White (self)
- **EXE column**: Red when executable was replaced/deleted; yellow when library was replaced
- Configurable themes via F2 Setup (dark, light, default, monochrome, etc.)

---

## 2. nvitop — NVIDIA GPU Process Viewer

### Overall Screen Layout (monitor mode)

```
┌──────────────────────────────────────────────────────────────┐
│  ┌── GPU 0: NVIDIA A100 ──────────────────────────────┐     │
│  │ Fan: 45%  Temp: 62C  GPU-Util: 78%  Pwr: 200W/250W│     │  ← PER-GPU HEADER
│  │ Mem: 8862MiB / 11019MiB  Mem-Util: 42%            │     │
│  │ Press ^C(INT)/T(TERM)/K(KILL) to send signals      │     │
│  └────────────────────────────────────────────────────┘     │
│  PID  USER    CPU%  HOST-MEM  TIME    GPU-MEM   SM%   CMD  │  ← PROCESS TABLE
│  7890 jlu      12.3   240M    2:34.5  4862MiB   65%  python│
│  7891 root      0.0    64M    0:12.3     32MiB    2%  bash │
│  ...                                                         │
├──────────────────────────────────────────────────────────────┤
│ h:help  q:quit  /:sort  t:tree  e:env  Enter:graphs         │  ← STATUS BAR
│ Display: FULL  Sort: SM%  Order: DESC  Tagged: 0             │
└──────────────────────────────────────────────────────────────┘
```

### Per-GPU Header Metrics

| Metric | Description |
|--------|-------------|
| Device name + index | e.g., "NVIDIA A100-SXM4-40GB" |
| Fan speed | Percentage (0-100%) |
| Temperature | Celsius |
| GPU utilization | Streaming Multiprocessor utilization % |
| Memory usage | Used MiB / Total MiB |
| Memory utilization | Bandwidth utilization % |
| Power usage | Instant watts / power limit watts |

### Process Table Columns

| Column | Description |
|--------|-------------|
| PID | Process ID |
| USER | Username |
| CPU% | Host CPU utilization |
| HOST-MEM | Host memory usage |
| TIME | Running time duration |
| GPU-MEM | GPU memory consumed |
| SM% | Streaming Multiprocessor utilization |
| COMMAND | Executable command line |

### Display Modes

| Mode | Description |
|------|-------------|
| Full (f) | Maximum detail, scrollable |
| Compact (c) | Minimal view for small terminals |
| Auto (a) | Adapts to terminal size automatically |

### Interactive Features

| Key | Action |
|-----|--------|
| Up/Down / Alt-j/k / Tab / Mouse | Select/highlight a process |
| Space | Tag/untag process |
| Esc | Clear all selections |
| h / ? | Help screen |
| q | Quit |
| Ctrl-c / I | Send SIGINT |
| T | Send SIGTERM |
| K | Send SIGKILL |
| e | Show environment variables |
| t | Toggle tree view (GPU processes + ancestors) |
| Enter | Show process metrics with live graphs |
| r / Ctrl-r / F5 | Force refresh |
| , / . | Select sort column |
| / | Reverse sort order |
| oN/oU/oP/oG/oS/oC/oM/oT | Sort by Name/User/PID/GPU-Mem/SM%/CPU%/MEM%/Time |
| Left/Right | Scroll horizontally |

### Color Scheme

- **Bar charts**: Spectrum-like, color intensity indicates load:
  - Light = below threshold 1 (low load)
  - Moderate = between thresholds (medium load)
  - Heavy = above threshold 2 (high load)
- **Thresholds**: Configurable via CLI flags / env vars
- **Box drawing**: Fancy Unicode box characters (can switch to ASCII with `--no-unicode`)
- **Dimmed**: Other users' processes shown dimmed
- **Themes**: `--colorful` (default for dark terminals), `--light` (for light backgrounds)

---

## 3. nvidia-smi --loop — Basic GPU Monitoring

### Output Layout (default human-readable, no interactive mode)

```
┌──────────────────────────────────────────────────────────────────┐
│ NVIDIA-SMI 580.95.05        Driver Version: 580.95.05            │
│ CUDA Version: 13.0                                                │
├──────────────────────────────────────────────────────────────────┤
│ GPU  Name            Persistence-M| Bus-Id        Disp.A | Vol.  │
│ Fan  Temp  Perf  Pwr:Usage/Cap|         Memory-Usage | GPU-Util│
├══════════════════════════════════════════════════════════════════┤
│  0   NVIDIA A100          On   | 00000000:00:04.0 Off |      0  │
│ 45%   62C    P0   200W / 250W |   8862MiB / 11019MiB |     78%  │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│ Processes:                                                       │
│  GPU   PID   Type   Process name                   GPU Memory    │
│    0   7890   C+G   python3                           4862MiB    │
└──────────────────────────────────────────────────────────────────┘
```

### Header Info (non-repeating)

| Field | Description |
|-------|-------------|
| NVIDIA-SMI version | Tool version |
| Driver Version | Installed NVIDIA driver |
| CUDA Version | Supported CUDA toolkit version |

### Per-GPU Table (rows repeated per GPU)

**Row 1 (GPU identity):**

| Column | Description |
|--------|-------------|
| GPU | Index number (0, 1, 2...) |
| Name | GPU model name |
| Persistence-M | Persistence mode (On/Off) |
| Bus-Id | PCI bus ID |
| Disp.A | Display active (On/Off) |
| Volatile Uncorr. ECC | Uncorrectable ECC error count |

**Row 2 (metrics):**

| Column | Description |
|--------|-------------|
| Fan | Speed percentage (N/A if fanless) |
| Temp | Temperature Celsius |
| Perf | Performance state (P0=best, P12=worst) |
| Pwr:Usage/Cap | Watts usage / limit |
| Memory-Usage | Used MiB / Total MiB |
| GPU-Util | Utilization percentage |
| Compute M. | Compute mode |
| MIG M. | Multi-Instance GPU mode |

### Process Table (bottom section)

| Column | Description |
|--------|-------------|
| GPU | GPU index |
| PID | Process ID |
| Type | C (Compute), G (Graphics), C+G |
| Process name | Executable name/path |
| GPU Memory | Memory consumed (MiB) |

### --loop Behavior

- `nvidia-smi --loop=1` re-queries and re-prints the entire output table every N seconds
- `nvidia-smi --loop-ms=500` same but in milliseconds
- **Not interactive** — pure stdout reprint (no curses/TUI)
- Terminal scrolls with each refresh; no fixed-position layout
- `nvidia-smi --query-gpu=... --format=csv --loop=1` for machine-parseable output
- Minimal color: basic ANSI terminal colors, no structured UI

### Key queryable GPU attributes (via `--query-gpu`)

```
name, index, uuid, utilization.gpu, utilization.memory,
temperature.gpu, temperature.memory,
power.draw, power.draw.average, power.limit,
clocks.gr, clocks.sm, clocks.mem, clocks.video,
pcie.link.gen.current, pcie.link.width.current,
memory.total, memory.used, memory.free, memory.reserved,
ecc.errors.corrected.volatile.total, ecc.errors.uncorrected.volatile.total
```

---

## 4. jtop — Jetson GPU/System Monitor

### Overall Screen Layout (tabbed TUI, 7 tabs)

```
┌──────────────────────────────────────────────────────────────┐
│  jtop v4.x — NVIDIA Jetson Orin (JetPack 6.0)               │  ← HEADER (always visible)
│  ┌─ ALL ── GPU ── CPU ── MEM ── ENG ── CTRL ── INFO ──┐    │
│  │                                                      │    │
│  │  [Tab content area]                                  │    │  ← TAB CONTENT
│  │                                                      │    │
│  │  GPU Processes:                                      │    │
│  │  PID  NAME       GPU-MEM  UTIL                       │    │
│  │  7890 python3    4862MiB  65%                        │    │
│  │                                                      │    │
│  └──────────────────────────────────────────────────────┘    │
│  q:quit  Tab:switch  Arrows:navigate  [hint per tab]        │  ← STATUS/HELP
└──────────────────────────────────────────────────────────────┘
```

### Tab 1: ALL — Dashboard Overview

Gives a single-screen summary of everything:

- **CPU**: Per-core load percentages (bar chart)
- **Memory**: RAM/Swap/EMC/Iram usage bars
- **GPU**: GPU load % + current frequency
- **Disk**: Disk usage (used / total)
- **Fan**: Fan speed RPM / PWM
- **Status indicators**: jetson_clocks on/off, NVPmodel
- **Temperatures**: with warning (>=84C) and critical (>=100C) thresholds
- **Power rails**: Instant and average power draw
- **HW Engines**: Status of hardware blocks
- **GPU process table**: PID, name, GPU memory, utilization

### Tab 2: GPU — GPU-specific View

- Real-time 10-second history chart of GPU load
- iGPU name and load percentage
- Current GPU governor
- GPU shared RAM bar (grey = total used)
- GPU flags:
  - 3D Scaling (enable/disable)
  - Railgate (power gating)
  - Power control
  - TPC PG (TPC power gate)
- Min/max/current GPU frequency + GPC frequency
- **GPU processes table** (sortable by clicking column headers)

### Tab 3: CPU — CPU-specific View

- Real-time per-core CPU usage chart
- Color coding: Green (user), Yellow (nice), Red (system)

### Tab 4: MEM — Memory-specific View

- Real-time memory usage chart
- Swap monitor
- Bar colors: Cyan (used RAM), Green (GPU shared), Blue (buffers), Yellow (cached), Red (swap)
- Keyboard: `c`=clear cache, `h`=toggle extra swap, `+`/`-`=adjust swap size

### Tab 5: ENG — Hardware Engines

- Status of hardware acceleration engines:
  - DLA (Deep Learning Accelerator)
  - VIC (Video Image Compositor)
  - PVA (Programmable Vision Accelerator)
  - NVENC/NVDEC (video encode/decode)
- Available on Orin/Xavier and newer boards

### Tab 6: CTRL — Control Panel

- **jetson_clocks**: Enable/disable (s/a key); enable on boot (e key)
- **NVPmodel**: Select power mode (`+`/`-`); modes needing reboot shown in amber, "D" marks default
- **Fan**: Toggle manual vs jetson_clocks mode (f key); adjust speed (p/m keys)
- Real-time fan chart (RPM/PWM)
- Power/voltage/current table with warning + critical current limits

### Tab 7: INFO — System Information

- Hardware model
- L4T (Linux for Tegra) version
- JetPack version
- CUDA, cuDNN, TensorRT versions
- Serial number
- Network interfaces
- Boot information
- Library versions

### Interactive Features

| Key | Context | Action |
|-----|---------|--------|
| Tab / Arrow keys | Global | Switch between 7 tabs |
| q | Global | Quit jtop |
| Click | GPU/MEM tables | Sort by column header |
| c | MEM tab | Clear cache |
| h | MEM tab | Toggle extra swap |
| + / - | MEM tab | Increase/decrease swap size |
| s / a | CTRL tab | Start/Stop jetson_clocks |
| e | CTRL tab | Toggle jetson_clocks on boot |
| + / - | CTRL tab | Change NVPmodel (power mode) |
| f | CTRL tab | Toggle fan mode (manual vs auto) |
| p / m | CTRL tab | Increase/decrease fan speed |

### Color/Visual Scheme

- **Tab labels**: Highlighted when active, dimmed when inactive
- **GPU bar**: Grey for shared RAM usage
- **CPU chart**: Green (user), Yellow (nice), Red (system)
- **Memory chart**: Cyan (RAM), Green (GPU shared), Blue (buffers), Yellow (cache), Red (swap)
- **Temperature**: Warning at >=84C (amber), Critical at >=100C (red)
- **NVPmodel**: Amber for modes needing reboot
- **Default mode**: Marked with "D"
- **Process highlight**: Selected row highlighted / inverted

---

## Comparison Summary Table

| Feature | htop | nvitop | nvidia-smi --loop | jtop |
|---------|------|--------|-------------------|------|
| Type | System process viewer | GPU process viewer | CLI query tool | Jetson system monitor |
| UI style | TUI (ncurses) | TUI (ncurses) | CLI stdout | TUI (ncurses) |
| Interactive | Yes | Yes | No | Yes |
| CPU monitoring | Per-core, aggregate | Host CPU% per process | No | Per-core charts |
| GPU monitoring | No | Comprehensive per-GPU | Basic metrics | Full Jetson GPU/SoC |
| Memory | System RAM + swap | GPU memory + host memory | GPU memory only | RAM + GPU shared + swap |
| Process management | Kill, renice, tag, filter | Kill signals (INT/TERM/KILL) | No | No process kill |
| Sorting | F6 column picker, I invert | Multiple `o*` shortcuts | N/A | Click column headers |
| Filtering | F4 substring filter | By user/PID | N/A | N/A |
| Tree view | F5 parent-child tree | t key GPU process tree | N/A | N/A |
| Colors | Polychrome bars + colored text | Spectrum bar charts + dimming | Minimal ANSI | Tab-themed bars |
| Customizable | F2 Setup (meters, columns, themes) | Themes (colorful/light), thresholds | --format=csv | N/A (fixed layout) |
| Config files | ~/.config/htop/htoprc | N/A | N/A | N/A |
| Mouse support | Yes | Yes | No | Yes |
| Help screen | F1 | h / ? | --help only | Per-tab hints |

## Key Design Takeaways for a GPU Top Tool

1. **Three-zone layout** (header / process table / footer) from htop is the gold standard.
2. **Per-GPU header cards** (nvitop style) with fan, temp, util, memory, power are essential.
3. **Process table** should show PID, USER, CPU%, GPU-MEM, SM%, COMMAND at minimum.
4. **Sorting** by any column (SM%, GPU-MEM, CPU%, PID) with visual indicator.
5. **Kill signals** with SIGINT/SIGTERM/SIGKILL keyboard shortcuts (nvitop style).
6. **Filtering** by command/substring (htop F4 style).
7. **Tree view** of GPU process hierarchies (nvitop style).
8. **Color coding**: spectrum bars for load (nvitop), colored process types (htop), threshold-based warnings.
9. **Mouse support** for selection and column sorting.