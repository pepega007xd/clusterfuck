#!/bin/bash

src=$(cat polotovar.rs | \
sed -e 's/\\n/\\\\n/g' | \
sed -e 's/"/\\"/g' | \
sed -e ':a;N;$!ba;s/\n/\\n/g')

dest=`cat polotovar.c`

marker="PLACEHOLDER"
echo "${dest/$marker/$src}" > cluster.c
