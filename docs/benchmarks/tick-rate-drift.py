# CitySim-rs — tick-rate 决策辅助（2.10 参数基线"tick 频率"行）
# Status: Proposed. 2026-11-18.
# 数值模型与 analysis.py 一致（精确有理数模拟，无浮点累积）。
# 用途：为 2.10 的 10/20/30 Hz 候选提供 dt 表示误差、漂移预算与"最大持续运行 tick 数"口径数据。
from fractions import Fraction as Fr

FORMATS = [(32, 32), (24, 40)]
CANDIDATES = (10, 20, 30)
REFERENCE = (16, 32)          # 2 的幂：dt = 1/tps 精确可表示，漂移为 0
N_LIST = (1_000_000, 100_000_000, 1_000_000_000)
V = 15                         # m/s，与 analysis.py 假设一致


def rnd_away(num, den):
    # round-to-nearest, ties away from zero, exact integer arithmetic
    if den < 0:
        num, den = -num, -den
    q, r = divmod(abs(num), den)
    if 2 * r >= den:
        q += 1
    return q if num >= 0 else -q


def rows(tps_list, title):
    print(f"== {title} ==")
    for tps in tps_list:
        for I, F in FORMATS:
            raw = rnd_away(1 << F, tps)              # dt = 1/tps 的定点表示
            err = Fr(raw, 1 << F) - Fr(1, tps)       # 秒，精确有理数
            per_tick = V * err                       # 米/tick（v 为整数，乘法无二次舍入）
            rel_ppm = float(err * tps) * 1e6
            drifts = "  ".join(f"drift@{n:,} = {float(per_tick * n):+.6f} m" for n in N_LIST)
            print(f"{tps:>2} Hz Q{I}.{F}: dt_raw = {raw}  dt 误差 = {float(err):+.3e} s "
                  f"({rel_ppm:+.3f} ppm)  {drifts}")
    print()


rows(CANDIDATES, "2.10 候选：dt = 1/tps 的表示误差与漂移（v = 15 m/s）")
rows(REFERENCE, "参考：2 的幂 tick 频率（dt 精确可表示，理想漂移 = 0）")

print("== 漂移预算 vs 最大持续运行 tick 数（2.10 新增行，Q32.32） ==")
print("drift(N) = N * v * (dt_fix - 1/tps)；dt 舍入误差是确定性系统偏差（非统计噪声）")
for budget in (0.001, 1.0):
    for tps in CANDIDATES:
        raw = rnd_away(1 << 32, tps)
        per_tick = abs(float(Fr(raw, 1 << 32) - Fr(1, tps)) * V)
        n = budget / per_tick
        print(f"漂移预算 {budget:g} m @ {tps:>2} Hz: N_max ≈ {n:.3g} ticks "
              f"(= {n / tps / 86400:.1f} 天连续运行)")
print()

print("== 交叉验证（模拟 == 解析）==")
for tps in CANDIDATES + REFERENCE:
    for I, F in FORMATS:
        raw = rnd_away(1 << F, tps)
        step = V * raw                       # 每 tick 原始增量（整数，精确）
        pos = step * 1_000_000               # 模拟 1e6 ticks（加法精确，无累积舍入）
        sim = float(Fr(pos, 1 << F) - Fr(V * 1_000_000, tps))
        analytic = float((Fr(raw, 1 << F) - Fr(1, tps)) * V * 1_000_000)
        assert abs(sim - analytic) < 1e-12, (tps, I, F, sim, analytic)
        print(f"{tps:>2} Hz Q{I}.{F}: 模拟 {sim:+.6f} m == 解析 {analytic:+.6f} m  OK")
print()

print("== 结论（供 2.10 tick 频率行定案参考）==")
print("1. Q32.32 三候选漂移同数量级（最坏约 1.7 mm/1e6 ticks），远小于任何合理的空间分辨率；")
print("   漂移不构成 tick 频率的决定性约束，选型应由模拟精度（交通/经济步进平滑度）与每 tick 负载决定。")
print("2. 10/20/30 Hz 的 dt 在二进制下均非精确表示，必须 from_ratio 派生并把舍入误差写入黄金测试（与修订五一致）。")
print("3. dt 舍入误差是固定系统偏差：10/20/30 Hz 均向上舍入（正偏差），floor 模式均向下（负偏差）；")
print("   '就近舍入'不消除漂移，只改变符号与量级。若要求极长运行（>1e10 ticks）漂移预算 < 1 m，")
print("   可考虑 2 的幂频率（16/32 Hz，dt 精确、漂移为 0）或其它漂移补偿机制（另行 ADR）。")
