--- bridge.lua — HTTP-first transport with hk subprocess fallback.
--- Pattern mirrors plugins/obsidian/src/bridge.ts and plugins/vscode/src/bridge.ts.

local M = {}

local BASE_URL = "http://127.0.0.1:2701"
local TIMEOUT_MS = 2000

--- Execute a shell command and return stdout, stderr, exit code.
---@param cmd string[]
---@return string stdout, string stderr, number code
local function shell(cmd)
  local stdout_file = vim.fn.tempname()
  local stderr_file = vim.fn.tempname()
  local quoted = table.concat(
    vim.tbl_map(function(a) return vim.fn.shellescape(a) end, cmd),
    " "
  )
  local code = vim.fn.system(
    quoted .. " >" .. stdout_file .. " 2>" .. stderr_file
  )
  local stdout = vim.fn.readfile(stdout_file)
  local stderr_lines = vim.fn.readfile(stderr_file)
  vim.fn.delete(stdout_file)
  vim.fn.delete(stderr_file)
  return table.concat(stdout, "\n"),
         table.concat(stderr_lines, "\n"),
         vim.v.shell_error
end

--- Locate the hk binary: $PATH first, then common install locations.
---@return string|nil
local function find_hk()
  if vim.fn.executable("hk") == 1 then return "hk" end
  for _, p in ipairs({
    vim.fn.expand("~/.cargo/bin/hk"),
    "/usr/local/bin/hk",
    "/opt/homebrew/bin/hk",
  }) do
    if vim.fn.executable(p) == 1 then return p end
  end
  return nil
end

--- HTTP GET via curl (non-blocking would require plenary; using synchronous curl).
---@param path string  e.g. "/health"
---@return table|nil decoded, string|nil err
local function http_get(path)
  local out, _, code = shell({
    "curl", "-sf", "--max-time", tostring(TIMEOUT_MS / 1000),
    BASE_URL .. path,
  })
  if code ~= 0 then return nil, "curl failed (exit " .. code .. ")" end
  local ok, decoded = pcall(vim.fn.json_decode, out)
  if not ok then return nil, "JSON decode error" end
  return decoded, nil
end

--- HTTP POST via curl.
---@param path string
---@param body table  will be JSON-encoded
---@return table|nil decoded, string|nil err
local function http_post(path, body)
  local json = vim.fn.json_encode(body)
  local out, _, code = shell({
    "curl", "-sf", "--max-time", tostring(TIMEOUT_MS / 1000),
    "-X", "POST",
    "-H", "Content-Type: application/json",
    "-d", json,
    BASE_URL .. path,
  })
  if code ~= 0 then return nil, "curl failed (exit " .. code .. ")" end
  local ok, decoded = pcall(vim.fn.json_decode, out)
  if not ok then return nil, "JSON decode error" end
  return decoded, nil
end

-- ── Public API ────────────────────────────────────────────────────────────────

--- Return the hook:// URI for a file path.
---@param path string
---@return string|nil uri, string|nil err
function M.file_uri(path)
  -- HTTP first
  local resp, err = http_get("/uri?path=" .. vim.uri_encode(path))
  if resp and resp.uri then return resp.uri, nil end

  -- Subprocess fallback
  local hk = find_hk()
  if not hk then return nil, "hk binary not found" end
  local out, serr, code = shell({ hk, "file", path })
  if code ~= 0 then return nil, serr end
  return vim.trim(out), nil
end

--- List all links for a URI.
---@param uri string
---@return table[]|nil links, string|nil err
function M.list_links(uri)
  local resp, err = http_get("/links?uri=" .. vim.uri_encode(uri))
  if resp then return resp, nil end

  local hk = find_hk()
  if not hk then return nil, "hk binary not found" end
  local out, serr, code = shell({ hk, "list", uri, "--json" })
  if code ~= 0 then return nil, serr end
  local ok, decoded = pcall(vim.fn.json_decode, out)
  if not ok then return nil, "JSON decode error" end
  return decoded, nil
end

--- Create a bidirectional link.
---@param uri_a string
---@param uri_b string
---@param note string|nil
---@return boolean ok, string|nil err
function M.create_link(uri_a, uri_b, note)
  local body = { uri_a = uri_a, uri_b = uri_b, note = note }
  local resp, err = http_post("/links", body)
  if resp and resp.ok then return true, nil end

  local hk = find_hk()
  if not hk then return false, "hk binary not found" end
  local args = { hk, "link", uri_a, uri_b, "--yes" }
  if note then vim.list_extend(args, { "--note", note }) end
  local _, serr, code = shell(args)
  if code ~= 0 then return false, serr end
  return true, nil
end

--- Generate purple numbers for a file.
---@param path string
---@return table[]|nil paragraphs, string|nil err
function M.purple(path)
  local resp, err = http_get("/purple?path=" .. vim.uri_encode(path))
  if resp then return resp, nil end

  local hk = find_hk()
  if not hk then return nil, "hk binary not found" end
  local out, serr, code = shell({ hk, "purple", path, "--format", "json" })
  if code ~= 0 then return nil, serr end
  local ok, decoded = pcall(vim.fn.json_decode, out)
  if not ok then return nil, "JSON decode error" end
  return decoded, nil
end

--- Open a hook:// URI (delegates to OS opener via hk).
---@param uri string
---@return boolean ok, string|nil err
function M.open_uri(uri)
  local hk = find_hk()
  if not hk then return false, "hk binary not found" end
  local _, serr, code = shell({ hk, "open", uri })
  if code ~= 0 then return false, serr end
  return true, nil
end

return M
