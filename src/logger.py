"""
Módulo de logging central para Paraguacraft.

Configuración:
  - Escribe en: %APPDATA%/ParaguacraftLauncher/paraguacraft_debug.log
  - Formato: [YYYY-MM-DD HH:MM:SS] [NIVEL   ] nombre.modulo - Mensaje
  - Rotación: 5 MB por archivo, máximo 3 backups (≈ 15 MB total en disco)
  - Nivel FILE: DEBUG (todo)
  - Sin handler de consola por defecto (el usuario no ve la terminal)

Uso en cada módulo:
    import logging
    log = logging.getLogger("paraguacraft.nombre_modulo")
    log.info("...")
    log.warning("...")
    log.error("...", exc_info=True)

El handler de archivo lo configura setup_logger(), llamado UNA sola vez
desde paragua.py (el punto de entrada). Los loggers hijos propagan
automáticamente al logger raíz "paraguacraft".
"""

import logging
import os
from logging.handlers import RotatingFileHandler

_DATA_DIR = os.path.join(
    os.environ.get("APPDATA", os.path.expanduser("~")),
    "ParaguacraftLauncher",
)
LOG_PATH = os.path.join(_DATA_DIR, "paraguacraft_debug.log")

_LOG_FORMAT = "[%(asctime)s] [%(levelname)-8s] %(name)s - %(message)s"
_DATE_FORMAT = "%Y-%m-%d %H:%M:%S"


def setup_logger(name: str = "paraguacraft") -> logging.Logger:
    """
    Configura el logger raíz de Paraguacraft con RotatingFileHandler.
    Idempotente: si ya tiene handlers, retorna el logger existente sin
    añadir duplicados.
    """
    logger = logging.getLogger(name)
    if logger.handlers:
        return logger

    logger.setLevel(logging.DEBUG)
    formatter = logging.Formatter(_LOG_FORMAT, datefmt=_DATE_FORMAT)

    try:
        os.makedirs(_DATA_DIR, exist_ok=True)
        fh = RotatingFileHandler(
            LOG_PATH,
            maxBytes=5 * 1024 * 1024,
            backupCount=3,
            encoding="utf-8",
        )
        fh.setLevel(logging.DEBUG)
        fh.setFormatter(formatter)
        logger.addHandler(fh)
    except OSError as exc:
        logging.basicConfig(
            level=logging.DEBUG,
            format=_LOG_FORMAT,
            datefmt=_DATE_FORMAT,
        )
        logger.warning(
            "No se pudo crear el archivo de log en disco (%s): %s. "
            "Los mensajes irán a stderr.",
            LOG_PATH,
            exc,
        )

    logger.propagate = False
    return logger


log = setup_logger()
