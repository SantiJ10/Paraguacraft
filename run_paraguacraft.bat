@echo off
setlocal
cd /d "%~dp0"

REM Inkscape y otros programas ponen su propio python.exe antes en el PATH.
REM El "Python Launcher" (py) suele apuntar a la instalacion oficial de python.org.
where py >nul 2>&1
if errorlevel 1 (
    echo.
    echo No se encontro el comando "py". Instala Python 3 desde https://www.python.org/downloads/
    echo En el instalador marca: "Add python.exe to PATH" y "Install launcher for all users".
    echo.
    pause
    exit /b 1
)

echo Usando: 
py -3 --version
echo.

py -3 -m pip --version >nul 2>&1
if errorlevel 1 (
    echo pip no esta disponible en ese Python. Reinstala Python y marca pip / tcl-tk.
    pause
    exit /b 1
)

echo Instalando dependencias si hace falta...
py -3 -m pip install -q -r requirements-paragua.txt

echo Iniciando Paraguacraft Launcher...
py -3 paragua.py
set ERR=%ERRORLEVEL%
if not "%ERR%"=="0" pause
exit /b %ERR%
