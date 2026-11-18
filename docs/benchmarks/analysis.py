from fractions import Fraction as Fr

FORMATS = [(48,16),(32,32),(24,40),(16,48)]  # (int bits incl sign, frac bits), all in i64

def rnd_away(num, den):
    # round-to-nearest, ties away from zero, exact integer arithmetic
    if den < 0: num, den = -num, -den
    q, r = divmod(abs(num), den)
    if 2*r >= den: q += 1
    return q if num >= 0 else -q

def floor_div(num, den):
    return num // den

def to_fix(x, F, mode):
    x = Fr(x)
    num, den = x.numerator * (1 << F), x.denominator
    return rnd_away(num, den) if mode == "round" else floor_div(num, den)

def mul(a, b, F, mode):
    p = a * b
    return rnd_away(p, 1 << F) if mode == "round" else (p >> F)

print("== 1. Range / resolution ==")
for I, F in FORMATS:
    maxv = 2 ** (I-1)
    print(f"Q{I}.{F}: range ±{maxv:,}  ulp={2.0**-F:.3e}  "
          f"as-seconds overflow after {maxv/31557600:,.1f} years")

print("\n== 2. Which formats hold a map coordinate (needs ±half-extent)? ==")
for ext_km in (8, 32, 128):
    half_m = ext_km * 1000 / 2
    ok = [f"Q{I}.{F}" for I, F in FORMATS if half_m < 2 ** (I-1)]
    print(f"{ext_km:>4} km map (±{half_m:,.0f} m): fits -> {ok}")

print("\n== 3. dt representability and drift (v=15 m/s, pos += v*dt, 1e6 ticks) ==")
for tps in (20, 60):
    dt = Fr(1, tps); v = Fr(15); N = 1_000_000
    exact = float(v * dt * N)
    for I, F in FORMATS:
        for mode in ("floor", "round"):
            fv, fdt = to_fix(v, F, mode), to_fix(dt, F, mode)
            pos = 0
            for _ in range(N):
                pos += mul(fv, fdt, F, mode)
            err = float(Fr(pos, 1 << F)) - exact
            print(f"{tps:>2} TPS Q{I}.{F} {mode:5}: error after {N:,} ticks = {err:+.6f} m")

print("\n== 4. Rounding bias: acc += x*c, c=0.3, x=1.0001, 1e6 steps ==")
c, x, N = Fr(3, 10), Fr(10001, 10000), 1_000_000
exact = float(c * x * N)
for I, F in FORMATS:
    for mode in ("floor", "round"):
        fc, fx = to_fix(c, F, mode), to_fix(x, F, mode)
        acc = 0
        for _ in range(N):
            acc += mul(fc, fx, F, mode)
        print(f"Q{I}.{F} {mode:5}: err = {float(Fr(acc,1<<F)) - exact:+.3e}")

print("\n== 5. Intermediate width for mul / div ==")
for I, F in FORMATS:
    print(f"Q{I}.{F}: i64*i64 needs up to 128 bits; div pre-shift (a<<{F}) needs {64+F} bits -> i128")
