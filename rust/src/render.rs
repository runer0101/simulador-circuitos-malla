//! Renderizado del esquema del circuito (PNG) y del reporte (PDF).
//!
//! El dibujo no pretende ser idéntico píxel a píxel al de Matplotlib: reproduce el mismo esquema
//! —tres mallas, seis resistencias, tres fuentes— con las mismas etiquetas y valores.

use ::image::{ExtendedColorType, ImageEncoder};
use plotters::prelude::*;
use printpdf::*;

use crate::api::Entry;
use crate::domain::MeshParams;

const ANCHO: u32 = 900;
const ALTO: u32 = 500;

fn rgb(hex: (u8, u8, u8)) -> RGBColor {
    RGBColor(hex.0, hex.1, hex.2)
}

/// Dibuja el esquema adaptativo del circuito y devuelve los bytes del PNG.
pub fn build_png(
    params: &MeshParams,
    resistencias: usize,
    voltajes: usize,
) -> Result<Vec<u8>, String> {
    let mut buffer = vec![0u8; (ANCHO * ALTO * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (ANCHO, ALTO)).into_drawing_area();
        root.fill(&RGBColor(255, 255, 255))
            .map_err(|e| e.to_string())?;

        let tinta = rgb((26, 43, 52));
        let cable = rgb((15, 118, 110));
        let resist = rgb((242, 193, 78));
        let fuente = rgb((226, 74, 74));

        let margen = 90i32;
        let arriba = 150i32;
        let abajo = 380i32;
        let paso = (ANCHO as i32 - 2 * margen) / 3;

        root.draw(&Text::new(
            format!("Circuito de mallas adaptativo ({resistencias}R, {voltajes}V)"),
            (ANCHO as i32 / 2 - 170, 40),
            ("sans-serif", 24).into_font().color(&tinta),
        ))
        .map_err(|e| e.to_string())?;

        for y in [arriba, abajo] {
            root.draw(&PathElement::new(
                vec![(margen, y), (ANCHO as i32 - margen, y)],
                ShapeStyle::from(&cable).stroke_width(3),
            ))
            .map_err(|e| e.to_string())?;
        }

        let valores_r = [
            params.r1, params.r2, params.r3, params.r4, params.r5, params.r6,
        ];
        let valores_v = [params.v1, params.v2, params.v3];
        let y_medio = (arriba + abajo) / 2;

        for rama in 0..3usize {
            let x = margen + paso * rama as i32 + paso / 2;

            root.draw(&PathElement::new(
                vec![(x, arriba), (x, abajo)],
                ShapeStyle::from(&cable).stroke_width(3),
            ))
            .map_err(|e| e.to_string())?;

            // Resistencia del tramo superior.
            root.draw(&Rectangle::new(
                [(x - 26, arriba - 12), (x + 26, arriba + 12)],
                ShapeStyle::from(&resist).filled(),
            ))
            .map_err(|e| e.to_string())?;
            root.draw(&Text::new(
                format!("R{}={:.1}\u{3a9}", rama * 2 + 1, valores_r[rama * 2]),
                (x - 34, arriba - 32),
                ("sans-serif", 14).into_font().color(&tinta),
            ))
            .map_err(|e| e.to_string())?;

            // Resistencia vertical.
            root.draw(&Rectangle::new(
                [(x - 12, y_medio - 34), (x + 12, y_medio + 34)],
                ShapeStyle::from(&resist).filled(),
            ))
            .map_err(|e| e.to_string())?;
            root.draw(&Text::new(
                format!("R{}={:.1}\u{3a9}", rama * 2 + 2, valores_r[rama * 2 + 1]),
                (x + 18, y_medio - 7),
                ("sans-serif", 14).into_font().color(&tinta),
            ))
            .map_err(|e| e.to_string())?;

            // Fuente de tensión.
            if rama < voltajes {
                root.draw(&Circle::new(
                    (x - 34, abajo - 34),
                    12,
                    ShapeStyle::from(&fuente).stroke_width(2),
                ))
                .map_err(|e| e.to_string())?;
                root.draw(&Text::new(
                    format!("V{}={:.1}V", rama + 1, valores_v[rama]),
                    (x - 76, abajo - 16),
                    ("sans-serif", 13).into_font().color(&tinta),
                ))
                .map_err(|e| e.to_string())?;
            }

            root.draw(&Text::new(
                format!("Malla {}", rama + 1),
                (x - 28, abajo + 28),
                ("sans-serif", 13).into_font().color(&tinta),
            ))
            .map_err(|e| e.to_string())?;
        }

        root.present().map_err(|e| e.to_string())?;
    }

    let mut png = Vec::new();
    ::image::codecs::png::PngEncoder::new(&mut png)
        .write_image(&buffer, ANCHO, ALTO, ExtendedColorType::Rgb8)
        .map_err(|e| e.to_string())?;
    Ok(png)
}

/// Genera el reporte PDF de una simulación.
pub fn build_pdf(entry: &Entry) -> Result<Vec<u8>, String> {
    let (doc, page, layer) = PdfDocument::new(
        "Reporte de simulacion de mallas",
        Mm(210.0),
        Mm(297.0),
        "Capa 1",
    );
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| format!("fuente: {e:?}"))?;
    let bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| format!("fuente: {e:?}"))?;
    let capa = doc.get_page(page).get_layer(layer);

    capa.use_text(
        "Reporte de simulacion de mallas",
        20.0,
        Mm(18.0),
        Mm(272.0),
        &bold,
    );
    capa.use_text(
        format!("ID: {}", entry.id),
        10.0,
        Mm(18.0),
        Mm(264.0),
        &font,
    );
    capa.use_text(
        format!("Fecha: {}", entry.timestamp),
        10.0,
        Mm(18.0),
        Mm(259.0),
        &font,
    );

    capa.use_text("Resultados de corrientes", 14.0, Mm(18.0), Mm(246.0), &bold);
    let mut y = 238.0;
    capa.use_text(
        "Malla   Corriente (A)   Interpretacion",
        10.0,
        Mm(18.0),
        Mm(y),
        &bold,
    );
    y -= 6.0;
    for (indice, clave) in ["I1", "I2", "I3"].iter().enumerate() {
        capa.use_text(
            format!(
                "{clave}      {:.6}       {}",
                entry.currents[indice], entry.interpretations[indice]
            ),
            10.0,
            Mm(18.0),
            Mm(y),
            &font,
        );
        y -= 6.0;
    }
    capa.use_text(
        format!("Potencia total: {:.4} W", entry.power_total),
        11.0,
        Mm(18.0),
        Mm(y - 4.0),
        &bold,
    );

    let mut yp = y - 16.0;
    capa.use_text("Parametros del circuito", 14.0, Mm(18.0), Mm(yp), &bold);
    yp -= 8.0;
    for (clave, valor) in [
        ("R1", entry.params.r1),
        ("R2", entry.params.r2),
        ("R3", entry.params.r3),
        ("R4", entry.params.r4),
        ("R5", entry.params.r5),
        ("R6", entry.params.r6),
        ("V1", entry.params.v1),
        ("V2", entry.params.v2),
        ("V3", entry.params.v3),
    ] {
        capa.use_text(
            format!("{clave}: {valor:.4}"),
            10.0,
            Mm(18.0),
            Mm(yp),
            &font,
        );
        yp -= 6.0;
    }

    doc.save_to_bytes().map_err(|e| format!("pdf: {e:?}"))
}
