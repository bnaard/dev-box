#!/bin/bash
# vim-loop.sh — Compatibility wrapper for historical calls.
# Runs one-shot Vim with startup terminal-query hardening.

exec vim --cmd "set t_u7=" --cmd "set t_RV=" "$@"
