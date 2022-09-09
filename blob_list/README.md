# blob_list

This is an experiment in using Rust to quickly list a directory on Azure blob storage.

I wanted to quickly debug the actual parsing / listing, so the Rust code itself doesn't currently handle authentication. Instead, I call the Rust program from a Python script [run_with_auth.py](run_with_auth.py), which uses internal [blobfile](https://github.com/christopher-hesse/blobfile) functions to authenticate. I tested this with `blobfile==1.3.3`.
