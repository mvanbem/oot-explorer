#!/bin/bash
set -e

wasm-pack build --target web --no-typescript $@
cp -v pkg/oot_explorer_web{_bg.wasm,.js} www/
