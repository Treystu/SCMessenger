/**
 * Shamir's Secret Sharing Implementation
 * Uses GF(2^8) arithmetic with Rijndael's finite field (poly: 0x11b)
 */

// Rijndael's finite field polynomial: x^8 + x^4 + x^3 + x + 1 (0x11b)
// We precompute log and exp tables for fast multiplication/division
const LOG_TABLE = new Uint8Array(256);
const EXP_TABLE = new Uint8Array(256);

function initTables() {
  let x = 1;
  for (let i = 0; i < 255; i++) {
    EXP_TABLE[i] = x;
    LOG_TABLE[x] = i;

    // Multiply by 3: (x * 2) ^ x
    // x * 2 computation with reduction
    let x2 = x << 1;
    if (x & 0x80) {
      // Check high bit of byte (since x is byte)
      x2 ^= 0x11b;
    }
    // Result is x2 ^ x (which is x*3)
    x = x2 ^ x;
    // ensure x is byte
    x &= 0xff;
  }
  // LOG[0] is undefined ideally, but we won't use it
}

initTables();

// GF(2^8) Addition/Subtraction (XOR)
function add(a: number, b: number): number {
  return a ^ b;
}

// GF(2^8) Multiplication
function mul(a: number, b: number): number {
  if (a === 0 || b === 0) return 0;
  const logA = LOG_TABLE[a];
  const logB = LOG_TABLE[b];
  const logSum = (logA + logB) % 255;
  return EXP_TABLE[logSum];
}

// GF(2^8) Division
function div(a: number, b: number): number {
  if (b === 0) throw new Error("Division by zero");
  if (a === 0) return 0;
  const logA = LOG_TABLE[a];
  const logB = LOG_TABLE[b];
  const logDiff = (logA - logB + 255) % 255;
  return EXP_TABLE[logDiff];
}

/**
 * Generate n distinct random shares for a secret, such that any k are required to reconstruct.
 * @param secret The secret bytes (Uint8Array)
 * @param n Total number of shares to generate
 * @param k Threshold number of shares to reconstruct
 */
export function split(
  secret: Uint8Array,
  n: number,
  k: number,
): Array<{ x: number; y: Uint8Array }> {
  if (k > n) throw new Error("Threshold k cannot be greater than n");
  if (k < 1) throw new Error("Threshold k must be at least 1");

  const len = secret.length;
  const shares: Array<{ x: number; y: Uint8Array }> = [];

  // Initialize shares
  for (let i = 0; i < n; i++) {
    shares.push({ x: i + 1, y: new Uint8Array(len) });
  }

  // Generate random polynomials for each byte of the secret
  for (let i = 0; i < len; i++) {
    // Polynomial: s + a1*x + a2*x^2 + ... + ak-1*x^(k-1)
    // coeffs[0] is the secret byte
    const coeffs = new Uint8Array(k);
    coeffs[0] = secret[i];

    // Generate random coefficients a1...ak-1
    if (k > 1) {
      const randomBytes = new Uint8Array(k - 1);
      crypto.getRandomValues(randomBytes);
      for (let j = 1; j < k; j++) {
        coeffs[j] = randomBytes[j - 1];
      }
    }

    // Evaluate polynomial for each share (x=1..n)
    for (let j = 0; j < n; j++) {
      const x = j + 1;
      // Horner's method or direct? Direct is simpler for small k
      // y = c0 + c1*x + c2*x^2 ...

      // Correct Horner's:
      // P(x) = c0 + x(c1 + x(c2 + ...))
      let val = 0;
      // Reverse coefficient iteration for Horner's not totally standard,
      // let's do direct sum for clarity on small finite fields

      // Val = Sum( c_m * x^m )
      for (let m = 0; m < k; m++) {
        // Calculate x^m
        let xPow = 1;
        for (let p = 0; p < m; p++) xPow = mul(xPow, x);

        const term = mul(coeffs[m], xPow);
        val = add(val, term);
      }
      shares[j].y[i] = val;
    }
  }

  return shares;
}

/**
 * Reconstruct the secret from k shares.
 * @param shares The array of shares (objects with x and y)
 */
export function combine(
  shares: Array<{ x: number; y: Uint8Array }>,
): Uint8Array {
  if (shares.length === 0) throw new Error("No shares provided");
  const len = shares[0].y.length;
  // We need at least k shares, but we assume the provided shares ARE the k shares (or more)
  // We use Lagrange Interpolation at x=0

  const reconstructed = new Uint8Array(len);

  for (let i = 0; i < len; i++) {
    // For each byte position
    let secretByte = 0;

    for (let j = 0; j < shares.length; j++) {
      const { x: xj, y: yBlock } = shares[j];
      const yj = yBlock[i];

      // Compute Lagrange basis polynomial L_j(0)
      // L_j(0) = Product ( (0 - xm) / (xj - xm) ) for m != j
      let numerator = 1;
      let denominator = 1;

      for (let m = 0; m < shares.length; m++) {
        if (m === j) continue;
        const xm = shares[m].x;

        // (0 - xm) = -xm = xm in GF(2^n) because addition is XOR
        numerator = mul(numerator, xm);
        // (xj - xm) = xj ^ xm
        denominator = mul(denominator, add(xj, xm));
      }

      const lagrange = mul(numerator, div(1, denominator)); // num * (1/den)
      const term = mul(yj, lagrange);

      secretByte = add(secretByte, term);
    }

    reconstructed[i] = secretByte;
  }

  return reconstructed;
}
