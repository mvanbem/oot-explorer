#!/bin/bash
set -e

(cd oot-explorer-web && npm install && npm run build)
mkdir -p -v www
cp -v oot-explorer-web/dist/* www/
cp -v oot-explorer-web/static/* www/
