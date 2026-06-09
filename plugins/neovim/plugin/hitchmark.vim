" plugin/hitchmark.vim — autoload shim for Neovim (Vim compatibility guard)
if exists('g:loaded_hitchmark') | finish | endif
let g:loaded_hitchmark = 1

" Commands are registered via init.lua / setup() — this file is a no-op
" when the plugin is loaded via a Lua package manager (lazy.nvim, packer, etc.)
