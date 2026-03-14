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
    initializeFormValidation();
    initializeAnimations();
    initializeTooltips();
    initializeParameterCountControls();
    initializeCircuitPreviewSync();

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

// Mostrar mensaje de error general
function showError(message) {
    // Crear o mostrar elemento de error
    let errorDiv = document.querySelector('.general-error');
    if (!errorDiv) {
        errorDiv = document.createElement('div');
        errorDiv.className = 'general-error error-msg';
        const form = document.querySelector('form');
        form.appendChild(errorDiv);
    }
    
    errorDiv.textContent = message;
    errorDiv.style.display = 'block';
    
    // Auto-ocultar después de 5 segundos
    setTimeout(() => {
        if (errorDiv) {
            errorDiv.style.display = 'none';
        }
    }, 5000);
}

// Mostrar estado de cálculo
function showCalculating() {
    const button = document.querySelector('button[type="submit"]');
    if (button) {
        isCalculating = true;
        button.disabled = true;
        button.innerHTML = '<span style="animation: spin 1s linear infinite;">⟳</span> Calculando...';
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

