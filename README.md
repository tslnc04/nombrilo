# Nombrilo

Nombrilo shows the block distribution for Minecraft worlds. Functional, but currently a work in progress.

## Usage

```
Usage: nombrilo [OPTIONS] [REGION]...

Arguments:
  [REGION]...  Region files or directories containing region files. Default is the current directory

Options:
  -n <N>                 Display the top N most common blocks. Default is 10
  -i, --ignore <IGNORE>  Blocks to ignore, with or without `minecraft:`. Default is none
  -s, --sorted           Sort the output by count. Default is false
  -v, --verbose          Print additional information, including time taken. Default is false
  -h, --help             Print help
  -V, --version          Print version
```

### Examples

Get the top 5 most common blocks in the overworld of `world`.

```
nombrilo -n 5 world/region
```

Get the sorted top 10 most common blocks in the nether of `world`, ignoring air.
```
nombrilo -s -i air world/DIM-1/region
```

Get the sorted top 5 most common blocks in all dimensions of `world`, ignoring air, water, and lava, printing how long it took.
```
nombrilo -v -s -n 5 -i air -i water -i lava world/region world/DIM-1/region world/DIM1/region
```

### Example Output

The output of the last example run on the Hermitcraft Season 9 world.
```
┌──────────────────────┬────────────┐
│ Block                │ Count      │
├──────────────────────┼────────────┤
│ minecraft:deepslate  │ 2176303158 │
├──────────────────────┼────────────┤
│ minecraft:stone      │ 1924685395 │
├──────────────────────┼────────────┤
│ minecraft:netherrack │ 666975631  │
├──────────────────────┼────────────┤
│ minecraft:bedrock    │ 189816967  │
├──────────────────────┼────────────┤
│ minecraft:tuff       │ 178996347  │
└──────────────────────┴────────────┘
Done in 2.533113483s.
```

## Name

Nombrilo means counter in Esperanto.

## Acknowledgment

Thanks to [`serde_json`](https://github.com/serde-rs/json), [`fastnbt`](https://github.com/owengage/fastnbt), and [`simdnbt`](https://github.com/azalea-rs/simdnbt) for inspiration.

## License

Copyright 2024 Kirsten Laskoski.

Licensed under MIT.

