set -eu
set +x
python3 - <<'PY'
import base64
import hashlib
import json
import os
import urllib.error
import urllib.request

endpoint = (os.environ['ACTIONS_RUNTIME_URL'].rstrip('/') +
            '/_apis/pipelines/workflows/' + os.environ['GITHUB_RUN_ID'] + '/artifacts')
token = os.environ['ACTIONS_RUNTIME_TOKEN']

def exchange(label, method, url, data, headers):
    request = urllib.request.Request(url, data=data, method=method, headers={
        'Authorization': 'Bearer ' + token, **headers,
    })
    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            print('ARTIFACT_PROTOCOL', label, 'HTTP', response.status)
            return response.read()
    except urllib.error.HTTPError as error:
        print('ARTIFACT_PROTOCOL', label, 'HTTP', error.code)
        raise SystemExit(1) from None
    except urllib.error.URLError:
        print('ARTIFACT_PROTOCOL', label, 'transport failure')
        raise SystemExit(1) from None

payload = b'a\nb\n'
checksum = base64.b64encode(hashlib.md5(payload).digest()).decode('ascii')
for name in ['tests', 'other']:
    created = exchange('create-' + name, 'POST', endpoint + '?api-version=6.0-preview',
                       json.dumps({'Type': 'actions_storage', 'Name': name}).encode(),
                       {'Content-Type': 'application/json'})
    container = json.loads(created)['fileContainerResourceUrl']
    separator = '&' if '?' in container else '?'
    exchange('upload-' + name, 'PUT', container + separator + 'itemPath=' + name + '%2Ffailed.txt',
             payload, {'Content-Type': 'application/octet-stream',
                       'x-actions-results-md5': checksum,
                       'Content-Range': 'bytes 0-3/4', 'x-tfs-filelength': '4'})
    exchange('finalize-' + name, 'PATCH', endpoint + '?artifactName=' + name + '&api-version=6.0-preview',
             b'{"Size":4}', {'Content-Type': 'application/json'})
PY
