--- commands.lua — user-facing Neovim commands for Hitchmark.

local M = {}
local bridge = require("hitchmark.bridge")

--- Yank the hook:// URI for the current file to the + register and notify.
function M.file()
  local path = vim.fn.expand("%:p")
  if path == "" then
    vim.notify("hitchmark: no file open", vim.log.levels.WARN)
    return
  end

  local uri, err = bridge.file_uri(path)
  if not uri then
    vim.notify("hitchmark: " .. (err or "unknown error"), vim.log.levels.ERROR)
    return
  end

  vim.fn.setreg("+", uri)
  vim.notify("Copied: " .. uri, vim.log.levels.INFO)
end

--- Link the current file to a URI entered via vim.ui.input.
function M.link()
  local path = vim.fn.expand("%:p")
  if path == "" then
    vim.notify("hitchmark: no file open", vim.log.levels.WARN)
    return
  end

  local uri_a, err_a = bridge.file_uri(path)
  if not uri_a then
    vim.notify("hitchmark: " .. (err_a or "unknown error"), vim.log.levels.ERROR)
    return
  end

  vim.ui.input({ prompt = "Link to URI: " }, function(uri_b)
    if not uri_b or vim.trim(uri_b) == "" then return end
    uri_b = vim.trim(uri_b)

    vim.ui.input({ prompt = "Note (optional): " }, function(note)
      note = (note and vim.trim(note) ~= "") and note or nil
      local ok, link_err = bridge.create_link(uri_a, uri_b, note)
      if ok then
        vim.notify("Linked: " .. uri_a .. " ↔ " .. uri_b, vim.log.levels.INFO)
      else
        vim.notify("hitchmark link failed: " .. (link_err or ""), vim.log.levels.ERROR)
      end
    end)
  end)
end

--- Show all links for the current file in the quickfix list.
function M.list()
  local path = vim.fn.expand("%:p")
  if path == "" then
    vim.notify("hitchmark: no file open", vim.log.levels.WARN)
    return
  end

  local uri, err = bridge.file_uri(path)
  if not uri then
    vim.notify("hitchmark: " .. (err or "unknown error"), vim.log.levels.ERROR)
    return
  end

  local links, list_err = bridge.list_links(uri)
  if not links then
    vim.notify("hitchmark list failed: " .. (list_err or ""), vim.log.levels.ERROR)
    return
  end

  if #links == 0 then
    vim.notify("hitchmark: no links for this file", vim.log.levels.INFO)
    return
  end

  -- Build quickfix entries
  local qf = {}
  for _, link in ipairs(links) do
    local other = (link.source == uri) and link.target or link.source
    table.insert(qf, {
      text = other .. (link.note and ("  -- " .. link.note) or ""),
      type = "I",
    })
  end

  vim.fn.setqflist(qf, "r")
  vim.fn.setqflist({}, "a", { title = "Hitchmark links for " .. vim.fn.fnamemodify(path, ":t") })
  vim.cmd("copen")
end

--- Annotate the current buffer with purple number virtual text.
function M.purple()
  local path = vim.fn.expand("%:p")
  if path == "" then
    vim.notify("hitchmark: no file open", vim.log.levels.WARN)
    return
  end

  local paragraphs, err = bridge.purple(path)
  if not paragraphs then
    vim.notify("hitchmark purple failed: " .. (err or ""), vim.log.levels.ERROR)
    return
  end

  local bufnr = vim.api.nvim_get_current_buf()
  local ns = vim.api.nvim_create_namespace("hitchmark_purple")
  vim.api.nvim_buf_clear_namespace(bufnr, ns, 0, -1)

  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  local line_idx = 0

  for _, para in ipairs(paragraphs) do
    -- Find the first line of this paragraph in the buffer
    local first_line = vim.split(para.text, "\n")[1]
    for i = line_idx, #lines - 1 do
      if lines[i + 1] == first_line then
        vim.api.nvim_buf_set_extmark(bufnr, ns, i, 0, {
          virt_text = { { "¶ " .. para.id, "Comment" } },
          virt_text_pos = "eol",
        })
        line_idx = i + 1
        break
      end
    end
  end

  vim.notify(string.format("hitchmark: annotated %d paragraph(s)", #paragraphs), vim.log.levels.INFO)
end

--- Open a hook:// URI under the cursor or entered via vim.ui.input.
function M.open()
  -- Try to grab URI under cursor first
  local word = vim.fn.expand("<cWORD>")
  local uri = word:match("hook://[^%s]+") or ""

  if uri == "" then
    vim.ui.input({ prompt = "Open URI: " }, function(input)
      if not input or vim.trim(input) == "" then return end
      do_open(vim.trim(input))
    end)
  else
    do_open(uri)
  end
end

function do_open(uri)
  local ok, err = bridge.open_uri(uri)
  if not ok then
    vim.notify("hitchmark open failed: " .. (err or ""), vim.log.levels.ERROR)
  end
end

return M
