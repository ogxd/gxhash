def gxhash32(input_bytes: bytes, seed: int) -> int:
    """
    Summary
    -------
    hashes an arbitrary stream of bytes to an u32


    Parameters
    ----------
    input_bytes (bytes): input bytes to hash

    seed (int): seed for the hash function


    Returns
    -------
    hash (int): u32 hash of the input bytes


    Example
    -------
    ```python
    import gxhash

    input_bytes =  bytes([42] * 1000)
    seed = 1234
    print(f"Hash is {gxhash.gxhash32(input_bytes, seed)}!")
    ```
    """

def gxhash32_nogil(input_bytes: bytes, seed: int) -> int:
    """
    Summary
    -------
    hashes an arbitrary stream of bytes to an u32 without the GIL


    Parameters
    ----------
    input_bytes (bytes): input bytes to hash

    seed (int): seed for the hash function


    Returns
    -------
    hash (int): u32 hash of the input bytes


    Example
    -------
    ```python
    import gxhash

    input_bytes =  bytes([42] * 1000)
    seed = 1234
    print(f"Hash is {gxhash.gxhash32_nogil(input_bytes, seed)}!")
    ```
    """

def gxhash64(input_bytes: bytes, seed: int) -> int:
    """
    Summary
    -------
    hashes an arbitrary stream of bytes to an u64


    Parameters
    ----------
    input_bytes (bytes): input bytes to hash

    seed (int): seed for the hash function


    Returns
    -------
    hash (int): u64 hash of the input bytes


    Example
    -------
    ```python
    import gxhash

    input_bytes =  bytes([42] * 1000)
    seed = 1234
    print(f"Hash is {gxhash.gxhash64(input_bytes, seed)}!")
    ```
    """

def gxhash64_nogil(input_bytes: bytes, seed: int) -> int:
    """
    Summary
    -------
    hashes an arbitrary stream of bytes to an u64 without the GIL


    Parameters
    ----------
    input_bytes (bytes): input bytes to hash

    seed (int): seed for the hash function


    Returns
    -------
    hash (int): u64 hash of the input bytes


    Example
    -------
    ```python
    import gxhash

    input_bytes =  bytes([42] * 1000)
    seed = 1234
    print(f"Hash is {gxhash.gxhash64_nogil(input_bytes, seed)}!")
    ```
    """

def gxhash128(input_bytes: bytes, seed: int) -> int:
    """
    Summary
    -------
    hashes an arbitrary stream of bytes to an u128


    Parameters
    ----------
    input_bytes (bytes): input bytes to hash

    seed (int): seed for the hash function


    Returns
    -------
    hash (int): u128 hash of the input bytes


    Example
    -------
    ```python
    import gxhash

    input_bytes =  bytes([42] * 1000)
    seed = 1234
    print(f"Hash is {gxhash.gxhash128(input_bytes, seed)}!")
    ```
    """

def gxhash128_nogil(input_bytes: bytes, seed: int) -> int:
    """
    Summary
    -------
    hashes an arbitrary stream of bytes to an u128 without the GIL


    Parameters
    ----------
    input_bytes (bytes): input bytes to hash

    seed (int): seed for the hash function


    Returns
    -------
    hash (int): u128 hash of the input bytes


    Example
    -------
    ```python
    import gxhash

    input_bytes =  bytes([42] * 1000)
    seed = 1234
    print(f"Hash is {gxhash.gxhash128_nogil(input_bytes, seed)}!")
    ```
    """
