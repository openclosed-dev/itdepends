# itdepends

## How to Build

```
cargo build --release
```

## How to Use

```shell
# Output the dependencies in JSON format
mvn org.apache.maven.plugins:maven-dependency-plugin:3.10.0:tree '-DoutputType=json' '-DoutputFile=deps-tree.json'
# Analyze the file
itdepends deps-tree.json
```
