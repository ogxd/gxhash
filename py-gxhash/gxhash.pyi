from typing import Protocol

class File(Protocol):
    def fileno(self) -> int: ...

def gxhash32(file: File, seed: int) -> int:
    """
    Summary
    -------
    hashes an arbitrary stream of bytes to an u32


    Parameters
    ----------
    file (File)
        file-like object

    seed (int)
        seed for the hash function


    Returns
    -------
    hash (int)
        u32 hash of the input bytes


    Example
    -------
    ```python
    file = TemporaryFile()
    file.write(bytes([42] * 1000))
    file.seek(0)
    seed = 1234
    print(f"Hash is {gxhash.gxhash32(file, seed)}!")
    ```
    """

async def gxhash32_async(file: File, seed: int) -> int:
    """
    Summary
    -------
    hashes an arbitrary stream of bytes to an u32 without the GIL


    Parameters
    ----------
    file (File)
        file-like object

    seed (int)
        seed for the hash function


    Returns
    -------
    hash (Awaitable[int])
        u32 hash of the input bytes


    Example
    -------
    ```python
    file = TemporaryFile()
    file.write(bytes([42] * 1000))
    file.seek(0)
    seed = 1234
    print(f"Hash is {gxhash.gxhash32_async(file, seed)}!")
    ```
    """

def gxhash64(file: File, seed: int) -> int:
    """
    Summary
    -------
    hashes an arbitrary stream of bytes to an u64


    Parameters
    ----------
    file (File)
        file-like object

    seed (int)
        seed for the hash function


    Returns
    -------
    hash (int)
        u64 hash of the input bytes


    Example
    -------
    ```python
    file = TemporaryFile()
    file.write(bytes([42] * 1000))
    file.seek(0)
    seed = 1234
    print(f"Hash is {gxhash.gxhash64(file, seed)}!")
    ```
    """

async def gxhash64_async(file: File, seed: int) -> int:
    """
    Summary
    -------
    hashes an arbitrary stream of bytes to an u64 without the GIL


    Parameters
    ----------
    file (File)
        file-like object

    seed (int)
        seed for the hash function


    Returns
    -------
    hash (Awaitable[int])
        u64 hash of the input bytes


    Example
    -------
    ```python
    file = TemporaryFile()
    file.write(bytes([42] * 1000))
    file.seek(0)
    seed = 1234
    print(f"Hash is {gxhash.gxhash64_async(file, seed)}!")
    ```
    """

def gxhash128(file: File, seed: int) -> int:
    """
    Summary
    -------
    hashes an arbitrary stream of bytes to an u128


    Parameters
    ----------
    file (File)
        file-like object

    seed (int)
        seed for the hash function


    Returns
    -------
    hash (int)
        u128 hash of the input bytes


    Example
    -------
    ```python
    file = TemporaryFile()
    file.write(bytes([42] * 1000))
    file.seek(0)
    seed = 1234
    print(f"Hash is {gxhash.gxhash128(file, seed)}!")
    ```
    """

async def gxhash128_async(file: File, seed: int) -> int:
    """
    Summary
    -------
    hashes an arbitrary stream of bytes to an u128 without the GIL


    Parameters
    ----------
    file (File)
        file-like object

    seed (int)
        seed for the hash function


    Returns
    -------
    hash (Awaitable[int])
        u128 hash of the input bytes


    Example
    -------
    ```python
    file = TemporaryFile()
    file.write(bytes([42] * 1000))
    file.seek(0)
    seed = 1234
    print(f"Hash is {gxhash.gxhash128_async(file, seed)}!")
    ```
    """
