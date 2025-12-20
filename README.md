# Tetris Sim (Rust) — How to Build & Play

I built this project as a Rust implementation of **Tetris** with **two modes**:

- **Graphics mode (recommended):** plays in a window (keyboard controls)
- **Text mode:** plays in the terminal (full command interpreter: multipliers, aliases, macros, scripts)

---

## 1) Requirements

You need the Rust toolchain:

```bash
cargo --version
rustc --version
```

If you don’t have Rust installed, install it with `rustup` (official Rust installer), then reopen your terminal.

---

## 2) Build with Cargo

From the project root (the folder that contains `Cargo.toml`):

```bash
cargo build
```

---

## 3) Run

This repo contains **two binaries**, so you must specify one with `--bin`.

### Run: Graphics Mode (windowed)
```bash
cargo run --bin tetris -- -startlevel 0 -scriptfile1 tetris_sequence1.txt -scriptfile2 tetris_sequence2.txt
```

### Run: Text Mode (terminal / command interpreter)
```bash
cargo run --bin text -- -startlevel 0 -scriptfile1 tetris_sequence1.txt -scriptfile2 tetris_sequence2.txt
```

---

## 4) Command-line Flags (both modes)

- `-startlevel <n>`: starting level (`0..4`)
- `-seed <n>`: RNG seed (optional; useful if you want reproducible randomness)
- `-scriptfile1 <file>`: sequence file for Player 1 (used at level 0)
- `-scriptfile2 <file>`: sequence file for Player 2 (used at level 0)

Example:
```bash
cargo run --bin tetris -- -startlevel 3 -seed 123
```

### Defaults
- If you start the program **with no flags**, it defaults to **Level 0** (`-startlevel 0`).
- For **Level 0**, the program uses **`tetris_sequence1.txt`** and **`tetris_sequence2.txt`** as the default sequence files.

---

## 5) Graphics Mode Controls (play in the window)

Graphics mode is **not** played by typing commands into the terminal.  
You launch it from the terminal, then **play using your keyboard in the game window**.

### Movement / Rotation
- Move left: **Left Arrow** or **A**
- Move right: **Right Arrow** or **D**
- Soft drop (down 1 step): **Down Arrow** or **S**
- Rotate CCW: **Q**
- Rotate CW: **E**
- Hard drop: **Space**

### Level / Game Control
- Level up: **PageUp** or **Fn + Up Arrow** for Mac users
- Level down: **PageDown** or **Fn + Down Arrow**
- Restart: **R**
- Quit: **Esc**

### Special Actions (after clearing ≥ 2 lines on a drop)
When you clear **2 or more lines** on a drop, the window will prompt you for a special action:

- **B** = blind  
- **H** = heavy  
- **F** = force → then press **I / J / L / S / T / O / Z**

---

## 6) Text Mode — Full Command List

Text mode is the “type commands” version. You type tokens and press Enter.

### Movement / Rotation
- `left`
- `right`
- `down`
- `cw`
- `ccw`
- `drop`

### Levels
- `levelup`
- `leveldown`

### Game / Flow
- `restart`
- `quit`

### Force the current block (manual override)
These commands replace the **currently falling** block:

- `I`
- `J`
- `L`
- `S`
- `T`
- `O`
- `Z`

### Random / no-random (levels 3 and 4)
- `random`  
  Enables random generation (levels 3–4).

- `norandom <filename>`  
  Uses a fixed sequence file instead of random generation (levels 3–4).

### Sequence scripts (stacked input)
- `sequence <filename>`  
  Reads commands from a file. When the file ends, input returns to the previous input source.

### Aliases (create an alias for an existing command)
- `rename <newname> <existingcommand>`

Example:
```text
rename a left
a
```

### Macros (define a command that expands into multiple tokens)
- `macro <name> <sequence-of-commands>`

Example:
```text
macro zig left left down cw
zig
```

### Multipliers (repeat commands)
You can repeat many commands by adding a number:

- Prefix form: `3left`, `2cw`, `5down`
- Suffix form: `left3`, `cw2`, `down5`

Notes:
- Multipliers are **ignored** for:
  - `drop`, `restart`, `quit`, `sequence`, `random`, `norandom`, `macro`

---

## 7) Sequence File Format (level 0 / norandom)

A sequence file is whitespace-separated block letters, for example:

```text
I J L S Z T O
```

Level 0 loops this sequence forever.

---