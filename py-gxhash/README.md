# gxhash-py

Python bindings for [GxHash](https://github.com/ogxd/gxhash), a blazingly fast and robust non-cryptographic hashing algorithm.

## Features

- **Blazingly Fast**: Minimal-overhead binding to leverage the full speed of GxHash.
- **Zero Python**: Pure Rust backend with zero additional Python runtime overhead.
- **Fine-Grained Control**: Build true multi-threaded or async hashing pipelines with GIL-free APIs.
- **Faster File Hashing**: Hash files using memory-mapped I/O via Rust — 3x faster than Python's sequential I/O.
- **Async-Ready**: Tokio-powered async hashing for fast and efficient concurrency.
- **Fully Typesafe**: Predictable, clean API with complete type safety.

## Installation

```bash
pip install gxhash
```

## Usage

Hashing bytes.

```python
from gxhash import GxHash32

def main():
    gxhash = GxHash32(seed=0)
    result = gxhash.hash(b"Hello, world!")

if __name__ == "__main__":
    main()
```

Hashing bytes asynchronously.

```python
from asyncio import run
from gxhash import GxHash128

async def main():
    gxhash = GxHash128(seed=0)
    result = await gxhash.hash_async(b"Hello, world!")

if __name__ == "__main__":
    run(main())
```

Hashing a file.

```python
from gxhash import GxHash64

def main():
    gxhash = GxHash64(seed=0)
    file = open("path/to/file.dmg", "rb")
    result = gxhash.hash_file(file)

if __name__ == "__main__":
    main()
```

Hashing a file asynchronously.

```python
from asyncio import run
from gxhash import GxHash128

async def main():
    gxhash = GxHash128(seed=0)
    file = open("path/to/file.dmg", "rb")
    result = await gxhash.hash_file_async(file)

if __name__ == "__main__":
    run(main())
```
