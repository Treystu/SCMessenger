#!/usr/bin/env python3
"""RFC 7748 section 6.1 X25519 known-answer vectors, checked in pure Python.

Why this exists (`#545`). The `rand 0.8 -> 0.10` upgrade is justified by the
claim that it changes no derivation: `x25519-dalek` 3.0 moves to `rand_core` 0.10
and the DH math is untouched. The issue's acceptance asks for RFC 7748 section
6.1 vectors to pin that claim to a vector instead of to inspection. This script
is that vector check, and it runs without a Rust toolchain, so the constants the
Rust test will carry can be validated before the migration is attempted.

It is an independent implementation of the RFC text (section 5 pseudo-code) and
is not a substitute for the Rust suite: it proves the vectors and the arithmetic,
not that `scmessenger-core` compiles or interoperates.

Vectors are quoted from RFC 7748:

- section 6.1 (curve25519 Diffie-Hellman): Alice's and Bob's private keys, their
  public keys, and their shared secret.
- section 5.2 (single X25519 call): one input scalar / u-coordinate pair with its
  expected output.
- section 5.2 (iterated X25519): the one-iteration and 1,000-iteration results.
  The 1,000,000-iteration vector is quoted in the RFC but is not evaluated here:
  a pure-Python ladder would need minutes per run, and this script is meant to be
  cheap enough to run in CI. It is marked NOT EVALUATED in the output rather than
  silently omitted.

Exit code 0 when every evaluated vector matches, 1 otherwise.
"""

import sys

P = 2**255 - 19
A24 = 121665
BITS = 255


def _decode_scalar(raw: bytes) -> int:
    """Clamp the scalar as RFC 7748 section 5 requires."""
    if len(raw) != 32:
        raise ValueError("scalar must be 32 bytes")
    k = bytearray(raw)
    k[0] &= 248
    k[31] &= 127
    k[31] |= 64
    return int.from_bytes(k, "little")


def _decode_u(raw: bytes) -> int:
    """Decode an untrusted u-coordinate: mask the high bit, per the RFC."""
    if len(raw) != 32:
        raise ValueError("u-coordinate must be 32 bytes")
    u = bytearray(raw)
    u[31] &= 127
    return int.from_bytes(u, "little")


def x25519(scalar: bytes, u_coordinate: bytes) -> bytes:
    """X25519(scalar, u) as specified by RFC 7748 section 5."""
    k = _decode_scalar(scalar)
    x1 = _decode_u(u_coordinate)
    x2, z2 = 1, 0
    x3, z3 = x1, 1
    swap = 0

    for t in range(BITS - 1, -1, -1):
        k_t = (k >> t) & 1
        swap ^= k_t
        if swap:
            x2, x3 = x3, x2
            z2, z3 = z3, z2
        swap = k_t

        a = (x2 + z2) % P
        aa = a * a % P
        b = (x2 - z2) % P
        bb = b * b % P
        e = (aa - bb) % P
        c = (x3 + z3) % P
        d = (x3 - z3) % P
        da = d * a % P
        cb = c * b % P
        x3 = pow(da + cb, 2, P)
        diff = (da - cb) % P
        z3 = x1 * diff % P * diff % P
        x2 = aa * bb % P
        z2 = e * ((aa + A24 * e) % P) % P

    if swap:
        x2, x3 = x3, x2
        z2, z3 = z3, z2

    return (x2 * pow(z2, P - 2, P) % P).to_bytes(32, "little")


BASE_POINT = bytes([9] + [0] * 31)

# RFC 7748 section 6.1.
ALICE_PRIVATE = bytes.fromhex(
    "77076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a"
)
ALICE_PUBLIC = bytes.fromhex(
    "8520f0098930a754748b7ddcb43ef75a0dbf3a0d26381af4eba4a98eaa9b4e6a"
)
BOB_PRIVATE = bytes.fromhex(
    "5dab087e624a8a4b79e17f8b83800ee66f3bb1292618b6fd1c2f8b27ff88e0eb"
)
BOB_PUBLIC = bytes.fromhex(
    "de9edb7d7b7dc1b4d35b61c2ece435373f8343c85b78674dadfc7e146f882b4f"
)
SHARED_SECRET = bytes.fromhex(
    "4a5d9d5ba4ce2de1728e3bf480350f25e07e21c947d19e3376f09b3c1e161742"
)

# RFC 7748 section 5.2, single call.
SCALAR_INPUT = bytes.fromhex(
    "a546e36bf0527c9d3b16154b82465edd62144c0ac1fc5a18506a2244ba449ac4"
)
U_INPUT = bytes.fromhex(
    "e6db6867583030db3594c1a424b15f7c726624ec26b3353b10a903a6d0ab1c4c"
)
U_OUTPUT = bytes.fromhex(
    "c3da55379de9c6908e94ea4df28d084f32eccf03491c71f754b4075577a28552"
)

# RFC 7748 section 5.2, iterated calls.
ITERATION_1 = bytes.fromhex(
    "422c8e7a6227d7bca1350b3e2bb7279f7897b87bb6854b783c60e80311ae3079"
)
ITERATION_1000 = bytes.fromhex(
    "684cf59ba83309552800ef566f2f4d3c1c3887c49360e3875f2eb94d99532c51"
)


def main() -> int:
    failures = []

    def check(label, got, want):
        if got == want:
            print(f"[OK]   {label}: {got.hex()}")
        else:
            failures.append(label)
            print(f"[FAIL] {label}: got {got.hex()} want {want.hex()}")

    # Section 6.1: the derivation must be the RFC's, from the private keys.
    check("6.1 Alice public key", x25519(ALICE_PRIVATE, BASE_POINT), ALICE_PUBLIC)
    check("6.1 Bob public key", x25519(BOB_PRIVATE, BASE_POINT), BOB_PUBLIC)

    # Section 6.1: both sides reach the same secret, and it is the quoted one.
    alice_side = x25519(ALICE_PRIVATE, BOB_PUBLIC)
    bob_side = x25519(BOB_PRIVATE, ALICE_PUBLIC)
    check("6.1 Alice-side shared secret", alice_side, SHARED_SECRET)
    check("6.1 Bob-side shared secret", bob_side, SHARED_SECRET)
    if alice_side != bob_side:
        failures.append("6.1 shared secret disagreement")
        print("[FAIL] 6.1 the two sides disagree")

    # Section 5.2, single call.
    check("5.2 single call", x25519(SCALAR_INPUT, U_INPUT), U_OUTPUT)

    # Section 5.2, iterated calls: k starts at the base point and each round
    # feeds the previous result in as the next u-coordinate.
    k = BASE_POINT
    u = BASE_POINT
    for i in range(1, 1001):
        k, u = x25519(k, u), k
        if i == 1:
            check("5.2 after 1 iteration", k, ITERATION_1)
        elif i == 1000:
            check("5.2 after 1,000 iterations", k, ITERATION_1000)

    print("[INFO] 5.2 after 1,000,000 iterations: NOT EVALUATED in this script "
          "(pure-Python cost); the RFC value is quoted in the Rust test.")

    if failures:
        print(f"[FAIL] {len(failures)} vector(s) did not match: {failures}")
        return 1
    print("[DONE] every evaluated RFC 7748 vector matched")
    return 0


if __name__ == "__main__":
    sys.exit(main())
