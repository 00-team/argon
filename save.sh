#!/bin/bash

cargo build -r
cp target/release/argon ~/.local/bin/
