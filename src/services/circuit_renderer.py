import io
from typing import Dict

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.patches import Circle, FancyBboxPatch


def dibujar_circuito(vals: Dict[str, float]) -> io.BytesIO:
    fig, ax = plt.subplots(figsize=(10.8, 5.4))
    fig.patch.set_facecolor("#eef2f7")
    ax.set_facecolor("#eef2f7")
    ax.axis("off")

    # Panel del diagrama para mejorar el contraste visual.
    panel = FancyBboxPatch(
        (0.3, 0.45),
        9.8,
        4.4,
        boxstyle="round,pad=0.02,rounding_size=0.12",
        facecolor="#ffffff",
        edgecolor="#c9d2df",
        linewidth=1.8,
        zorder=0,
    )
    ax.add_patch(panel)

    x0, x1, x2, x3 = 1.0, 3.75, 6.5, 9.25
    yb, yt = 1.45, 3.85
    wire_color = "#1f2937"

    def draw_wire(xs, ys):
        ax.plot(xs, ys, color=wire_color, lw=3.1, solid_capstyle="round", zorder=2)

    def draw_resistor_h(x_center, y, label):
        resistor = FancyBboxPatch(
            (x_center - 0.62, y - 0.12),
            1.24,
            0.24,
            boxstyle="round,pad=0.01,rounding_size=0.04",
            facecolor="#fef3c7",
            edgecolor="#b45309",
            linewidth=1.7,
            zorder=3,
        )
        ax.add_patch(resistor)
        ax.text(x_center, y + 0.28, label, color="#7c2d12", fontsize=10.8, ha="center", va="bottom", fontweight="bold")

    def draw_resistor_v(x, y_center, label):
        resistor = FancyBboxPatch(
            (x - 0.12, y_center - 0.62),
            0.24,
            1.24,
            boxstyle="round,pad=0.01,rounding_size=0.04",
            facecolor="#fee2e2",
            edgecolor="#9f1239",
            linewidth=1.7,
            zorder=3,
        )
        ax.add_patch(resistor)
        ax.text(x + 0.28, y_center, label, color="#881337", fontsize=10.8, ha="left", va="center", fontweight="bold")

    # Malla principal.
    draw_wire([x0, x3], [yt, yt])
    draw_wire([x0, x3], [yb, yb])
    draw_wire([x0, x0], [yb, yt])
    draw_wire([x1, x1], [yb, yt])
    draw_wire([x2, x2], [yb, yt])
    draw_wire([x3, x3], [yb, yt])

    for x in [x0, x1, x2, x3]:
        for y in [yb, yt]:
            ax.add_patch(Circle((x, y), 0.055, color="#111827", zorder=4))

    draw_resistor_h((x0 + x1) / 2, yt, f"R₁={vals['R1']}Ω")
    draw_resistor_h((x1 + x2) / 2, yt, f"R₃={vals['R3']}Ω")
    draw_resistor_h((x2 + x3) / 2, yt, f"R₅={vals['R5']}Ω")

    draw_resistor_v(x0, (yb + yt) / 2, f"R₂={vals['R2']}Ω")
    draw_resistor_v(x1, (yb + yt) / 2, f"R₄={vals['R4']}Ω")
    draw_resistor_v(x2, (yb + yt) / 2, f"R₆={vals['R6']}Ω")

    # Fuente principal.
    source_x, source_y = x0, yt - 0.46
    source = Circle((source_x, source_y), 0.2, fill=False, edgecolor="#0369a1", linewidth=2, zorder=4)
    ax.add_patch(source)
    ax.text(
        source_x - 0.16,
        source_y + 0.12,
        "+",
        color="#0369a1",
        fontsize=10.5,
        ha="center",
        va="center",
        fontweight="bold",
    )
    ax.text(
        source_x - 0.16,
        source_y - 0.12,
        "-",
        color="#0369a1",
        fontsize=10.5,
        ha="center",
        va="center",
        fontweight="bold",
    )
    ax.text(
        source_x - 0.42,
        source_y,
        f"V₁={vals['V1']}V",
        color="#0c4a6e",
        fontsize=10.8,
        ha="right",
        va="center",
        fontweight="bold",
    )

    def draw_current_arrow(x_start, x_end, label):
        y_arrow = yb - 0.22
        ax.annotate(
            "",
            xy=(x_end, y_arrow),
            xytext=(x_start, y_arrow),
            arrowprops=dict(arrowstyle="->", lw=2.2, color="#047857"),
            zorder=5,
        )
        ax.text(
            (x_start + x_end) / 2,
            y_arrow - 0.12,
            label,
            color="#065f46",
            fontsize=11.5,
            ha="center",
            va="top",
            fontweight="bold",
        )

    draw_current_arrow(x0 + 0.45, x1 - 0.45, "I₁")
    draw_current_arrow(x1 + 0.45, x2 - 0.45, "I₂")
    draw_current_arrow(x2 + 0.45, x3 - 0.45, "I₃")

    ax.text((x0 + x1) / 2, 0.78, "Malla 1\n(Cocina)", color="#334155", fontsize=10.2, ha="center", va="center")
    ax.text((x1 + x2) / 2, 0.78, "Malla 2\n(Sala)", color="#334155", fontsize=10.2, ha="center", va="center")
    ax.text((x2 + x3) / 2, 0.78, "Malla 3\n(Dormitorios)", color="#334155", fontsize=10.2, ha="center", va="center")

    ax.text(
        5.2,
        4.63,
        "Circuito de mallas - vista técnica",
        color="#1e293b",
        fontsize=13,
        ha="center",
        va="center",
        fontweight="bold",
    )

    ax.set_xlim(0, 10.4)
    ax.set_ylim(0.4, 5.0)

    plt.tight_layout()
    buf = io.BytesIO()
    plt.savefig(buf, format="png", bbox_inches="tight", facecolor=fig.get_facecolor())
    plt.close(fig)
    buf.seek(0)
    return buf
