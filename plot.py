import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns


sns.set(style="whitegrid")
df = pd.read_csv("results.csv")

# normalize mode names
df["mode"] = df["mode"].str.lower()
df_grouped = (
    df.groupby(["mode", "d", "weight"], as_index=False)
      .agg({
          "avg_time_ms": "mean"
      })
)


# 📈 1. COST vs DEGREE (MAIN RESULT)
plt.figure()

sns.lineplot(
    data=df_grouped,
    x="d",
    y="avg_time_ms",
    hue="mode",
    marker="o"
)

plt.title("Encoding Cost vs Degree d")
plt.xlabel("Degree (d)")
plt.ylabel("Time per Encoding (ms)")
plt.legend(title="Construction")
plt.tight_layout()
plt.savefig("cost_vs_d.png", dpi=300)
plt.show(block=False)


# 📈 2. COST vs WEIGHT
plt.figure()

sns.lineplot(
    data=df_grouped,
    x="weight",
    y="avg_time_ms",
    hue="mode",
    style="d",        # 👈 KEY FIX
    marker="o"
)

plt.title("Encoding Cost vs Weight")
plt.xlabel("Weight")
plt.ylabel("Time per Encoding (ms)")
plt.legend(title="Construction")
plt.tight_layout()
plt.savefig("cost_vs_weight.png", dpi=300)
plt.show(block=False)

# 📊 3. SPEEDUP ANALYSIS (KEY PART)
# reshape table
pivot = df_grouped.pivot_table(
    index=["d", "weight"],
    columns="mode",
    values="avg_time_ms"
)

# compute % improvements
if "brakedown" in pivot.columns and "hybrid" in pivot.columns:
    pivot["hybrid_vs_brakedown_%"] = (
        (pivot["brakedown"] - pivot["hybrid"]) /
        pivot["brakedown"] * 100
    )

if "spielman" in pivot.columns and "hybrid" in pivot.columns:
    pivot["hybrid_vs_spielman_%"] = (
        (pivot["spielman"] - pivot["hybrid"]) /
        pivot["spielman"] * 100
    )

print("\n=== SPEEDUP TABLE (%) ===")
print(pivot.round(2))


# 📊 4. VISUALIZE SPEEDUP

pivot_reset = pivot.reset_index()

if "hybrid_vs_brakedown_%" in pivot_reset.columns:
    plt.figure()

    sns.lineplot(
        data=pivot_reset,
        x="d",
        y="hybrid_vs_brakedown_%",
        hue="weight",
        marker="o"
    )

    plt.title("Hybrid Speedup vs Brakedown (%)")
    plt.xlabel("Degree (d)")
    plt.ylabel("Speedup (%)")
    plt.legend(title="Weight")
    plt.tight_layout()
    plt.savefig("hybrid_vs_brakedown.png", dpi=300)
    plt.show(block=False)

if "hybrid_vs_spielman_%" in pivot_reset.columns:
    plt.figure()

    sns.lineplot(
        data=pivot_reset,
        x="d",
        y="hybrid_vs_spielman_%",
        hue="weight",
        marker="o"
    )

    plt.title("Hybrid Speedup vs Spielman (%)")
    plt.xlabel("Degree (d)")
    plt.ylabel("Speedup (%)")
    plt.legend(title="Weight")
    plt.tight_layout()
    plt.savefig("hybrid_vs_spielman.png", dpi=300)
    plt.show()


# 📊 5. SUMMARY PRINT (FOR REPORT)

if "hybrid_vs_brakedown_%" in pivot.columns:
    avg_speedup = pivot["hybrid_vs_brakedown_%"].mean()
    print(f"\nAverage Hybrid vs Brakedown speedup: {avg_speedup:.2f}%")

if "hybrid_vs_spielman_%" in pivot.columns:
    avg_speedup = pivot["hybrid_vs_spielman_%"].mean()
    print(f"Average Hybrid vs Spielman speedup: {avg_speedup:.2f}%")