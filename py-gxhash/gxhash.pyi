from pathlib import Path
from typing import Protocol

class Hasher(Protocol):
    def __init__(self, *, seed: int) -> None:
        """
        Summary
        -------
        Initialise `Hasher` with a `seed`.
        The `seed` should not be exposed as it is used to deterministically generate the hash.
        An exposed `seed` would put your service at a higher risk of a DoS attack.

        Parameters
        ----------
        seed (`int`)
            a seed for the hasher

        Example
        -------
        ```python
        hasher = GxHash128(seed=1234)
        ```
        """
    def hash(self, bytes: bytes) -> int:
        """
        Summary
        -------
        Hashes `bytes` to an `int`.
        If your input is in `bytes`, this is the most performant variant of the hasher.

        Parameters
        ----------
        bytes (`bytes`)
            input bytes

        Returns
        -------
        hash (`int`)
            the hash of the input bytes

        Example
        -------
        ```python
        hasher = GxHash128(1234)
        print(f"Hash is {hasher.hash(bytes([42] * 1000))}!")
        ```
        """
    async def hash_async(self, bytes: bytes) -> int:
        """
        Summary
        -------
        Hashes `bytes` to an `int` asynchronously.
        This method allows you to perform multiple hashes with true multi-threaded parallelism.
        If called sequentially, this method is slightly less performant than the default `hash` method.
        Otherwise, this variant offers the best raw multi-threaded performance.

        Parameters
        ----------
        bytes (`bytes`)
            input bytes

        Returns
        -------
        hash (`int`)
            the hash of the input bytes

        Example
        -------
        ```python
        hasher = GxHash128(seed=1234)
        print(f"Hash is {await hasher.hash_async(bytes([42] * 1000))}!")
        ```
        """
    def hash_file(self, file_path: str | Path) -> int:
        """
        Summary
        -------
        Hashes a file to an `int`.
        This method memory maps the file entirely in Rust.
        This operation is many times faster than reading the file in Python and passing the bytes to the hasher.
        If your input is already in `bytes`, this method may be slightly less performant than `hash` and `hash_async`.

        Parameters
        ----------
        file_path (`str | Path`)
            a file object

        Returns
        -------
        hash (`int`)
            the hash of the input file

        Example
        -------
        ```python
        hasher = GxHash128(seed=1234)
        print(f"Hash is {hasher.hash_file('./data/large_file.bin')}!")
        ```
        """
    async def hash_file_async(self, file_path: str | Path) -> int:
        """
        Summary
        -------
        Asynchronous variant of `hash_file`.
        This method allows you to perform multiple hashes with true multi-threaded parallelism.
        If called sequentially, this method is slightly less performant than `hash_file`.
        It is only ever faster than a multi-threaded `hash_async` when the input is a file,
        and that is due to the performance overhead of reading a file in Python.

        Parameters
        ----------
        file_path (`str | Path`)
            a file object

        Returns
        -------
        hash (`int`)
            the hash of the input file

        Example
        -------
        ```python
        hasher = GxHash128(seed=1234)
        print(f"Hash is {await hasher.hash_file_async('./data/large_file.bin')}!")
        ```
        """

class GxHash32(Hasher):
    """
    Summary
    -------
    This class exposes GxHash's 32-bit hash methods.
    """

class GxHash64(Hasher):
    """
    Summary
    -------
    This class exposes GxHash's 64-bit hash methods.
    """

class GxHash128(Hasher):
    """
    Summary
    -------
    This class exposes GxHash's 128-bit hash methods.
    """
