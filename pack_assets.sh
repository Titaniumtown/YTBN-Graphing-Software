#!/bin/bash
cd assets
rm -fr data.tar | true
tar -cvf data.tar *.*
mv data.tar ../