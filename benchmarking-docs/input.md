# Input data

The input format follows a specific YAML syntax.
For this, suppose that a variable

-  `test_one_dim` is required and is a one-dimensional scalar.
-  `test_four_dim` is required and is a four-dimensional vector.

```yaml
vars:
  - name: test_one_dim
    min_val: -10
    max_val: 10
    values: [ 20 ]

  - name: test_four_dim
    min_val: -10
    max_val: 10
    values: [ 20, 30, 40, 50 ]
```

Note that the fields `min_val` and `max_val` are **not** required in this implementation.
They are used for homomorphic encryption and thus ignored by our implementation.

The dimensionality is encoded in the array notation.
As YAML also supports an ordered list, this representation is also possible.
