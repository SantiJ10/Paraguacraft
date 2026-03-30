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
if getattr(sys, 'frozen', False):
    directorio_actual = os.path.dirname(sys.executable)
    viejo_exe = os.path.join(directorio_actual, "Paraguacraft_Viejo.exe")
    if os.path.exists(viejo_exe):
        try:
            os.remove(viejo_exe)
        except:
            pass

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

LAUNCHER_VERSION = "1.0.2"
UPDATE_URL = "https://raw.githubusercontent.com/SantiJ10/Paraguacraft/refs/heads/main/version.txt"

class ParaguaCraftLauncher(ctk.CTk):
    def __init__(self):
        super().__init__()
        self.title(f"ParaguaCraft Launcher v{LAUNCHER_VERSION}")
        self.centrar_ventana(self, 600, 780)
        self.resizable(False, False)

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

        self.frame_lista = ctk.CTkFrame(self.frame_main, border_width=2, border_color="#3a3a3a")
        self.frame_lista.pack(pady=5, fill="x", padx=50)
        
        self.scroll_versiones = ctk.CTkScrollbar(self.frame_lista)
        self.scroll_versiones.pack(side="right", fill="y", padx=(0, 2), pady=2)
        
        self.lista_versiones = tk.Listbox(
            self.frame_lista, yscrollcommand=self.scroll_versiones.set, 
            bg="#242424", fg="#e0e0e0", font=("Consolas", 11), 
            selectbackground="#1f538d", borderwidth=0, highlightthickness=0, height=8
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

        # Activamos el sensor magnético para arrastrar y soltar
        try:
            windnd.hook_dropurls(self.winfo_id(), self.procesar_archivos_soltados)
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
                # Intento 1: Por internet (Para cuando lo usan tus amigos)
                server = JavaServer.lookup(SERVER_IP)
                status = server.status()
            except Exception:
                try:
                    # Intento 2: Red local / Bucle interno (Para cuando lo usás vos en tu PC)
                    server_local = JavaServer.lookup("127.0.0.1:25565")
                    status = server_local.status()
                except Exception:
                    # Si los dos fallan, ahí sí está apagado posta
                    self.after(0, lambda: self.lbl_server.configure(text="🔴 El Server está OFFLINE", text_color="#e74c3c"))
                    return

            # Si cualquiera de los dos intentos funcionó, lo pintamos de verde
            texto_online = f"🟢 ONLINE ({status.players.online}/{status.players.max} Jugadores)"
            self.after(0, lambda: self.lbl_server.configure(text=texto_online, text_color="#2ecc71"))
        
        threading.Thread(target=hacer_ping, daemon=True).start()

    # 🚀 MOTOR DE AUTO-UPDATER 
    # 🚀 MOTOR DE AUTO-UPDATER DINÁMICO
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
            f"¡Salió la versión {version_remota} de Paraguacraft!\n\n¿Querés actualizarlo ahora automáticamente? (Tarda unos segundos)"
        )
        if respuesta:
            # Le pasamos el número de versión nuevo a la función de descarga
            threading.Thread(target=self.ejecutar_actualizacion, args=(version_remota,), daemon=True).start()

    def ejecutar_actualizacion(self, version_remota):
        try:
            self.actualizar_progreso("Descargando actualización (Esto puede tardar)...")
            self.boton_jugar.configure(state="disabled", text="ACTUALIZANDO...")
            
            # MAGIA INGENIERIL: Armamos el link dinámico con la versión que leyó del texto
            url_descarga_dinamica = f"https://github.com/SantiJ10/Paraguacraft/releases/download/v.{version_remota}/Paraguacraft.exe"
            
            r = requests.get(url_descarga_dinamica, stream=True, timeout=30)
            r.raise_for_status() # Esto corta si hay un error 404
            
            exe_actual = sys.executable 
            directorio = os.path.dirname(exe_actual)
            nombre_exe = os.path.basename(exe_actual)
            nuevo_exe = os.path.join(directorio, "Paraguacraft_Nuevo.exe")
            
            with open(nuevo_exe, "wb") as f:
                for chunk in r.iter_content(chunk_size=8192):
                    f.write(chunk)
                    
            self.actualizar_progreso("Aplicando actualización...")
            bat_path = os.path.join(directorio, "update_paraguacraft.bat")
            
            bat_content = f"""@echo off
timeout /t 2 /nobreak > NUL
:bucle
del /f /q "{nombre_exe}"
if exist "{nombre_exe}" (
    timeout /t 1 /nobreak > NUL
    goto bucle
)
ren "Paraguacraft_Nuevo.exe" "{nombre_exe}"
del "%~f0"
"""
            with open(bat_path, "w") as f:
                f.write(bat_content)
                
            subprocess.Popen([bat_path], creationflags=subprocess.CREATE_NO_WINDOW if platform.system() == "Windows" else 0)
            
            messagebox.showinfo("¡Actualización Lista!", "La actualización se descargó correctamente.\n\nEl launcher se cerrará ahora para instalarla. Por favor, vuelve a abrirlo en unos segundos para jugar.")
            os._exit(0)
            
        except Exception as e:
            self.actualizar_progreso("Error al actualizar.")
            self.after(0, lambda: messagebox.showerror("Error", f"No se pudo actualizar: {str(e)}"))
            self.after(0, lambda: self.boton_jugar.configure(state="normal", text="INICIAR PARAGUACRAFT"))

    def iniciar_discord_rpc(self):
        try:
            from pypresence import Presence
            self.rpc = Presence(DISCORD_APP_ID)
            self.rpc.connect()
            self.rpc.update(state="En el menú principal", details="Preparándose para jugar", large_image="logo", large_text="Paraguacraft Launcher")
        except: pass

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
                    
                    # --- CARGAR EL TEMA VISUAL ---
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
            # 1. Cargamos las que ya tiene instaladas la PC (Acá van a estar tus Fabric locales)
            versiones_locales = minecraft_launcher_lib.utils.get_installed_versions(mine_dir)
            lista_locales = [v["id"] for v in versiones_locales]
            
            # 2. Le pedimos a Mojang la lista oficial
            versiones_oficiales = minecraft_launcher_lib.utils.get_version_list()
            
            # FILTRO INGENIERO: Solo "releases" y filtramos la mugre
            lista_oficiales = []
            palabras_prohibidas = ["rc", "pre", "snapshot", "alpha", "beta"]
            
            for v in versiones_oficiales:
                if v["type"] == "release":
                    version_limpia = v["id"].lower()
                    # Si no contiene ninguna palabra prohibida, la aceptamos
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
        # Hacemos la ventana un pelín más ancha para que respiren los textos
        self.centrar_ventana(vent, 480, 780) 
        vent.grab_set()

        if getattr(sys, 'frozen', False):
            icono_ajustes = os.path.join(sys._MEIPASS, "iconomc.ico")
        else:
            icono_ajustes = "iconomc.ico"
        if os.path.exists(icono_ajustes):
            try: vent.iconbitmap(icono_ajustes)
            except: pass

        # Contenedor principal con barra de scroll para que no se apriete nada
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
        
        # El anchor="w" hace que todos queden pegados a la izquierda como soldados
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
        
        ctk.CTkLabel(frame_skins, text="Gestor de Skins (Offline)", font=ctk.CTkFont(weight="bold")).pack(pady=(10, 5))
        
        frame_skin_buttons = ctk.CTkFrame(frame_skins, fg_color="transparent")
        frame_skin_buttons.pack(fill="x", pady=(0, 10), padx=10)
        
        ctk.CTkButton(frame_skin_buttons, text="🖼️ Elegir Skin", command=self.elegir_skin, fg_color="#8e44ad", hover_color="#5e3370").pack(side="left", padx=5, expand=True, fill="x")
        ctk.CTkButton(frame_skin_buttons, text="❌ Borrar", command=self.borrar_skin, fg_color="#c0392b", hover_color="#922b21", width=80).pack(side="right", padx=5)

        # --- 6. BOTONES FINALES ---
        frame_botones_finales = ctk.CTkFrame(scroll_main, fg_color="transparent")
        frame_botones_finales.pack(fill="x", padx=10, pady=15)

        self.btn_tienda = ctk.CTkButton(frame_botones_finales, text="🛒 Tienda de Mods", command=self.abrir_tienda, fg_color="#27ae60", hover_color="#2ecc71", font=ctk.CTkFont(weight="bold"), height=40)
        self.btn_tienda.pack(side="left", expand=True, fill="x", padx=(0, 5))

        ctk.CTkButton(frame_botones_finales, text="🚑 Reparación (SOS)", command=self.ejecutar_reparacion_sos, fg_color="#c0392b", hover_color="#922b21", height=40).pack(side="right", expand=True, fill="x", padx=(5, 0))

    def elegir_skin(self):
        ruta = filedialog.askopenfilename(title="Skin (.png)", filetypes=[("PNG", "*.png")])
        if ruta:
            mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            skin_path = os.path.join(mine_dir, "resourcepacks", "ParaguacraftBrandPack", "assets", "minecraft", "textures", "entity", "player")
            os.makedirs(skin_path, exist_ok=True)
            try:
                shutil.copy(ruta, os.path.join(skin_path, "steve.png"))
                shutil.copy(ruta, os.path.join(skin_path, "alex.png"))
                messagebox.showinfo("Skin Guardada", "¡Skin aplicada!")
            except: messagebox.showerror("Error", "No se pudo copiar la skin.")

    def borrar_skin(self):
        mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
        skin_path = os.path.join(mine_dir, "resourcepacks", "ParaguacraftBrandPack", "assets", "minecraft", "textures", "entity", "player")
        try:
            if os.path.exists(os.path.join(skin_path, "steve.png")): os.remove(os.path.join(skin_path, "steve.png"))
            if os.path.exists(os.path.join(skin_path, "alex.png")): os.remove(os.path.join(skin_path, "alex.png"))
            messagebox.showinfo("Skin Borrada", "Skin restablecida a la de defecto (Steve).")
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

    # EL CAMBIO EN LA FIRMA: Ahora recibe tipo_cliente
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

            self.after(0, lambda: self.lbl_estado.configure(text="Lanzando Paraguacraft..."))
            
            threading.Thread(target=self._hilo_ninja_renombrar, args=(version_jugada,), daemon=True).start()

            start_time = datetime.now()
            
            # EL CAMBIO EN LA LLAMADA AL CORE: Le enviamos el tipo_cliente a core.py
# EL CAMBIO EN LA LLAMADA AL CORE
            lanzar_minecraft(version_jugada, usuario, ram, gc_type, self.opt_var.get(), tipo_cliente, self.papa_var.get(), self.mesa_var.get(), self.consola_var.get(), self.actualizar_progreso, uuid_real, token_real, self.lan_distancia_var.get())
            self.after(0, lambda: self.lbl_estado.configure(text="Lanzando Paraguacraft..."))
            
            threading.Thread(target=self._hilo_ninja_renombrar, args=(version_jugada,), daemon=True).start()
            
            # --- NUEVO: ACTIVAMOS EL ESPÍA DE DISCORD ---
            threading.Thread(target=self._hilo_discord_rpc_dinamico, args=(version_jugada, usuario, tipo_cliente), daemon=True).start()

            start_time = datetime.now()
            
            # EL CAMBIO EN LA LLAMADA AL CORE: Le enviamos el tipo_cliente a core.py
            lanzar_minecraft(version_jugada, usuario, ram, gc_type, self.opt_var.get(), tipo_cliente, self.papa_var.get(), self.mesa_var.get(), self.consola_var.get(), self.actualizar_progreso, uuid_real, token_real)
            
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
        ram_total_gb = max(1, int(psutil.virtual_memory().total / (1024 ** 3)))
        if ram_total_gb <= 8:
            self.combo_ram.set("3GB")
            self.opt_var.set(True) 
            self.tipo_cliente_var.set("Fabric") # <--- Corregido acá
        elif ram_total_gb <= 16:
            self.combo_ram.set("6GB")
            self.opt_var.set(False) 
        else:
            self.combo_ram.set("8GB") 
            self.opt_var.set(False) 
        
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

    # =======================================================
    # 🛒 SPRINT 3: TIENDA DE MODS MULTIPROPÓSITO (MODRINTH)
    # =======================================================
    
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

        # Armamos los filtros de la API dinámicamente
        facets = f'[["versions:{version_actual}"],["project_type:{tipo_proyecto}"]'
        # Shaders y Texturas no dependen de Fabric/Forge en Modrinth, así que solo filtramos motor si es un Mod o Modpack
        if tipo_proyecto in ["mod", "modpack"]:
            facets += f',["categories:{tipo}"]'
        facets += ']'
        
        url = f'https://api.modrinth.com/v2/search?query={query}&limit=8&facets={facets}'
        
        try:
            r = requests.get(url, headers={"User-Agent": "ParaguacraftLauncher/1.0"}, timeout=10)
            if r.status_code == 200:
                data = r.json()
                self.after(0, self.mostrar_resultados_tienda, data.get("hits", []), tipo_proyecto)
            else:
                self.after(0, self.mostrar_error_tienda, f"Error de la API: {r.status_code}")
        except Exception:
            self.after(0, self.mostrar_error_tienda, "No hay conexión a internet o la API está caída.")

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
            
            # FIX DEL BUG VISUAL: Empaquetamos el botón PRIMERO a la derecha
            btn_instalar = ctk.CTkButton(frame_mod, text="⬇️ Instalar", width=80, fg_color="#d35400", hover_color="#a04000",
                                         command=lambda s=slug, t=titulo, tp=tipo_proyecto: self.instalar_contenido_tienda(s, t, tp))
            btn_instalar.pack(side="right", padx=15, pady=10)
            
            # Y DESPUÉS el texto, para que no lo empuje
            frame_info = ctk.CTkFrame(frame_mod, fg_color="transparent")
            frame_info.pack(side="left", fill="both", expand=True, padx=10, pady=10)
            
            ctk.CTkLabel(frame_info, text=titulo, font=ctk.CTkFont(weight="bold", size=15)).pack(anchor="w")
            ctk.CTkLabel(frame_info, text=f"Por {autor}", font=ctk.CTkFont(size=11, slant="italic"), text_color="gray").pack(anchor="w")
            ctk.CTkLabel(frame_info, text=desc, font=ctk.CTkFont(size=12), justify="left").pack(anchor="w", pady=(2,0))

    def instalar_contenido_tienda(self, slug, titulo, tipo_proyecto):
        # 1. Obtenemos versión y cliente actuales de la interfaz principal
        seleccion = self.lista_versiones.curselection()
        if seleccion: version_actual = self.lista_versiones.get(seleccion[0])
        else: version_actual = "1.21.1" 
        
        tipo_cliente_ui = self.tipo_cliente_var.get().split()[0] # Saca "Vanilla", "Fabric" o "Forge"
        
        # 2. Reconstruimos la ruta exacta del aislamiento de instancias (Igual que en core.py)
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

        # 4. Lanzamos el hilo de descarga para no congelar el launcher
        tipo_loader = tipo_cliente_ui.lower()
        threading.Thread(target=self._hilo_descarga_tienda, args=(slug, titulo, version_actual, tipo_loader, carpeta_destino), daemon=True).start()

    def _hilo_descarga_tienda(self, slug, titulo, version, tipo_loader, carpeta_destino):
        self.actualizar_progreso(f"Buscando archivo de {titulo}...")
        
        # Le pedimos a Modrinth la versión exacta del archivo
        url_api = f"https://api.modrinth.com/v2/project/{slug}/version"
        params = {"game_versions": f'["{version}"]'}
        
        # Si va a la carpeta mods, le exigimos a la API que sea compatible con nuestro motor
        if "mods" in carpeta_destino:
            if tipo_loader == "vanilla": tipo_loader = "fabric" # Fallback de seguridad
            params["loaders"] = f'["{tipo_loader}"]'
            
        try:
            r = requests.get(url_api, params=params, headers={"User-Agent": "ParaguacraftLauncher/1.0"}, timeout=10)
            if r.status_code == 200:
                data = r.json()
                if len(data) > 0:
                    # Agarramos el archivo más reciente (el primero de la lista)
                    archivo = data[0]["files"][0]
                    nombre_archivo = archivo["filename"]
                    url_descarga = archivo["url"]
                    
                    ruta_final = os.path.join(carpeta_destino, nombre_archivo)
                    
                    self.actualizar_progreso(f"Descargando {titulo}...")
                    r_dl = requests.get(url_descarga, stream=True, timeout=20)
                    with open(ruta_final, "wb") as f:
                        shutil.copyfileobj(r_dl.raw, f)
                        
                    self.actualizar_progreso("¡Instalación exitosa!")
                    self.after(0, lambda: messagebox.showinfo("Tienda Paraguacraft", f"¡'{titulo}' se instaló correctamente!\n\nSe guardó en tu perfil de {version} ({tipo_loader.capitalize()})."))
                else:
                    self.actualizar_progreso("Versión no compatible.")
                    self.after(0, lambda: messagebox.showerror("Error de Compatibilidad", f"El creador de '{titulo}' no subió una versión compatible con Minecraft {version} en {tipo_loader.capitalize()}."))
            else:
                self.actualizar_progreso("Error en la API.")
        except Exception as e:
            self.actualizar_progreso("Error de red.")
            self.after(0, lambda: messagebox.showerror("Error de Conexión", f"No se pudo descargar el archivo: {e}"))

    def _hilo_discord_rpc_dinamico(self, version_jugada, usuario, tipo_cliente):
        if not self.rpc: return
        import time, os, minecraft_launcher_lib

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

        try:
            while not os.path.exists(log_path) and self.hilo_juego_activo: time.sleep(1)
            
            with open(log_path, 'r', encoding='utf-8', errors='ignore') as f:
                while self.hilo_juego_activo:
                    linea = f.readline()
                    if not linea:
                        time.sleep(0.5)
                        f.seek(f.tell()) # Actualiza el puntero para ver cambios en vivo
                        continue

                    # --- DETECCIÓN DE ESTADOS ---
                    # A. Si hosteás LAN con e4mc
                    if "Local game hosted on domain" in linea:
                        try:
                            ip_lan = linea.split("[")[1].split("]")[0]
                            estado_actual = f"🏠 Hosteando LAN: {ip_lan}"
                        except: estado_actual = "🏠 Hosteando Mundo LAN"
                        
                    # B. Si te conectás a un server (Hypixel, etc)
                    elif "Connecting to" in linea:
                        ip = linea.split("Connecting to ")[1].split(",")[0].split(":")[0].strip()
                        estado_actual = f"🌐 Multijugador: {ip}"
                            
                    # C. Si entrás a un mundo local normal
                    elif "Starting integrated minecraft server" in linea:
                        if "Hosteando" not in estado_actual: # No pisar el estado LAN si ya lo detectó
                            estado_actual = "🌍 Jugando en Mundo Local"
                        
                    # D. Si volvés al menú
                    elif "Disconnecting from" in linea or "Stopping server" in linea:
                        estado_actual = "En el menú principal"

                    self.rpc.update(state=estado_actual, details=f"👤 {usuario} | 🎮 {version_jugada}", large_image="logo")

        except Exception as e: print(f"Error RPC: {e}")

    def procesar_archivos_soltados(self, archivos):
        try:
            # 1. Miramos qué versión tiene elegida el usuario en la lista
            seleccion = self.lista_versiones.curselection()
            if not seleccion:
                messagebox.showwarning("Aviso", "¡Elegí una versión en la lista primero para saber dónde instalar los mods/shaders!")
                return
            
            version_actual = self.lista_versiones.get(seleccion[0])
            tipo_cliente_ui = self.tipo_cliente_var.get().split()[0]
            
            # 2. Armamos la ruta a la caja fuerte de esa versión (Instancia Aislada)
            folder_name = f"Paraguacraft_{version_actual}_{tipo_cliente_ui}".replace(".", "_")
            mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            instancia_dir = os.path.join(mine_dir, "instancias", folder_name)
            
            archivos_procesados = []
            
            # 3. Analizamos archivo por archivo de los que tiró el usuario
            for archivo_bytes in archivos:
                # windnd nos devuelve la ruta en bytes, la pasamos a texto normal
                ruta_archivo = archivo_bytes.decode('gbk') 
                nombre_archivo = os.path.basename(ruta_archivo)
                carpeta_destino = ""
                
                # REGLA A: Si es un .jar, es un Mod. Va derecho a la carpeta mods.
                if ruta_archivo.endswith('.jar'):
                    carpeta_destino = os.path.join(instancia_dir, "mods")
                    
                # REGLA B: Si es un .zip, somos ingenieros y miramos qué tiene adentro
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
                
                # 4. Si sabemos dónde va, lo copiamos
                if carpeta_destino:
                    os.makedirs(carpeta_destino, exist_ok=True)
                    destino_final = os.path.join(carpeta_destino, nombre_archivo)
                    shutil.copyfile(ruta_archivo, destino_final)
                    archivos_procesados.append(nombre_archivo)
            
            # 5. Le avisamos que salió todo bien
            if archivos_procesados:
                lista_nombres = "\n".join(archivos_procesados)
                messagebox.showinfo("Instalación Mágica ✨", f"Se instalaron correctamente en tu perfil de {version_actual}:\n\n{lista_nombres}")
                
        except Exception as e:
            messagebox.showerror("Error al procesar", f"Hubo un problema instalando el archivo:\n{e}")

if __name__ == "__main__":
    app = ParaguaCraftLauncher()
    app.mainloop()