# Tetris (Rust) — Graphics + Text Mode

A **two-player, competitive Tetris-style** game written in Rust.

This repo includes:
- **Graphics mode (recommended)** powered by `macroquad` (portable windowed 2D rendering)
- **Text mode** (terminal) with a full **command interpreter** (multipliers, aliases, macros, script playback)

> If you’re coming from the original “BiQuadris”-style assignment: this project mirrors that gameplay structure (two boards, special actions, levels, scripts, scoring), but the project name + UI text are **Tetris**.

---

## Features

### Core gameplay
- **Two simultaneous boards** (Player 1 vs Player 2)
- Standard tetrominoes: **I, J, L, O, S, T, Z**
- **Next piece preview**
- **Ghost / phantom piece** (landing outline)
- **Levels 0–4**:
  - Level 0: scripted sequence (repeat loop)
  - Levels 1–4: weighted/random generation (logic supported; interactive toggles are in text mode)
  - Higher levels can be “heavy” (extra gravity behavior)

### Special actions (competitive)
After a player clears **≥ 2 rows on a drop**, they can apply a special action to the opponent:
- **blind**: hides a central region of the opponent board
- **heavy**: opponent’s next horizontal move causes extra forced drops
- **force**: force opponent’s current falling block to a specific type (I/J/L/S/T/O/Z)

### Two ways to play
- **Graphics**: play with keyboard controls in a window
- **Text**: play by typing commands (`left`, `drop`, `3down`, macros, etc.)

---

## Requirements

- Rust toolchain (edition 2021) with `cargo`
- For graphics mode: a machine that can open a window (OpenGL-capable environment)

If you’re on Linux and see missing system-lib errors (X11/GL/ALSA), install the typical dev packages for windowing + OpenGL + audio on your distro.

---

## Quick Start

### 1) Unzip and enter the folder
```bash
unzip tetris_rust_graphics.zip
cd tetris

