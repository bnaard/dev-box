#!/usr/bin/env bash
set -euo pipefail

vim_args=(
  --cmd "set t_u7="
  --cmd "set t_RV="
  -c "if executable('aibox-copy') | xnoremap <silent> y y:<C-u>call system('aibox-copy', getreg('\"'))<CR>:echo 'copied to clipboard'<CR> | endif"
  -c "if executable('aibox-copy') | nnoremap <silent> Y yy:call system('aibox-copy', getreg('\"'))<CR>:echo 'copied line to clipboard'<CR> | endif"
)

exec vim "${vim_args[@]}" "$@"
