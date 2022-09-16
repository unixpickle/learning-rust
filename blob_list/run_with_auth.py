import argparse
import datetime
import json
import subprocess

import blobfile._azure as az
import blobfile._ops as ops
import xmltodict


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("az_dir", type=str)
    args = parser.parse_args()

    account, container, prefix = az.split_path(args.az_dir)
    params = dict(comp="list", restype="container", delimiter="/", prefix=prefix)

    # https://github.com/christopher-hesse/blobfile/blob/b14b012626d0af7887d84dcc76fdbe47e9b58215/blobfile/_azure.py#L1304
    req = az.Request(
        url=az.build_url(account, "/{container}", container=container),
        method="GET",
        params=params,
        data=None,
        success_codes=(200, 404, az.INVALID_HOSTNAME_STATUS),
    )

    # pylint: disable=protected-access
    auth = az.access_token_manager.get_token(ops.default_context._conf, key=(account, container))
    headers = {}

    # https://github.com/christopher-hesse/blobfile/blob/b14b012626d0af7887d84dcc76fdbe47e9b58215/blobfile/_azure.py#L227
    headers["x-ms-version"] = "2019-02-02"
    headers["x-ms-date"] = datetime.datetime.utcnow().strftime("%a, %d %b %Y %H:%M:%S GMT")
    data = req.data
    if data is not None and isinstance(data, dict):
        data = xmltodict.unparse(data).encode("utf8")

    result = az.Request(
        method=req.method,
        url=req.url,
        params=params,
        headers=headers,
        data=data,
        preload_content=req.preload_content,
        success_codes=tuple(req.success_codes),
        retry_codes=tuple(req.retry_codes),
    )

    kind, token = auth
    if kind == az.SHARED_KEY:
        # make sure we are signing the request that has the ms headers added already
        headers["Authorization"] = az.sign_with_shared_key(result, token)
    elif kind == az.OAUTH_TOKEN:
        headers["Authorization"] = f"Bearer {token}"
    elif kind == az.ANONYMOUS:
        pass

    subprocess.check_call(
        [
            "./target/debug/blob_list",
            "--headers-json",
            json.dumps(headers),
            "--params-json",
            json.dumps(params),
            req.url,
        ]
    )


if __name__ == "__main__":
    main()
