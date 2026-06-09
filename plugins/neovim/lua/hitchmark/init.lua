--- init.lua — Hitchmark Neovim plugin entry point.
---
--- Usage (lazy.nvim):
---   { "elijah/hitchmark", config = function() require("hitchmark").setup() end }
---
--- Usage (packer):
---   use { "elijah/hitchmark", config = function() require("hitchmark").setup() end }

local M = {}

---@class HitchmarkConfig
---@field serve_url string?    Base URL for hk serve (default: "http://127.0.0.1:2701")
---@field hk_path string?      Explicit path to hk binary (default: auto-detect)
---@field keymaps table?       Custom keymaps (set to false to disable all)

---@type HitchmarkConfig
local defaults = {
  serve_url = "http://127.0.0.1:2701",
  hk_path = nil,
  keymaps = {
    file   = "<leader>hf",
    link   = "<leader>hl",
    list   = "<leader>hL",
    purple = "<leader>hp",
    open   = "<leader>ho",
  },
}

--- Configure and register the plugin.
---@param opts HitchmarkConfig?
function M.setup(opts)
  local config = vim.tbl_deep_extend("force", defaults, opts or {})

  -- Register :Hk* commands
  vim.api.nvim_create_user_command("HkFile",   require("hitchmark.commands").file,   { desc = "Copy hook:// URI for current file" })
  vim.api.nvim_create_user_command("HkLink",   require("hitchmark.commands").link,   { desc = "Link current file to a URI" })
  vim.api.nvim_create_user_command("HkList",   require("hitchmark.commands").list,   { desc = "List links for current file (quickfix)" })
  vim.api.nvim_create_user_command("HkPurple", require("hitchmark.commands").purple, { desc = "Annotate buffer with purple numbers" })
  vim.api.nvim_create_user_command("HkOpen",   require("hitchmark.commands").open,   { desc = "Open a hook:// URI" })

  -- Keymaps (unless disabled)
  if config.keymaps ~= false then
    local km = config.keymaps
    local cmds = require("hitchmark.commands")
    if km.file   then vim.keymap.set("n", km.file,   cmds.file,   { desc = "Hitchmark: copy URI" }) end
    if km.link   then vim.keymap.set("n", km.link,   cmds.link,   { desc = "Hitchmark: link file" }) end
    if km.list   then vim.keymap.set("n", km.list,   cmds.list,   { desc = "Hitchmark: list links" }) end
    if km.purple then vim.keymap.set("n", km.purple, cmds.purple, { desc = "Hitchmark: purple numbers" }) end
    if km.open   then vim.keymap.set("n", km.open,   cmds.open,   { desc = "Hitchmark: open URI" }) end
  end
end

return M
