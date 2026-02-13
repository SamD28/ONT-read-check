# Read check

simple stats for ONT reads

```
read_check <PATH>
```

will tell you basic stats emited as YAML such as number of reads, number of bases, N50 ect

by adding

```
-g
```

Will hash 32mers into a hashmap to calculate depth and therefore estimated genome size for assemblers
