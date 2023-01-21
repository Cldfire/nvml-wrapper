#!/bin/bash

# Writes all unwrapped function names to `unwrapped_functions.txt`. This can
# help discover functions to work on wrapping.
#
# `ripgrep` must be installed and available. `cargo install ripgrep`

all_functions=$(rg 'pub unsafe fn (\w+)' -oNr '$1' ../nvml-wrapper-sys/src/bindings.rs | sort)
readarray -t all_functions_arr <<< "$all_functions"

output=""

for name in "${all_functions_arr[@]}"
do
    if [[ $name = "new" ]]
    then
        continue
    fi

    # filter out function names that appear in the wrapper source
    if ! rg -U "lib[ \n]*\.${name}[ \n]*\." -q src/* ;
    then
        output+="${name}"
        output+=$'\n'
    fi
done

echo "$output" > unwrapped_functions.txt
