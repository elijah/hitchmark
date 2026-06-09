-- tests/minimal_init.lua — minimal Neovim init for running plenary tests in CI.
-- Usage: nvim --headless -u tests/minimal_init.lua -c "PlenaryBustedFile tests/bridge_spec.lua"

vim.opt.runtimepath:prepend(vim.fn.fnamemodify(debug.getinfo(1).source:sub(2), ":h:h"))

-- Add plenary to rtp if installed via luarocks or local clone
local plenary_path = vim.fn.expand("~/.local/share/nvim/site/pack/packer/start/plenary.nvim")
if vim.fn.isdirectory(plenary_path) == 1 then
  vim.opt.runtimepath:prepend(plenary_path)
end
