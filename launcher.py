import customtkinter as ctk
import tkinter as tk
import threading
import os
import platform
import subprocess
import webbrowser
import json
import minecraft_launcher_lib
from tkinter import filedialog, messagebox
import shutil
from core import lanzar_minecraft

ctk.set_appearance_mode("dark")
ctk.set_default_color_theme("blue")

CONFIG_FILE = "paraguacraft_config.json"

class ParaguaCraftLauncher(ctk.CTk):
    def __init__(self):
        super().__init__()
        self.title("ParaguaCraft Launcher")
        self.geometry("600x750")
        self.resizable(False, False)

        # Variables de configuración
        self.gc_var = ctk.StringVar(value="G1GC (Equilibrado / Recomendado)")
        self.opt_var = ctk.BooleanVar(value=False)
        self.optimode_var = ctk.BooleanVar(value=True) # Mods activados por defecto
        self.papa_var = ctk.BooleanVar(value=False)
        self.cerrar_var = ctk.BooleanVar(value=False)
        self.mesa_var = ctk.BooleanVar(value=False)
        self.consola_var = ctk.BooleanVar(value=False)

        self.frame_main = ctk.CTkFrame(self)
        self.frame_main.pack(pady=15, padx=20, fill="both", expand=True)

        self.label_titulo = ctk.CTkLabel(self.frame_main, text="ParaguaCraft Launcher", font=ctk.CTkFont(size=28, weight="bold"))
        self.label_titulo.pack(pady=(20, 10))

        # --- ZONA DE USUARIO OFFLINE (GIGANTE Y CENTRADA) ---
        self.frame_user = ctk.CTkFrame(self.frame_main, fg_color="transparent")
        self.frame_user.pack(pady=10, fill="x", padx=50)
        
        self.entry_usuario = ctk.CTkEntry(
            self.frame_user, 
            placeholder_text="Ingresá tu Nombre de Jugador", 
            width=350, 
            height=40, 
            font=ctk.CTkFont(size=16),
            justify="center"
        )
        self.entry_usuario.pack(pady=5)

        # --- LISTA DE VERSIONES (CON BORDES) ---
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

        # --- AJUSTES RÁPIDOS ---
        self.frame_opciones = ctk.CTkFrame(self.frame_main, fg_color="transparent")
        self.frame_opciones.pack(pady=15, fill="x", padx=50)

        self.label_ram = ctk.CTkLabel(self.frame_opciones, text="RAM:")
        self.label_ram.pack(side="left", padx=(0, 10))
        self.combo_ram = ctk.CTkComboBox(self.frame_opciones, values=["2GB", "4GB", "6GB", "8GB", "16GB", "32GB"], width=90)
        self.combo_ram.pack(side="left")

        self.btn_config = ctk.CTkButton(self.frame_opciones, text="Ajustes Extra", command=self.abrir_config, fg_color="#4a4a4a", width=120)
        self.btn_config.pack(side="right")

        # Ícono de Carpeta en vez de texto
        self.btn_carpeta = ctk.CTkButton(self.frame_opciones, text="📂", command=self.abrir_carpeta_minecraft, fg_color="#2b7b4b", width=40)
        self.btn_carpeta.pack(side="right", padx=10)

        # --- ESTADO Y PROGRESO ---
        self.lbl_estado = ctk.CTkLabel(self.frame_main, text="Listo", text_color="gray")
        self.lbl_estado.pack(pady=5)

        self.barra_progreso = ctk.CTkProgressBar(self.frame_main, width=350)
        self.barra_progreso.pack(pady=5)
        self.barra_progreso.set(0)

        self.boton_jugar = ctk.CTkButton(self.frame_main, text="INICIAR PARAGUACRAFT", command=self.iniciar_juego_thread, font=ctk.CTkFont(size=20, weight="bold"), height=50, width=300)
        self.boton_jugar.pack(pady=(20, 10))

        self.cargar_configuracion()
        threading.Thread(target=self.cargar_versiones, daemon=True).start()

    def cargar_configuracion(self):
        if os.path.exists(CONFIG_FILE):
            try:
                with open(CONFIG_FILE, "r") as f:
                    datos = json.load(f)
                    self.entry_usuario.insert(0, datos.get("usuario", ""))
                    self.combo_ram.set(datos.get("ram", "8GB"))
                    self.gc_var.set(datos.get("gc", "G1GC (Equilibrado / Recomendado)"))
                    self.opt_var.set(datos.get("opt_minimos", False))
                    self.optimode_var.set(datos.get("optimode", True))
                    self.papa_var.set(datos.get("papa_mode", False))
                    self.cerrar_var.set(datos.get("cerrar_launcher", False))
                    self.mesa_var.set(datos.get("mesa", False))
                    self.consola_var.set(datos.get("consola", False))
            except: pass

    def guardar_configuracion(self):
        datos = {
            "usuario": self.entry_usuario.get(),
            "ram": self.combo_ram.get(),
            "gc": self.gc_var.get(),
            "opt_minimos": self.opt_var.get(),
            "optimode": self.optimode_var.get(),
            "papa_mode": self.papa_var.get(),
            "cerrar_launcher": self.cerrar_var.get(),
            "mesa": self.mesa_var.get(),
            "consola": self.consola_var.get()
        }
        with open(CONFIG_FILE, "w") as f:
            json.dump(datos, f)

    def cargar_versiones(self):
        try:
            versiones = minecraft_launcher_lib.utils.get_version_list()
            lista_releases = [v["id"] for v in versiones if v["type"] == "release" or v["id"].startswith("1.")]
            for ver in lista_releases:
                self.lista_versiones.insert(tk.END, ver)
            self.lista_versiones.selection_set(0)
        except:
            self.lbl_estado.configure(text="Error al cargar versiones.")

    def abrir_config(self):
        vent = ctk.CTkToplevel(self)
        vent.title("Ajustes Avanzados")
        vent.geometry("400x520")
        vent.grab_set()

        ctk.CTkLabel(vent, text="Rendimiento y Opciones", font=ctk.CTkFont(weight="bold")).pack(pady=15)
        
        # GARBAGE COLLECTOR
        ctk.CTkLabel(vent, text="Garbage Collector (Java):").pack(pady=(5,0))
        opciones_gc = [
            "G1GC (Equilibrado / Recomendado)",
            "ZGC (Latencia Ultra Baja, requiere RAM)",
            "Shenandoah (Rendimiento fluido de fondo)",
            "CMS (Para versiones antiguas)"
        ]
        ctk.CTkComboBox(vent, values=opciones_gc, variable=self.gc_var, width=320).pack(pady=10)

        ctk.CTkCheckBox(vent, text="Aplicar gráficos al mínimo", variable=self.opt_var).pack(pady=5)
        ctk.CTkCheckBox(vent, text="Auto-descargar Mods Optimización", variable=self.optimode_var).pack(pady=5)
        ctk.CTkCheckBox(vent, text="Modo 'PC Papa' (Resolución 800x600)", variable=self.papa_var).pack(pady=5)
        ctk.CTkCheckBox(vent, text="Cerrar launcher al iniciar", variable=self.cerrar_var).pack(pady=5)
        ctk.CTkCheckBox(vent, text="OpenGL Mesa", variable=self.mesa_var).pack(pady=5)
        ctk.CTkCheckBox(vent, text="Mostrar Consola", variable=self.consola_var).pack(pady=5)

        ctk.CTkButton(vent, text="Elegir Skin Offline (.png)", command=self.elegir_skin, fg_color="#8e44ad").pack(pady=20)

    def elegir_skin(self):
        ruta = filedialog.askopenfilename(title="Skin (.png)", filetypes=[("PNG", "*.png")])
        if ruta:
            mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
            skin_path = os.path.join(mine_dir, "resourcepacks", "ParaguacraftBrandPack", "assets", "minecraft", "textures", "entity", "player")
            os.makedirs(skin_path, exist_ok=True)
            shutil.copy(ruta, os.path.join(skin_path, "steve.png"))
            shutil.copy(ruta, os.path.join(skin_path, "alex.png"))
            messagebox.showinfo("Skin", "¡Skin aplicada con éxito!")

    def abrir_carpeta_minecraft(self):
        minecraft_directory = minecraft_launcher_lib.utils.get_minecraft_directory()
        if platform.system() == "Windows": os.startfile(minecraft_directory)
        else: subprocess.Popen(["xdg-open", minecraft_directory])

    def actualizar_progreso(self, mensaje):
        self.lbl_estado.configure(text=mensaje)
        self.barra_progreso.step()

    def iniciar_juego_thread(self):
        seleccion = self.lista_versiones.curselection()
        if not seleccion:
            self.lbl_estado.configure(text="¡Selecciona una versión!")
            return

        usuario = self.entry_usuario.get().strip()
        if not usuario:
            messagebox.showwarning("Falta nombre", "Ponete un nombre de jugador para entrar offline.")
            return

        version = self.lista_versiones.get(seleccion[0])
        ram = self.combo_ram.get().replace("B", "")
        gc_type = self.gc_var.get().split()[0]

        self.guardar_configuracion()

        self.boton_jugar.configure(state="disabled", text="TRABAJANDO...")
        self.barra_progreso.set(0)

        hilo = threading.Thread(target=self.ejecutar_motor, args=(version, usuario, ram, gc_type), daemon=True)
        hilo.start()

    def ejecutar_motor(self, version, usuario, ram, gc_type):
        try:
            if self.cerrar_var.get():
                self.withdraw()
            lanzar_minecraft(version, usuario, ram, gc_type, self.opt_var.get(), self.optimode_var.get(), self.papa_var.get(), self.mesa_var.get(), self.consola_var.get(), self.actualizar_progreso)
            if not self.cerrar_var.get():
                self.lbl_estado.configure(text="Juego finalizado.")
        except Exception as e:
            self.deiconify()
            self.lbl_estado.configure(text=f"Error: {str(e)[:40]}")
        finally:
            if self.cerrar_var.get():
                self.destroy()
            else:
                self.boton_jugar.configure(state="normal", text="INICIAR PARAGUACRAFT")

if __name__ == "__main__":
    app = ParaguaCraftLauncher()
    app.mainloop()