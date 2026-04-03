import sys
import os
import urllib.parse
import webbrowser
import platform
import subprocess
import json
import threading
import shutil
import zipfile
import io
import requests
import psutil
from datetime import datetime
from mcstatus import JavaServer
import threading
import windnd
import hashlib
from PIL import Image, ImageTk
from http.server import SimpleHTTPRequestHandler, HTTPServer
from src.modelo import GestorMods, TiendaAPI, CreadorServidor
from src.controlador import GameBooster

if sys.platform == "win32":
    import win32gui
    import win32con

if sys.platform == "win32":
    if sys.stdout is None: sys.stdout = open(os.devnull, "w")
    if sys.stderr is None: sys.stderr = open(os.devnull, "w")

import customtkinter as ctk
import tkinter as tk
from tkinter import filedialog, messagebox
import minecraft_launcher_lib
from core import lanzar_minecraft

ctk.set_appearance_mode("dark")
ctk.set_default_color_theme("blue")

# LIMPIADOR DE ACTUALIZACIONES
# LIMPIADOR DE ACTUALIZACIONES SILENCIOSO
if getattr(sys, 'frozen', False):
    import os, sys
    exe_actual = sys.executable
    viejo_exe = exe_actual + ".old"
    if os.path.exists(viejo_exe):
        try:
            os.remove(viejo_exe)
        except Exception as e:
            print(f"No se pudo limpiar el exe viejo: {e}")

APPDATA_DIR = os.path.join(os.getenv('APPDATA'), "ParaguacraftLauncher")
os.makedirs(APPDATA_DIR, exist_ok=True)
CONFIG_FILE = os.path.join(APPDATA_DIR, "paraguacraft_config.json")
SESSION_FILE = os.path.join(APPDATA_DIR, "paraguacraft_session.json")
LOG_DIR = os.path.join(minecraft_launcher_lib.utils.get_minecraft_directory(), "logs")

# URL Configuracion
CLIENT_ID = "72fb7c48-c2f5-4d13-b0e7-9835b3b906c0" 
REDIRECT_URI = "https://login.live.com/oauth20_desktop.srf"
DISCORD_APP_ID = "1487516329631154206" 
SERVER_IP = "process-import.gl.at.ply.gg:2055" 
MODS_ZIP_URL = "" 

LAUNCHER_VERSION = "1.0.7"
UPDATE_URL = "https://raw.githubusercontent.com/SantiJ10/Paraguacraft/refs/heads/main/version.txt"

class ParaguaCraftLauncher(ctk.CTk):
    def __init__(self):
        super().__init__()
        self.title(f"ParaguaCraft Launcher v{LAUNCHER_VERSION}")
        self.centrar_ventana(self, 600, 780)
        self.resizable(True, True)

        icono_base = "iconomc.ico"
        if getattr(sys, 'frozen', False):
            icono_path = os.path.join(sys._MEIPASS, icono_base)
        else:
            icono_path = icono_base
            
        if os.path.exists(icono_path):
            try: self.iconbitmap(icono_path)
            except Exception as e: print(f"No se pudo cargar el icono: {e}")
        self.limpiador_deep_var = ctk.BooleanVar(value=False)
        self.backup_var = ctk.BooleanVar(value=True) 
        
        self.veces_iniciadas = ctk.IntVar(value=0)
        self.horas_jugadas_total = ctk.DoubleVar(value=0.0)

        self.gc_var = ctk.StringVar(value="G1GC (Equilibrado / Recomendado)")
        self.opt_var = ctk.BooleanVar(value=False)
        self.tipo_cliente_var = ctk.StringVar(value="Fabric")
        self.papa_var = ctk.BooleanVar(value=False)
        self.mesa_var = ctk.BooleanVar(value=False)
        self.consola_var = ctk.BooleanVar(value=False)
        self.lan_distancia_var = ctk.BooleanVar(value=False)
        
        self.ms_data = None
        self.rpc = None
        self.usuarios_guardados = []
        self.hilo_juego_activo = False

        self.frame_main = ctk.CTkFrame(self)
        self.frame_main.pack(pady=15, padx=20, fill="both", expand=True)

        self.label_titulo = ctk.CTkLabel(self.frame_main, text="ParaguaCraft Launcher", font=ctk.CTkFont(size=28, weight="bold"))
        self.label_titulo.pack(pady=(20, 5))
        # Contenedor del Avatar
        self.frame_avatar = ctk.CTkFrame(self.frame_main, fg_color="transparent")
        self.frame_avatar.pack(pady=5)
        
        self.lbl_avatar = ctk.CTkLabel(self.frame_avatar, text="", width=64, height=64)
        self.lbl_avatar.pack(side="left", padx=10)

        self.lbl_server = ctk.CTkLabel(self.frame_main, text=f"🟡 Comprobando estado de {SERVER_IP}...", font=ctk.CTkFont(size=12, slant="italic"))
        self.lbl_server.pack(pady=(0, 10))

        # --- CONTENEDOR DE USUARIO Y MULTICUENTA ---
        self.frame_user = ctk.CTkFrame(self.frame_main, fg_color="transparent")
        self.frame_user.pack(pady=5, fill="x", padx=50)
        
        self.tema_var = ctk.StringVar(value="dark") # Variable para guardar el tema visual

        self.frame_login_input = ctk.CTkFrame(self.frame_user, fg_color="transparent")
        self.frame_login_input.pack(pady=5)
        
        self.combo_usuario = ctk.CTkComboBox(
            self.frame_login_input, values=["JugadorOffline"], 
            width=300, height=40, font=ctk.CTkFont(size=16), justify="center"
        )
        self.combo_usuario.pack(side="left", padx=5)

        # Botón del Gestor Multicuenta (Eliminar historial)
        self.btn_borrar_user = ctk.CTkButton(
            self.frame_login_input, text="❌", width=40, height=40, 
            command=self.borrar_usuario_seleccionado, fg_color="#c0392b", hover_color="#922b21"
        )
        self.btn_borrar_user.pack(side="left")

        self.btn_microsoft = ctk.CTkButton(
            self.frame_user, text="Iniciar Sesión con Microsoft", 
            command=self.login_microsoft, fg_color="#107C10", hover_color="#0b5e0b",
            font=ctk.CTkFont(weight="bold")
        )
        self.btn_microsoft.pack(pady=5)

        self.label_version = ctk.CTkLabel(self.frame_main, text="Seleccionar Versión:", font=ctk.CTkFont(weight="bold"))
        self.label_version.pack(pady=(10, 5))

        # FIX VISUAL
        self.frame_lista = ctk.CTkFrame(self.frame_main, border_width=2, border_color="#3a3a3a", width=400, height=160)
        self.frame_lista.pack(pady=5) # Le sacamos el fill="x" y el expand
        self.frame_lista.pack_propagate(False)
        
        self.scroll_versiones = ctk.CTkScrollbar(self.frame_lista)
        self.scroll_versiones.pack(side="right", fill="y", padx=(0, 2), pady=2)
        
        self.lista_versiones = tk.Listbox(
            self.frame_lista, yscrollcommand=self.scroll_versiones.set, 
            bg="#242424", fg="#e0e0e0", font=("Consolas", 11), 
            selectbackground="#1f538d", borderwidth=0, highlightthickness=0
        )
        self.lista_versiones.pack(side="left", fill="both", expand=True, padx=5, pady=5)
        self.scroll_versiones.configure(command=self.lista_versiones.yview)

        self.frame_opciones = ctk.CTkFrame(self.frame_main, fg_color="transparent")
        self.frame_opciones.pack(pady=10, fill="x", padx=50)

        ram_total = max(2, int(psutil.virtual_memory().total / (1024**3)))
        ram_disponible = max(1, ram_total - 1)
        opciones_ram = [f"{i}GB" for i in range(1, ram_disponible + 1)]

        self.label_ram = ctk.CTkLabel(self.frame_opciones, text="RAM:")
        self.label_ram.pack(side="left", padx=(0, 10))
        self.combo_ram = ctk.CTkComboBox(self.frame_opciones, values=opciones_ram, width=90)
        self.combo_ram.pack(side="left")

        self.btn_config = ctk.CTkButton(self.frame_opciones, text="Ajustes Extra", command=self.abrir_config, fg_color="#4a4a4a", width=120)
        self.btn_config.pack(side="right")

        self.btn_carpeta = ctk.CTkButton(self.frame_opciones, text="📂", command=self.abrir_carpeta_minecraft, fg_color="#2b7b4b", width=40)
        self.btn_carpeta.pack(side="right", padx=10)

        # --- BOTÓN NUEVO DEL GESTOR DE MODS ---
        self.btn_gestor_mods = ctk.CTkButton(self.frame_opciones, text="🧩 Gestor de Mods", command=self.abrir_gestor_mods, fg_color="#8e44ad", hover_color="#5e3370", width=140)
        self.btn_gestor_mods.pack(side="right", padx=(0, 10))


        self.lbl_estado = ctk.CTkLabel(self.frame_main, text="Listo", text_color="gray")
        self.lbl_estado.pack(pady=5)

        self.barra_progreso = ctk.CTkProgressBar(self.frame_main, width=350)
        self.barra_progreso.pack(pady=5)
        self.barra_progreso.set(0)

        self.boton_jugar = ctk.CTkButton(self.frame_main, text="INICIAR PARAGUACRAFT", command=self.iniciar_juego_thread, font=ctk.CTkFont(size=20, weight="bold"), height=50, width=300)
        self.boton_jugar.pack(pady=(10, 10))

        self.cargar_configuracion()
        self.cargar_sesion_microsoft()
        
        self.auto_configurar_hardware()

        threading.Thread(target=self.cargar_versiones, daemon=True).start()
        threading.Thread(target=self.ping_servidor, daemon=True).start()
        
        threading.Thread(target=self.buscar_actualizaciones, daemon=True).start()
        threading.Thread(target=self.iniciar_discord_rpc, daemon=True).start()

        # --- MONITOR DE TELEMETRÍA (HUD) --- 
        self.lbl_telemetria = ctk.CTkLabel(
            self.frame_main, 
            text="⚡ Iniciando sensores de hardware...", 
            text_color="#2ecc71", 
            font=ctk.CTkFont(size=12, weight="bold")
        )
        self.lbl_telemetria.pack(side="bottom", pady=(5, 0))
        threading.Thread(target=self._hilo_telemetria, daemon=True).start()

        try:
            windnd.hook_dropfiles(self.winfo_id(), self.procesar_archivos_soltados)
        except Exception as e:
            print("No se pudo iniciar el Drag & Drop:", e)

    def centrar_ventana(self, ventana, ancho, alto):
        # Saca la resolución de la pantalla
        pantalla_ancho = ventana.winfo_screenwidth()
        pantalla_alto = ventana.winfo_screenheight()
        
        # Calcula el centro exacto
        x = int((pantalla_ancho / 2) - (ancho / 2))
        y = int((pantalla_alto / 2) - (alto / 2))
        
        # Aplica la geometría
        ventana.geometry(f"{ancho}x{alto}+{x}+{y}")

    # --- NUEVO MOTOR DE PING (MCSTATUS) ---
    def ping_servidor(self):
        def hacer_ping():
            try:
                # Intento 1: Por internet
                server = JavaServer.lookup(SERVER_IP)
                status = server.status()
            except Exception:
                try:
                    # Intento 2: Red local
                    server_local = JavaServer.lookup("127.0.0.1:25565")
                    status = server_local.status()
                except Exception:
                    # Si los dos fallan, esta apagado
                    self.after(0, lambda: self.lbl_server.configure(text="🔴 El Server está OFFLINE", text_color="#e74c3c"))
                    return

            # Si cualquiera de los dos intentos funcionó, lo pintamos de verde
            texto_online = f"🟢 ONLINE ({status.players.online}/{status.players.max} Jugadores)"
            self.after(0, lambda: self.lbl_server.configure(text=texto_online, text_color="#2ecc71"))
        
        threading.Thread(target=hacer_ping, daemon=True).start()

    #  MOTOR DE AUTO-UPDATER DINÁMICO
    def buscar_actualizaciones(self):
        try:
            if not UPDATE_URL: return
            r = requests.get(UPDATE_URL, timeout=10) 
            if r.status_code == 200:
                version_remota = r.text.strip()
                if version_remota != LAUNCHER_VERSION:
                    self.after(0, self.preguntar_actualizacion, version_remota)
        except Exception as e:
            print(f"Error de conexión: {e}")
    

    def preguntar_actualizacion(self, version_remota):
        respuesta = messagebox.askyesno(
            "¡Actualización Disponible!",
            f"¡Salió la versión {version_remota} de Paraguacraft!\n\n¿Querés actualizar ahora? Es una descarga rápida y 100% segura."
        )
        if respuesta:
            # Apuntamos a la nueva función
            threading.Thread(target=self.ejecutar_actualizacion_segura, args=(version_remota,), daemon=True).start()

    def ejecutar_actualizacion_segura(self, version_remota):
        import requests
        import sys
        import os
        import subprocess
        from tkinter import messagebox

        try:
            self.actualizar_progreso("Preparando actualización segura...")
            self.boton_jugar.configure(state="disabled", text="ACTUALIZANDO...")

            # URL directa al .exe en tu GitHub
            url_descarga_exe = f"https://github.com/SantiJ10/Paraguacraft/releases/download/v.{version_remota}/Paraguacraft.exe"

            r = requests.get(url_descarga_exe, stream=True, timeout=30)
            r.raise_for_status()

            total_length = int(r.headers.get('content-length', 0))
            descargado = 0

            exe_actual = sys.executable
            
            # Si estás probando el código desde VS Code (.py), cancela para no romper nada
            if not getattr(sys, 'frozen', False):
                messagebox.showinfo("Desarrollo", "Estás en entorno Python. Simulando actualización exitosa.")
                self.boton_jugar.configure(state="normal", text="INICIAR PARAGUACRAFT")
                return

            ruta_vieja = exe_actual + ".old"

            # 1. El truco maestro: Renombramos el archivo que se está ejecutando AHORA MISMO
            if os.path.exists(ruta_vieja):
                os.remove(ruta_vieja) # Por si quedó basura de antes
            os.rename(exe_actual, ruta_vieja)

            # 2. Descargamos el archivo nuevo y lo guardamos con el nombre original impecable
            with open(exe_actual, "wb") as f:
                for chunk in r.iter_content(chunk_size=8192):
                    if chunk:
                        f.write(chunk)
                        descargado += len(chunk)
                        if total_length > 0:
                            porcentaje = descargado / total_length
                            self.barra_progreso.set(porcentaje)
                            self.lbl_estado.configure(text=f"Descargando versión {version_remota}: {int(porcentaje * 100)}%")
                            self.update() # Mantiene la barra fluida

            self.actualizar_progreso("¡Actualización completada! Reiniciando...")
            
            # 3. Lanzamos el launcher nuevo y suicidamos al viejo al instante
            subprocess.Popen([exe_actual])
            sys.exit()

        except Exception as e:
            print(f"Error crítico en actualización: {e}")
            self.actualizar_progreso("Error al actualizar. Reintentá más tarde.")
            self.after(0, lambda: messagebox.showerror("Error", f"Fallo en la conexión: {str(e)}"))
            self.after(0, lambda: self.boton_jugar.configure(state="normal", text="INICIAR PARAGUACRAFT"))
            
            # Si se cortó el internet a mitad de camino, restauramos el nombre viejo para no romper tu launcher
            try:
                if os.path.exists(ruta_vieja) and not os.path.exists(exe_actual):
                    os.rename(ruta_vieja, exe_actual)
            except: pass

    def _hilo_telemetria(self):
        import time, psutil
        while True:
            try:
                # Lee los sensores cada 1 segundo
                cpu = psutil.cpu_percent(interval=1)
                ram_libre = psutil.virtual_memory().available / (1024**3)
                ram_total = psutil.virtual_memory().total / (1024**3)
                
                # Lógica de colores según el estrés del hardware
                color = "#2ecc71" # Verde (Óptimo)
                if cpu > 85 or ram_libre < 3: color = "#e74c3c" # Rojo (Peligro/Stuttering inminente)
                elif cpu > 60: color = "#f1c40f" # Amarillo (Carga pesada)
                
                texto = f"⚡ CPU: {cpu}%  |  💾 RAM Libre: {ram_libre:.1f}GB / {ram_total:.1f}GB"
                self.after(0, lambda t=texto, c=color: self.lbl_telemetria.configure(text=t, text_color=c))
            except: pass

    def iniciar_discord_rpc(self):
        try:
            from pypresence import Presence
            self.rpc = Presence(DISCORD_APP_ID)
            self.rpc.connect()
            self.rpc.update(state="En el menú principal", details="Preparándose para jugar", large_image="logo", large_text="Paraguacraft Launcher")
        except: 
            self.rpc = None # ¡CLAVE! Si falla, anulamos la conexión

    def login_microsoft(self):
        try:
            from minecraft_launcher_lib.microsoft_account import get_login_url, complete_login, url_contains_auth_code, get_auth_code_from_url
            url = get_login_url(CLIENT_ID, REDIRECT_URI)
            webbrowser.open(url)
            dialog = ctk.CTkInputDialog(text="1. Iniciá sesión en el navegador.\n2. Copiá TODO el link de arriba.\n3. Pegalo acá abajo:", title="Conectar Cuenta Premium")
            url_respuesta = dialog.get_input()
            if url_respuesta:
                if url_contains_auth_code(url_respuesta):
                    auth_code = get_auth_code_from_url(url_respuesta)
                    self.lbl_estado.configure(text="Verificando licencia con Mojang...")
                    self.update()
                    account_data = complete_login(CLIENT_ID, None, REDIRECT_URI, auth_code)
                    self.ms_data = account_data
                    self.combo_usuario.set(account_data["name"])
                    self.combo_usuario.configure(state="disabled") 
                    self.btn_microsoft.configure(text="Desconectar Premium", command=self.logout_microsoft, fg_color="#C70039", hover_color="#900C3F")
                    with open(SESSION_FILE, "w") as f: json.dump(account_data, f)
                    self.lbl_estado.configure(text=f"¡Bienvenido, {account_data['name']} (Premium)!")
                    messagebox.showinfo("Login Exitoso", f"Cuenta Premium verificada: {account_data['name']}")
                else:
                    messagebox.showerror("Error", "No copiaste bien el link o cancelaste el login.")
        except Exception as e:
            messagebox.showerror("Error de Login", str(e))

    def cargar_sesion_microsoft(self):
        if os.path.exists(SESSION_FILE):
            try:
                with open(SESSION_FILE, "r") as f:
                    self.ms_data = json.load(f)
                    self.combo_usuario.set(self.ms_data["name"])
                    self.combo_usuario.configure(state="disabled")
                    self.btn_microsoft.configure(text="Desconectar Premium", command=self.logout_microsoft, fg_color="#C70039", hover_color="#900C3F")
                    self.lbl_estado.configure(text="Cuenta Premium cargada.")
            except: pass

    def logout_microsoft(self):
        self.ms_data = None
        if os.path.exists(SESSION_FILE): os.remove(SESSION_FILE)
        self.combo_usuario.configure(state="normal")
        if self.usuarios_guardados: self.combo_usuario.set(self.usuarios_guardados[0])
        else: self.combo_usuario.set("")
        self.btn_microsoft.configure(text="Iniciar Sesión con Microsoft", command=self.login_microsoft, fg_color="#107C10", hover_color="#0b5e0b")
        self.lbl_estado.configure(text="Sesión cerrada. Modo Offline activado.")

    def cargar_configuracion(self):
        if os.path.exists(CONFIG_FILE):
            try:
                with open(CONFIG_FILE, "r") as f:
                    datos = json.load(f)
                    self.usuarios_guardados = datos.get("historial_usuarios", [])
                    if self.usuarios_guardados:
                        self.combo_usuario.configure(values=self.usuarios_guardados)
                        self.combo_usuario.set(datos.get("usuario", self.usuarios_guardados[0]))
                    
                    self.combo_ram.set(datos.get("ram", "4GB"))
                    self.gc_var.set(datos.get("gc", "G1GC (Equilibrado / Recomendado)"))
                    self.opt_var.set(datos.get("opt_minimos", False))
                    self.lan_distancia_var.set(datos.get("lan_distancia", False))
                    
                    # --- EL NUEVO SELECTOR DE CLIENTE ---
                    self.tipo_cliente_var.set(datos.get("tipo_cliente", "Fabric"))
                    
                    self.papa_var.set(datos.get("papa_mode", False))
                    self.mesa_var.set(datos.get("mesa", False))
                    self.consola_var.set(datos.get("consola", False))
                    
                    self.veces_iniciadas.set(datos.get("veces_iniciadas", 0))
                    self.horas_jugadas_total.set(datos.get("horas_jugadas_total", 0.0))
                    self.limpiador_deep_var.set(datos.get("limpiador_deep", False))
                    self.backup_var.set(datos.get("backup_on_launch", True))
                    
                    if hasattr(self, 'tema_var'):
                        self.tema_var.set(datos.get("tema", "dark"))
                        ctk.set_appearance_mode(self.tema_var.get())
            except: pass

    def guardar_configuracion(self):
        usuario_actual = self.combo_usuario.get().strip()
        if usuario_actual and usuario_actual not in self.usuarios_guardados and not self.ms_data:
            self.usuarios_guardados.append(usuario_actual)
            self.combo_usuario.configure(values=self.usuarios_guardados)

        try:
            datos = {
                "usuario": usuario_actual, "historial_usuarios": self.usuarios_guardados,
                "ram": self.combo_ram.get(), "gc": self.gc_var.get(),
                "opt_minimos": self.opt_var.get(), 
                "lan_distancia": self.lan_distancia_var.get(),
                
                # --- GUARDAR EL SELECTOR DE CLIENTE ---
                "tipo_cliente": self.tipo_cliente_var.get(), 
                
                "papa_mode": self.papa_var.get(),
                "mesa": self.mesa_var.get(), "consola": self.consola_var.get(),
                "veces_iniciadas": self.veces_iniciadas.get(),
                "horas_jugadas_total": self.horas_jugadas_total.get(),
                "limpiador_deep": self.limpiador_deep_var.get(),
                "backup_on_launch": self.backup_var.get()
            }
            
            # --- GUARDAR EL TEMA VISUAL ---
            if hasattr(self, 'tema_var'):
                datos["tema"] = self.tema_var.get()
                
            with open(CONFIG_FILE, "w") as f: json.dump(datos, f)
        except: pass

    def cargar_versiones(self):
        try:
            mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            versiones_locales = minecraft_launcher_lib.utils.get_installed_versions(mine_dir)
            lista_locales = [v["id"] for v in versiones_locales]
            
            versiones_oficiales = minecraft_launcher_lib.utils.get_version_list()
            
            lista_oficiales = []
            palabras_prohibidas = ["rc", "pre", "snapshot", "alpha", "beta"]
            
            for v in versiones_oficiales:
                if v["type"] == "release":
                    version_limpia = v["id"].lower()
                    if not any(palabra in version_limpia for palabra in palabras_prohibidas):
                        lista_oficiales.append(v["id"])

            # 3. Juntamos las locales (Fabric/Forge ya instalados) con las Vanilla limpias
            versiones_finales = lista_locales.copy()
            for ver in lista_oficiales:
                if ver not in versiones_finales: versiones_finales.append(ver)

            self.lista_versiones.delete(0, tk.END)
            for ver in versiones_finales: self.lista_versiones.insert(tk.END, ver)
            
            try:
                indice_target = versiones_finales.index("1.21.1")
                self.lista_versiones.selection_set(indice_target)
            except: self.lista_versiones.selection_set(0)
        except: self.lbl_estado.configure(text="Error al cargar versiones.")

    def abrir_config(self):
        vent = ctk.CTkToplevel(self)
        vent.title("Ajustes Avanzados")
        self.centrar_ventana(vent, 480, 780) 
        vent.grab_set()
        vent.resizable(True, True)

        if getattr(sys, 'frozen', False):
            icono_ajustes = os.path.join(sys._MEIPASS, "iconomc.ico")
        else:
            icono_ajustes = "iconomc.ico"
        if os.path.exists(icono_ajustes):
            try: vent.iconbitmap(icono_ajustes)
            except: pass

        scroll_main = ctk.CTkScrollableFrame(vent, fg_color="transparent")
        scroll_main.pack(fill="both", expand=True, padx=10, pady=10)

        borde_color = ("#c0c0c0", "#3a3a3a")

        # --- 1. PERSONALIZACIÓN Y SISTEMA ---
        ctk.CTkLabel(scroll_main, text="Personalización y Sistema", font=ctk.CTkFont(weight="bold", size=15)).pack(pady=(10, 5))
        frame_top = ctk.CTkFrame(scroll_main, fg_color="transparent")
        frame_top.pack(fill="x", padx=10, pady=5)
        
        ctk.CTkLabel(frame_top, text="Tema Visual:").pack(side="left", padx=(0, 10))
        combo_tema = ctk.CTkComboBox(frame_top, values=["dark", "light", "system"], command=self.cambiar_tema, width=100)
        combo_tema.set(self.tema_var.get())
        combo_tema.pack(side="left")
        
        ctk.CTkButton(frame_top, text="🔍 Auto-Configurar", command=self.auto_configurar_hardware, fg_color="#2980b9", hover_color="#1f618d", width=120).pack(side="right")

        # --- 2. MOTOR Y RENDIMIENTO JAVA ---
        ctk.CTkLabel(scroll_main, text="Motor y Rendimiento", font=ctk.CTkFont(weight="bold", size=15)).pack(pady=(15, 5))
        frame_motor = ctk.CTkFrame(scroll_main, border_width=2, border_color=borde_color, corner_radius=10)
        frame_motor.pack(fill="x", padx=10, pady=5)

        ctk.CTkLabel(frame_motor, text="Garbage Collector (Java):", text_color="gray").pack(pady=(10,0))
        opciones_gc = ["G1GC (Equilibrado / Recomendado)", "ZGC (Latencia Ultra Baja, requiere RAM)", "Shenandoah (Rendimiento fluido de fondo)", "CMS (Para versiones antiguas)"]
        ctk.CTkComboBox(frame_motor, values=opciones_gc, variable=self.gc_var, width=380).pack(pady=(5, 10), padx=10)

        ctk.CTkLabel(frame_motor, text="Motor del Juego:", text_color="gray").pack(pady=(5,0))
        opciones_cliente = ["Vanilla (Juego Base)", "Fabric (Optimizado/Recomendado)", "Forge (Mods Clásicos)"]
        ctk.CTkComboBox(frame_motor, values=opciones_cliente, variable=self.tipo_cliente_var, width=380).pack(pady=(5, 10), padx=10)

        # --- 3. AJUSTES DEL JUEGO (Todos los checks alineados perfecto) ---
        frame_ajustes = ctk.CTkFrame(scroll_main, border_width=2, border_color=borde_color, corner_radius=10)
        frame_ajustes.pack(fill="x", padx=10, pady=10)
        
        ctk.CTkLabel(frame_ajustes, text="Ajustes de Lanzamiento", font=ctk.CTkFont(weight="bold")).pack(pady=(10, 5))
        
        # El anchor="w" hace que todos queden pegados a la izquierda
        ctk.CTkCheckBox(frame_ajustes, text="📉 Aplicar gráficos al mínimo (Mejora FPS)", variable=self.opt_var).pack(pady=5, padx=20, anchor="w")
        ctk.CTkCheckBox(frame_ajustes, text="🥔 Modo 'PC Papa' (Resolución 800x600)", variable=self.papa_var).pack(pady=5, padx=20, anchor="w")
        ctk.CTkCheckBox(frame_ajustes, text="🌐 Multijugador LAN a distancia (e4mc)", variable=self.lan_distancia_var).pack(pady=5, padx=20, anchor="w")
        ctk.CTkCheckBox(frame_ajustes, text="🖥️ Forzar OpenGL Mesa (Para errores gráficos)", variable=self.mesa_var).pack(pady=5, padx=20, anchor="w")
        ctk.CTkCheckBox(frame_ajustes, text="📜 Mostrar Consola de Depuración", variable=self.consola_var).pack(pady=(5, 10), padx=20, anchor="w")

        # --- 4. MANTENIMIENTO ---
        frame_mantenimiento = ctk.CTkFrame(scroll_main, border_width=2, border_color=borde_color, corner_radius=10)
        frame_mantenimiento.pack(fill="x", padx=10, pady=10)
        
        ctk.CTkLabel(frame_mantenimiento, text="Mantenimiento Automático", font=ctk.CTkFont(weight="bold")).pack(pady=(10, 5))
        ctk.CTkCheckBox(frame_mantenimiento, text="💾 Auto-Backup Salvavidas (Zip al iniciar)", variable=self.backup_var).pack(pady=5, padx=20, anchor="w")
        ctk.CTkCheckBox(frame_mantenimiento, text="🧹 Limpieza Profunda al Iniciar (logs/crash)", variable=self.limpiador_deep_var).pack(pady=(5, 10), padx=20, anchor="w")

        # --- 5. GESTOR DE SKINS ---
        frame_skins = ctk.CTkFrame(scroll_main, border_width=2, border_color=borde_color, corner_radius=10)
        frame_skins.pack(fill="x", padx=10, pady=10)
        
        ctk.CTkLabel(frame_skins, text="Gestor de Skins", font=ctk.CTkFont(weight="bold")).pack(pady=(10, 5))
        
        # Fila 1: Opciones Offline
        frame_skin_buttons = ctk.CTkFrame(frame_skins, fg_color="transparent")
        frame_skin_buttons.pack(fill="x", pady=(0, 5), padx=10)
        
        ctk.CTkButton(frame_skin_buttons, text="🖼️ Skin Offline", command=self.elegir_skin, fg_color="#8e44ad", hover_color="#5e3370").pack(side="left", padx=5, expand=True, fill="x")
        ctk.CTkButton(frame_skin_buttons, text="❌ Borrar", command=self.borrar_skin, fg_color="#c0392b", hover_color="#922b21", width=80).pack(side="right", padx=5)

        # Fila 2: Opción Premium (Botón ancho y dorado)
        frame_skin_premium = ctk.CTkFrame(frame_skins, fg_color="transparent")
        frame_skin_premium.pack(fill="x", pady=(0, 10), padx=10)
        
        ctk.CTkButton(frame_skin_premium, text="👑 Cambiar Skin Premium (API Mojang)", command=self.cambiar_skin_premium_api, fg_color="#d4af37", text_color="black", hover_color="#b5952f").pack(fill="x", padx=5)
        # --- 6. BOTONES FINALES ---
        frame_botones_finales = ctk.CTkFrame(scroll_main, fg_color="transparent")
        frame_botones_finales.pack(fill="x", padx=10, pady=15)

        # Fila 1: Tienda y SOS (Lado a lado)
        frame_fila1 = ctk.CTkFrame(frame_botones_finales, fg_color="transparent")
        frame_fila1.pack(fill="x", pady=(0, 5))
        
        self.btn_tienda = ctk.CTkButton(frame_fila1, text="🛒 Tienda de Mods", command=self.abrir_tienda, fg_color="#27ae60", hover_color="#2ecc71", font=ctk.CTkFont(weight="bold"), height=40)
        self.btn_tienda.pack(side="left", expand=True, fill="x", padx=(0, 5))

        ctk.CTkButton(frame_fila1, text="🚑 Reparación (SOS)", command=self.ejecutar_reparacion_sos, fg_color="#c0392b", hover_color="#922b21", height=40).pack(side="right", expand=True, fill="x", padx=(5, 0))

        # Fila 2 y 3: Las nuevas herramientas del Sprint
        ctk.CTkButton(frame_botones_finales, text="🔄 Smart Updater (Actualizar Mods)", command=self.actualizar_mods_instancia, fg_color="#2980b9", hover_color="#1f618d", height=35).pack(fill="x", pady=5)
        
        ctk.CTkButton(frame_botones_finales, text="🖥️ Creador de Servidor Dedicado Local", command=self.crear_servidor_local, fg_color="#8e44ad", hover_color="#5e3370", height=35).pack(fill="x", pady=5)

    def abrir_gestor_mods(self):
        seleccion = self.lista_versiones.curselection()
        if not seleccion:
            messagebox.showwarning("Aviso", "Seleccioná una versión en la lista primero.")
            return
            
        version_actual = self.lista_versiones.get(seleccion[0])
        tipo_cliente_ui = self.tipo_cliente_var.get().split()[0]
        
        folder_name = f"Paraguacraft_{version_actual}_{tipo_cliente_ui}".replace(".", "_")
        mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
        mods_dir = os.path.join(mine_dir, "instancias", folder_name, "mods")

        if not os.path.exists(mods_dir):
            messagebox.showinfo("Gestor", "No hay mods instalados para esta versión todavía.")
            return

        vent_gestor = ctk.CTkToplevel(self)
        vent_gestor.title(f"Gestor de Mods - {version_actual}")
        self.centrar_ventana(vent_gestor, 450, 550)
        vent_gestor.grab_set()

        ctk.CTkLabel(vent_gestor, text="Interruptor de Mods", font=ctk.CTkFont(size=20, weight="bold")).pack(pady=10)
        ctk.CTkLabel(vent_gestor, text="Apagá los mods que colisionen (ej. Iris).", text_color="gray").pack(pady=(0,10))

        scroll_mods = ctk.CTkScrollableFrame(vent_gestor, width=400, height=400)
        scroll_mods.pack(pady=10, padx=20, fill="both", expand=True)

        from src.modelo import GestorLocalMods
        lista_mods = GestorLocalMods.obtener_lista_mods(mods_dir)

        if not lista_mods:
            ctk.CTkLabel(scroll_mods, text="Carpeta de mods vacía.").pack(pady=20)
            return

        # Generador dinámico de interruptores
        def crear_switch(mod_info):
            estado_inicial = mod_info["estado"] == "Activo"
            # Limpiamos el nombre para que quede estético en la ventana
            nombre_limpio = mod_info["archivo"].replace(".jar", "").replace(".disabled", "")

            var = ctk.BooleanVar(value=estado_inicial)

            def alternar():
                nuevo_nombre = GestorLocalMods.alternar_estado_mod(mods_dir, mod_info["archivo"])
                if nuevo_nombre:
                    mod_info["archivo"] = nuevo_nombre # Se actualiza en memoria para el próximo clic

            switch = ctk.CTkSwitch(scroll_mods, text=nombre_limpio, variable=var, command=alternar)
            switch.pack(pady=8, anchor="w", padx=10)

        for mod in lista_mods:
            crear_switch(mod)

    def elegir_skin(self):
        ruta = filedialog.askopenfilename(title="Skin (.png)", filetypes=[("PNG", "*.png")])
        if ruta:
            mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            
            # Las nuevas rutas obligatorias para 1.19.3+ (Steve es wide, Alex es slim)
            skin_path_wide = os.path.join(mine_dir, "resourcepacks", "ParaguacraftBrandPack", "assets", "minecraft", "textures", "entity", "player", "wide")
            skin_path_slim = os.path.join(mine_dir, "resourcepacks", "ParaguacraftBrandPack", "assets", "minecraft", "textures", "entity", "player", "slim")
            # Ruta clásica por si jugás en la 1.16 o 1.8
            skin_path_old = os.path.join(mine_dir, "resourcepacks", "ParaguacraftBrandPack", "assets", "minecraft", "textures", "entity")
            
            os.makedirs(skin_path_wide, exist_ok=True)
            os.makedirs(skin_path_slim, exist_ok=True)
            os.makedirs(skin_path_old, exist_ok=True)
            
            try:
                # 1. Aplicamos a Steve (Brazos gruesos)
                shutil.copy(ruta, os.path.join(skin_path_wide, "steve.png"))
                shutil.copy(ruta, os.path.join(skin_path_old, "steve.png"))
                
                # 2. Aplicamos a Alex (Brazos finos, por si a tu amigo le toca esta)
                shutil.copy(ruta, os.path.join(skin_path_slim, "alex.png"))
                shutil.copy(ruta, os.path.join(skin_path_old, "alex.png"))
                
                messagebox.showinfo("Skin Guardada", "¡Skin aplicada para todas las versiones y modelos!")
                self.actualizar_avatar_visual()
            except: messagebox.showerror("Error", "No se pudo copiar la skin.")

    def borrar_skin(self):
        mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
        rutas_limpiar = [
            os.path.join(mine_dir, "resourcepacks", "ParaguacraftBrandPack", "assets", "minecraft", "textures", "entity", "player", "wide"),
            os.path.join(mine_dir, "resourcepacks", "ParaguacraftBrandPack", "assets", "minecraft", "textures", "entity", "player", "slim"),
            os.path.join(mine_dir, "resourcepacks", "ParaguacraftBrandPack", "assets", "minecraft", "textures", "entity")
        ]
        
        try:
            for ruta in rutas_limpiar:
                if os.path.exists(os.path.join(ruta, "steve.png")): os.remove(os.path.join(ruta, "steve.png"))
                if os.path.exists(os.path.join(ruta, "alex.png")): os.remove(os.path.join(ruta, "alex.png"))
            
            messagebox.showinfo("Skin Borrada", "Skin restablecida a la de defecto (Steve/Alex).")
            self.actualizar_avatar_visual()
        except: pass

    def descargar_mods_zip(self):
        if not MODS_ZIP_URL:
            messagebox.showinfo("Ups", "Santi todavía no subió el link de los mods.")
            return
        threading.Thread(target=self._hilo_descarga_mods, daemon=True).start()

    def _hilo_descarga_mods(self):
        try:
            self.lbl_estado.configure(text="Descargando mods oficiales...")
            r = requests.get(MODS_ZIP_URL, stream=True)
            if r.status_code == 200:
                mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
                mods_dir = os.path.join(mine_dir, "mods")
                os.makedirs(mods_dir, exist_ok=True)
                z = zipfile.ZipFile(io.BytesIO(r.content))
                z.extractall(mods_dir)
                self.lbl_estado.configure(text="¡Mods instalados con éxito!")
                messagebox.showinfo("Mods", "¡Paquete instalado correctamente!")
            else: self.lbl_estado.configure(text="Error al descargar mods.")
        except Exception as e: self.lbl_estado.configure(text=f"Error: {str(e)}")

    def abrir_carpeta_minecraft(self):
        minecraft_directory = minecraft_launcher_lib.utils.get_minecraft_directory()
        if platform.system() == "Windows": os.startfile(minecraft_directory)
        else: subprocess.Popen(["xdg-open", minecraft_directory])

    def actualizar_progreso(self, mensaje):
        self.after(0, self._actualizar_gui, mensaje)

    def _actualizar_gui(self, mensaje):
        self.lbl_estado.configure(text=mensaje)
        self.barra_progreso.step()

    def _hacer_backup_real(self):
        try:
            mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            saves_dir = os.path.join(mine_dir, "saves")
            if os.path.exists(saves_dir):
                backup_folder = os.path.join(mine_dir, "backups_paraguacraft")
                os.makedirs(backup_folder, exist_ok=True)
                fecha = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
                archivo_zip = os.path.join(backup_folder, f"backup_mundos_{fecha}")
                shutil.make_archive(archivo_zip, 'zip', saves_dir)
        except Exception: pass

    def iniciar_juego_thread(self):
        seleccion = self.lista_versiones.curselection()
        if not seleccion:
            self.lbl_estado.configure(text="¡Selecciona una versión!")
            return

        usuario = self.combo_usuario.get().strip()
        if not usuario:
            messagebox.showwarning("Falta nombre", "Ponete un nombre de jugador para entrar offline.")
            return

        version = self.lista_versiones.get(seleccion[0])
        ram_seleccionada = self.combo_ram.get()
        ram = ram_seleccionada.replace("GB", "G") 
        gc_type = self.gc_var.get().split()[0]
        tipo_cliente = self.tipo_cliente_var.get().split()[0]

        self.veces_iniciadas.set(self.veces_iniciadas.get() + 1)
        self.guardar_configuracion()
        
        self.boton_jugar.configure(state="disabled", text="TRABAJANDO...")
        self.barra_progreso.set(0)

        if self.rpc:
            try: self.rpc.update(state=f"Jugando Paraguacraft {version}", details=f"Jugador: {usuario}", large_image="logo")
            except: pass

        uuid_real = None
        token_real = None
        if self.ms_data:
            uuid_real = self.ms_data["id"]
            token_real = self.ms_data["access_token"]

        self.hilo_juego_activo = True
        
        hilo = threading.Thread(target=self.ejecutar_motor, args=(version, usuario, ram, gc_type, tipo_cliente, uuid_real, token_real), daemon=True)
        hilo.start()

    def ejecutar_motor(self, version_jugada, usuario, ram, gc_type, tipo_cliente, uuid_real, token_real):
        try:
            if self.backup_var.get():
                self.after(0, lambda: self.lbl_estado.configure(text="Creando auto-backup salvavidas (Zip)..."))
                self._hacer_backup_real()
            
            if self.limpiador_deep_var.get() and os.path.exists(LOG_DIR):
                self.after(0, lambda: self.lbl_estado.configure(text="Ejecutando limpiador profundo invisible..."))
                try:
                    for filename in os.listdir(LOG_DIR):
                        if filename.endswith(".log.gz") or filename == "latest.log":
                            os.remove(os.path.join(LOG_DIR, filename))
                except Exception: pass

            # 1. ACTIVAMOS EL BOOSTER DESDE EL CONTROLADOR (Limpia procesos)
            self.after(0, lambda: GameBooster.activar_modo_gamer(self.actualizar_progreso))

            self.after(0, lambda: self.lbl_estado.configure(text="Lanzando Paraguacraft..."))
            
            # 2. LANZAMOS LOS HILOS SECUNDARIOS (Ninja y Discord)
            threading.Thread(target=self._hilo_ninja_renombrar, args=(version_jugada,), daemon=True).start()
            threading.Thread(target=self._hilo_discord_rpc_dinamico, args=(version_jugada, usuario, tipo_cliente), daemon=True).start()

            # 3. LANZAMOS EL JUEGO REAL
            start_time = datetime.now()
            lanzar_minecraft(version_jugada, usuario, ram, gc_type, self.opt_var.get(), tipo_cliente, self.papa_var.get(), self.mesa_var.get(), self.consola_var.get(), self.actualizar_progreso, uuid_real, token_real, self.lan_distancia_var.get())
            
            self.hilo_juego_activo = False
            
            end_time = datetime.now()
            duracion = end_time - start_time
            horas_jugadas_sesion = duracion.total_seconds() / 3600.0
            
            nuevo_total_horas = self.horas_jugadas_total.get() + horas_jugadas_sesion
            self.horas_jugadas_total.set(nuevo_total_horas)
            self.guardar_configuracion()
            
            self.after(0, lambda: self.lbl_estado.configure(text=f"Juego finalizado (Jugaste {horas_jugadas_sesion:.2f}hs)"))
                
        except Exception as e:
            self.hilo_juego_activo = False
            error_msg = f"Error: {str(e)[:40]}"
            self.after(0, self.deiconify)
            self.after(0, lambda m=error_msg: self.lbl_estado.configure(text=m))
        finally:
            self.after(0, lambda: self.boton_jugar.configure(state="normal", text="INICIAR PARAGUACRAFT"))

    def _hilo_ninja_renombrar(self, version_jugada):
        import time
        import win32gui
        
        while self.hilo_juego_activo:
            time.sleep(1)
            def callback(hwnd, extra):
                if win32gui.IsWindowVisible(hwnd):
                    titulo = win32gui.GetWindowText(hwnd)
                    if "Minecraft" in titulo and "ParaguaCraft Launcher" not in titulo:
                        nuevo_titulo = titulo.replace("Minecraft*", "Paraguacraft").replace("Minecraft", "Paraguacraft")
                        if nuevo_titulo != titulo:
                            win32gui.SetWindowText(hwnd, nuevo_titulo)
            win32gui.EnumWindows(callback, None)
    
    # --- FUNCIONES NUEVAS: MULTICUENTA, TEMAS Y HARDWARE ---

    def borrar_usuario_seleccionado(self):
        usuario = self.combo_usuario.get().strip()
        if usuario in self.usuarios_guardados:
            self.usuarios_guardados.remove(usuario)
            self.combo_usuario.configure(values=self.usuarios_guardados)
            self.combo_usuario.set(self.usuarios_guardados[0] if self.usuarios_guardados else "")
            self.guardar_configuracion()
            messagebox.showinfo("Gestor de Cuentas", f"La cuenta '{usuario}' fue eliminada del historial.")

    def cambiar_tema(self, valor):
        ctk.set_appearance_mode(valor)
        self.tema_var.set(valor)
        self.guardar_configuracion()

    def auto_configurar_hardware(self):
        # Lee RAM y Núcleos del Procesador físicos
        ram_total_gb = max(1, int(psutil.virtual_memory().total / (1024 ** 3)))
        cpu_cores = psutil.cpu_count(logical=False) or 2 

        if ram_total_gb <= 8 or cpu_cores <= 2:
            # PC Básica: Deja 3GB de RAM, activa gráficos al mínimo y fuerza Fabric
            self.combo_ram.set("3GB")
            self.opt_var.set(True) 
            self.tipo_cliente_var.set("Fabric") 
            self.papa_var.set(ram_total_gb <= 4) # Si tiene 4 o menos, activa Modo Papa
        elif ram_total_gb <= 16 and cpu_cores <= 4:
            # PC Media
            self.combo_ram.set("6GB")
            self.opt_var.set(False) 
            self.papa_var.set(False)
        else:
            # PC Alta (Sobrada)
            self.combo_ram.set("8GB") 
            self.opt_var.set(False) 
            self.papa_var.set(False)
        
        self.guardar_configuracion()

    def ejecutar_reparacion_sos(self):
        respuesta = messagebox.askyesno("Reparación SOS", "Esto va a borrar la caché, los logs viejos y va a resetear la configuración gráfica (options.txt) de TODAS tus instancias.\n\nTus mundos y servers NO se van a borrar.\n\n¿Querés proceder?")
        if respuesta:
            try:
                mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
                
                # Limpieza de basura general
                for carpeta in ["logs", "crash-reports", "webcache2"]:
                    ruta = os.path.join(mine_dir, carpeta)
                    if os.path.exists(ruta): shutil.rmtree(ruta, ignore_errors=True)
                
                # Resetear opciones de las instancias aisladas
                instancias_dir = os.path.join(mine_dir, "instancias")
                if os.path.exists(instancias_dir):
                    for carpeta_instancia in os.listdir(instancias_dir):
                        ruta_opt = os.path.join(instancias_dir, carpeta_instancia, "options.txt")
                        if os.path.exists(ruta_opt): os.remove(ruta_opt)
                        
                messagebox.showinfo("SOS Completado", "Se reparó el caché con éxito. Volvé a iniciar el juego.")
            except Exception as e:
                messagebox.showerror("Error", f"No se pudo completar la limpieza: {e}")

    def actualizar_mods_instancia(self):
        seleccion = self.lista_versiones.curselection()
        if not seleccion: return
        version_actual = self.lista_versiones.get(seleccion[0])
        tipo_cliente_ui = self.tipo_cliente_var.get().split()[0]
        
        folder_name = f"Paraguacraft_{version_actual}_{tipo_cliente_ui}".replace(".", "_")
        mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
        mods_dir = os.path.join(mine_dir, "instancias", folder_name, "mods")
        
        if not os.path.exists(mods_dir):
            messagebox.showinfo("Aviso", "No hay mods instalados en esta versión todavía.")
            return

        def _hilo_mvc():
            # Acá el Controlador (launcher.py) le pasa la tarea al Modelo (GestorMods)
            # y le pasa 'self.actualizar_progreso' para que el Modelo pueda actualizar la UI sin romper Tkinter
            actualizados, reparados = GestorMods.verificar_y_actualizar(mods_dir, version_actual, self.actualizar_progreso)
            
            # Una vez que el Modelo termina, el Controlador manda el pop-up
            mensaje_final = f"Proceso finalizado.\n\nMods actualizados: {actualizados}\nArchivos corruptos reparados: {reparados}"
            self.after(0, lambda: messagebox.showinfo("Smart Updater Pro", mensaje_final))

        threading.Thread(target=_hilo_mvc, daemon=True).start()
    
    def abrir_tienda(self):
        vent_tienda = ctk.CTkToplevel(self)
        vent_tienda.title("Tienda Paraguacraft (Mods, Shaders, Texturas)")
        self.centrar_ventana(vent_tienda, 650, 600) # Un poquito más ancha
        vent_tienda.grab_set()

        # --- BARRA DE BÚSQUEDA Y FILTROS ---
        frame_busqueda = ctk.CTkFrame(vent_tienda, fg_color="transparent")
        frame_busqueda.pack(pady=15, padx=20, fill="x")

        # Nuevo Filtro de Categorías
        self.tipo_busqueda_var = ctk.StringVar(value="Mods")
        combo_filtro = ctk.CTkComboBox(frame_busqueda, values=["Mods", "Shaders", "Texturas", "Modpacks"], variable=self.tipo_busqueda_var, width=110)
        combo_filtro.pack(side="left", padx=(0, 10))

        self.entry_busqueda = ctk.CTkEntry(frame_busqueda, placeholder_text="Ej: Sodium, Iris, Bare Bones...", width=250)
        self.entry_busqueda.pack(side="left", padx=(0, 10), expand=True, fill="x")
        self.entry_busqueda.bind("<Return>", lambda event: self.buscar_mods())

        btn_buscar = ctk.CTkButton(frame_busqueda, text="🔍 Buscar", width=80, command=self.buscar_mods)
        btn_buscar.pack(side="left")

        # --- CONTENEDOR DE RESULTADOS ---
        self.scroll_resultados = ctk.CTkScrollableFrame(vent_tienda, width=550, height=450)
        self.scroll_resultados.pack(pady=10, padx=20, fill="both", expand=True)
        
        ctk.CTkLabel(self.scroll_resultados, text="Elegí una categoría y buscá para empezar.\nEl sistema filtrará automáticamente por tu versión del menú principal.", text_color="gray").pack(pady=50)

    def buscar_mods(self):
        query = self.entry_busqueda.get().strip()
        if not query: return
        
        for widget in self.scroll_resultados.winfo_children(): widget.destroy()
        ctk.CTkLabel(self.scroll_resultados, text="Conectando con la base de datos de Modrinth...", font=ctk.CTkFont(slant="italic")).pack(pady=20)
        
        threading.Thread(target=self._hilo_buscar_modrinth, args=(query,), daemon=True).start()

    def _hilo_buscar_modrinth(self, query):
        seleccion = self.lista_versiones.curselection()
        if seleccion: version_actual = self.lista_versiones.get(seleccion[0])
        else: version_actual = "1.21.1" 
            
        tipo = self.tipo_cliente_var.get().lower().replace(" (mods clásicos)", "").replace(" (optimizado/recomendado)", "").replace(" (juego base)", "").strip()
        if tipo == "vanilla": tipo = "fabric" 
        
        categoria_elegida = self.tipo_busqueda_var.get()
        tipo_proyecto = "mod"
        if categoria_elegida == "Shaders": tipo_proyecto = "shader"
        elif categoria_elegida == "Texturas": tipo_proyecto = "resourcepack"
        elif categoria_elegida == "Modpacks": tipo_proyecto = "modpack"

        # DELEGAMOS LA BÚSQUEDA AL MODELO
        exito, resultado = TiendaAPI.buscar_mods(query, version_actual, tipo, tipo_proyecto)
        
        # ACTUAMOS SEGÚN LA RESPUESTA
        if exito:
            self.after(0, self.mostrar_resultados_tienda, resultado, tipo_proyecto)
        else:
            self.after(0, self.mostrar_error_tienda, resultado)

    def mostrar_error_tienda(self, mensaje):
        for widget in self.scroll_resultados.winfo_children(): widget.destroy()
        ctk.CTkLabel(self.scroll_resultados, text=f"🔴 {mensaje}", text_color="#e74c3c").pack(pady=20)

    def mostrar_resultados_tienda(self, hits, tipo_proyecto):
        for widget in self.scroll_resultados.winfo_children(): widget.destroy()
        
        if not hits:
            ctk.CTkLabel(self.scroll_resultados, text="No se encontró contenido compatible con esta versión.").pack(pady=20)
            return
            
        for mod in hits:
            frame_mod = ctk.CTkFrame(self.scroll_resultados, fg_color=("#e0e0e0", "#2b2b2b"), corner_radius=10)
            frame_mod.pack(fill="x", pady=5, padx=5)
            
            titulo = mod.get("title", "Desconocido")
            autor = mod.get("author", "Autor Desconocido")
            desc = mod.get("description", "")[:90] + "..." 
            slug = mod.get("slug")
            
            btn_instalar = ctk.CTkButton(frame_mod, text="⬇️ Instalar", width=80, fg_color="#d35400", hover_color="#a04000",
                                         command=lambda s=slug, t=titulo, tp=tipo_proyecto: self.instalar_contenido_tienda(s, t, tp))
            btn_instalar.pack(side="right", padx=15, pady=10)
            
            frame_info = ctk.CTkFrame(frame_mod, fg_color="transparent")
            frame_info.pack(side="left", fill="both", expand=True, padx=10, pady=10)
            
            ctk.CTkLabel(frame_info, text=titulo, font=ctk.CTkFont(weight="bold", size=15)).pack(anchor="w")
            ctk.CTkLabel(frame_info, text=f"Por {autor}", font=ctk.CTkFont(size=11, slant="italic"), text_color="gray").pack(anchor="w")
            ctk.CTkLabel(frame_info, text=desc, font=ctk.CTkFont(size=12), justify="left").pack(anchor="w", pady=(2,0))

    def instalar_contenido_tienda(self, slug, titulo, tipo_proyecto):
        seleccion = self.lista_versiones.curselection()
        if seleccion: version_actual = self.lista_versiones.get(seleccion[0])
        else: version_actual = "1.21.1" 
        
        tipo_cliente_ui = self.tipo_cliente_var.get().split()[0] # Saca "Vanilla", "Fabric" o "Forge"
        
        # 2. Reconstruye la ruta exacta del aislamiento de instancias (Igual que en core.py)
        folder_name = f"Paraguacraft_{version_actual}_{tipo_cliente_ui}".replace(".", "_")
        mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
        instancia_dir = os.path.join(mine_dir, "instancias", folder_name)
        
        # 3. Direccionador de carpetas inteligente
        if tipo_proyecto == "mod": carpeta_destino = os.path.join(instancia_dir, "mods")
        elif tipo_proyecto == "shader": carpeta_destino = os.path.join(instancia_dir, "shaderpacks")
        elif tipo_proyecto == "resourcepack": carpeta_destino = os.path.join(instancia_dir, "resourcepacks")
        else:
            messagebox.showwarning("Aviso", "La instalación automática de Modpacks completos llegará en una próxima actualización.")
            return

        os.makedirs(carpeta_destino, exist_ok=True)

        # 4. Lanza el hilo de descarga para no congelar el launcher
        tipo_loader = tipo_cliente_ui.lower()
        threading.Thread(target=self._hilo_descarga_tienda, args=(slug, titulo, version_actual, tipo_loader, carpeta_destino), daemon=True).start()

    def _hilo_descarga_tienda(self, slug, titulo, version, tipo_loader, carpeta_destino):
        self.actualizar_progreso(f"Buscando archivo de {titulo}...")
        
        # DELEGAMOS LA DESCARGA AL MODELO
        exito, mensaje = TiendaAPI.descargar_mod(slug, version, tipo_loader, carpeta_destino, self.actualizar_progreso)
        
        # ACTUAMOS SEGÚN LA RESPUESTA (Solo mostramos cartelitos)
        if exito:
            self.actualizar_progreso("¡Instalación exitosa!")
            self.after(0, lambda: messagebox.showinfo("Tienda Paraguacraft", f"¡'{titulo}' se instaló correctamente!\n\nSe guardó en tu perfil de {version} ({tipo_loader.capitalize()})."))
        else:
            self.actualizar_progreso("Error al instalar.")
            if mensaje == "incompatible":
                self.after(0, lambda: messagebox.showerror("Error de Compatibilidad", f"El creador de '{titulo}' no subió una versión compatible con Minecraft {version} en {tipo_loader.capitalize()}."))
            else:
                self.after(0, lambda: messagebox.showerror("Error de Conexión", f"No se pudo descargar el archivo: {mensaje}"))

    def _hilo_discord_rpc_dinamico(self, version_jugada, usuario, tipo_cliente):
        if not self.rpc: return
        import time, os, minecraft_launcher_lib, re

        try:
            # 1. Ubicación de la instancia
            version_base = version_jugada.strip()
            if version_base.startswith("fabric-loader-"):
                version_base = version_base.split("-")[-1]
                tipo_cliente = "Fabric"
            folder_name = f"Paraguacraft_{version_base}_{tipo_cliente}".replace(".", "_")
            log_path = os.path.join(minecraft_launcher_lib.utils.get_minecraft_directory(), "instancias", folder_name, "logs", "latest.log")
            
            try:
                if os.path.exists(log_path): os.remove(log_path)
            except: pass
                
            estado_actual = "En el menú principal"
            self.rpc.update(state=estado_actual, details=f"👤 {usuario} | 🎮 {version_jugada}", large_image="logo")

            while not os.path.exists(log_path) and self.hilo_juego_activo: time.sleep(1)
            
            with open(log_path, 'r', encoding='utf-8', errors='ignore') as f:
                while self.hilo_juego_activo:
                    linea = f.readline()
                    if not linea:
                        time.sleep(0.5)
                        f.seek(f.tell()) 
                        continue

                    # --- DETECCIÓN DE ESTADOS ---
                    if "Local game hosted on" in linea:
                        match = re.search(r'\[(.*?)\]', linea)
                        if match: estado_actual = f"🏠 Hosteando LAN: {match.group(1)}"
                    elif "Connecting to" in linea:
                        ip = linea.split("Connecting to ")[1].split(",")[0].split(":")[0].strip()
                        estado_actual = f"🌐 Multijugador: {ip}"
                    elif "Starting integrated minecraft server" in linea:
                        if "Hosteando" not in estado_actual: estado_actual = "🌍 Jugando en Mundo Local"
                    elif "Disconnecting from" in linea or "Stopping server" in linea:
                        estado_actual = "En el menú principal"

                    self.rpc.update(state=estado_actual, details=f"👤 {usuario} | 🎮 {version_jugada}", large_image="logo")

        except Exception as e: 
            print(f"Error RPC: {e}")
            self.rpc = None

    def procesar_archivos_soltados(self, archivos):
        try:
            # 1. Mira qué versión tiene elegida el usuario en la lista
            seleccion = self.lista_versiones.curselection()
            if not seleccion:
                messagebox.showwarning("Aviso", "¡Elegí una versión en la lista primero para saber dónde instalar los mods/shaders!")
                return
            
            version_actual = self.lista_versiones.get(seleccion[0])
            tipo_cliente_ui = self.tipo_cliente_var.get().split()[0]
            
            # 2. Arma la ruta a la caja fuerte de esa versión (Instancia Aislada)
            folder_name = f"Paraguacraft_{version_actual}_{tipo_cliente_ui}".replace(".", "_")
            mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            instancia_dir = os.path.join(mine_dir, "instancias", folder_name)
            
            archivos_procesados = []
            
            # 3. Analiza archivo por archivo de los que tiró el usuario
            for archivo_bytes in archivos:
                # windnd devuelve la ruta en bytes, la pasa a texto normal
                ruta_archivo = archivo_bytes.decode('gbk') 
                nombre_archivo = os.path.basename(ruta_archivo)
                carpeta_destino = ""
                
                # REGLA A: Si es un .jar, es un Mod. Va derecho a la carpeta mods.
                if ruta_archivo.endswith('.jar'):
                    carpeta_destino = os.path.join(instancia_dir, "mods")
                    
                # REGLA B: Si es un .zip
                elif ruta_archivo.endswith('.zip'):
                    with zipfile.ZipFile(ruta_archivo, 'r') as z:
                        archivos_zip = z.namelist()
                        
                        # Si tiene una carpeta "shaders", es un Shaderpack
                        if any(f.startswith('shaders/') for f in archivos_zip):
                            carpeta_destino = os.path.join(instancia_dir, "shaderpacks")
                            
                        # Si tiene el archivo de descripción oficial, es un Resourcepack (Texturas)
                        elif 'pack.mcmeta' in archivos_zip:
                            carpeta_destino = os.path.join(instancia_dir, "resourcepacks")
                            
                        # Si es un zip pero no tiene nada de eso, a veces los creadores empaquetan mods ahí
                        else:
                            carpeta_destino = os.path.join(instancia_dir, "mods")
                
                # 4. Si se sabe dónde va, lo copia
                if carpeta_destino:
                    os.makedirs(carpeta_destino, exist_ok=True)
                    destino_final = os.path.join(carpeta_destino, nombre_archivo)
                    shutil.copyfile(ruta_archivo, destino_final)
                    archivos_procesados.append(nombre_archivo)
            
            if archivos_procesados:
                lista_nombres = "\n".join(archivos_procesados)
                messagebox.showinfo("Instalación Mágica ✨", f"Se instalaron correctamente en tu perfil de {version_actual}:\n\n{lista_nombres}")
                
        except Exception as e:
            messagebox.showerror("Error al procesar", f"Hubo un problema instalando el archivo:\n{e}")

    def crear_servidor_local(self):
        carpeta_server = filedialog.askdirectory(title="Elegí una carpeta VACÍA para tu servidor")
        if not carpeta_server: return
        
        seleccion = self.lista_versiones.curselection()
        sugerencia = self.lista_versiones.get(seleccion[0]) if seleccion else "1.21.1"
        if "fabric" in sugerencia: sugerencia = sugerencia.split("-")[-1]
        
        dialog = ctk.CTkInputDialog(text="Ingresá la versión EXACTA (ej: 1.21.1):", title="Versión del Server")
        ver_server = dialog.get_input()
        if not ver_server: return

        def _hilo_server():
            # Le pasamos la tarea pesada al Modelo
            exito, mensaje = CreadorServidor.descargar_y_preparar(carpeta_server, ver_server, self.actualizar_progreso)
            
            # El Modelo responde, la Vista muestra los pop-ups
            if exito:
                self.after(0, lambda: messagebox.showinfo("Servidor Creado Mágicamente ✨", mensaje))
            else:
                self.actualizar_progreso("Error al crear servidor.")
                self.after(0, lambda: messagebox.showerror("Error", mensaje))

        threading.Thread(target=_hilo_server, daemon=True).start()

    def actualizar_avatar_visual(self):
        usuario = self.combo_usuario.get().strip()
        mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
        
        try:
            if self.ms_data: # ES PREMIUM
                url = f"https://mc-heads.net/avatar/{usuario}/64"
                img_data = requests.get(url).content
                img = Image.open(io.BytesIO(img_data))
            else: # ES OFFLINE
                path_skin = os.path.join(mine_dir, "resourcepacks", "ParaguacraftBrandPack", "assets", "minecraft", "textures", "entity", "player", "steve.png")
                if os.path.exists(path_skin):
                    img_full = Image.open(path_skin)
                    # El área de la cara en una skin de MC es (8, 8, 16, 16)
                    img = img_full.crop((8, 8, 16, 16)).resize((64, 64), Image.Resampling.NEAREST)
                else:
                    return # Si no hay skin, no muestra nada

            ctk_img = ctk.CTkImage(light_image=img, dark_image=img, size=(64, 64))
            self.lbl_avatar.configure(image=ctk_img)
        except: pass

    def cambiar_skin_premium_api(self):
        if not self.ms_data:
            messagebox.showwarning("Premium Requerido", "Esta función es solo para cuentas Premium conectadas.")
            return
            
        ruta = filedialog.askopenfilename(title="Elegí tu nueva Skin (.png)", filetypes=[("PNG", "*.png")])
        if not ruta: return
        
        token = self.ms_data["access_token"]
        url = "https://api.minecraftservices.com/minecraft/profile/skins"
        headers = {"Authorization": f"Bearer {token}"}
        # 'variant': 'classic' para skin normal, 'slim' para Alex
        payload = {'variant': 'classic'}
        files = [('file', ('skin.png', open(ruta, 'rb'), 'image/png'))]
        
        try:
            r = requests.post(url, headers=headers, data=payload, files=files)
            if r.status_code == 200:
                messagebox.showinfo("Éxito", "¡Skin Premium actualizada en los servidores de Mojang!")
                self.actualizar_avatar_visual()
            else:
                messagebox.showerror("Error", f"No se pudo cambiar: {r.status_code}")
        except Exception as e:
            messagebox.showerror("Error", str(e))

    def iniciar_servidor_skins_local(self):
        class SkinHandler(SimpleHTTPRequestHandler):
            def do_GET(self):
                mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
                path_skin = os.path.join(mine_dir, "resourcepacks", "ParaguacraftBrandPack", "assets", "minecraft", "textures", "entity", "player", "steve.png")
                if self.path == "/skin.png":
                    self.send_response(200)
                    self.send_header("Content-type", "image/png")
                    self.end_headers()
                    with open(path_skin, 'rb') as f: self.wfile.write(f.read())
                else:
                    self.send_error(404)

        def _run_server():
            try:
                server = HTTPServer(('0.0.0.0', 8080), SkinHandler)
                server.serve_forever()
            except: pass

        threading.Thread(target=_run_server, daemon=True).start()

if __name__ == "__main__":
    app = ParaguaCraftLauncher()
    app.mainloop()