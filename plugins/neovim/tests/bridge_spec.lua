-- tests/bridge_spec.lua — unit tests for bridge.lua using busted/plenary.
-- Run: nvim --headless -u tests/minimal_init.lua -c "PlenaryBustedFile tests/bridge_spec.lua"

local bridge = require("hitchmark.bridge")

describe("hitchmark.bridge", function()

  -- ── file_uri ──────────────────────────────────────────────────────────────

  describe("file_uri (subprocess fallback)", function()
    it("returns nil and an error when hk is not found", function()
      -- Patch find_hk to return nil
      local orig = vim.fn.executable
      vim.fn.executable = function() return 0 end

      local uri, err = bridge.file_uri("/does/not/exist.md")
      assert.is_nil(uri)
      assert.is_not_nil(err)

      vim.fn.executable = orig
    end)
  end)

  -- ── list_links ────────────────────────────────────────────────────────────

  describe("list_links (subprocess fallback)", function()
    it("returns nil and an error when hk is not found", function()
      local orig = vim.fn.executable
      vim.fn.executable = function() return 0 end

      local links, err = bridge.list_links("hook://file/dGVzdA")
      assert.is_nil(links)
      assert.is_not_nil(err)

      vim.fn.executable = orig
    end)
  end)

  -- ── create_link ───────────────────────────────────────────────────────────

  describe("create_link (subprocess fallback)", function()
    it("returns false and an error when hk is not found", function()
      local orig = vim.fn.executable
      vim.fn.executable = function() return 0 end

      local ok, err = bridge.create_link("hook://file/a", "hook://file/b")
      assert.is_false(ok)
      assert.is_not_nil(err)

      vim.fn.executable = orig
    end)
  end)

  -- ── purple ────────────────────────────────────────────────────────────────

  describe("purple (subprocess fallback)", function()
    it("returns nil and an error when hk is not found", function()
      local orig = vim.fn.executable
      vim.fn.executable = function() return 0 end

      local paras, err = bridge.purple("/some/file.md")
      assert.is_nil(paras)
      assert.is_not_nil(err)

      vim.fn.executable = orig
    end)
  end)

  -- ── open_uri ──────────────────────────────────────────────────────────────

  describe("open_uri", function()
    it("returns false and an error when hk is not found", function()
      local orig = vim.fn.executable
      vim.fn.executable = function() return 0 end

      local ok, err = bridge.open_uri("hook://file/dGVzdA")
      assert.is_false(ok)
      assert.is_not_nil(err)

      vim.fn.executable = orig
    end)
  end)

end)
