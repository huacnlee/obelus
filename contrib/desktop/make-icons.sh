#!/bin/sh
# Copies the site's favicon from the mark beside this script.
#
# The favicon is the mark without the plate: a tab has its own background,
# and a dark plate in a light tab bar is a hole in it. `obelus-mark.svg`
# rather than `obelus.svg`, because a favicon is read at sixteen pixels and
# the bar in the icon is half a pixel there -- the plate is what carries it
# at that size, so a mark without one is drawn on its own grid. Copied and
# not converted, and copied here so that nobody has to remember to.
set -eu

cd "$(dirname "$0")"
cp obelus-mark.svg ../../docs/obelus.svg
echo 'docs/obelus.svg copied from obelus-mark.svg'
