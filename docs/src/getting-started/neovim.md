# Neovim Plugin

The Hitchmark Neovim plugin provides `hook://` URI management and purple-number annotations directly inside Neovim. It communicates with `hk serve` over HTTP when the server is running, and falls back to shelling out to `hk` when it isn't.

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
```
Then in `init.lua`:
```lua
require("hitchmark").setup()
```

## Commands

| Command | Default keymap | Description |
|---------|---------------|-------------|
| `:HkFile` | `<leader>hf` | Copy `hook://` URI for the current file to `+` register |
| `:HkLink` | `<leader>hl` | Link current file to a URI (prompts for target URI) |
| `:HkList` | `<leader>hL` | Show all links for current file in the quickfix list |
| `:HkPurple` | `<leader>hp` | Annotate buffer with purple-number virtual text (EOL) |
| `:HkOpen` | `<leader>ho` | Open `hook://` URI under cursor, or prompt for one |

## Configuration

```lua
require("hitchmark").setup({
  -- Base URL for hk serve (default: "http://127.0.0.1:2701")
  serve_url = "http://127.0.0.1:2701",

  -- Explicit path to hk binary (default: auto-detect from $PATH)
  hk_path = nil,

  -- Keymaps (false to disable all defaults)
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

1. **HTTP-first** — if `hk serve` is running, all operations use the REST API (fast, no process overhead)
2. **Subprocess fallback** — if the server is not running, each operation shells out to `hk` directly
3. **Purple numbers** — `:HkPurple` fetches paragraph IDs and renders them as EOL virtual text via `nvim_buf_set_extmark`; they don't modify the file on disk

## Requirements

- Neovim 0.8+
- `hk` binary on `$PATH` (or set `hk_path` in config)
- Optional: `hk serve` running for faster HTTP transport
- Optional: [plenary.nvim](https://github.com/nvim-lua/plenary.nvim) for running tests

## Running tests

```bash
nvim --headless \
  -u plugins/neovim/tests/minimal_init.lua \
  -c "PlenaryBustedFile plugins/neovim/tests/bridge_spec.lua"
```
