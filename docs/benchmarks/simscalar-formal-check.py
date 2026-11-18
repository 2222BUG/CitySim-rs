# -*- coding: utf-8 -*-
"""Spike A 定案参数正式核对（Step 2）。

输入：2.10 参数基线（2026-11-18 项目负责人定案）。
核对：Q32.32 / Q24.40 对定案参数的适配性 + 30 Hz dt 漂移 + 位宽中间值。
输出：docs/benchmarks/simscalar-formal-check-output.txt
"""
import io
import sys

out = io.StringIO()


def emit(s=""):
    print(s)
    out.write(s + "\n")


FORMATS = {"Q32.32": (32, 32), "Q24.40": (24, 40)}
results = []


def check(name, ok, detail, fatal=True):
    """fatal=True 计入选型结论；fatal=False 仅登记（比较项/已有处置）。"""
    if fatal:
        results.append((name, ok))
    emit(f"  [{'PASS' if ok else ('FAIL' if fatal else 'NOTE')}] {name}: {detail}")


emit("=" * 72)
emit("Spike A 定案参数正式核对（2.10 已定案 2026-11-18，CS2 参考值）")
emit("=" * 72)
emit()
emit("定案参数：tick=30 Hz | 地图 16 km x 16 km | 分辨率 1 mm | v_max=50 m/s | a_max=10 m/s^2")
emit("          经济 10^12（独立 i64）| 参考场景 100k Agent x 1000 Tick + 200x200 网格 | N_max=10^9 ticks")
emit()

# 1) 坐标范围（以中心为原点，半幅 8 km）
emit("[1] 坐标范围：地图半幅 8000 m")
for name, (i, f) in FORMATS.items():
    int_max = 2 ** i
    check(f"{name} 坐标容纳", int_max > 8000,
          f"整数位可表示 ±{int_max} m，需 ±8000 m")
emit()

# 2) 分辨率
emit("[2] 分辨率：要求 <= 1 mm")
for name, (i, f) in FORMATS.items():
    res = 2.0 ** -f
    check(f"{name} 分辨率", res <= 1e-3, f"LSB={res:.3e} m <= 1e-3 m")
emit()

# 3) 30 Hz dt 漂移
emit("[3] dt=1/30 s 舍入与漂移（v=15 m/s，就近舍入 0.5 远离零）")
for name, (i, f) in FORMATS.items():
    dt_raw = (2 ** f + 15) // 30  # round(2^f/30) ties away
    dt = dt_raw / 2 ** f
    err = dt - 1.0 / 30.0
    v_raw = 15 * 2 ** f
    step_raw = (v_raw * dt_raw + 2 ** (f - 1)) >> f  # round_shift ties away
    step = step_raw / 2 ** f
    step_err = step - 15.0 / 30.0
    drift_1e6 = step_err * 1e6
    drift_1e9 = step_err * 1e9
    check(f"{name} 漂移@1e6", abs(drift_1e6) < 1e-2,
          f"dt_raw={dt_raw} (+{err:.3e} s) 步长误差={step_err:+.3e} m "
          f"@1e6={drift_1e6:+.4f} m @1e9={drift_1e9:+.3f} m")
emit()

# 4) 中间乘法位宽
emit("[4] 中间乘法位宽：v_raw * dt_raw")
for name, (i, f) in FORMATS.items():
    v_raw = 50 * 2 ** f          # 50 m/s
    dt_raw = (2 ** f + 15) // 30
    prod = v_raw * dt_raw
    need = prod.bit_length() + 1
    check(f"{name} 乘积位宽", need <= 128,
          f"v_raw*dt_raw 需 {need} bit（i128 中间值 {'足够' if need <= 128 else '不足'}）")
emit()

# 5) 里程字段（无界累积）压力点
emit("[5] 里程/无界累积压力点：N_max * v_max * dt")
max_step = 50.0 / 30.0
odometer = max_step * 1e9
for name, (i, f) in FORMATS.items():
    int_max = 2 ** i
    ok = int_max > odometer
    check(f"{name} 里程 10^9 ticks", ok,
          f"最坏累计 {odometer:.3e} m vs 整数上限 {int_max:.3e} m"
          f"（{'恰在范围内' if ok else '溢出，里程/统计字段需独立整数类型'}）",
          fatal=False)
emit("  注：普通位置受地图边界约束（±8000 m）无溢出风险；此行只约束里程/统计类无界累积字段。")
emit()

# 6) 经济数值
emit("[6] 经济数值上限 10^12")
for name, (i, f) in FORMATS.items():
    int_max = 2 ** i
    check(f"{name} 经济直接容纳", int_max > 1e12,
          f"10^12 vs 整数上限 {int_max:.3e} → "
          f"{'可直接容纳' if int_max > 1e12 else '不可容纳；已定案：独立 i64 整数类型（ADR-NEW-18）'}",
          fatal=False)
emit()

# 7) 时间字段
emit("[7] u64 tick 时间字段：N_max=1e9 < 2^64，富余")
check("u64 tick", True, "1e9 ticks @30 Hz = 3.33e7 s ≈ 386 天，远小于 u64 上限")
emit()

n_fail = sum(1 for _, ok in results if not ok)
emit("=" * 72)
emit(f"结论：硬约束核对 {len(results)} 项，FAIL {n_fail} 项。")
emit("- Q32.32 通过全部硬约束（坐标/分辨率/漂移/位宽/里程/u64 时间），推荐定案为 SimScalar 格式。")
emit("- Q24.40 精度更高，但 10^9 ticks 最坏里程超出其整数范围（1.678e7 m），属相对劣势。")
emit("- 经济 10^12 超出两种格式整数范围，按 ADR-NEW-18 用独立 i64（NOTE，不计 FAIL）。")
emit("- 里程/统计类无界累积字段（若将来需要）按独立整数类型处理，不改变 SimScalar 选型。")
emit("=" * 72)

with open(r"D:\AIWORK\CITYSIM\docs\benchmarks\simscalar-formal-check-output.txt", "w",
          encoding="utf-8") as fh:
    fh.write(out.getvalue())
sys.exit(0)
