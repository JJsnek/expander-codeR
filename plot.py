import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns

sns.set(style="whitegrid")

# =========================
# LOAD DATA
# =========================

df = pd.read_csv("results.csv")
df["mode"] = df["mode"].str.lower()

# =========================
# GROUP DATA
# =========================

df_grouped = (
    df.groupby(["mode", "d", "weight"], as_index=False)
      .agg({"avg_time_ms": "mean"})
)

# Config label (Table-style)
df_grouped["config"] = (
    "(" + df_grouped["weight"].astype(str) +
    "," + df_grouped["d"].astype(str) + ")"
)

# =========================
# 1. TABLE-STYLE BAR PLOT
# =========================

plt.figure(figsize=(8,5))

sns.barplot(
    data=df_grouped,
    x="config",
    y="avg_time_ms",
    hue="mode"
)

plt.title("Encoding Cost Across Expander Constructions")
plt.xlabel("(c_n, d_n)")
plt.ylabel("Time (ms)")
plt.legend(title="Mode")
plt.tight_layout()
plt.savefig("table1_style.png", dpi=300)
plt.show(block=False)

# =========================
# 2. SPEEDUP TABLE
# =========================

pivot = df_grouped.pivot_table(
    index=["d", "weight"],
    columns="mode",
    values="avg_time_ms"
)

# Hybrid vs Brakedown
if "brakedown" in pivot.columns and "hybrid" in pivot.columns:
    pivot["hybrid_vs_brakedown_%"] = (
        (pivot["brakedown"] - pivot["hybrid"]) /
        pivot["brakedown"] * 100
    )

# Hybrid vs Spielman
if "spielman" in pivot.columns and "hybrid" in pivot.columns:
    pivot["hybrid_vs_spielman_%"] = (
        (pivot["spielman"] - pivot["hybrid"]) /
        pivot["spielman"] * 100
    )

# Brakedown vs Spielman
if "brakedown" in pivot.columns and "spielman" in pivot.columns:
    pivot["brakedown_vs_spielman_%"] = (
        (pivot["spielman"] - pivot["brakedown"]) /
        pivot["spielman"] * 100
    )

print("\n=== SPEEDUP TABLE (%) ===")
print(pivot.round(2))

pivot_reset = pivot.reset_index()

# =========================
# 3. SPEEDUP PLOTS
# =========================

# Hybrid vs Brakedown
plt.figure(figsize=(8,5))

if "hybrid_vs_brakedown_%" in pivot_reset.columns:
    sns.lineplot(
        data=pivot_reset,
        x="d",
        y="hybrid_vs_brakedown_%",
        hue="weight",
        style="weight",
        marker="o"
    )

    plt.axhline(0, linestyle="--")
    plt.title("Hybrid vs Brakedown Speedup (%)")
    plt.xlabel("d")
    plt.ylabel("Speedup (%)")
    plt.legend(title="Weight (c_n)")
    plt.tight_layout()
    plt.savefig("hybrid_vs_brakedown.png", dpi=300)
    plt.show(block=False)

# Hybrid vs Spielman
plt.figure(figsize=(8,5))

if "hybrid_vs_spielman_%" in pivot_reset.columns:
    sns.lineplot(
        data=pivot_reset,
        x="d",
        y="hybrid_vs_spielman_%",
        hue="weight",
        style="weight",
        marker="o"
    )

    plt.axhline(0, linestyle="--")
    plt.title("Hybrid vs Spielman Speedup (%)")
    plt.xlabel("d")
    plt.ylabel("Speedup (%)")
    plt.legend(title="Weight (c_n)")
    plt.tight_layout()
    plt.savefig("hybrid_vs_spielman.png", dpi=300)
    plt.show(block=False)

# Brakedown vs Spielman
plt.figure(figsize=(8,5))

if "brakedown_vs_spielman_%" in pivot_reset.columns:
    sns.lineplot(
        data=pivot_reset,
        x="d",
        y="brakedown_vs_spielman_%",
        hue="weight",
        style="weight",
        marker="o"
    )

    plt.axhline(0, linestyle="--")
    plt.title("Brakedown vs Spielman Speedup (%)")
    plt.xlabel("d")
    plt.ylabel("Speedup (%)")
    plt.legend(title="Weight (c_n)")
    plt.tight_layout()
    plt.savefig("brakedown_vs_spielman.png", dpi=300)
    plt.show()

# =========================
# 4. SUMMARY
# =========================

if "hybrid_vs_brakedown_%" in pivot.columns:
    avg_speedup = pivot["hybrid_vs_brakedown_%"].mean()
    print(f"\nAverage Hybrid vs Brakedown: {avg_speedup:.2f}%")

if "hybrid_vs_spielman_%" in pivot.columns:
    avg_speedup = pivot["hybrid_vs_spielman_%"].mean()
    print(f"Average Hybrid vs Spielman: {avg_speedup:.2f}%")

# =========================
# 5. N-SCALING EXPERIMENT
# =========================

df_n = pd.read_csv("results_n_scaling.csv")
df_n["mode"] = df_n["mode"].str.lower()

df_n_grouped = (
    df_n.groupby(["mode", "n"], as_index=False)
        .agg({"avg_time_ms": "mean"})
)

plt.figure(figsize=(8,5))

sns.lineplot(
    data=df_n_grouped,
    x="n",
    y="avg_time_ms",
    hue="mode",
    marker="o"
)

plt.title("Scalability of Expander-Based Encoding")
plt.xlabel("n")
plt.ylabel("Time (ms)")
plt.legend(title="Mode")
plt.tight_layout()
plt.savefig("cost_vs_n.png", dpi=300)
plt.show(block=False)

# =========================
# 6. COST MODEL VALIDATION
# =========================

df_c = pd.read_csv("results_cost_model.csv")

plt.figure(figsize=(7,5))

sns.scatterplot(
    data=df_c,
    x="model_value",
    y="measured_ms"
)

plt.title("Empirical Validation of Encoding Cost Model")
plt.xlabel("(1/(1-α)) * (c + (α/ρ)d)")
plt.ylabel("Measured Time (ms)")
plt.tight_layout()
plt.savefig("cost_model_validation.png", dpi=300)
plt.show()