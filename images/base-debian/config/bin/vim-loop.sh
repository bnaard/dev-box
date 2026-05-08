#!/bin/bash
# vim-loop.sh — Compatibility wrapper for historical calls.
# Runs Vim with startup terminal-query hardening, then keeps the pane alive.

vim --cmd "set t_u7=" --cmd "set t_RV=" "$@"
exec bash
