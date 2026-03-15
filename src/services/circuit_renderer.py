import io
from typing import Dict

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.patches import Circle, FancyBboxPatch


def dibujar_circuito(vals: Dict[str, float], resistance_count: int = 6, voltage_count: int = 3) -> io.BytesIO:
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

    # El diagrama se compacta según la cantidad activa de componentes.
    mesh_count = max(1, min(3, max((resistance_count + 1) // 2, voltage_count)))

    x_start = 1.0
    spacing = 2.75
    boundaries = [x_start + i * spacing for i in range(mesh_count + 1)]
    yb, yt = 1.45, 3.85

    panel_width = 1.3 + mesh_count * spacing
    panel.set_width(panel_width)

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
        ax.text(
            x_center,
            y + 0.28,
            label,
            color="#7c2d12",
            fontsize=10.8,
            ha="center",
            va="bottom",
            fontweight="bold",
        )

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
        ax.text(
            x + 0.28,
            y_center,
            label,
            color="#881337",
            fontsize=10.8,
            ha="left",
            va="center",
            fontweight="bold",
        )

    # Malla principal.
    draw_wire([boundaries[0], boundaries[-1]], [yt, yt])
    draw_wire([boundaries[0], boundaries[-1]], [yb, yb])
    for x in boundaries:
        draw_wire([x, x], [yb, yt])

    for x in boundaries:
        for y in [yb, yt]:
            ax.add_patch(Circle((x, y), 0.055, color="#111827", zorder=4))

    # Resistencias activas según el conteo elegido por usuario.
    resistance_specs = [
        (1, "h", 0, f"R₁={vals['R1']}Ω"),
        (2, "v", 0, f"R₂={vals['R2']}Ω"),
        (3, "h", 1, f"R₃={vals['R3']}Ω"),
        (4, "v", 1, f"R₄={vals['R4']}Ω"),
        (5, "h", 2, f"R₅={vals['R5']}Ω"),
        (6, "v", 2, f"R₆={vals['R6']}Ω"),
    ]
    for idx, orientation, slot, label in resistance_specs:
        if idx > resistance_count or slot >= mesh_count:
            continue
        if orientation == "h":
            draw_resistor_h((boundaries[slot] + boundaries[slot + 1]) / 2, yt, label)
        else:
            draw_resistor_v(boundaries[slot], (yb + yt) / 2, label)

    def draw_voltage_source(x_source, label):
        source_y = yt - 0.46
        source = Circle((x_source, source_y), 0.2, fill=False, edgecolor="#0369a1", linewidth=2, zorder=4)
        ax.add_patch(source)
        ax.text(
            x_source - 0.16,
            source_y + 0.12,
            "+",
            color="#0369a1",
            fontsize=10.5,
            ha="center",
            va="center",
            fontweight="bold",
        )
        ax.text(
            x_source - 0.16,
            source_y - 0.12,
            "-",
            color="#0369a1",
            fontsize=10.5,
            ha="center",
            va="center",
            fontweight="bold",
        )
        ax.text(
            x_source - 0.42,
            source_y,
            label,
            color="#0c4a6e",
            fontsize=10.3,
            ha="right",
            va="center",
            fontweight="bold",
        )

    # Fuentes activas según conteo elegido por usuario.
    voltage_labels = [f"V₁={vals['V1']}V", f"V₂={vals['V2']}V", f"V₃={vals['V3']}V"]
    for i in range(min(voltage_count, mesh_count)):
        draw_voltage_source(boundaries[i], voltage_labels[i])

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

    mesh_names = ["Cocina", "Sala", "Dormitorios"]
    for i in range(mesh_count):
        draw_current_arrow(boundaries[i] + 0.45, boundaries[i + 1] - 0.45, f"I{i + 1}")
        ax.text(
            (boundaries[i] + boundaries[i + 1]) / 2,
            0.78,
            f"Malla {i + 1}\\n({mesh_names[i]})",
            color="#334155",
            fontsize=10.2,
            ha="center",
            va="center",
        )

    ax.text(
        (boundaries[0] + boundaries[-1]) / 2,
        4.63,
        f"Circuito de mallas adaptativo ({resistance_count}R, {voltage_count}V)",
        color="#1e293b",
        fontsize=13,
        ha="center",
        va="center",
        fontweight="bold",
    )

    ax.set_xlim(0, panel_width + 0.6)
    ax.set_ylim(0.4, 5.0)

    plt.tight_layout()
    buf = io.BytesIO()
    plt.savefig(buf, format="png", bbox_inches="tight", facecolor=fig.get_facecolor())
    plt.close(fig)
    buf.seek(0)
    return buf
