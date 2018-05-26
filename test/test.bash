#!/usr/bin/env bash

# set up directories
cd "$(dirname "$0")"/original
rm -rf ../actual
mkdir -p ../actual

# Transform the files
for i in *.bash
do
  ../../shellharden --transform "$i" > ../actual/"$i" 2>> ../actual/errors.txt
done

# Check the results
cd ..
diff -C3 expected/ actual/ && echo Tests passed
