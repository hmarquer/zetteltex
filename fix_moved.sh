#!/bin/bash
find docs -type f -name "*.md" -print0 | while IFS= read -r -d '' f; do
    # API.md -> ../02-guia-tecnica/api.md or 02-guia-tecnica/api.md depending on depth
    # The easiest is to use sed with relative paths, but we can also just use absolute references if we are not careful.
    # Actually, simpler to just run python to fix the links since it can resolve them accurately, but let's do it with sed carefully for the known occurrences.
done
