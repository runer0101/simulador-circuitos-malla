import logging

from flask import Blueprint, abort, render_template, request, send_file

from src.services.circuit_renderer import dibujar_circuito
from src.services.mesh_analyzer import MeshAnalyzer, get_default_values
from src.validators.inputs import parse_form_data, validate_parameters

logger = logging.getLogger(__name__)
web_bp = Blueprint("web", __name__)


@web_bp.route("/", methods=["GET", "POST"])
def home():
    default_vals = get_default_values()
    vals = default_vals.copy()
    error = None
    I1 = I2 = I3 = A = B = interpretaciones = None

    if request.method == "POST":
        vals, error = parse_form_data(request.form, default_vals)

        if not error:
            try:
                R1, R2, R3, R4, R5, R6 = vals["R1"], vals["R2"], vals["R3"], vals["R4"], vals["R5"], vals["R6"]
                V1, V2, V3 = vals["V1"], vals["V2"], vals["V3"]

                I1, I2, I3, A, B = MeshAnalyzer.calcular_corrientes(R1, R2, R3, R4, R5, R6, V1, V2, V3)
                interpretaciones = MeshAnalyzer.interpretar_corrientes(I1, I2, I3)
                logger.info(f"Cálculo exitoso: I1={I1:.3f}A, I2={I2:.3f}A, I3={I3:.3f}A")
            except ValueError as exc:
                error = str(exc)
                logger.warning(f"Error de validación: {error}")
            except Exception:
                error = "Error inesperado durante el cálculo. Verifica los valores ingresados."
                logger.exception(
                    "Error inesperado en home",
                    extra={
                        "method": request.method,
                        "path": request.path,
                        "query": request.query_string.decode("utf-8"),
                    },
                )

    template_data = {
        "vals": vals,
        "error": error,
        "I1": I1,
        "I2": I2,
        "I3": I3,
        "A": A,
        "B": B,
        "interpretaciones": interpretaciones,
        "default_vals": default_vals,
    }

    return render_template("index.html", **template_data)


@web_bp.route("/circuito.png")
def circuito_png():
    vals = get_default_values()
    for key in vals.keys():
        val = request.args.get(key)
        if val is not None and val.strip() != "":
            try:
                vals[key] = float(val.replace(",", "."))
            except ValueError:
                abort(400, description=f"{key}: Debe ser un número válido")

    try:
        validate_parameters(vals)
    except ValueError as exc:
        abort(400, description=str(exc))

    try:
        resistance_count = int(request.args.get("resistance_count", "6"))
        voltage_count = int(request.args.get("voltage_count", "3"))
    except ValueError:
        abort(400, description="Conteos de componentes inválidos")

    resistance_count = max(1, min(6, resistance_count))
    voltage_count = max(1, min(3, voltage_count))

    buf = dibujar_circuito(vals, resistance_count=resistance_count, voltage_count=voltage_count)
    return send_file(buf, mimetype="image/png")
