# Development

## Prerequisites

- Node.js 20+
- `pnpm`
- Rust toolchain
- Tauri prerequisites for your platform
- Discord desktop client if you want Rich Presence sync

## Run in Development

```bash
corepack enable
pnpm install
pnpm tauri dev
```

## Build

Build the frontend bundle:

```bash
pnpm build
```

Build desktop packages:

```bash
pnpm tauri build
```

## Stack

- Tauri 2
- React 19
- TypeScript
- Rust
- DaisyUI + Tailwind CSS

