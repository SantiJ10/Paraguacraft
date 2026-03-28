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

APPDATA_DIR = os.path.join(os.getenv('APPDATA'), "ParaguacraftLauncher")
os.makedirs(APPDATA_DIR, exist_ok=True)
CONFIG_FILE = os.path.join(APPDATA_DIR, "paraguacraft_config.json")
SESSION_FILE = os.path.join(APPDATA_DIR, "paraguacraft_session.json")
LOG_DIR = os.path.join(minecraft_launcher_lib.utils.get_minecraft_directory(), "logs")

# === CONFIGURACIÓN DE PARAGUACRAFT (DATOS BLINDADOS) ===
CLIENT_ID = "72fb7c48-c2f5-4d13-b0e7-9835b3b906c0" 
REDIRECT_URI = "https://login.live.com/oauth20_desktop.srf"
DISCORD_APP_ID = "1487516329631154206" 
SERVER_IP = "process-import.gl.at.ply.gg:2055" 
MODS_ZIP_URL = "" 
LAUNCHER_VERSION = "1.0.0"
UPDATE_URL = "" 
# =====================================================

class ParaguaCraftLauncher(ctk.CTk):
    def __init__(self):
        super().__init__()
        self.title(f"ParaguaCraft Launcher v{LAUNCHER_VERSION}")
        self.geometry("600x780") 
        self.resizable(False, False)

        self.limpiador_deep_var = ctk.BooleanVar(value=False)
        self.backup_var = ctk.BooleanVar(value=True) 
        
        self.veces_iniciadas = ctk.IntVar(value=0)
        self.horas_jugadas_total = ctk.DoubleVar(value=0.0)

        self.gc_var = ctk.StringVar(value="G1GC (Equilibrado / Recomendado)")
        self.opt_var = ctk.BooleanVar(value=False)
        self.optimode_var = ctk.BooleanVar(value=True)
        self.papa_var = ctk.BooleanVar(value=False)
        self.mesa_var = ctk.BooleanVar(value=False)
        self.consola_var = ctk.BooleanVar(value=False)
        
        self.ms_data = None
        self.rpc = None
        self.usuarios_guardados = []
        self.hilo_juego_activo = False # Para apagar el Terminator cuando cerres el juego

        self.frame_main = ctk.CTkFrame(self)
        self.frame_main.pack(pady=15, padx=20, fill="both", expand=True)

        self.label_titulo = ctk.CTkLabel(self.frame_main, text="ParaguaCraft Launcher", font=ctk.CTkFont(size=28, weight="bold"))
        self.label_titulo.pack(pady=(20, 5))

        self.lbl_server = ctk.CTkLabel(self.frame_main, text=f"🟡 Comprobando estado de {SERVER_IP}...", font=ctk.CTkFont(size=12, slant="italic"))
        self.lbl_server.pack(pady=(0, 10))

        self.frame_user = ctk.CTkFrame(self.frame_main, fg_color="transparent")
        self.frame_user.pack(pady=5, fill="x", padx=50)
        
        self.combo_usuario = ctk.CTkComboBox(
            self.frame_user, values=["JugadorOffline"], 
            width=350, height=40, font=ctk.CTkFont(size=16), justify="center"
        )
        self.combo_usuario.pack(pady=5)

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
        threading.Thread(target=self.cargar_versiones, daemon=True).start()
        threading.Thread(target=self.ping_servidor, daemon=True).start()
        threading.Thread(target=self.buscar_actualizaciones, daemon=True).start()
        threading.Thread(target=self.iniciar_discord_rpc, daemon=True).start()

    def ping_servidor(self):
        try:
            r = requests.get(f"https://api.mcsrvstat.us/2/{SERVER_IP}", timeout=5)
            if r.status_code == 200:
                data = r.json()
                if data.get("online"):
                    jug = data["players"]["online"]
                    max_j = data["players"]["max"]
                    self.lbl_server.configure(text=f"🟢 El Server está ONLINE ({jug}/{max_j})", text_color="#2ecc71")
                else: self.lbl_server.configure(text=f"🔴 El Server está OFFLINE", text_color="#e74c3c")
        except: self.lbl_server.configure(text="No se pudo comprobar el servidor", text_color="gray")

    def buscar_actualizaciones(self):
        try:
            if not UPDATE_URL: return
            r = requests.get(UPDATE_URL, timeout=3)
            if r.status_code == 200:
                version_remota = r.text.strip()
                if version_remota != LAUNCHER_VERSION and len(version_remota) < 10:
                    messagebox.showinfo("¡Actualización Disponible!", f"Salió la versión {version_remota} del launcher.\n¡Pedile el nuevo archivo a Santi!")
        except: pass

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
                else: messagebox.showerror("Error", "No copiaste bien el link o cancelaste el login.")
        except Exception as e: messagebox.showerror("Error de Login", str(e))

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
                    self.optimode_var.set(datos.get("optimode", True))
                    self.papa_var.set(datos.get("papa_mode", False))
                    self.mesa_var.set(datos.get("mesa", False))
                    self.consola_var.set(datos.get("consola", False))
                    
                    self.veces_iniciadas.set(datos.get("veces_iniciadas", 0))
                    self.horas_jugadas_total.set(datos.get("horas_jugadas_total", 0.0))
                    self.limpiador_deep_var.set(datos.get("limpiador_deep", False))
                    self.backup_var.set(datos.get("backup_on_launch", True))
            except: pass

    def guardar_configuracion(self):
        usuario_actual = self.combo_usuario.get().strip()
        if usuario_actual and usuario_actual not in self.usuarios_guardados and not self.ms_data:
            self.usuarios_guardados.append(usuario_actual)
            self.combo_usuario.configure(values=self.usuarios_guardados)

        try:
            datos = {
                "usuario": usuario_actual, "historial_usuarios": self.usuarios_guardados, "ram": self.combo_ram.get(), "gc": self.gc_var.get(),
                "opt_minimos": self.opt_var.get(), "optimode": self.optimode_var.get(), "papa_mode": self.papa_var.get(),
                "mesa": self.mesa_var.get(), "consola": self.consola_var.get(), "veces_iniciadas": self.veces_iniciadas.get(),
                "horas_jugadas_total": self.horas_jugadas_total.get(), "limpiador_deep": self.limpiador_deep_var.get(), "backup_on_launch": self.backup_var.get()
            }
            with open(CONFIG_FILE, "w") as f: json.dump(datos, f)
        except: pass

    def cargar_versiones(self):
        try:
            mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            versiones_locales = minecraft_launcher_lib.utils.get_installed_versions(mine_dir)
            lista_locales = [v["id"] for v in versiones_locales]
            versiones_oficiales = minecraft_launcher_lib.utils.get_version_list()
            lista_oficiales = [v["id"] for v in versiones_oficiales if v["type"] == "release" or v["id"].startswith("1.")]
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
        vent.geometry("450x700")
        vent.grab_set()

        ctk.CTkLabel(vent, text="Rendimiento y Opciones", font=ctk.CTkFont(weight="bold")).pack(pady=15)
        ctk.CTkLabel(vent, text="Garbage Collector (Java):").pack(pady=(5,0))
        opciones_gc = ["G1GC (Equilibrado / Recomendado)", "ZGC (Latencia Ultra Baja, requiere RAM)", "Shenandoah (Rendimiento fluido de fondo)", "CMS (Para versiones antiguas)"]
        ctk.CTkComboBox(vent, values=opciones_gc, variable=self.gc_var, width=350).pack(pady=10)

        ctk.CTkCheckBox(vent, text="Aplicar gráficos al mínimo (Mejora FPS)", variable=self.opt_var).pack(pady=5)
        ctk.CTkCheckBox(vent, text="Auto-descargar Mods de Optimización", variable=self.optimode_var).pack(pady=5)
        ctk.CTkCheckBox(vent, text="Modo 'PC Papa' (Resolución 800x600)", variable=self.papa_var).pack(pady=5)
        ctk.CTkCheckBox(vent, text="OpenGL Mesa", variable=self.mesa_var).pack(pady=5)
        ctk.CTkCheckBox(vent, text="Mostrar Consola", variable=self.consola_var).pack(pady=5)
        
        frame_premium = ctk.CTkFrame(vent, fg_color="#2b2b2b")
        frame_premium.pack(pady=15, padx=20, fill="x")
        ctk.CTkLabel(frame_premium, text="Mantenimiento Automático", font=ctk.CTkFont(weight="bold")).pack(pady=5)
        ctk.CTkCheckBox(frame_premium, text="💾 Auto-Backup Salvavidas (Zip al iniciar)", variable=self.backup_var).pack(pady=5, padx=10, anchor="w")
        ctk.CTkCheckBox(frame_premium, text="🧹 Limpieza Profunda al Iniciar (logs/crash)", variable=self.limpiador_deep_var).pack(pady=5, padx=10, anchor="w")
        
        frame_skins = ctk.CTkFrame(vent, fg_color="#2b2b2b")
        frame_skins.pack(pady=15, padx=20, fill="x")
        ctk.CTkLabel(frame_skins, text="Gestor de Skins (Offline)", font=ctk.CTkFont(weight="bold")).pack(pady=5)
        btn_skin = ctk.CTkButton(frame_skins, text="🖼️ Elegir Skin Personalizada", command=self.elegir_skin, fg_color="#8e44ad", hover_color="#5e3370")
        btn_skin.pack(pady=5, padx=10, side="left")
        btn_skin_reset = ctk.CTkButton(frame_skins, text="❌ Borrar", command=self.borrar_skin, fg_color="#c0392b", hover_color="#922b21", width=60)
        btn_skin_reset.pack(pady=5, padx=10, side="right")

        btn_mods = ctk.CTkButton(vent, text="📦 Descargar Mods del Servidor (1-Clic)", command=self.descargar_mods_zip, fg_color="#d35400", hover_color="#a04000")
        btn_mods.pack(pady=10)

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

        self.veces_iniciadas.set(self.veces_iniciadas.get() + 1)
        self.guardar_configuracion()
        
        self.boton_jugar.configure(state="disabled", text="TRABAJANDO...")
        self.barra_progreso.set(0)

        # 🌟 EL ARREGLO DE DISCORD: AHORA DICE "JUGANDO PARAGUACRAFT" MÁS LA VERSIÓN
        if self.rpc:
            try: self.rpc.update(state=f"Jugando Paraguacraft {version}", details=f"Jugador: {usuario}", large_image="logo")
            except: pass

        uuid_real = None
        token_real = None
        if self.ms_data:
            uuid_real = self.ms_data["id"]
            token_real = self.ms_data["access_token"]

        self.hilo_juego_activo = True
        hilo = threading.Thread(target=self.ejecutar_motor, args=(version, usuario, ram, gc_type, uuid_real, token_real), daemon=True)
        hilo.start()

    def ejecutar_motor(self, version_jugada, usuario, ram, gc_type, uuid_real, token_real):
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
            
            # 🔥 INICIAMOS EL TERMINATOR DE VENTANAS
            threading.Thread(target=self._hilo_ninja_renombrar, args=(version_jugada,), daemon=True).start()

            start_time = datetime.now()
            
            lanzar_minecraft(version_jugada, usuario, ram, gc_type, self.opt_var.get(), self.optimode_var.get(), self.papa_var.get(), self.mesa_var.get(), self.consola_var.get(), self.actualizar_progreso, uuid_real, token_real)
            
            self.hilo_juego_activo = False # Apagamos el Terminator
            
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

    # 🪟 EL TERMINATOR DE VENTANAS (Chequea y renombra constantemente)
    def _hilo_ninja_renombrar(self, version_jugada):
        if sys.platform != "win32": return
        import time
        # Mientras el juego esté abierto, va a vigilar cada 2 segundos
        while self.hilo_juego_activo:
            time.sleep(2)
            try:
                def callback(hwnd, windows_list):
                    if win32gui.IsWindowVisible(hwnd):
                        text = win32gui.GetWindowText(hwnd)
                        # Si Minecraft cambia su nombre a "Minecraft", lo pisamos.
                        if "Minecraft" in text and "Launcher" not in text: 
                            win32gui.SetWindowText(hwnd, f"Paraguacraft {version_jugada}")
                    return True
                
                matching_windows = []
                win32gui.EnumWindows(callback, matching_windows)
            except Exception: pass

if __name__ == "__main__":
    app = ParaguaCraftLauncher()
    app.mainloop()