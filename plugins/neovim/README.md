# Hitchmark — Neovim Plugin

Stable `hook://` URI links and purple paragraph IDs for Neovim.

Mirrors the same HTTP-first + subprocess fallback pattern as the VS Code and Obsidian plugins — works with or without `hk serve` running.

## Installation

**lazy.nvim:**
```lua
{
  "elijah/hitchmark",
  config = function()
    require("hitchmark").setup()
  end,
}
```

**packer.nvim:**
```lua
use {
  "elijah/hitchmark",
  config = function()
    require("hitchmark").setup()
  end,
}
```

**vim-plug:**
```vim
Plug 'elijah/hitchmark'
" then in your init.lua:
lua require("hitchmark").setup()
```

## Requirements

- Neovim 0.8+
- `hk` CLI on `$PATH` (or configured via `hk_path`)
- Optional: `hk serve` running for faster HTTP transport

## Commands

| Command | Default keymap | Description |
|---------|---------------|-------------|
| `:HkFile` | `<leader>hf` | Copy `hook://` URI for current file to `+` register |
| `:HkLink` | `<leader>hl` | Link current file to a URI (prompted) |
| `:HkList` | `<leader>hL` | Show all links for current file in quickfix list |
| `:HkPurple` | `<leader>hp` | Annotate buffer with purple number virtual text |
| `:HkOpen` | `<leader>ho` | Open `hook://` URI under cursor or prompted |

## Configuration

```lua
require("hitchmark").setup({
  -- Base URL for hk serve (default: "http://127.0.0.1:2701")
  serve_url = "http://127.0.0.1:2701",

  -- Explicit path to hk binary (default: auto-detect from $PATH and common locations)
  hk_path = nil,

  -- Keymaps (set to false to disable all, or override individual keys)
  keymaps = {
    file   = "<leader>hf",
    link   = "<leader>hl",
    list   = "<leader>hL",
    purple = "<leader>hp",
    open   = "<leader>ho",
  },
})
```

## How it works

1. **HTTP first** — if `hk serve` is running at `serve_url`, all operations use the REST API (fast, no subprocess overhead)
2. **Subprocess fallback** — if the server is not running, each operation shells out to `hk` directly
3. **Purple numbers** — `:HkPurple` fetches paragraph IDs and renders them as EOL virtual text using `nvim_buf_set_extmark`; they don't modify the file

## Running tests

Requires [plenary.nvim](https://github.com/nvim-lua/plenary.nvim):

```bash
nvim --headless \
  -u plugins/neovim/tests/minimal_init.lua \
  -c "PlenaryBustedFile plugins/neovim/tests/bridge_spec.lua"
```
