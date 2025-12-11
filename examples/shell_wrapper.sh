#!/usr/bin/env bash
# Wrapper script for reponest to enable cd functionality
# Inspired by Yazi: https://yazi-rs.github.io/docs/quick-start#shell-wrapper
#
# How to use:
#   Add this function to your ~/.bashrc or ~/.zshrc

function reponest() {
	local tmp="$(mktemp -t "reponest-cwd.XXXXXX")" cwd
	command reponest "$@" --cwd-file="$tmp"
	
	if [ -f "$tmp" ]; then
		cwd="$(cat "$tmp")"
		if [ -n "$cwd" ] && [ "$cwd" != "$PWD" ]; then
			builtin cd -- "$cwd" && echo "Changed directory to: $cwd"
		fi
		rm -f -- "$tmp"
	fi
}
