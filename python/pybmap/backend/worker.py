"""Single-threaded JSON-lines command worker."""

import sys

from .protocol import (
    ProtocolError,
    encode_message,
    error_response,
    parse_request_line,
    success_response,
)
from .service import BackendService


def run(stdin=None, stdout=None, stderr=None, service=None):
    stdin = stdin or sys.stdin
    stdout = stdout or sys.stdout
    stderr = stderr or sys.stderr
    service = service or BackendService()

    for line in stdin:
        request_id = ""
        try:
            request = parse_request_line(line)
            request_id = request.request_id
            result = service.handle(request.method, request.params)
            message = success_response(request_id, result)
        except ProtocolError as exc:
            request_id = exc.request_id or request_id
            message = error_response(
                request_id, exc.code, exc.message, exc.retryable, exc.details
            )
        except Exception as exc:  # The wire never exposes exception types or traces.
            print("backend internal error: %s" % exc, file=stderr)
            message = error_response(request_id, "INTERNAL", "internal backend error")

        stdout.write(encode_message(message))
        stdout.flush()
        if service.stopping:
            break
    return 0
