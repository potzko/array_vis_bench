# `rotation_auxiliary`

Auxiliary rotation: copies the smaller side to a buffer and back. It asks for
half the array as scratch, so a sort allocates that once and every rotation
copies through it; a caller that passes less gets a buffer allocated through
the logger for that call.

Its catalog components are declared in `rotation_auxiliary.spec`.
