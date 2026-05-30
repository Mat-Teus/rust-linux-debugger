# Rust Linux Debugger

A simple Linux debugger written in Rust using `ptrace`.

This project is being developed as a low-level systems programming study project focused on:
- Linux internals
- process control
- debugging concepts
- memory manipulation
- breakpoints
- ELF/DWARF exploration

Inspired by the "Writing a Linux Debugger" series.

---

## Features

Current features:
- Process tracing with `ptrace`
- Fork/exec debugger architecture
- Interactive debugger prompt
- Continue execution command
- Process synchronization using `waitpid`
- Software Breakpoints with INT3
- Register Inspection
- Memory Reading Writing 

Planned features:
- ELF parsing
- DWARF debug symbols
- Single stepping
- Stack tracing

---

## Technologies

- Rust
- Linux
- ptrace
- waitpid
- execv

---

## Project Structure

```text
Parent Process (Debugger)
        |
        | ptrace
        v
Child Process (Debugee)
