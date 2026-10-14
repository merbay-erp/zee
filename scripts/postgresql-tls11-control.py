#!/usr/bin/env python3
"""K-256: legacy TLS only in an owned loopback positive-control fixture."""
import socket
import ssl
import struct
import time
import warnings


def validate_control(control):
    keys = {'protocol', 'server_protocol', 'hostname_verified', 'certificate_verified', 'cipher'}
    if not isinstance(control, dict) or set(control) != keys:
        raise ValueError('TLS1.1 capable-positive control is incomplete')
    if control['protocol'] != 'TLSv1.1' or control['server_protocol'] != 'TLSv1.1':
        raise ValueError('TLS1.1 was not actually negotiated and observed by server')
    if control['hostname_verified'] is not True or control['certificate_verified'] is not True:
        raise ValueError('TLS1.1 positive control did not verify hostname and CA')
    cipher = control['cipher']
    if not isinstance(cipher, str) or not 1 <= len(cipher) <= 128 or not cipher.isascii() or any(ord(c) < 33 for c in cipher):
        raise ValueError('TLS1.1 positive cipher is absent or malformed')


def probe(port, ca):
    # This context never reaches production/native TLS settings. Lowering the
    # cipher security level is needed solely to prove the fixture supports TLS1.1.
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
    with warnings.catch_warnings():
        warnings.simplefilter('ignore', DeprecationWarning)
        context.minimum_version = ssl.TLSVersion.TLSv1_1
        context.maximum_version = ssl.TLSVersion.TLSv1_1
    context.set_ciphers('ALL:@SECLEVEL=0')
    context.load_verify_locations(cafile=str(ca))
    deadline = time.monotonic() + 5

    def remaining():
        budget = deadline-time.monotonic()
        if budget <= 0:
            raise TimeoutError('TLS1.1 positive control deadline')
        return budget

    def send(stream, data):
        stream.settimeout(remaining())
        stream.sendall(data)

    def receive(stream, size):
        value = bytearray()
        while len(value) < size:
            stream.settimeout(remaining())
            chunk = stream.recv(size-len(value))
            if not chunk:
                raise ValueError('TLS1.1 positive control EOF')
            value.extend(chunk)
        return bytes(value)

    def frame(stream):
        kind = receive(stream, 1)
        size = struct.unpack('!I', receive(stream, 4))[0]
        if not 4 <= size <= 65536:
            raise ValueError('TLS1.1 positive frame limit')
        return kind, receive(stream, size-4)

    with socket.create_connection(('127.0.0.1', port), timeout=remaining()) as raw:
        send(raw, struct.pack('!II', 8, 80877103))
        if receive(raw, 1) != b'S':
            raise ValueError('TLS1.1 fixture refused SSL request')
        raw.settimeout(remaining())
        with context.wrap_socket(raw, server_hostname='localhost') as stream:
            protocol = stream.version()
            cipher = stream.cipher()[0]
            body = struct.pack('!I', 196608) + b'user\0postgres\0database\0postgres\0\0'
            send(stream, struct.pack('!I', len(body)+4)+body)
            for _ in range(64):
                kind, data = frame(stream)
                if kind == b'E':
                    raise ValueError('TLS1.1 positive startup error')
                if kind == b'Z':
                    break
            else:
                raise ValueError('TLS1.1 startup never ready')
            query = b'SELECT version FROM pg_stat_ssl WHERE pid=pg_backend_pid()\0'
            send(stream, b'Q'+struct.pack('!I', len(query)+4)+query)
            observed = None
            for _ in range(64):
                kind, data = frame(stream)
                if kind == b'D':
                    if len(data) < 6 or struct.unpack('!H', data[:2])[0] != 1:
                        raise ValueError('TLS1.1 positive row shape')
                    size = struct.unpack('!i', data[2:6])[0]
                    if size < 0 or len(data) != 6+size:
                        raise ValueError('TLS1.1 positive row size')
                    observed = data[6:].decode('ascii')
                if kind == b'E':
                    raise ValueError('TLS1.1 positive query error')
                if kind == b'Z':
                    break
            else:
                raise ValueError('TLS1.1 query never ready')
            control = {'protocol': protocol, 'server_protocol': observed, 'cipher': cipher,
                       'hostname_verified': context.check_hostname,
                       'certificate_verified': context.verify_mode == ssl.CERT_REQUIRED}
            validate_control(control)
            send(stream, b'X'+struct.pack('!I', 4))
            return control
