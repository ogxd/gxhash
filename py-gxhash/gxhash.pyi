from typing import Protocol

class File(Protocol):
    def fileno(self) -> int:
        """
        Summary
        -------
        Returns the file descriptor of the file.
        Some file-like objects like `io.BytesIO` have an unimplemented `fileno` method.
        If you are uncertain whether the file has a valid `fileno` method,
        you should write to a `tempfile.TemporaryFile` and pass that to the hasher.

        Returns
        -------
        file_descriptor (`int`)
            file descriptor of the file
        """

class Hasher(Protocol):
    def __init__(self, *, seed: int) -> None:
        """
        Summary
        -------
        Initialise `Hasher` with a `seed`.
        The `seed` should not be exposed as it is used to deterministically generate the hash.
        An exposed `seed` would put your service at risk of a DoS attack.

        Parameters
        ----------
        seed (`int`)
            seed for the hasher

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
            hash of the input bytes

        Example
        -------
        ```python
        hasher = GxHash128(1234)
        print(f"Hash is {hasher.hash(bytes([42] * 1000))}!")
        ```
        """
    def hash_nogil(self, bytes: bytes) -> int:
        """
        Summary
        -------
        Hashes `bytes` to an `int` without the GIL.
        This allows you to perform multiple hashes with true multi-threaded parallelism.
        If called sequentially, this method is slightly less performant than the default `hash` method.

        Parameters
        ----------
        bytes (`bytes`)
            input bytes

        Returns
        -------
        hash (`int`)
            hash of the input bytes

        Example
        -------
        ```python
        hasher = GxHash128(seed=1234)
        input_bytes = bytes([42] * 1000)
        thread_pool = ThreadPoolExecutor()
        future = thread_pool.submit(hasher.hash_nogil, input_bytes)
        hash_result = await wrap_future(future)
        print(f"Hash is {hash_result}!")
        ```
        """
    def hash_file(self, file: File) -> int:
        """
        Summary
        -------
        Hashes a `File` to an `int`.
        This method duplicates the file descriptor and memory maps the file entirely in Rust.
        This operation is many times faster than reading the file in Python and passing the bytes to the hasher.
        If your input is already in `bytes`, this method is slightly less performant than `hash` and `hash_nogil`.

        Parameters
        ----------
        file (`File`)
            file object

        Returns
        -------
        hash (`int`)
            hash of the input file

        Example
        -------
        Converting `bytes` to a `TemporaryFile` and hashing.

        ```python
        hasher = GxHash128(seed=1234)
        file = TemporaryFile()
        file.write(bytes([42] * 1000))
        file.seek(0)
        print(f"Hash is {hasher.hash_file(file)}!")
        ```

        Hashing a file directly.

        ```python
        with Path('really_large_file.img').open('rb') as file:
            hasher = GxHash128(seed=1234)
            print(f"Hash is {hasher.hash_file(file)}!")
        ```
        """
    async def hash_file_async(self, file: File) -> int:
        """
        Summary
        -------
        Asynchronous variant of `hash_file`.
        This method allows you to perform multiple hashes with true multi-threaded parallelism.
        If called sequentially, this method is slightly less performant than `hash_file`.
        In terms of multi-threaded performance, this method is less performant than an asynchronous `hash_nogil`.
        It is only ever faster than `hash_nogil` when the input is a `File`, and that is due to
        the performance overhead of reading a `File` in Python.

        Parameters
        ----------
        file (File)
            file object

        Returns
        -------
        hash (int)
            hash of the input file

        Example
        -------
        Converting `bytes` to a `TemporaryFile` and hashing.

        ```python
        hasher = GxHash128(seed=1234)
        file = TemporaryFile()
        file.write(bytes([42] * 1000))
        file.seek(0)
        print(f"Hash is {await hasher.hash_file_asymc(file)}!")
        ```

        Hashing a file directly.

        ```python
        with Path('really_large_file.img').open('rb') as file:
            hasher = GxHash128(seed=1234)
            print(f"Hash is {await hasher.hash_file_async(file)}!")
        ```
        """

class GxHash32(Hasher): ...
class GxHash64(Hasher): ...
class GxHash128(Hasher): ...
