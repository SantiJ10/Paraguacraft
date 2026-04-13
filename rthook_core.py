import sys
import os
import importlib.util

if hasattr(sys, '_MEIPASS'):
    if sys._MEIPASS not in sys.path:
        sys.path.insert(0, sys._MEIPASS)
    _core_path = os.path.join(sys._MEIPASS, 'core.py')
    if os.path.exists(_core_path) and 'core' not in sys.modules:
        try:
            _spec = importlib.util.spec_from_file_location('core', _core_path)
            _mod  = importlib.util.module_from_spec(_spec)
            sys.modules['core'] = _mod
            _spec.loader.exec_module(_mod)
        except Exception:
            pass
