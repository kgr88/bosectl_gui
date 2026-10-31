"""Versioned newline-delimited JSON contract for the GUI sidecar."""

import json
import re
from collections import namedtuple


PROTOCOL_VERSION = 3
MAX_LINE_BYTES = 64 * 1024
MAX_REQUEST_ID_LENGTH = 128

METHODS = (
    "hello",
    "discover",
    "connect",
    "disconnect",
    "get_snapshot",
    "poll_snapshot",
    "set_noise_control",
    "set_eq",
    "set_setting",
    "get_profiles",
    "set_profile",
    "delete_profile",
    "shutdown",
)

ERROR_CODES = (
    "INVALID_REQUEST",
    "REQUEST_TOO_LARGE",
    "UNSUPPORTED_PROTOCOL",
    "UNKNOWN_METHOD",
    "INVALID_PARAMS",
    "BACKEND_UNAVAILABLE",
    "BACKEND_CRASHED",
    "BACKEND_TIMEOUT",
    "BLUETOOTH_UNAVAILABLE",
    "BLUETOOTH_DISABLED",
    "DEVICE_NOT_FOUND",
    "DEVICE_NOT_PAIRED",
    "SERVICE_NOT_FOUND",
    "ACCESS_DENIED",
    "CONNECTION_FAILED",
    "NOT_CONNECTED",
    "DEVICE_TIMEOUT",
    "DEVICE_LOST",
    "UNSUPPORTED_DEVICE",
    "UNSUPPORTED_OPERATION",
    "STALE_GENERATION",
    "CONFLICT",
    "SHUTTING_DOWN",
    "INTERNAL",
)

EVENTS = ("snapshot_changed", "connection_changed", "backend_error")
DEVICE_TYPES = ("qc_ultra2", "qc35")
NOISE_CONTROL_KINDS = ("cnc", "anr", "mode")
SETTING_KEYS = (
    "sidetone",
    "auto_pause",
    "spatial_audio",
    "wind_block",
    "anc_toggle",
)

Request = namedtuple("Request", ["request_id", "method", "params"])


class ProtocolError(Exception):
    def __init__(self, code, message, request_id="", retryable=False, details=None):
        super().__init__(message)
        self.code = code
        self.message = message
        self.request_id = request_id
        self.retryable = retryable
        self.details = details


def _fail(message, request_id="", code="INVALID_PARAMS"):
    raise ProtocolError(code, message, request_id=request_id)


def _is_int(value):
    return isinstance(value, int) and not isinstance(value, bool)


def _require_object(value, field, request_id):
    if not isinstance(value, dict):
        _fail("%s must be an object" % field, request_id)
    return value


def _strict_keys(value, required, optional, request_id, field="params"):
    keys = set(value)
    missing = set(required) - keys
    extra = keys - set(required) - set(optional)
    if missing:
        _fail("%s is missing: %s" % (field, ", ".join(sorted(missing))), request_id)
    if extra:
        _fail(
            "%s contains unknown fields: %s" % (field, ", ".join(sorted(extra))),
            request_id,
        )


def _generation(params, request_id):
    value = params.get("generation")
    if not _is_int(value) or value < 0:
        _fail("generation must be a non-negative integer", request_id)


def _nonempty_string(value, field, request_id, maximum=256):
    if not isinstance(value, str) or not value or len(value) > maximum:
        _fail(
            "%s must be a non-empty string of at most %d characters" % (field, maximum),
            request_id,
        )


def _validate_target(target, request_id):
    target = _require_object(target, "target", request_id)
    kind = target.get("kind")
    if kind == "discovered":
        _strict_keys(target, ("kind", "device_id"), (), request_id, "target")
        _nonempty_string(target["device_id"], "target.device_id", request_id)
    elif kind == "manual":
        _strict_keys(
            target, ("kind", "address", "device_type"), (), request_id, "target"
        )
        _nonempty_string(target["address"], "target.address", request_id, 64)
        if not re.match(r"^[0-9A-Fa-f]{2}(?::[0-9A-Fa-f]{2}){5}$", target["address"]):
            _fail("target.address must be a Bluetooth MAC address", request_id)
        if target["device_type"] not in DEVICE_TYPES:
            _fail("target.device_type is not supported", request_id)
    else:
        _fail("target.kind must be discovered or manual", request_id)


def _validate_profile(profile, request_id):
    profile = _require_object(profile, "profile", request_id)
    _strict_keys(
        profile,
        ("name", "cnc_level", "auto_cnc", "spatial", "wind_block", "anc_toggle"),
        ("profile_id",),
        request_id,
        "profile",
    )
    if "profile_id" in profile and profile["profile_id"] is not None:
        _nonempty_string(profile["profile_id"], "profile.profile_id", request_id)
    _nonempty_string(profile["name"], "profile.name", request_id, 64)
    if not _is_int(profile["cnc_level"]) or not 0 <= profile["cnc_level"] <= 10:
        _fail("profile.cnc_level must be an integer from 0 to 10", request_id)
    if profile["spatial"] not in ("off", "room", "motion"):
        _fail("profile.spatial must be off, room, or motion", request_id)
    for key in ("auto_cnc", "wind_block", "anc_toggle"):
        if not isinstance(profile[key], bool):
            _fail("profile.%s must be a boolean" % key, request_id)


def validate_params(method, params, request_id):
    params = _require_object(params, "params", request_id)
    if method in ("hello", "discover", "disconnect", "get_snapshot", "shutdown"):
        _strict_keys(params, (), (), request_id)
    elif method == "poll_snapshot":
        _strict_keys(params, ("generation", "fields"), (), request_id)
        _generation(params, request_id)
        fields = params["fields"]
        if (
            not isinstance(fields, list)
            or not fields
            or len(fields) > 2
            or len(set(fields)) != len(fields)
            or any(field not in ("battery", "noise_control") for field in fields)
        ):
            _fail("fields must contain battery and/or noise_control", request_id)
    elif method == "connect":
        _strict_keys(params, ("target",), (), request_id)
        _validate_target(params["target"], request_id)
    elif method == "set_noise_control":
        _strict_keys(params, ("generation", "kind", "value"), (), request_id)
        _generation(params, request_id)
        kind = params["kind"]
        value = params["value"]
        if kind not in NOISE_CONTROL_KINDS:
            _fail("kind must be cnc, anr, or mode", request_id)
        if kind == "cnc" and (not _is_int(value) or not 0 <= value <= 10):
            _fail("cnc value must be an integer from 0 to 10", request_id)
        if kind != "cnc":
            _nonempty_string(value, "value", request_id, 64)
    elif method == "set_eq":
        _strict_keys(params, ("generation", "bass", "mid", "treble"), (), request_id)
        _generation(params, request_id)
        for band in ("bass", "mid", "treble"):
            if not _is_int(params[band]) or not -10 <= params[band] <= 10:
                _fail("%s must be an integer from -10 to 10" % band, request_id)
    elif method == "set_setting":
        _strict_keys(params, ("generation", "key", "value"), (), request_id)
        _generation(params, request_id)
        if params["key"] not in SETTING_KEYS:
            _fail("key is not a supported setting", request_id)
        value = params["value"]
        key = params["key"]
        boolean_keys = ("auto_pause", "wind_block", "anc_toggle")
        if key in boolean_keys and not isinstance(value, bool):
            _fail("%s must be a boolean" % key, request_id)
        elif key == "sidetone" and value not in ("off", "low", "medium", "high"):
            _fail("sidetone has an invalid value", request_id)
        elif key == "spatial_audio" and value not in ("off", "room", "motion"):
            _fail("spatial_audio has an invalid value", request_id)
    elif method == "get_profiles":
        _strict_keys(params, ("generation",), (), request_id)
        _generation(params, request_id)
    elif method == "set_profile":
        _strict_keys(params, ("generation", "profile"), (), request_id)
        _generation(params, request_id)
        _validate_profile(params["profile"], request_id)
    elif method == "delete_profile":
        _strict_keys(params, ("generation", "profile_id"), (), request_id)
        _generation(params, request_id)
        _nonempty_string(params["profile_id"], "profile_id", request_id)
    else:
        _fail("unknown method: %s" % method, request_id, "UNKNOWN_METHOD")
    return params


def parse_request_line(line):
    if isinstance(line, bytes):
        try:
            text = line.decode("utf-8")
        except UnicodeDecodeError:
            raise ProtocolError("INVALID_REQUEST", "request must be UTF-8")
    else:
        text = line
    if not isinstance(text, str):
        raise ProtocolError("INVALID_REQUEST", "request line must be text")
    if len(text.encode("utf-8")) > MAX_LINE_BYTES:
        raise ProtocolError("REQUEST_TOO_LARGE", "request exceeds 65536 bytes")
    try:
        message = json.loads(
            text,
            parse_constant=lambda value: (_ for _ in ()).throw(
                ValueError("non-finite JSON number: %s" % value)
            ),
        )
    except (TypeError, ValueError):
        raise ProtocolError("INVALID_REQUEST", "request is not valid JSON")
    if not isinstance(message, dict):
        raise ProtocolError("INVALID_REQUEST", "request must be a JSON object")

    request_id = message.get("id") if isinstance(message.get("id"), str) else ""
    extra = set(message) - {"v", "id", "method", "params"}
    if extra:
        _fail(
            "request contains unknown fields: %s" % ", ".join(sorted(extra)),
            request_id,
            "INVALID_REQUEST",
        )
    if message.get("v") != PROTOCOL_VERSION or isinstance(message.get("v"), bool):
        _fail("unsupported protocol version", request_id, "UNSUPPORTED_PROTOCOL")
    if not request_id or len(request_id) > MAX_REQUEST_ID_LENGTH:
        _fail(
            "id must be a non-empty string of at most 128 characters",
            "",
            "INVALID_REQUEST",
        )
    method = message.get("method")
    if method not in METHODS:
        _fail("unknown method", request_id, "UNKNOWN_METHOD")
    params = validate_params(method, message.get("params"), request_id)
    return Request(request_id, method, params)


def success_response(request_id, result):
    return {"v": PROTOCOL_VERSION, "id": request_id, "result": result}


def error_response(request_id, code, message, retryable=False, details=None):
    error = {"code": code, "message": message, "retryable": bool(retryable)}
    if details is not None:
        error["details"] = details
    return {"v": PROTOCOL_VERSION, "id": request_id, "error": error}


def event_message(event, data):
    if event not in EVENTS:
        raise ValueError("unknown event: %s" % event)
    return {"v": PROTOCOL_VERSION, "event": event, "data": data}


def encode_message(message):
    encoded = json.dumps(
        message, ensure_ascii=False, allow_nan=False, separators=(",", ":")
    )
    if len(encoded.encode("utf-8")) + 1 > MAX_LINE_BYTES:
        raise ProtocolError("REQUEST_TOO_LARGE", "message exceeds 65536 bytes")
    return encoded + "\n"
