# SkEmbed
A command line tool for embedding data within the unused portions of the Minecraft skin texture.

## Usage
This will take data from `data.txt`, embed it within `in-skin.png`, and write it to `out-skin.png`:

```skembed embed --data data.txt --output out-skin.png in-skin.png```

To overwrite the skin, use the `-w` or `--overwrite` switch:

```skembed embed --data data.txt --overwrite skin.png```

This will read the data from `in-skin.png` and print it to the console, if the data is valid UTF-8:

```skembed extract in-skin.png```

To write to a file, use the `-o` or `--output` argument:

```skembed extract --output out-data.txt in-skin.png```
