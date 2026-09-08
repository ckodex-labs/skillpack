# Schema inference rules

- Integers: all sampled values match `^-?\d+$`.
- Floats: decimal point or exponent present in any sampled value.
- Booleans: case-insensitive true/false/yes/no set.
- Dates: ISO-8601 prefix match on every sampled value.
- Fallback: string.
