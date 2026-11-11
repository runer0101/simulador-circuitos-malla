FROM python:3.11-slim

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    MPLBACKEND=Agg \
    PORT=8000 \
    LOG_FORMAT=json

WORKDIR /app

COPY requirements.txt ./
RUN python -m pip install --no-cache-dir --upgrade pip \
    && python -m pip install --no-cache-dir -r requirements.txt

COPY app.py wsgi.py gunicorn.conf.py healthcheck.py ./
COPY src ./src
COPY templates ./templates
COPY static ./static

RUN useradd --create-home --uid 10001 appuser \
    && chown -R appuser:appuser /app
USER appuser

EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD ["python", "healthcheck.py"]

CMD ["gunicorn", "--config", "gunicorn.conf.py", "wsgi:app"]
