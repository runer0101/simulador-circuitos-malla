/*
    main.js - Scripts interactivos para la simulación de mallas residenciales
    Autor: runer0101
    Este archivo contiene funciones para mejorar la experiencia del usuario.
    Incluye validación de formularios, animaciones y efectos visuales.
*/

// Variables globales
let isCalculating = false;

// Evento que se ejecuta al cargar la página
document.addEventListener('DOMContentLoaded', function() {
    console.log('Simulación de mallas cargada correctamente.');

    // Inicializar funcionalidades
    initializeTheme();
    initializeFormValidation();
    initializeAnimations();
    initializeTooltips();
    initializeParameterCountControls();
    initializeCircuitPreviewSync();
    initializeExportButtons();
    initializeHistoryControls();
    initializeCompareForm();
    initializeReportControls();
    initializeCounterAnimations();

    const btnEjemplo = document.querySelector('#btn-ejemplo');
    if (btnEjemplo) {
        btnEjemplo.addEventListener('click', function(e) {
            e.preventDefault();
            cargarEjemplo();
        });
    }

    if (document.querySelector('.result-card')) {
        highlightCircuitElements();
    }
});

// Función global para cargar ejemplo (llamada desde HTML)
function cargarEjemplo() {
    fetch('/api/example')
        .then(response => response.json())
        .then(data => {
            for (const key in data) {
                const input = document.querySelector(`input[name="${key}"]`);
                if (input) {
                    input.value = data[key];
                    // Disparar eventos para validación visual
                    input.dispatchEvent(new Event('input'));
                    input.dispatchEvent(new Event('blur'));
                }
            }
        })
        .catch(err => {
            showError('No se pudo cargar el ejemplo.');
        });
}

// Validación en tiempo real del formulario
function initializeFormValidation() {
    const inputs = document.querySelectorAll('input[type="number"]');
    
    inputs.forEach(input => {
        input.addEventListener('input', function() {
            validateInput(this);
        });
        
        input.addEventListener('blur', function() {
            validateInput(this);
        });
    });
    
    // Validación del formulario al enviar
    const form = document.querySelector('form');
    if (form) {
        form.addEventListener('submit', function(e) {
            if (!validateForm()) {
                e.preventDefault();
                showError('Por favor, corrige los errores en el formulario antes de continuar.');
                return false;
            }
            
            showCalculating();
        });
    }
}

function initializeParameterCountControls() {
    const resistanceCountInput = document.querySelector('#resistance-count');
    const voltageCountInput = document.querySelector('#voltage-count');
    const resistanceValue = document.querySelector('#resistance-count-value');
    const voltageValue = document.querySelector('#voltage-count-value');
    const resistanceMinus = document.querySelector('#resistance-minus');
    const resistancePlus = document.querySelector('#resistance-plus');
    const voltageMinus = document.querySelector('#voltage-minus');
    const voltagePlus = document.querySelector('#voltage-plus');

    if (
        !resistanceCountInput ||
        !voltageCountInput ||
        !resistanceValue ||
        !voltageValue ||
        !resistanceMinus ||
        !resistancePlus ||
        !voltageMinus ||
        !voltagePlus
    ) {
        return;
    }

    const inputs = document.querySelectorAll('input[type="number"]');
    inputs.forEach(input => {
        input.dataset.defaultValue = input.value;
    });

    function toggleParameterField(name, active) {
        const input = document.querySelector(`input[name="${name}"]`);
        const label = document.querySelector(`label[for="${name}"]`);

        if (!input || !label) {
            return;
        }

        if (!active) {
            input.dataset.lastValue = input.value;
            if (name.startsWith('V')) {
                input.value = '0';
            } else {
                input.value = '1000';
            }
        } else if (input.dataset.lastValue) {
            input.value = input.dataset.lastValue;
        }

        label.style.display = active ? '' : 'none';
        input.style.display = active ? '' : 'none';
        input.disabled = false;
        input.required = active;
        input.dataset.inactive = active ? 'false' : 'true';

        if (!active) {
            clearFieldError(input);
        }
    }

    let resistanceCount = parseInt(resistanceCountInput.value, 10) || 6;
    let voltageCount = parseInt(voltageCountInput.value, 10) || 3;

    function renderCounts() {
        resistanceValue.textContent = String(resistanceCount);
        voltageValue.textContent = String(voltageCount);
        resistanceCountInput.value = String(resistanceCount);
        voltageCountInput.value = String(voltageCount);
    }

    function applyCounts() {
        renderCounts();

        for (let i = 1; i <= 6; i += 1) {
            toggleParameterField(`R${i}`, i <= resistanceCount);
        }

        for (let i = 1; i <= 3; i += 1) {
            toggleParameterField(`V${i}`, i <= voltageCount);
        }

        document.querySelectorAll('input[type="number"]').forEach(input => {
            if (!input.disabled) {
                validateInput(input);
            }
        });

        const preview = document.querySelector('#circuit-preview');
        if (preview) {
            preview.dispatchEvent(new Event('input-sync'));
        }
    }

    resistanceMinus.addEventListener('click', () => {
        resistanceCount = Math.max(1, resistanceCount - 1);
        applyCounts();
    });
    resistancePlus.addEventListener('click', () => {
        resistanceCount = Math.min(6, resistanceCount + 1);
        applyCounts();
    });
    voltageMinus.addEventListener('click', () => {
        voltageCount = Math.max(1, voltageCount - 1);
        applyCounts();
    });
    voltagePlus.addEventListener('click', () => {
        voltageCount = Math.min(3, voltageCount + 1);
        applyCounts();
    });

    applyCounts();
}

function initializeCircuitPreviewSync() {
    const preview = document.querySelector('#circuit-preview');
    if (!preview) {
        return;
    }

    const endpoint = preview.dataset.endpoint;
    const fields = ['R1', 'R2', 'R3', 'R4', 'R5', 'R6', 'V1', 'V2', 'V3'];
    const inputs = fields
        .map(name => document.querySelector(`input[name="${name}"]`))
        .filter(Boolean);
    const resistanceCountInput = document.querySelector('#resistance-count');
    const voltageCountInput = document.querySelector('#voltage-count');

    let timerId = null;

    function hasValidValues() {
        return inputs.every(input => input.value.trim() !== '' && !Number.isNaN(parseFloat(input.value)));
    }

    function updatePreview() {
        if (!hasValidValues()) {
            return;
        }

        const params = new URLSearchParams();
        inputs.forEach(input => {
            params.set(input.name, input.value.trim().replace(',', '.'));
        });
        if (resistanceCountInput) {
            params.set('resistance_count', resistanceCountInput.value);
        }
        if (voltageCountInput) {
            params.set('voltage_count', voltageCountInput.value);
        }

        preview.src = `${endpoint}?${params.toString()}`;
    }

    inputs.forEach(input => {
        input.addEventListener('input', () => {
            if (timerId) {
                clearTimeout(timerId);
            }
            timerId = setTimeout(updatePreview, 250);
        });
    });

    preview.addEventListener('input-sync', () => {
        if (timerId) {
            clearTimeout(timerId);
        }
        timerId = setTimeout(updatePreview, 50);
    });
}

// Validar un campo individual
function validateInput(input) {
    if (input.disabled || input.dataset.inactive === 'true') {
        clearFieldError(input);
        return true;
    }

    const value = parseFloat(input.value);
    const fieldName = input.getAttribute('name');
    
    // Limpiar errores previos
    clearFieldError(input);
    
    // Validaciones
    if (input.value === '') {
        showFieldError(input, 'Este campo es obligatorio');
        return false;
    }
    
    if (isNaN(value)) {
        showFieldError(input, 'Debe ser un número válido');
        return false;
    }
    
    // Validaciones específicas por tipo de campo
    if (fieldName.startsWith('R')) {
        if (value <= 0) {
            showFieldError(input, 'La resistencia debe ser mayor que cero');
            return false;
        }

        if (value < 0.01 || value > 1000) {
            showFieldError(input, 'La resistencia debe estar entre 0.01Ω y 1000Ω');
            return false;
        }
    }

    if (fieldName.startsWith('V') && (value < 0 || value > 500)) {
        showFieldError(input, 'El voltaje debe estar entre 0V y 500V');
        return false;
    }
    
    // Si llegamos aquí, el campo es válido
    showFieldSuccess(input);
    return true;
}

// Validar todo el formulario
function validateForm() {
    const inputs = document.querySelectorAll('input[type="number"]');
    let isValid = true;
    
    inputs.forEach(input => {
        if (input.disabled || input.dataset.inactive === 'true') {
            return;
        }
        if (!validateInput(input)) {
            isValid = false;
        }
    });
    
    return isValid;
}

// Mostrar error en un campo específico
function showFieldError(input, message) {
    input.classList.add('error');
    input.classList.remove('success');
    
    // Crear o actualizar mensaje de error
    let errorMsg = input.parentNode.querySelector('.field-error');
    if (!errorMsg) {
        errorMsg = document.createElement('div');
        errorMsg.className = 'field-error';
        input.parentNode.appendChild(errorMsg);
    }
    errorMsg.textContent = message;
}

// Mostrar éxito en un campo
function showFieldSuccess(input) {
    input.classList.add('success');
    input.classList.remove('error');
    clearFieldError(input);
}

// Limpiar errores de un campo
function clearFieldError(input) {
    input.classList.remove('error', 'success');
    const errorMsg = input.parentNode.querySelector('.field-error');
    if (errorMsg) {
        errorMsg.remove();
    }
}

// === Fase 6: Toast notifications y helpers UI ===

const TOAST_ICONS = {
    error: '<svg class="toast-icon icon-error" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="8" x2="12" y2="12"></line><line x1="12" y1="16" x2="12.01" y2="16"></line></svg>',
    success: '<svg class="toast-icon icon-success" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>',
    info: '<svg class="toast-icon icon-info" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="16" x2="12" y2="12"></line><line x1="12" y1="8" x2="12.01" y2="8"></line></svg>'
};

function showToast(message, type = 'info', duration = 3500) {
    const container = document.querySelector('#toast-container');
    if (!container) return;
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.setAttribute('role', type === 'error' ? 'alert' : 'status');
    toast.innerHTML = (TOAST_ICONS[type] || TOAST_ICONS.info) + '<span>' + message + '</span>';
    container.appendChild(toast);
    setTimeout(function() {
        toast.classList.add('fade-out');
        setTimeout(function() { toast.remove(); }, 320);
    }, duration);
}

function showError(message) {
    showToast(message, 'error', 4500);
}

function showSuccess(message) {
    showToast(message, 'success', 3000);
}

function showInfo(message) {
    showToast(message, 'info', 3000);
}

function showConfirm(message, options) {
    const opts = options || {};
    const root = document.querySelector('#confirm-root');
    if (!root) {
        if (window.confirm(message)) {
            if (typeof opts.onConfirm === 'function') opts.onConfirm();
            return Promise.resolve(true);
        }
        if (typeof opts.onCancel === 'function') opts.onCancel();
        return Promise.resolve(false);
    }
    return new Promise(function(resolve) {
        const overlay = document.createElement('div');
        overlay.className = 'confirm-overlay';
        overlay.innerHTML = (
            '<div class="confirm-dialog" role="dialog" aria-modal="true">' +
                '<h4>' + (opts.title || 'Confirmar') + '</h4>' +
                '<p>' + message + '</p>' +
                '<div class="confirm-actions">' +
                    '<button type="button" class="confirm-btn cancel" data-action="cancel">' + (opts.cancelLabel || 'Cancelar') + '</button>' +
                    '<button type="button" class="confirm-btn confirm" data-action="confirm">' + (opts.confirmLabel || 'Confirmar') + '</button>' +
                '</div>' +
            '</div>'
        );
        root.appendChild(overlay);
        function close(result) {
            overlay.remove();
            document.removeEventListener('keydown', onKey);
            if (result && typeof opts.onConfirm === 'function') opts.onConfirm();
            if (!result && typeof opts.onCancel === 'function') opts.onCancel();
            resolve(result);
        }
        function onKey(e) {
            if (e.key === 'Escape') close(false);
            if (e.key === 'Enter') close(true);
        }
        overlay.addEventListener('click', function(e) {
            if (e.target === overlay) close(false);
            if (e.target.dataset.action === 'cancel') close(false);
            if (e.target.dataset.action === 'confirm') close(true);
        });
        document.addEventListener('keydown', onKey);
        const confirmBtn = overlay.querySelector('[data-action="confirm"]');
        if (confirmBtn) confirmBtn.focus();
    });
}

// === Fase 6: Theme toggle (dark mode) ===

const THEME_STORAGE_KEY = 'scm-theme';

function applyTheme(theme) {
    const root = document.documentElement;
    if (theme === 'dark') {
        root.setAttribute('data-theme', 'dark');
    } else {
        root.removeAttribute('data-theme');
    }
}

function initializeTheme() {
    let stored = null;
    try {
        stored = localStorage.getItem(THEME_STORAGE_KEY);
    } catch (e) {
        stored = null;
    }
    if (!stored) {
        const prefersDark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
        stored = prefersDark ? 'dark' : 'light';
    }
    applyTheme(stored);
    const toggle = document.querySelector('#theme-toggle');
    if (toggle) {
        toggle.addEventListener('click', function() {
            const current = document.documentElement.getAttribute('data-theme') === 'dark' ? 'dark' : 'light';
            const next = current === 'dark' ? 'light' : 'dark';
            applyTheme(next);
            try { localStorage.setItem(THEME_STORAGE_KEY, next); } catch (e) { /* ignore */ }
            showToast(next === 'dark' ? 'Modo oscuro activado' : 'Modo claro activado', 'info', 1800);
        });
    }
}

// Mostrar estado de cálculo
function showCalculating() {
    const button = document.querySelector('button[type="submit"]');
    if (button) {
        isCalculating = true;
        button.disabled = true;
        button.innerHTML = '<span style="animation: spin 1s linear infinite; display:inline-block;">⟳</span> Calculando...';
        button.style.opacity = '0.8';
        button.style.cursor = 'not-allowed';

        // Agregar indicador visual al formulario
        const form = document.querySelector('.formulario');
        if (form) {
            form.style.opacity = '0.6';
            form.style.pointerEvents = 'none';
        }
    }
}

// Inicializar animaciones
function initializeAnimations() {
    // Animación de aparición de resultados
    const resultCards = document.querySelectorAll('.result-card');
    resultCards.forEach((card, index) => {
        card.style.animationDelay = `${index * 0.2}s`;
        card.classList.add('slide-in');
    });
}

// Inicializar tooltips informativos
function initializeTooltips() {
    // Agregar tooltips a elementos con información técnica
    const resistanceInputs = document.querySelectorAll('input[name^="R"]');
    const voltageInputs = document.querySelectorAll('input[name^="V"]');
    
    resistanceInputs.forEach(input => {
        input.title = 'Resistencia en Ohmios (Ω). Representa la oposición al flujo de corriente.';
    });
    
    voltageInputs.forEach(input => {
        input.title = 'Voltaje en Voltios (V). Representa la diferencia de potencial eléctrico.';
    });
}

// Función para resaltar elementos del circuito según los resultados
function highlightCircuitElements() {
    const resultCards = document.querySelectorAll('.result-card');
    
    resultCards.forEach((card, index) => {
        card.addEventListener('mouseenter', function() {
            // Resaltar elementos correspondientes en el SVG
            const mallaClass = `malla${index + 1}`;
            const svgElements = document.querySelectorAll(`[class*="${mallaClass}"]`);
            
            svgElements.forEach(element => {
                element.style.filter = 'brightness(1.3)';
                element.style.transition = 'filter 0.3s ease';
            });
        });
        
        card.addEventListener('mouseleave', function() {
            // Restaurar elementos del SVG
            const mallaClass = `malla${index + 1}`;
            const svgElements = document.querySelectorAll(`[class*="${mallaClass}"]`);
            
            svgElements.forEach(element => {
                element.style.filter = 'brightness(1)';
            });
        });
    });
}

// === Fase 5: Exportación CSV/PDF ===

function collectFormParams() {
    const keys = ['R1', 'R2', 'R3', 'R4', 'R5', 'R6', 'V1', 'V2', 'V3'];
    const payload = {};
    let valid = true;
    keys.forEach(function(key) {
        const input = document.querySelector(`input[name="${key}"]`);
        if (!input || input.value.trim() === '') {
            valid = false;
            return;
        }
        const value = parseFloat(input.value.replace(',', '.'));
        if (Number.isNaN(value)) {
            valid = false;
            return;
        }
        payload[key] = value;
    });
    return { payload, valid };
}

function downloadFile(url, body) {
    return fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
    }).then(function(response) {
        if (!response.ok) {
            throw new Error('Export failed');
        }
        return response.blob().then(function(blob) {
            const disposition = response.headers.get('Content-Disposition') || '';
            const match = disposition.match(/filename="?([^"]+)"?/);
            const filename = match ? match[1] : 'simulacion';
            const objectUrl = URL.createObjectURL(blob);
            const link = document.createElement('a');
            link.href = objectUrl;
            link.download = filename;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
            URL.revokeObjectURL(objectUrl);
            const ext = (filename.split('.').pop() || 'archivo').toUpperCase();
            showSuccess('Archivo ' + ext + ' descargado: ' + filename);
        });
    });
}

function initializeExportButtons() {
    const csvBtn = document.querySelector('#btn-export-csv');
    const pdfBtn = document.querySelector('#btn-export-pdf');

    function handle(button, url) {
        if (!button) return;
        button.addEventListener('click', function() {
            const simulationId = button.dataset.simulationId;
            let body;
            if (simulationId) {
                body = { id: simulationId };
            } else {
                const collected = collectFormParams();
                if (!collected.valid) {
                    showError('Verifica los valores del formulario antes de exportar.');
                    return;
                }
                body = collected.payload;
            }
            button.disabled = true;
            const originalText = button.innerHTML;
            button.innerHTML = button.innerHTML.replace(/Exportar/, 'Generando…');
            downloadFile(url, body)
                .catch(function() {
                    showError('No se pudo generar el archivo de exportación.');
                })
                .then(function() {
                    button.disabled = false;
                    button.innerHTML = originalText;
                });
        });
    }

    handle(csvBtn, '/api/export/csv');
    handle(pdfBtn, '/api/export/pdf');
}

// === Fase 5: Historial de simulaciones ===

function formatDate(iso) {
    try {
        const d = new Date(iso);
        return d.toLocaleString();
    } catch (e) {
        return iso;
    }
}

function severityClassFor(value) {
    const m = Math.abs(value);
    if (m < 1) return 'severity-low';
    if (m < 10) return 'severity-normal';
    if (m < 50) return 'severity-high';
    return 'severity-critical';
}

function buildSparkline(values) {
    const max = Math.max.apply(null, values.concat([0.0001]));
    const min = Math.min.apply(null, values.concat([0]));
    const range = (max - min) || 1;
    const width = 60;
    const height = 18;
    const barWidth = width / values.length;
    let bars = '';
    for (let i = 0; i < values.length; i += 1) {
        const norm = (values[i] - min) / range;
        const h = Math.max(2, norm * height);
        const y = height - h;
        const x = i * barWidth + 1;
        const w = Math.max(2, barWidth - 2);
        const colorClass = severityClassFor(values[i]).replace('severity-', 'severity-');
        bars += '<rect class="' + colorClass + '" x="' + x + '" y="' + y + '" width="' + w + '" height="' + h + '" rx="1"></rect>';
    }
    return '<svg class="sparkline" viewBox="0 0 ' + width + ' ' + height + '" width="' + width + '" height="' + height + '" aria-hidden="true">' + bars + '</svg>';
}

function renderHistory(items) {
    const container = document.querySelector('#history-list');
    if (!container) return;
    if (!items || items.length === 0) {
        container.innerHTML = '<div class="history-empty">No hay simulaciones registradas todavía. Ejecuta un cálculo para empezar.</div>';
        return;
    }
    const rows = items.map(function(item) {
        const params = item.params || {};
        const currents = item.currents || {};
        const i1Class = severityClassFor(currents.I1 || 0);
        const i2Class = severityClassFor(currents.I2 || 0);
        const i3Class = severityClassFor(currents.I3 || 0);
        const spark = buildSparkline([Math.abs(currents.I1 || 0), Math.abs(currents.I2 || 0), Math.abs(currents.I3 || 0)]);
        return (
            '<article class="history-item" data-id="' + item.id + '">' +
                '<header class="history-item-header">' +
                    '<span class="history-item-id">#' + item.id + '</span>' +
                    '<span class="history-item-time">' + formatDate(item.timestamp) + '</span>' +
                '</header>' +
                '<div class="history-item-currents">' +
                    '<span class="' + i1Class + '">I₁=' + (currents.I1 || 0).toFixed(3) + 'A</span>' +
                    '<span class="' + i2Class + '">I₂=' + (currents.I2 || 0).toFixed(3) + 'A</span>' +
                    '<span class="' + i3Class + '">I₃=' + (currents.I3 || 0).toFixed(3) + 'A</span>' +
                    spark +
                    '<span class="history-item-power">P=' + (item.power_total || 0).toFixed(2) + 'W</span>' +
                '</div>' +
                '<details class="history-item-details">' +
                    '<summary>Ver parámetros</summary>' +
                    '<div class="history-params">' +
                        'R1=' + (params.R1 || 0) + ' · R2=' + (params.R2 || 0) + ' · R3=' + (params.R3 || 0) + '<br>' +
                        'R4=' + (params.R4 || 0) + ' · R5=' + (params.R5 || 0) + ' · R6=' + (params.R6 || 0) + '<br>' +
                        'V1=' + (params.V1 || 0) + ' · V2=' + (params.V2 || 0) + ' · V3=' + (params.V3 || 0) +
                    '</div>' +
                '</details>' +
                '<div class="history-item-actions">' +
                    '<button type="button" class="history-item-reuse" data-action="reuse" data-id="' + item.id + '">↺ Reusar en calculadora</button>' +
                '</div>' +
            '</article>'
        );
    }).join('');
    container.innerHTML = rows;
    // Reuse handlers
    container.querySelectorAll('[data-action="reuse"]').forEach(function(btn) {
        btn.addEventListener('click', function(e) {
            e.stopPropagation();
            const id = btn.dataset.id;
            reuseHistoryItem(id);
        });
    });
    container.querySelectorAll('.history-item').forEach(function(card) {
        card.addEventListener('click', function() {
            reuseHistoryItem(card.dataset.id);
        });
    });
}

function reuseHistoryItem(id) {
    fetch('/api/history/' + encodeURIComponent(id))
        .then(function(r) { return r.json(); })
        .then(function(data) {
            if (!data || !data.success) {
                showError('No se pudo cargar la simulación.');
                return;
            }
            const item = data.item;
            const params = item.params || {};
            Object.keys(params).forEach(function(key) {
                const input = document.querySelector('input[name="' + key + '"]');
                if (input) {
                    input.value = params[key];
                    input.dispatchEvent(new Event('input'));
                    input.dispatchEvent(new Event('blur'));
                }
            });
            const target = document.querySelector('#calculadora');
            if (target) {
                target.scrollIntoView({ behavior: 'smooth', block: 'start' });
            }
            showSuccess('Parámetros cargados desde #' + id);
        })
        .catch(function() {
            showError('No se pudo cargar la simulación.');
        });
}

function refreshHistory() {
    const container = document.querySelector('#history-list');
    if (!container) return;
    container.innerHTML = '<div class="history-empty">Cargando historial…</div>';
    fetch('/api/history')
        .then(function(r) { return r.json(); })
        .then(function(data) {
            if (data && data.success) {
                renderHistory(data.items || []);
            } else {
                container.innerHTML = '<div class="history-empty">No se pudo cargar el historial.</div>';
            }
        })
        .catch(function() {
            container.innerHTML = '<div class="history-empty">No se pudo cargar el historial.</div>';
        });
}

function initializeHistoryControls() {
    const refresh = document.querySelector('#btn-history-refresh');
    const clear = document.querySelector('#btn-history-clear');
    if (refresh) {
        refresh.addEventListener('click', refreshHistory);
    }
    if (clear) {
        clear.addEventListener('click', function() {
            showConfirm('¿Eliminar todas las simulaciones del historial?', {
                title: 'Limpiar historial',
                confirmLabel: 'Sí, limpiar',
                cancelLabel: 'Cancelar',
            }).then(function(ok) {
                if (!ok) return;
                fetch('/api/history', { method: 'DELETE' })
                    .then(function(r) { return r.json(); })
                    .then(function(data) {
                        const removed = (data && data.removed) || 0;
                        showSuccess(removed > 0 ? 'Se eliminaron ' + removed + ' simulaciones' : 'Historial ya estaba vacío');
                        refreshHistory();
                        refreshReport();
                    })
                    .catch(function() { showError('No se pudo limpiar el historial.'); });
            });
        });
    }
    if (document.querySelector('#history-list')) {
        refreshHistory();
    }
}

// === Fase 5: Comparación de escenarios ===

function renderCompareResult(data) {
    const container = document.querySelector('#compare-result');
    if (!container) return;
    if (!data || !data.success) {
        container.innerHTML = '<div class="compare-error">No se pudo realizar la comparación.</div>';
        return;
    }
    const base = data.baseline;
    const opt = data.optimized;
    const comp = data.comparison || {};
    const labelMap = {
        mejora: { cls: 'improve', text: 'Mejora: menor consumo en el escenario optimizado', icon: '↓' },
        empeoramiento: { cls: 'worse', text: 'Empeoramiento: mayor consumo en el escenario optimizado', icon: '↑' },
        'sin cambio': { cls: 'neutral', text: 'Sin cambio en el consumo total', icon: '→' },
    };
    const verdict = comp.power_reduction_label ? labelMap[comp.power_reduction_label] : null;
    const verdictHtml = verdict
        ? '<div class="compare-verdict ' + verdict.cls + '"><span style="font-size:1.3rem;font-weight:700;">' + verdict.icon + '</span><span>' + verdict.text + '</span></div>'
        : '';

    function deltaCell(value, percent, isCurrentOrPower) {
        const abs = Math.abs(value);
        const sign = value > 0 ? '+' : '';
        const valueText = sign + value.toFixed(4);
        let pctText = '';
        if (percent === null || percent === undefined) {
            pctText = 'N/A';
        } else {
            const pctSign = percent > 0 ? '+' : '';
            pctText = pctSign + percent.toFixed(1) + '%';
        }
        let arrowCls = 'flat';
        let arrow = '→';
        if (isCurrentOrPower) {
            if (value < 0) { arrowCls = 'down'; arrow = '↓'; }
            else if (value > 0) { arrowCls = 'up'; arrow = '↑'; }
        }
        const cellCls = value < 0 ? 'good' : value > 0 ? 'bad' : 'neutral';
        const barPct = Math.min(100, abs * 10);
        const barSpan = '<span class="compare-delta-bar"><span class="' + cellCls + '" style="width:' + barPct.toFixed(1) + '%;left:' + (value < 0 ? (50 - barPct / 2) : '50%') + '"></span></span>';
        return (
            '<td class="delta-' + cellCls + '">' +
                '<span class="compare-arrow ' + arrowCls + '">' + arrow + '</span>' +
                valueText + ' (' + pctText + ')' +
                barSpan +
            '</td>'
        );
    }

    container.innerHTML = (
        verdictHtml +
        '<table class="compare-table">' +
            '<thead>' +
                '<tr><th>Variable</th><th>Base</th><th>Optimizado</th><th>Diferencia</th></tr>' +
            '</thead>' +
            '<tbody>' +
                '<tr><td>I₁ (A)</td><td>' + base.currents.I1.toFixed(4) + '</td><td>' + opt.currents.I1.toFixed(4) + '</td>' + deltaCell(comp.deltas.I1.absolute, comp.deltas.I1.percent, true) + '</tr>' +
                '<tr><td>I₂ (A)</td><td>' + base.currents.I2.toFixed(4) + '</td><td>' + opt.currents.I2.toFixed(4) + '</td>' + deltaCell(comp.deltas.I2.absolute, comp.deltas.I2.percent, true) + '</tr>' +
                '<tr><td>I₃ (A)</td><td>' + base.currents.I3.toFixed(4) + '</td><td>' + opt.currents.I3.toFixed(4) + '</td>' + deltaCell(comp.deltas.I3.absolute, comp.deltas.I3.percent, true) + '</tr>' +
                '<tr><td>Potencia (W)</td><td>' + base.power_total.toFixed(3) + '</td><td>' + opt.power_total.toFixed(3) + '</td>' + deltaCell(comp.deltas.power_total.absolute, comp.deltas.power_total.percent, true) + '</tr>' +
                '<tr><td>Corriente máx. (A)</td><td>' + base.max_current.toFixed(4) + '</td><td>' + opt.max_current.toFixed(4) + '</td>' + deltaCell(comp.deltas.max_current.absolute, comp.deltas.max_current.percent, true) + '</tr>' +
                '<tr><td>Severidad máx.</td><td>' + comp.baseline_max_severity + '</td><td>' + comp.optimized_max_severity + '</td><td>—</td></tr>' +
            '</tbody>' +
        '</table>'
    );
}

function initializeCompareForm() {
    const form = document.querySelector('#compare-form');
    if (!form) return;
    form.addEventListener('submit', function(event) {
        event.preventDefault();
        const fields = ['R1', 'R2', 'R3', 'R4', 'R5', 'R6', 'V1', 'V2', 'V3'];
        const baseline = {};
        const optimized = {};
        let valid = true;
        fields.forEach(function(key) {
            const baseInput = document.querySelector(`#cmp_${key}_b`);
            const optInput = document.querySelector(`#cmp_${key}_o`);
            if (!baseInput || !optInput) {
                valid = false;
                return;
            }
            const baseVal = parseFloat(baseInput.value.replace(',', '.'));
            const optVal = parseFloat(optInput.value.replace(',', '.'));
            if (Number.isNaN(baseVal) || Number.isNaN(optVal)) {
                valid = false;
                return;
            }
            baseline[key] = baseVal;
            optimized[key] = optVal;
        });
        if (!valid) {
            showError('Verifica los valores de los escenarios antes de comparar.');
            return;
        }
        const container = document.querySelector('#compare-result');
        if (container) {
            container.innerHTML = '<div class="history-empty">Comparando…</div>';
        }
        fetch('/api/compare', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ baseline: baseline, optimized: optimized }),
        })
            .then(function(r) { return r.json(); })
            .then(function(data) { renderCompareResult(data); })
            .catch(function() {
                if (container) {
                    container.innerHTML = '<div class="compare-error">No se pudo realizar la comparación.</div>';
                }
            });
    });
}

// === Fase 5: Reporte técnico agregado ===

function renderReport(data) {
    const container = document.querySelector('#report-summary');
    if (!container) return;
    const report = (data && data.report) || {};
    const total = report.total_simulations || 0;
    if (total === 0) {
        container.innerHTML = '<div class="history-empty">Sin datos: ejecuta simulaciones para generar el reporte.</div>';
        return;
    }
    const sev = report.by_severity || {};
    const sevTotal = (sev.baja || 0) + (sev.normal || 0) + (sev.alta || 0) + (sev.critica || 0);
    const sevBars = ['baja', 'normal', 'alta', 'critica'].map(function(key) {
        const count = sev[key] || 0;
        const pct = sevTotal > 0 ? (count / sevTotal) * 100 : 0;
        return (
            '<div class="report-bar-row">' +
                '<span class="report-bar-label">' + key + '</span>' +
                '<div class="report-bar"><div class="report-bar-fill severity-' + key + '" style="width:' + pct.toFixed(1) + '%"></div></div>' +
                '<span class="report-bar-value">' + count + '</span>' +
            '</div>'
        );
    }).join('');
    container.innerHTML = (
        '<div class="report-grid">' +
            '<div class="report-cell"><svg class="report-cell-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"></polyline></svg><span class="report-label">Simulaciones</span><span class="report-value">' + total + '</span></div>' +
            '<div class="report-cell"><svg class="report-cell-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="3" width="18" height="18" rx="2"></rect><line x1="9" y1="9" x2="15" y2="9"></line><line x1="9" y1="15" x2="15" y2="15"></line></svg><span class="report-label">Capacidad</span><span class="report-value">' + (data.history_capacity || '—') + '</span></div>' +
            '<div class="report-cell"><svg class="report-cell-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"></path></svg><span class="report-label">Corriente máx. observada</span><span class="report-value">' + (report.max_current_observed || 0).toFixed(3) + ' A</span></div>' +
            '<div class="report-cell"><svg class="report-cell-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><line x1="12" y1="1" x2="12" y2="23"></line><path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"></path></svg><span class="report-label">Potencia promedio</span><span class="report-value">' + (report.avg_power_watts || 0).toFixed(3) + ' W</span></div>' +
            '<div class="report-cell"><svg class="report-cell-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><polyline points="6 9 12 15 18 9"></polyline></svg><span class="report-label">Potencia mín.</span><span class="report-value">' + (report.min_power_watts || 0).toFixed(3) + ' W</span></div>' +
            '<div class="report-cell"><svg class="report-cell-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><polyline points="18 15 12 9 6 15"></polyline></svg><span class="report-label">Potencia máx.</span><span class="report-value">' + (report.max_power_watts || 0).toFixed(3) + ' W</span></div>' +
        '</div>' +
        '<div class="report-canvas-wrap">' +
            '<div class="report-canvas-title">Potencia en el tiempo (W)</div>' +
            '<canvas id="power-chart" aria-label="Gráfico de potencia por simulación"></canvas>' +
        '</div>' +
        '<div class="report-bars">' + sevBars + '</div>' +
        '<div class="report-window">' +
            '<span>Primera: ' + (report.first_simulation_at || '—') + '</span><br>' +
            '<span>Última: ' + (report.last_simulation_at || '—') + '</span>' +
        '</div>'
    );
    // Render the chart after DOM is in place
    const canvas = container.querySelector('#power-chart');
    if (canvas) {
        fetchHistoryForChart().then(function(items) {
            drawPowerChart(canvas, items);
        });
    }
}

function fetchHistoryForChart() {
    return fetch('/api/history')
        .then(function(r) { return r.json(); })
        .then(function(data) {
            if (data && data.success) {
                // items are in reverse chronological order (newest first)
                return (data.items || []).slice().reverse();
            }
            return [];
        })
        .catch(function() { return []; });
}

function getCssVar(name, fallback) {
    const root = document.documentElement;
    const value = getComputedStyle(root).getPropertyValue(name).trim();
    return value || fallback;
}

function drawPowerChart(canvas, items) {
    if (!canvas || !canvas.getContext) return;
    const ctx = canvas.getContext('2d');
    const dpr = window.devicePixelRatio || 1;
    const cssWidth = canvas.clientWidth || 400;
    const cssHeight = canvas.clientHeight || 110;
    canvas.width = cssWidth * dpr;
    canvas.height = cssHeight * dpr;
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, cssWidth, cssHeight);

    const padding = { top: 6, right: 6, bottom: 18, left: 30 };
    const w = cssWidth - padding.left - padding.right;
    const h = cssHeight - padding.top - padding.bottom;

    if (!items || items.length === 0) {
        ctx.fillStyle = getCssVar('--muted', '#999');
        ctx.font = '12px "Space Grotesk", sans-serif';
        ctx.textAlign = 'center';
        ctx.fillText('Sin datos', cssWidth / 2, cssHeight / 2);
        return;
    }

    const values = items.map(function(it) { return it.power_total || 0; });
    const max = Math.max.apply(null, values.concat([0.001]));
    const min = Math.min.apply(null, values.concat([0]));
    const range = (max - min) || 1;

    // grid
    ctx.strokeStyle = getCssVar('--line', '#ddd');
    ctx.lineWidth = 1;
    for (let i = 0; i < 4; i += 1) {
        const y = padding.top + (h * i) / 3;
        ctx.beginPath();
        ctx.moveTo(padding.left, y);
        ctx.lineTo(padding.left + w, y);
        ctx.stroke();
    }

    // axis labels
    ctx.fillStyle = getCssVar('--muted', '#888');
    ctx.font = '10px "JetBrains Mono", monospace';
    ctx.textAlign = 'right';
    ctx.fillText(max.toFixed(1), padding.left - 4, padding.top + 8);
    ctx.fillText(min.toFixed(1), padding.left - 4, padding.top + h);

    // line
    const brand = getCssVar('--brand', '#0f766e');
    const brandSoft = getCssVar('--brand-soft', '#1f9d8f');
    ctx.strokeStyle = brand;
    ctx.lineWidth = 2;
    ctx.beginPath();
    values.forEach(function(v, i) {
        const x = padding.left + (items.length === 1 ? w / 2 : (i / (items.length - 1)) * w);
        const y = padding.top + h - ((v - min) / range) * h;
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
    });
    ctx.stroke();

    // fill under
    ctx.lineTo(padding.left + w, padding.top + h);
    ctx.lineTo(padding.left, padding.top + h);
    ctx.closePath();
    const grad = ctx.createLinearGradient(0, padding.top, 0, padding.top + h);
    grad.addColorStop(0, brandSoft);
    grad.addColorStop(1, 'rgba(31, 157, 143, 0.05)');
    ctx.fillStyle = grad;
    ctx.globalAlpha = 0.4;
    ctx.fill();
    ctx.globalAlpha = 1;

    // points
    ctx.fillStyle = brand;
    values.forEach(function(v, i) {
        const x = padding.left + (items.length === 1 ? w / 2 : (i / (items.length - 1)) * w);
        const y = padding.top + h - ((v - min) / range) * h;
        ctx.beginPath();
        ctx.arc(x, y, 2.5, 0, Math.PI * 2);
        ctx.fill();
    });

    // x labels (first and last)
    ctx.fillStyle = getCssVar('--muted', '#888');
    ctx.textAlign = 'left';
    ctx.fillText('#' + (items[0].id || '').slice(0, 6), padding.left, padding.top + h + 12);
    if (items.length > 1) {
        ctx.textAlign = 'right';
        ctx.fillText('#' + (items[items.length - 1].id || '').slice(0, 6), padding.left + w, padding.top + h + 12);
    }
}

function refreshReport() {
    const container = document.querySelector('#report-summary');
    if (!container) return;
    container.innerHTML = '<div class="history-empty">Cargando reporte…</div>';
    fetch('/api/report')
        .then(function(r) { return r.json(); })
        .then(function(data) { renderReport(data); })
        .catch(function() {
            container.innerHTML = '<div class="history-empty">No se pudo cargar el reporte.</div>';
        });
}

function initializeReportControls() {
    const refresh = document.querySelector('#btn-report-refresh');
    if (refresh) {
        refresh.addEventListener('click', refreshReport);
    }
    if (document.querySelector('#report-summary')) {
        refreshReport();
    }
}

// === Fase 6: Counter animation ===

function animateNumber(element, target, duration, prefix, suffix) {
    if (!element) return;
    const dur = duration || 600;
    const start = performance.now();
    const decimals = (String(target).split('.')[1] || '').length;
    function frame(now) {
        const progress = Math.min(1, (now - start) / dur);
        const eased = 1 - Math.pow(1 - progress, 3);
        const current = target * eased;
        if (progress < 1) {
            element.textContent = prefix + current.toFixed(decimals) + suffix;
            requestAnimationFrame(frame);
        } else {
            element.textContent = prefix + Number(target).toFixed(decimals) + suffix;
        }
    }
    requestAnimationFrame(frame);
}

function initializeCounterAnimations() {
    const reduceMotion = window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    if (reduceMotion) return;

    document.querySelectorAll('.result-card .current-value').forEach(function(el) {
        // Keep the server-rendered "I₁ = 2.143 A" intact: only the number is animated, and
        // the text is left untouched until the first frame so a paused tab never loses the value.
        const parts = (el.textContent || '').match(/^(.*?)(-?\d+(?:\.\d+)?)(.*)$/);
        if (!parts) return;
        const value = parseFloat(parts[2]);
        if (Number.isNaN(value)) return;
        const prefix = parts[1];
        const suffix = parts[3];
        setTimeout(function() { animateNumber(el, value, 700, prefix, suffix); }, 80);
    });
}

