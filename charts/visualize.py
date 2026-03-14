import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.gridspec as gridspec
import matplotlib.ticker as mticker
import seaborn as sns
import os
import sys

# ── Config ───────────────────────────────────────────────────────────────────
CSV_PATH = os.path.join(os.path.dirname(__file__), "..", "output", "transactions.csv")
OUT_DIR  = os.path.dirname(__file__)

DARK_BG      = "#0d1117"
PANEL_BG     = "#161b22"
ACCENT_CYAN  = "#39d0d8"
ACCENT_GREEN = "#3fb950"
ACCENT_RED   = "#f85149"
ACCENT_PURPLE= "#bc8cff"
ACCENT_YELLOW= "#e3b341"
TEXT_MUTED   = "#8b949e"
TEXT_MAIN    = "#e6edf3"

plt.rcParams.update({
    "figure.facecolor":  DARK_BG,
    "axes.facecolor":    PANEL_BG,
    "axes.edgecolor":    "#30363d",
    "axes.labelcolor":   TEXT_MAIN,
    "axes.titlecolor":   TEXT_MAIN,
    "axes.titlesize":    12,
    "axes.labelsize":    10,
    "xtick.color":       TEXT_MUTED,
    "ytick.color":       TEXT_MUTED,
    "xtick.labelsize":   8,
    "ytick.labelsize":   8,
    "grid.color":        "#21262d",
    "grid.linestyle":    "--",
    "grid.alpha":        0.6,
    "legend.facecolor":  PANEL_BG,
    "legend.edgecolor":  "#30363d",
    "legend.labelcolor": TEXT_MAIN,
    "legend.fontsize":   8,
    "text.color":        TEXT_MAIN,
    "font.family":       "monospace",
})

# ── Load data ─────────────────────────────────────────────────────────────────
def load_data(path: str) -> pd.DataFrame:
    if not os.path.exists(path):
        print(f"[ERROR] CSV not found: {path}")
        print("  → Run the Rust tracker first to generate output/transactions.csv")
        sys.exit(1)

    df = pd.read_csv(path)
    df["block_time"] = pd.to_datetime(df["block_time"], errors="coerce")
    df["amount_sol"] = pd.to_numeric(df["amount_sol"], errors="coerce").fillna(0)
    df["fee_sol"]    = pd.to_numeric(df["fee_sol"],    errors="coerce").fillna(0)
    return df

# ── Chart helpers ─────────────────────────────────────────────────────────────
def styled_ax(ax, title: str):
    ax.set_title(title, pad=12, fontsize=11, color=TEXT_MAIN, fontweight="bold")
    ax.grid(True, axis="y")
    ax.spines[["top","right","left","bottom"]].set_visible(False)

# ── 1. Transaction type distribution (donut) ──────────────────────────────────
def chart_tx_types(ax, df):
    counts = df["tx_type"].value_counts()
    colors = [ACCENT_CYAN, ACCENT_PURPLE, ACCENT_YELLOW, ACCENT_GREEN][:len(counts)]
    wedges, texts, autotexts = ax.pie(
        counts,
        labels=None,
        autopct="%1.1f%%",
        colors=colors,
        startangle=140,
        wedgeprops={"width": 0.55, "edgecolor": DARK_BG, "linewidth": 2},
        pctdistance=0.75,
    )
    for at in autotexts:
        at.set_fontsize(8)
        at.set_color(TEXT_MAIN)
    ax.legend(
        wedges, counts.index.tolist(),
        loc="lower center",
        ncol=2,
        bbox_to_anchor=(0.5, -0.12),
        framealpha=0,
    )
    styled_ax(ax, "Transaction Type Distribution")

# ── 2. SOL flow over time (line) ──────────────────────────────────────────────
def chart_sol_flow(ax, df):
    sol = df[df["token_symbol"] == "SOL"].copy()
    sol = sol.dropna(subset=["block_time"]).sort_values("block_time")

    sol_in  = sol[sol["direction"] == "IN"]
    sol_out = sol[sol["direction"] == "OUT"]

    ax.plot(sol_in["block_time"],  sol_in["amount_sol"],
            "o-", color=ACCENT_GREEN,  markersize=4, linewidth=1.5, label="IN")
    ax.plot(sol_out["block_time"], sol_out["amount_sol"],
            "o-", color=ACCENT_RED,    markersize=4, linewidth=1.5, label="OUT")

    ax.legend()
    ax.xaxis.set_major_formatter(plt.matplotlib.dates.DateFormatter("%m-%d\n%H:%M"))
    ax.yaxis.set_major_formatter(mticker.FormatStrFormatter("%.4f"))
    ax.set_ylabel("Amount (SOL)")
    styled_ax(ax, "SOL Flow Over Time")

# ── 3. Transaction status bar ─────────────────────────────────────────────────
def chart_status(ax, df):
    counts = df["status"].value_counts()
    color_map = {"SUCCESS": ACCENT_GREEN, "FAILED": ACCENT_RED}
    colors = [color_map.get(s, ACCENT_YELLOW) for s in counts.index]
    bars = ax.bar(counts.index, counts.values, color=colors,
                  width=0.45, edgecolor=DARK_BG, linewidth=1.5)
    for bar, val in zip(bars, counts.values):
        ax.text(bar.get_x() + bar.get_width() / 2, bar.get_height() + 0.3,
                str(val), ha="center", va="bottom", fontsize=9, color=TEXT_MAIN)
    ax.set_ylabel("Count")
    styled_ax(ax, "Transaction Status")

# ── 4. Fee distribution (histogram) ──────────────────────────────────────────
def chart_fees(ax, df):
    fees_lamports = (df["fee_sol"] * 1e9).astype(int)
    ax.hist(fees_lamports, bins=15, color=ACCENT_PURPLE,
            edgecolor=DARK_BG, linewidth=1, alpha=0.9)
    ax.set_xlabel("Fee (lamports)")
    ax.set_ylabel("Count")
    styled_ax(ax, "Fee Distribution")

# ── 5. Direction breakdown ────────────────────────────────────────────────────
def chart_direction(ax, df):
    sol_df = df[df["token_symbol"] == "SOL"]
    totals = sol_df.groupby("direction")["amount_sol"].sum().reindex(["IN","OUT","N/A"]).fillna(0)
    color_map = {"IN": ACCENT_GREEN, "OUT": ACCENT_RED, "N/A": TEXT_MUTED}
    colors = [color_map.get(d, ACCENT_YELLOW) for d in totals.index]
    bars = ax.bar(totals.index, totals.values, color=colors,
                  width=0.45, edgecolor=DARK_BG, linewidth=1.5)
    for bar, val in zip(bars, totals.values):
        ax.text(bar.get_x() + bar.get_width() / 2, bar.get_height() + 0.001,
                f"{val:.4f}", ha="center", va="bottom", fontsize=8, color=TEXT_MAIN)
    ax.set_ylabel("Total SOL")
    styled_ax(ax, "SOL IN vs OUT (Total)")

# ── 6. Cumulative SOL balance ─────────────────────────────────────────────────
def chart_cumulative(ax, df):
    sol = df[df["token_symbol"] == "SOL"].copy()
    sol = sol.dropna(subset=["block_time"]).sort_values("block_time")
    sol["signed"] = sol.apply(
        lambda r: r["amount_sol"] if r["direction"] == "IN"
                  else -r["amount_sol"] if r["direction"] == "OUT"
                  else 0,
        axis=1,
    )
    sol["cumulative"] = sol["signed"].cumsum()

    ax.fill_between(sol["block_time"], sol["cumulative"],
                    alpha=0.25, color=ACCENT_CYAN)
    ax.plot(sol["block_time"], sol["cumulative"],
            color=ACCENT_CYAN, linewidth=2)
    ax.axhline(0, color=TEXT_MUTED, linewidth=0.8, linestyle="--")
    ax.xaxis.set_major_formatter(plt.matplotlib.dates.DateFormatter("%m-%d\n%H:%M"))
    ax.yaxis.set_major_formatter(mticker.FormatStrFormatter("%.4f"))
    ax.set_ylabel("Cumulative SOL")
    styled_ax(ax, "Cumulative SOL Balance Change")

# ── Main ──────────────────────────────────────────────────────────────────────
def main():
    df = load_data(CSV_PATH)
    print(f"[INFO] Loaded {len(df)} transactions from {CSV_PATH}")

    wallet = df["from_address"].mode()[0] if not df.empty else "unknown"
    short_wallet = wallet[:8] + "…" + wallet[-4:] if len(wallet) > 12 else wallet

    fig = plt.figure(figsize=(18, 11), facecolor=DARK_BG)
    fig.suptitle(
        f"Solana Wallet Tracker  ◎  {short_wallet}",
        fontsize=15, fontweight="bold", color=ACCENT_CYAN, y=0.98
    )

    gs = gridspec.GridSpec(2, 3, figure=fig, hspace=0.42, wspace=0.32)

    chart_tx_types  (fig.add_subplot(gs[0, 0]), df)
    chart_sol_flow  (fig.add_subplot(gs[0, 1:]),df)
    chart_status    (fig.add_subplot(gs[1, 0]), df)
    chart_fees      (fig.add_subplot(gs[1, 1]), df)
    chart_direction (fig.add_subplot(gs[1, 2]), df)

    # Cumulative gets its own second figure
    fig2, ax2 = plt.subplots(figsize=(14, 4), facecolor=DARK_BG)
    fig2.suptitle(
        f"Cumulative SOL  ◎  {short_wallet}",
        fontsize=12, color=ACCENT_CYAN
    )
    chart_cumulative(ax2, df)
    fig2.tight_layout()

    out1 = os.path.join(OUT_DIR, "dashboard.png")
    out2 = os.path.join(OUT_DIR, "cumulative.png")
    fig.savefig(out1,  dpi=150, bbox_inches="tight", facecolor=DARK_BG)
    fig2.savefig(out2, dpi=150, bbox_inches="tight", facecolor=DARK_BG)

    print(f"[✔] Saved → {out1}")
    print(f"[✔] Saved → {out2}")
    plt.show()

if __name__ == "__main__":
    main()