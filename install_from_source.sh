#!/bin/bash

set -e

./builddist.sh
cd dist
./install.sh