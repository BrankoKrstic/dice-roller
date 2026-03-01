#!/bin/bash

stylance --watch . &
cargo leptos watch
wait
