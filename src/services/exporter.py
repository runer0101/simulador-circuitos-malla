"""Servicio de exportación de resultados a CSV y PDF."""

from __future__ import annotations

import csv
import io
from datetime import datetime, timezone
from typing import Any, Dict, List

from reportlab.lib import colors
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.lib.units import mm
from reportlab.platypus import (
    Paragraph,
    SimpleDocTemplate,
    Spacer,
    Table,
    TableStyle,
)


def _build_rows(simulation: Dict[str, Any]) -> List[List[str]]:
    params = simulation.get("params", {})
    currents = simulation.get("currents", {})
    interpretations = simulation.get("interpretations", {})

    rows: List[List[str]] = [
        ["Malla", "Corriente (A)", "Interpretación"],
    ]
    for key in ("I1", "I2", "I3"):
        value = currents.get(key, 0.0)
        rows.append([key, f"{value:.6f}", interpretations.get(key, "")])
    rows.append(["Potencia total (W)", f"{simulation.get('power_total', 0.0):.4f}", ""])
    rows.append(["--- Parámetros ---", "", ""])
    for key in ("R1", "R2", "R3", "R4", "R5", "R6", "V1", "V2", "V3"):
        rows.append([key, f"{float(params.get(key, 0.0)):.4f}", ""])
    return rows


def build_csv(simulation: Dict[str, Any]) -> bytes:
    buffer = io.StringIO()
    writer = csv.writer(buffer)
    writer.writerow(["simulador-circuitos-malla"])
    writer.writerow(["id", simulation.get("id", "")])
    writer.writerow(["timestamp", simulation.get("timestamp", "")])
    writer.writerow([])
    for row in _build_rows(simulation):
        writer.writerow(row)
    return buffer.getvalue().encode("utf-8")


def build_pdf(simulation: Dict[str, Any]) -> bytes:
    buffer = io.BytesIO()
    doc = SimpleDocTemplate(
        buffer,
        pagesize=A4,
        leftMargin=18 * mm,
        rightMargin=18 * mm,
        topMargin=18 * mm,
        bottomMargin=18 * mm,
        title="Reporte de simulación de mallas",
    )
    styles = getSampleStyleSheet()
    title_style = ParagraphStyle(
        "title",
        parent=styles["Heading1"],
        textColor=colors.HexColor("#0f766e"),
        spaceAfter=8,
    )
    subtitle_style = ParagraphStyle(
        "subtitle",
        parent=styles["Normal"],
        textColor=colors.HexColor("#4e5a62"),
        fontSize=9,
        spaceAfter=12,
    )
    body_style = styles["BodyText"]
    section_style = ParagraphStyle(
        "section",
        parent=styles["Heading2"],
        textColor=colors.HexColor("#18353d"),
        spaceBefore=10,
        spaceAfter=6,
    )

    story: List[Any] = []
    story.append(Paragraph("Reporte de simulación de mallas", title_style))
    timestamp = simulation.get("timestamp") or datetime.now(timezone.utc).isoformat()
    sim_id = simulation.get("id", "n/a")
    story.append(
        Paragraph(
            f"<b>ID:</b> {sim_id}<br/><b>Fecha:</b> {timestamp}",
            subtitle_style,
        )
    )

    story.append(Paragraph("Resultados de corrientes", section_style))
    currents = simulation.get("currents", {})
    interpretations = simulation.get("interpretations", {})
    currents_table = Table(
        [
            ["Malla", "Corriente (A)", "Interpretación"],
            *[
                [
                    key,
                    f"{float(currents.get(key, 0.0)):.6f}",
                    Paragraph(interpretations.get(key, ""), body_style),
                ]
                for key in ("I1", "I2", "I3")
            ],
        ],
        colWidths=[28 * mm, 38 * mm, 100 * mm],
    )
    currents_table.setStyle(
        TableStyle(
            [
                ("BACKGROUND", (0, 0), (-1, 0), colors.HexColor("#0f766e")),
                ("TEXTCOLOR", (0, 0), (-1, 0), colors.whitesmoke),
                ("FONTNAME", (0, 0), (-1, 0), "Helvetica-Bold"),
                ("GRID", (0, 0), (-1, -1), 0.4, colors.HexColor("#bcb9ac")),
                ("VALIGN", (0, 0), (-1, -1), "TOP"),
                ("LEFTPADDING", (0, 0), (-1, -1), 6),
                ("RIGHTPADDING", (0, 0), (-1, -1), 6),
                ("TOPPADDING", (0, 0), (-1, -1), 6),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 6),
            ]
        )
    )
    story.append(currents_table)
    story.append(Spacer(1, 6 * mm))

    story.append(Paragraph("Parámetros del circuito", section_style))
    params = simulation.get("params", {})
    param_keys = ("R1", "R2", "R3", "R4", "R5", "R6", "V1", "V2", "V3")
    params_table = Table(
        [
            ["Componente", "Valor"],
            *[[key, f"{float(params.get(key, 0.0)):.4f}"] for key in param_keys],
        ],
        colWidths=[60 * mm, 50 * mm],
    )
    params_table.setStyle(
        TableStyle(
            [
                ("BACKGROUND", (0, 0), (-1, 0), colors.HexColor("#e6862e")),
                ("TEXTCOLOR", (0, 0), (-1, 0), colors.whitesmoke),
                ("FONTNAME", (0, 0), (-1, 0), "Helvetica-Bold"),
                ("GRID", (0, 0), (-1, -1), 0.4, colors.HexColor("#bcb9ac")),
                ("LEFTPADDING", (0, 0), (-1, -1), 6),
                ("RIGHTPADDING", (0, 0), (-1, -1), 6),
                ("TOPPADDING", (0, 0), (-1, -1), 4),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 4),
            ]
        )
    )
    story.append(params_table)
    story.append(Spacer(1, 6 * mm))

    story.append(Paragraph("Resumen energético", section_style))
    power = simulation.get("power_total", 0.0)
    summary_table = Table(
        [
            ["Potencia total disipada (W)", f"{float(power):.4f}"],
        ],
        colWidths=[110 * mm, 50 * mm],
    )
    summary_table.setStyle(
        TableStyle(
            [
                ("BACKGROUND", (0, 0), (-1, -1), colors.HexColor("#fffaf0")),
                ("GRID", (0, 0), (-1, -1), 0.4, colors.HexColor("#bcb9ac")),
                ("LEFTPADDING", (0, 0), (-1, -1), 6),
                ("RIGHTPADDING", (0, 0), (-1, -1), 6),
                ("TOPPADDING", (0, 0), (-1, -1), 6),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 6),
                ("FONTNAME", (0, 0), (0, 0), "Helvetica-Bold"),
            ]
        )
    )
    story.append(summary_table)
    story.append(Spacer(1, 4 * mm))
    story.append(
        Paragraph(
            "Documento generado automáticamente por simulador-circuitos-malla.",
            subtitle_style,
        )
    )

    doc.build(story)
    return buffer.getvalue()
