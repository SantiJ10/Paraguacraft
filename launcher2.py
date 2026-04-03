import customtkinter as ctk
import json
import os
import threading
import webbrowser
from PIL import Image
from io import BytesIO
import requests
import psutil
import math

# Configuración base del tema oscuro
ctk.set_appearance_mode("Dark")
ctk.set_default_color_theme("green")

class NuevoParaguacraft(ctk.CTk):
    def __init__(self):
        super().__init__()

        self.title("Paraguacraft Launcher")
        self.geometry("1100x650")
        self.minsize(900, 550)

        # --- CARGA DE CONFIGURACIÓN GLOBAL ---
        self.ruta_config = "paraguacraft_config.json"
        self.config_actual = {"ram_asignada": 4, "cerrar_al_jugar": True, "mostrar_consola": False, "usuario": ""}
        if os.path.exists(self.ruta_config):
            with open(self.ruta_config, "r") as f:
                self.config_actual.update(json.load(f))
                
        # --- CARGA DE SESIÓN DE MICROSOFT GLOBAL ---
        self.ruta_sesion = "paraguacraft_session.json"
        self.ms_data = None
        if os.path.exists(self.ruta_sesion):
            with open(self.ruta_sesion, "r") as f:
                self.ms_data = json.load(f)

        self.grid_rowconfigure(0, weight=0) # Barra Superior
        self.grid_rowconfigure(1, weight=1) # Contenido
        self.grid_columnconfigure(0, weight=1)

        # --- TOP NAVIGATION (Lunar Style) ---
        self.topnav_frame = ctk.CTkFrame(self, height=70, corner_radius=0, fg_color="#101010")
        self.topnav_frame.grid(row=0, column=0, sticky="ew")
        self.topnav_frame.pack_propagate(False)

        self.logo_label = ctk.CTkLabel(self.topnav_frame, text="PARAGUACRAFT", font=ctk.CTkFont(size=24, weight="bold"))
        self.logo_label.pack(side="left", padx=20)

        frame_nav_center = ctk.CTkFrame(self.topnav_frame, fg_color="transparent")
        frame_nav_center.pack(side="left", expand=True, padx=20)

        def crear_boton_nav(parent, text, vista):
            btn = ctk.CTkButton(parent, text=text, command=lambda v=vista: self.cambiar_vista(v), fg_color="transparent", text_color=("gray10", "gray90"), hover_color=("gray70", "gray30"), anchor="center", font=ctk.CTkFont(size=14))
            btn.pack(side="left", padx=5, pady=15)
            return btn

        self.btn_inicio = crear_boton_nav(frame_nav_center, "Inicio", "inicio")
        self.btn_servidores = crear_boton_nav(frame_nav_center, "Servidores", "servidores") 
        self.btn_versiones = crear_boton_nav(frame_nav_center, "Versiones", "versiones")
        self.btn_skins = crear_boton_nav(frame_nav_center, "Skins", "skins") 
        self.btn_mods = crear_boton_nav(frame_nav_center, "Mods", "tienda_mods") # <-- AHORA VA A LA TIENDA

        self.frame_top_account = ctk.CTkFrame(self.topnav_frame, fg_color="transparent")
        self.frame_top_account.pack(side="right", padx=20)

        def menu_config_event(valor):
            if valor == "Discord RPC": self.cambiar_vista("discord")
            elif valor == "Almacenamiento": self.cambiar_vista("almacenamiento")
            self.combo_config.set("⚙️")

        self.combo_config = ctk.CTkOptionMenu(self.frame_top_account, values=["Discord RPC", "Almacenamiento"], command=menu_config_event, width=60, fg_color="#1E1E1E", button_color="#1E1E1E", button_hover_color="#2A2A2A", font=ctk.CTkFont(size=18))
        self.combo_config.set("⚙️")
        self.combo_config.pack(side="left", padx=5)

        self.nombre_display = self.ms_data["name"] if self.ms_data else self.config_actual.get("usuario", "Astronauta")
        
        def menu_cuenta_event(valor):
            if valor == "Cerrar Sesión":
                self.ms_data = None
                if os.path.exists(self.ruta_sesion): os.remove(self.ruta_sesion)
                self.config_actual["usuario"] = ""
                self.guardar_config()
                self.mostrar_pantalla_login()
            elif valor == "Ver Perfil":
                self.cambiar_vista("cuenta")
            self.combo_cuenta.set(f"👤 {self.nombre_display}")

        self.combo_cuenta = ctk.CTkOptionMenu(self.frame_top_account, values=["Ver Perfil", "Cerrar Sesión"], command=menu_cuenta_event, width=150, fg_color="#1E1E1E", button_color="#1E1E1E", button_hover_color="#2A2A2A", font=ctk.CTkFont(weight="bold"))
        self.combo_cuenta.set(f"👤 {self.nombre_display}")
        self.combo_cuenta.pack(side="left", padx=10)

        self.vista_actual = None
        
        if not self.ms_data and not self.config_actual.get("usuario", ""):
            self.mostrar_pantalla_login()
        else:
            self.cambiar_vista("inicio")

    def guardar_config(self):
        with open(self.ruta_config, "w") as f:
            json.dump(self.config_actual, f, indent=4)

    def mostrar_pantalla_login(self):
        self.topnav_frame.grid_remove()
        if self.vista_actual: self.vista_actual.destroy()

        self.login_frame = ctk.CTkFrame(self, corner_radius=0, fg_color="#121212")
        self.login_frame.grid(row=0, column=0, rowspan=2, sticky="nsew")
        self.login_frame.grid_columnconfigure(0, weight=1)
        self.login_frame.grid_rowconfigure((0, 5), weight=1)

        ctk.CTkLabel(self.login_frame, text="PARAGUACRAFT", font=ctk.CTkFont(size=45, weight="bold")).grid(row=1, column=0, pady=(0, 10))
        ctk.CTkLabel(self.login_frame, text="Inicia sesión para continuar", font=ctk.CTkFont(size=16), text_color="gray60").grid(row=2, column=0, pady=(0, 50))

        def login_microsoft():
            try:
                from minecraft_launcher_lib.microsoft_account import get_login_url, complete_login, url_contains_auth_code, get_auth_code_from_url
                from tkinter import messagebox
                CLIENT_ID = "72fb7c48-c2f5-4d13-b0e7-9835b3b906c0"
                REDIRECT_URI = "https://login.live.com/oauth20_desktop.srf"
                url = get_login_url(CLIENT_ID, REDIRECT_URI)
                webbrowser.open(url)
                dialog = ctk.CTkInputDialog(text="Copiá TODO el link final y pegalo acá:", title="Microsoft Login")
                url_respuesta = dialog.get_input()
                if url_respuesta and url_contains_auth_code(url_respuesta):
                    btn_ms.configure(text="Validando...", state="disabled")
                    self.update()
                    def proceso():
                        try:
                            auth_code = get_auth_code_from_url(url_respuesta)
                            self.ms_data = complete_login(CLIENT_ID, None, REDIRECT_URI, auth_code)
                            with open(self.ruta_sesion, "w") as f: json.dump(self.ms_data, f)
                            self.after(0, self.cerrar_pantalla_login)
                        except Exception as e:
                            self.after(0, lambda: messagebox.showerror("Error", f"Fallo al iniciar:\n{e}"))
                            self.after(0, lambda: btn_ms.configure(text="Login with Microsoft", state="normal"))
                    threading.Thread(target=proceso, daemon=True).start()
            except ImportError:
                print("Aviso: minecraft_launcher_lib no está instalado. Instalalo con pip install minecraft-launcher-lib")

        def login_invitado():
            dialog = ctk.CTkInputDialog(text="Ingresá tu nombre offline:", title="Guest")
            u = dialog.get_input()
            if u and u.strip():
                self.config_actual["usuario"] = u.strip()
                self.guardar_config()
                self.cerrar_pantalla_login()

        btn_ms = ctk.CTkButton(self.login_frame, text="Login with Microsoft Account", font=ctk.CTkFont(size=16, weight="bold"), height=50, width=320, fg_color="#0078D4", hover_color="#106EBE", command=login_microsoft)
        btn_ms.grid(row=3, column=0, pady=10)
        ctk.CTkButton(self.login_frame, text="Continue as Guest", font=ctk.CTkFont(size=16, weight="bold"), height=50, width=320, fg_color="#2A2A2A", hover_color="#3A3A3A", command=login_invitado).grid(row=4, column=0, pady=10)

    def cerrar_pantalla_login(self):
        if hasattr(self, 'login_frame'): self.login_frame.destroy()
        self.topnav_frame.grid()
        self.nombre_display = self.ms_data["name"] if self.ms_data else self.config_actual.get("usuario", "Astronauta")
        self.combo_cuenta.set(f"👤 {self.nombre_display}")
        self.cambiar_vista("inicio")

    def cambiar_vista(self, nombre_vista):
        if self.vista_actual is not None:
            self.vista_actual.destroy()

        self.vista_actual = ctk.CTkFrame(self, fg_color="transparent")
        self.vista_actual.grid(row=1, column=0, sticky="nsew", padx=20, pady=20)

        if nombre_vista == "inicio":
            import os
            import threading
            self.vista_actual.grid_columnconfigure(0, weight=1)
            self.vista_actual.grid_rowconfigure(0, weight=1) # Espacio gigante para el Banner
            self.vista_actual.grid_rowconfigure(1, weight=0) # Barra de Launch

            # --- 1. BANNER GIGANTE (Lunar Home Style) ---
            frame_banner = ctk.CTkFrame(self.vista_actual, fg_color="#121212", corner_radius=15)
            frame_banner.grid(row=0, column=0, sticky="nsew", padx=20, pady=(10, 15))
            frame_banner.grid_propagate(False)

            # Intentar cargar imagen real desde la carpeta assets
            ruta_banner = os.path.join(os.getcwd(), "assets", "main_banner.png")
            if os.path.exists(ruta_banner):
                from PIL import Image
                img_data = Image.open(ruta_banner)
                # La hacemos bien grande para que cubra el fondo
                img_banner = ctk.CTkImage(light_image=img_data, dark_image=img_data, size=(1200, 600))
                lbl_fondo = ctk.CTkLabel(frame_banner, text="", image=img_banner)
                lbl_fondo.pack(expand=True, fill="both")
            else:
                # Diseño default elegante si no pusiste la foto todavía
                ctk.CTkLabel(frame_banner, text="P A R A G U A C R A F T", font=ctk.CTkFont(size=70, weight="bold"), text_color="#1A1A1A").place(relx=0.5, rely=0.4, anchor="center")
                ctk.CTkLabel(frame_banner, text=f"Welcome back, {self.nombre_display}", font=ctk.CTkFont(size=32, weight="bold"), text_color="white").place(relx=0.5, rely=0.5, anchor="center")
                ctk.CTkLabel(frame_banner, text="Crea una carpeta 'assets' y guardá una foto llamada 'main_banner.png' para cambiar este fondo.", font=ctk.CTkFont(size=14), text_color="gray50").place(relx=0.5, rely=0.6, anchor="center")

            # --- 2. BOTTOM LAUNCH BAR (Lunar Style) ---
            frame_bottom = ctk.CTkFrame(self.vista_actual, fg_color="#181818", corner_radius=12, height=90)
            frame_bottom.grid(row=1, column=0, sticky="ew", padx=20, pady=(0, 20))
            frame_bottom.pack_propagate(False)

            v_activa = self.config_actual.get("version_activa", "1.20.4")
            m_activo = self.config_actual.get("motor_activo", "Fabric")

            # Izquierda: Info de versión
            info_frame = ctk.CTkFrame(frame_bottom, fg_color="transparent")
            info_frame.pack(side="left", padx=25, pady=15)
            lbl_status_top = ctk.CTkLabel(info_frame, text="Ready to play", font=ctk.CTkFont(size=13, weight="bold"), text_color="#2ECC71")
            lbl_status_top.pack(anchor="w")
            ctk.CTkLabel(info_frame, text=f"Minecraft {v_activa} ({m_activo})", font=ctk.CTkFont(size=20, weight="bold"), text_color="white").pack(anchor="w", pady=(2,0))

            # Derecha: Botones
            frame_botones = ctk.CTkFrame(frame_bottom, fg_color="transparent")
            frame_botones.pack(side="right", padx=20, pady=20)

            # Botón Abrir Carpeta
            def abrir_instancia():
                import platform
                try:
                    import minecraft_launcher_lib
                    mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
                except:
                    mine_dir = os.path.join(os.getenv('APPDATA'), ".minecraft")
                nombre_limpio = m_activo.replace(" ", "_").replace("+", "Plus")
                folder_name = f"Paraguacraft_{v_activa}_{nombre_limpio}"
                ruta = os.path.join(mine_dir, "instancias", folder_name)
                os.makedirs(ruta, exist_ok=True)
                if platform.system() == "Windows": os.startfile(ruta)

            btn_folder = ctk.CTkButton(frame_botones, text="📁", width=50, height=50, font=ctk.CTkFont(size=20), fg_color="#2A2A2A", hover_color="#3A3A3A", command=abrir_instancia)
            btn_folder.pack(side="left", padx=(0, 10))

            btn_jugar = ctk.CTkButton(frame_botones, text="LAUNCH GAME", font=ctk.CTkFont(size=18, weight="bold"), height=50, width=250, fg_color="#2ECC71", hover_color="#27AE60", text_color="black")
            btn_jugar.pack(side="left")

            # Barra de progreso integrada
            barra_carga_juego = ctk.CTkProgressBar(self.vista_actual, height=4, progress_color="#2ECC71", fg_color="#181818")
            barra_carga_juego.set(0)
            barra_carga_juego.grid(row=2, column=0, sticky="ew", padx=35, pady=(0, 10))
            barra_carga_juego.grid_remove() # Oculta hasta que se inicie

            def actualizar_ui_juego(mensaje):
                lbl_status_top.configure(text=mensaje, text_color="#3498DB")
                barra_carga_juego.step()

            def lanzar_juego():
                try:
                    from core import lanzar_minecraft
                except ImportError:
                    actualizar_ui_juego("Error: core.py no encontrado.")
                    return
                
                ram_gb = self.config_actual.get("ram_asignada", 4)
                
                btn_jugar.configure(text="STARTING...", state="disabled", fg_color="#2A2A2A", text_color="gray")
                barra_carga_juego.grid() # Mostramos la barrita verde
                barra_carga_juego.set(0)
                self.update()

                def hilo_ejecucion():
                    try:
                        lanzar_minecraft(
                            version=v_activa,
                            username=self.nombre_display,
                            max_ram=f"{ram_gb}G",
                            gc_type="G1GC",
                            optimizar=self.config_actual.get("opt_minimos", False),
                            motor_elegido=m_activo,
                            papa_mode=self.config_actual.get("papa_mode", False),
                            usar_mesa=False,
                            mostrar_consola=self.config_actual.get("mostrar_consola", False),
                            progress_callback=lambda m: self.after(0, actualizar_ui_juego, m),
                            lan_distancia=self.config_actual.get("lan_distancia", False)
                        )
                        
                        if self.config_actual.get("cerrar_al_jugar"):
                            self.after(0, self.destroy)
                        else:
                            self.after(0, lambda: btn_jugar.configure(text="LAUNCH GAME", state="normal", fg_color="#2ECC71", text_color="black"))
                            self.after(0, lambda: lbl_status_top.configure(text="Ready to play", text_color="#2ECC71"))
                            self.after(0, barra_carga_juego.grid_remove)
                            
                    except Exception as e:
                        self.after(0, lambda: lbl_status_top.configure(text=f"Crash: {str(e)[:40]}", text_color="#E74C3C"))
                        self.after(0, lambda: btn_jugar.configure(text="RETRY", state="normal", fg_color="#2ECC71", text_color="black"))

                threading.Thread(target=hilo_ejecucion, daemon=True).start()

            btn_jugar.configure(command=lanzar_juego)

        elif nombre_vista == "versiones":
            # --- PROPORCIONES PERFECTAS LUNAR STYLE (75% Grilla / 25% Panel Derecho) ---
            self.vista_actual.grid_columnconfigure(0, weight=3) # Toma 3 partes del espacio
            self.vista_actual.grid_columnconfigure(1, weight=1) # Toma 1 parte del espacio
            self.vista_actual.grid_rowconfigure(0, weight=1)

            # --- PANEL DERECHO: Detalles de la Selección (Compacto) ---
            frame_right = ctk.CTkFrame(self.vista_actual, fg_color="#181818", corner_radius=10)
            frame_right.grid(row=0, column=1, sticky="nsew", padx=(10, 20), pady=10)
            # Le ponemos un ancho mínimo para que no se aplaste demasiado si achicás la ventana
            frame_right.grid_columnconfigure(0, weight=1)

            ctk.CTkLabel(frame_right, text="SELECTED VERSION", font=ctk.CTkFont(size=14, weight="bold")).pack(anchor="w", padx=20, pady=(20, 10))

            # Contenedor de la foto del panel derecho
            lbl_img_right = ctk.CTkLabel(frame_right, text="", height=100, fg_color="#2A2A2A", corner_radius=8)
            lbl_img_right.pack(fill="x", padx=20, pady=10)

            lbl_version_title = ctk.CTkLabel(frame_right, text="Minecraft", font=ctk.CTkFont(size=20, weight="bold"))
            lbl_version_title.pack(anchor="w", padx=20, pady=(10, 0))

            # Achicamos un poco el wraplength para que encaje en el panel más fino
            lbl_version_desc = ctk.CTkLabel(frame_right, text="Seleccioná una versión de la grilla.", wraplength=220, justify="left", text_color="gray60", font=ctk.CTkFont(size=12))
            lbl_version_desc.pack(anchor="w", padx=20, pady=(5, 20))

            # Desplegable de Sub-Versiones
            frame_dropdown = ctk.CTkFrame(frame_right, fg_color="transparent")
            frame_dropdown.pack(fill="x", padx=20, pady=5)
            ctk.CTkLabel(frame_dropdown, text="Version", font=ctk.CTkFont(size=13, weight="bold")).pack(side="left")
            combo_subversion = ctk.CTkOptionMenu(frame_dropdown, values=["-"], width=100, fg_color="#2A2A2A", button_color="#3A3A3A")
            combo_subversion.pack(side="right")

            # Desplegable de Loader / Addons
            frame_motor = ctk.CTkFrame(frame_right, fg_color="transparent")
            frame_motor.pack(fill="x", padx=20, pady=15)
            ctk.CTkLabel(frame_motor, text="Addons", font=ctk.CTkFont(size=13, weight="bold")).pack(side="left")
            combo_motor = ctk.CTkOptionMenu(frame_motor, values=["-"], width=130, fg_color="#2A2A2A", button_color="#3A3A3A")
            combo_motor.pack(side="right")

            # --- BOTONES INFERIORES DEL PANEL DERECHO ---
            frame_launch = ctk.CTkFrame(frame_right, fg_color="transparent")
            frame_launch.pack(side="bottom", fill="x", padx=20, pady=20)

            def abrir_ajustes_version():
                self.config_actual["version_activa"] = combo_subversion.get()
                self.config_actual["motor_activo"] = combo_motor.get()
                self.guardar_config()
                self.cambiar_vista("gestor_version")

            btn_gear = ctk.CTkButton(frame_launch, text="⚙️", width=40, height=45, font=ctk.CTkFont(size=20), fg_color="#2A2A2A", hover_color="#3A3A3A", command=abrir_ajustes_version)
            btn_gear.pack(side="left", padx=(0, 10))

            def guardar_y_lanzar():
                self.config_actual["version_activa"] = combo_subversion.get()
                self.config_actual["motor_activo"] = combo_motor.get()
                self.guardar_config()
                self.cambiar_vista("inicio") 

            btn_launch = ctk.CTkButton(frame_launch, text="LAUNCH GAME", font=ctk.CTkFont(size=15, weight="bold"), height=45, fg_color="#2ECC71", hover_color="#27AE60", text_color="black", command=guardar_y_lanzar)
            btn_launch.pack(side="left", expand=True, fill="x")

            def actualizar_panel_derecho(titulo, desc, subversiones, menu_motores, img_obj):
                lbl_version_title.configure(text=f"Minecraft {titulo}")
                lbl_version_desc.configure(text=desc)
                combo_subversion.configure(values=subversiones)
                combo_subversion.set(subversiones[0])
                
                combo_motor.configure(values=menu_motores)
                combo_motor.set(menu_motores[0])
                
                if img_obj:
                    lbl_img_right.configure(image=img_obj, text="")
                else:
                    lbl_img_right.configure(image="", text="[ Sin Imagen ]")

            # --- PANEL IZQUIERDO: Grilla de Versiones ---
            frame_left = ctk.CTkScrollableFrame(self.vista_actual, fg_color="transparent")
            frame_left.grid(row=0, column=0, sticky="nsew", padx=(0, 10))
            # Le damos 2 columnas a la grilla para que las tarjetas se estiren bien
            frame_left.grid_columnconfigure((0, 1), weight=1)

            def crear_tarjeta_lunar(fila, col, titulo, desc, subs, menu_motores):
                import os
                from PIL import Image
                
                card = ctk.CTkFrame(frame_left, corner_radius=15, fg_color="#1E1E1E", height=180)
                card.grid(row=fila, column=col, padx=10, pady=10, sticky="nsew")
                card.grid_propagate(False)

                nombre_archivo = titulo.replace(".", "_") 
                ruta_img = os.path.join(os.getcwd(), "assets", f"{nombre_archivo}.png")
                
                img_right_panel = None

                if os.path.exists(ruta_img):
                    img_data = Image.open(ruta_img)
                    # Tamaño dinámico para que llene bien la tarjeta
                    ctk_img_card = ctk.CTkImage(light_image=img_data, dark_image=img_data, size=(450, 180))
                    img_right_panel = ctk.CTkImage(light_image=img_data, dark_image=img_data, size=(280, 100))
                    
                    lbl_fondo = ctk.CTkLabel(card, text="", image=ctk_img_card, corner_radius=15)
                    lbl_fondo.place(relx=0.5, rely=0.5, anchor="center")
                    
                    # TEXTO FLOTANTE DIRECTO SOBRE LA IMAGEN (Sin cajas negras)
                    lbl = ctk.CTkLabel(card, text=f"PARAGUA {titulo}", font=ctk.CTkFont(size=32, weight="bold"), text_color="white", bg_color="transparent")
                    lbl.place(relx=0.5, rely=0.5, anchor="center")
                else:
                    ctk.CTkLabel(card, text="P A R A G U A", font=ctk.CTkFont(size=14, weight="bold"), text_color="gray50").pack(pady=(40, 0))
                    lbl = ctk.CTkLabel(card, text=titulo, font=ctk.CTkFont(size=45, weight="bold"), text_color="white")
                    lbl.pack()
                
                def _al_entrar(e): card.configure(border_width=2, border_color="#2ECC71")
                def _al_salir(e): card.configure(border_width=0)
                def _al_click(e): actualizar_panel_derecho(titulo, desc, subs, menu_motores, img_right_panel)

                elementos = [card, lbl]
                if 'lbl_fondo' in locals(): elementos.append(lbl_fondo)
                
                for el in elementos:
                    el.bind("<Enter>", _al_entrar)
                    el.bind("<Leave>", _al_salir)
                    el.bind("<Button-1>", _al_click)

            # --- DIBUJAMOS LA GRILLA DE VERSIONES ---
            crear_tarjeta_lunar(0, 0, "1.21", "The Tricky Trials Update. Includes Trial Chambers, new mobs, and items.", ["1.21.4", "1.21.3", "1.21.2", "1.21.1", "1.21"], ["Fabric + Iris", "Fabric (Solo)", "Vanilla", "Forge"])
            crear_tarjeta_lunar(0, 1, "1.20", "Trails & Tales. Archeology, armor trims, and camels.", ["1.20.6", "1.20.4", "1.20.2", "1.20.1", "1.20"], ["Fabric + Iris", "Fabric (Solo)", "Forge", "Vanilla"])
            crear_tarjeta_lunar(1, 0, "1.19", "The Wild Update. Deep dark, ancient cities, and mangrove swamps.", ["1.19.4", "1.19.3", "1.19.2", "1.19.1", "1.19"], ["Fabric + Iris", "Forge", "Vanilla"])
            crear_tarjeta_lunar(1, 1, "1.18", "Caves & Cliffs Part II. Increased world height and new cave generation.", ["1.18.2", "1.18.1", "1.18"], ["Fabric", "Forge", "Vanilla"])
            crear_tarjeta_lunar(2, 0, "1.17", "Caves & Cliffs Part I. Amethyst, copper, and deepslate.", ["1.17.1", "1.17"], ["Fabric", "Forge", "Vanilla"])
            crear_tarjeta_lunar(2, 1, "1.16", "The Nether Update. Piglins, bastions, and netherite.", ["1.16.5", "1.16.4", "1.16.1"], ["Fabric", "Forge + Optifine", "Vanilla"])
            crear_tarjeta_lunar(3, 0, "1.12", "The World of Color Update. Great for legacy modpacks.", ["1.12.2"], ["Forge + Optifine", "Vanilla"])
            crear_tarjeta_lunar(3, 1, "1.8", "The Bountiful Update. The absolute best version for competitive PvP.", ["1.8.9", "1.8.8"], ["OptiFine M6", "Forge + Optifine", "Paraguacraft PvP Mods"])
            crear_tarjeta_lunar(4, 0, "1.7", "The Update that Changed the World. Extremely stable legacy mods.", ["1.7.10"], ["Forge + Optifine", "Vanilla"])
            
            # Forzamos un click en la primera tarjeta para que el panel derecho inicie con datos
            actualizar_panel_derecho("1.21", "The Tricky Trials Update. Includes Trial Chambers, new mobs, and items.", ["1.21.4", "1.21.3", "1.21.2", "1.21.1", "1.21"], ["Fabric + Iris", "Fabric (Solo)", "Vanilla", "Forge"], None)
                
        elif nombre_vista == "gestor_version":
            version_edit = self.config_actual.get("version_activa", "1.20.4")
            
            self.vista_actual.grid_columnconfigure(0, weight=1)
            self.vista_actual.grid_rowconfigure(1, weight=1)

            frame_header = ctk.CTkFrame(self.vista_actual, fg_color="transparent", height=60)
            frame_header.grid(row=0, column=0, sticky="ew", padx=20, pady=10)
            
            frame_title = ctk.CTkFrame(frame_header, fg_color="transparent")
            frame_title.pack(side="left")
            ctk.CTkButton(frame_title, text="←", width=40, height=40, fg_color="#2A2A2A", hover_color="#3A3A3A", command=lambda: self.cambiar_vista("versiones")).pack(side="left", padx=(0, 15))
            ctk.CTkLabel(frame_title, text=f"MINECRAFT {version_edit}", font=ctk.CTkFont(size=24, weight="bold")).pack(side="left")

            frame_nav = ctk.CTkFrame(frame_header, fg_color="#181818", corner_radius=8)
            frame_nav.pack(side="right")
            
            frame_content = ctk.CTkFrame(self.vista_actual, fg_color="transparent")
            frame_content.grid(row=1, column=0, sticky="nsew", padx=20, pady=10)
            frame_content.grid_columnconfigure(0, weight=1)
            frame_content.grid_rowconfigure(0, weight=1)

            def cargar_subvista(subvista):
                for widget in frame_content.winfo_children(): widget.destroy()
                
                if subvista == "mods":
                    import os
                    motor_edit = self.config_actual.get("motor_activo", "Fabric")
                    nombre_limpio_motor = motor_edit.replace(" ", "_").replace("+", "Plus")
                    folder_name = f"Paraguacraft_{version_edit}_{nombre_limpio_motor}"
                    
                    try:
                        import minecraft_launcher_lib
                        mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
                    except ImportError:
                        mine_dir = os.path.join(os.getenv('APPDATA'), ".minecraft")
                        
                    mods_dir = os.path.join(mine_dir, "instancias", folder_name, "mods")

                    archivos_mods = [f for f in os.listdir(mods_dir) if f.endswith('.jar') or f.endswith('.disabled')] if os.path.exists(mods_dir) else []

                    if not archivos_mods:
                        frame_empty = ctk.CTkFrame(frame_content, fg_color="transparent")
                        frame_empty.grid(row=0, column=0, sticky="nsew")
                        frame_empty.grid_columnconfigure(0, weight=1)
                        frame_empty.grid_rowconfigure((0, 4), weight=1)

                        ctk.CTkLabel(frame_empty, text="🗂️", font=ctk.CTkFont(size=80), text_color="gray30").grid(row=1, column=0, pady=(0, 10))
                        ctk.CTkLabel(frame_empty, text="No Local Mods found", font=ctk.CTkFont(size=28, weight="bold")).grid(row=2, column=0, pady=(0, 10))
                        ctk.CTkLabel(frame_empty, text="Minecraft is better with mods! Explore new content.", text_color="gray60", font=ctk.CTkFont(size=14)).grid(row=3, column=0, pady=(0, 30))
                        ctk.CTkButton(frame_empty, text="🚀 Explore Online Mods", font=ctk.CTkFont(weight="bold", size=14), fg_color="#D35400", hover_color="#E67E22", width=180, height=45, command=lambda: self.cambiar_vista("tienda_mods")).grid(row=4, column=0, sticky="n")
                    else:
                        frame_lista_mods = ctk.CTkScrollableFrame(frame_content, fg_color="transparent")
                        frame_lista_mods.grid(row=0, column=0, sticky="nsew")
                        
                        top_mods = ctk.CTkFrame(frame_lista_mods, fg_color="transparent")
                        top_mods.pack(fill="x", pady=10, padx=10)
                        ctk.CTkLabel(top_mods, text=f"Mods Locales ({motor_edit})", font=ctk.CTkFont(size=18, weight="bold")).pack(side="left")
                        ctk.CTkButton(top_mods, text="🔄 Smart Updater", fg_color="#3498DB", hover_color="#2980B9", width=120).pack(side="right")

                        for mod_file in archivos_mods:
                            item_mod = ctk.CTkFrame(frame_lista_mods, fg_color="#1A1A1A", corner_radius=8)
                            item_mod.pack(fill="x", pady=5, padx=10)

                            nombre_limpio = mod_file.replace(".jar", "").replace(".disabled", "")
                            esta_activo = not mod_file.endswith(".disabled")

                            info = ctk.CTkFrame(item_mod, fg_color="transparent")
                            info.pack(side="left", padx=15, pady=10)
                            ctk.CTkLabel(info, text=nombre_limpio, font=ctk.CTkFont(size=14, weight="bold")).pack(anchor="w")
                            
                            estado_texto = "🟢 Activo" if esta_activo else "🔴 Apagado"
                            ctk.CTkLabel(info, text=estado_texto, text_color="gray50", font=ctk.CTkFont(size=11)).pack(anchor="w")

                            def toggle_mod(a, act):
                                import os
                                ruta_actual = os.path.join(mods_dir, a)
                                if act:
                                    nueva_ruta = ruta_actual + ".disabled"
                                else:
                                    nueva_ruta = ruta_actual.replace(".disabled", "")
                                if os.path.exists(ruta_actual): os.rename(ruta_actual, nueva_ruta)
                                cargar_subvista("mods")

                            sw_mod = ctk.CTkSwitch(item_mod, text="", progress_color="#2ECC71", width=50)
                            if esta_activo: sw_mod.select()
                            sw_mod.configure(command=lambda a=mod_file, act=esta_activo: toggle_mod(a, act))
                            sw_mod.pack(side="right", padx=20)
                
                elif subvista == "settings":
                    import psutil, math
                    ram_total_gb = max(1, math.floor(psutil.virtual_memory().total / (1024 ** 3)))
                    
                    scroll_ajustes = ctk.CTkScrollableFrame(frame_content, fg_color="transparent")
                    scroll_ajustes.grid(row=0, column=0, sticky="nsew")

                    # BOTONES DEL VIEJO LAUNCHER (Arriba)
                    frame_old_tools = ctk.CTkFrame(scroll_ajustes, fg_color="transparent")
                    frame_old_tools.pack(fill="x", pady=(0, 20))
                    
                    def auto_detectar():
                        if ram_total_gb <= 8:
                            self.config_actual["ram_asignada"] = 3
                            self.config_actual["opt_minimos"] = True
                            self.config_actual["papa_mode"] = True
                        else:
                            self.config_actual["ram_asignada"] = min(ram_total_gb // 2, 8)
                            self.config_actual["opt_minimos"] = False
                            self.config_actual["papa_mode"] = False
                        self.guardar_config()
                        cargar_subvista("settings") # Recargar para ver cambios
                        
                    def abrir_carpeta():
                        import os, platform
                        import minecraft_launcher_lib
                        directory = minecraft_launcher_lib.utils.get_minecraft_directory()
                        if platform.system() == "Windows": os.startfile(directory)
                        else: os.system(f"xdg-open '{directory}'")

                    ctk.CTkButton(frame_old_tools, text="🔍 Auto-Detect PC", fg_color="#3498DB", hover_color="#2980B9", command=auto_detectar).pack(side="left", padx=5)
                    ctk.CTkButton(frame_old_tools, text="📂 Open .minecraft", fg_color="#27AE60", hover_color="#2ECC71", command=abrir_carpeta).pack(side="left", padx=5)
                    
                    # Java Environment (RAM)
                    ctk.CTkLabel(scroll_ajustes, text="☕ Java Environment", font=ctk.CTkFont(size=18, weight="bold")).pack(anchor="w", pady=(10, 10))
                    frame_ram = ctk.CTkFrame(scroll_ajustes, fg_color="#181818", corner_radius=10)
                    frame_ram.pack(fill="x", pady=5)
                    lbl_ram = ctk.CTkLabel(frame_ram, text=f"Allocated Memory: {self.config_actual.get('ram_asignada', 4)} GB", font=ctk.CTkFont(size=14))
                    lbl_ram.pack(anchor="w", padx=20, pady=(15, 5))
                    
                    def cambiar_ram(v):
                        self.config_actual["ram_asignada"] = int(v)
                        lbl_ram.configure(text=f"Allocated Memory: {int(v)} GB")
                        self.guardar_config()
                        
                    slider_ram = ctk.CTkSlider(frame_ram, from_=1, to=ram_total_gb, number_of_steps=ram_total_gb-1, command=cambiar_ram)
                    slider_ram.set(self.config_actual.get("ram_asignada", 4))
                    slider_ram.pack(fill="x", padx=20, pady=(5, 20))

                    # Game Settings & Maintenance
                    ctk.CTkLabel(scroll_ajustes, text="🎮 Game Settings & Maintenance", font=ctk.CTkFont(size=18, weight="bold")).pack(anchor="w", pady=(20, 10))
                    frame_game = ctk.CTkFrame(scroll_ajustes, fg_color="#181818", corner_radius=10)
                    frame_game.pack(fill="x", pady=5)
                    
                    def crear_switch(parent, texto, clave_config, defecto):
                        var = ctk.BooleanVar(value=self.config_actual.get(clave_config, defecto))
                        def evt(): self.config_actual[clave_config] = var.get(); self.guardar_config()
                        sw = ctk.CTkSwitch(parent, text=texto, variable=var, progress_color="#27AE60", command=evt)
                        sw.pack(anchor="w", padx=20, pady=10)
                        return sw

                    crear_switch(frame_game, "Close launcher on game start", "cerrar_al_jugar", True)
                    crear_switch(frame_game, "📉 Gráficos al Mínimo (Boost FPS)", "opt_minimos", False)
                    crear_switch(frame_game, "🥔 Papa Mode (800x600 Resolution)", "papa_mode", False)
                    crear_switch(frame_game, "🌐 Activar LAN a Distancia (e4mc)", "lan_distancia", False)
                    crear_switch(frame_game, "💾 Auto-Backup de Mundos al iniciar", "backup_var", True)
                    crear_switch(frame_game, "🧹 Limpieza profunda de Logs/Crash", "limpiador_deep_var", False)

            ctk.CTkButton(frame_nav, text="📁 Mods", fg_color="transparent", hover_color="#2A2A2A", command=lambda: cargar_subvista("mods")).pack(side="left", padx=5, pady=5)
            ctk.CTkButton(frame_nav, text="📦 Resource Packs", fg_color="transparent", hover_color="#2A2A2A").pack(side="left", padx=5)
            ctk.CTkButton(frame_nav, text="✨ Shaders", fg_color="transparent", hover_color="#2A2A2A").pack(side="left", padx=5)
            ctk.CTkButton(frame_nav, text="⚙️ Settings", fg_color="transparent", hover_color="#2A2A2A", command=lambda: cargar_subvista("settings")).pack(side="left", padx=5)

            cargar_subvista("mods") 

        elif nombre_vista == "tienda_mods":
            # --- LA TIENDA DE MODS ONLINE INTEGRADA (Modrinth API) ---
            self.vista_actual.grid_columnconfigure(0, weight=1)
            self.vista_actual.grid_rowconfigure(1, weight=1)

            frame_top_tienda = ctk.CTkFrame(self.vista_actual, fg_color="transparent")
            frame_top_tienda.grid(row=0, column=0, sticky="ew", padx=20, pady=10)
            
            ctk.CTkButton(frame_top_tienda, text="← Back", width=40, fg_color="#2A2A2A", hover_color="#3A3A3A", command=lambda: self.cambiar_vista("gestor_version")).pack(side="left", padx=(0, 15))
            
            # Selector inteligente con DATAPACKS y MODPACKS
            combo_tipo = ctk.CTkOptionMenu(frame_top_tienda, values=["mod", "shader", "resourcepack", "modpack", "datapack"], width=130, fg_color="#1E1E1E", button_color="#2A2A2A")
            combo_tipo.pack(side="left", padx=(0, 10))

            entry_busqueda = ctk.CTkEntry(frame_top_tienda, placeholder_text="Search mods, modpacks, plugins...", width=250, fg_color="#1E1E1E")
            entry_busqueda.pack(side="left")
            
            frame_lista_tienda = ctk.CTkScrollableFrame(self.vista_actual, fg_color="transparent")
            frame_lista_tienda.grid(row=1, column=0, sticky="nsew", padx=20, pady=10)

            # --- FUNCIONES DE LÓGICA (Definidas antes para evitar el NameError) ---

            def abrir_popup_instalacion(project_id, nombre_mod, tipo_proy):
                popup = ctk.CTkToplevel(self)
                popup.title(f"Install {nombre_mod}")
                popup.geometry("450x450")
                popup.attributes("-topmost", True)
                popup.grab_set()

                # Cabecera
                frame_head = ctk.CTkFrame(popup, fg_color="transparent")
                frame_head.pack(fill="x", padx=20, pady=15)
                ctk.CTkLabel(frame_head, text=f"⚙️ Install {nombre_mod}", font=ctk.CTkFont(size=20, weight="bold")).pack(side="left")
                
                # Pestañas (Tabview)
                tabview = ctk.CTkTabview(popup, width=400, height=300, fg_color="#181818", segmented_button_selected_color="#2ECC71", segmented_button_selected_hover_color="#27AE60")
                tabview.pack(padx=20, pady=10, expand=True, fill="both")
                tabview.add("Compatible Profiles")
                tabview.add("+ Create New Pack")

                # TAB 1: Instalar en el perfil actual
                tab1 = tabview.tab("Compatible Profiles")
                v_activa = self.config_actual.get("version_activa", "1.20.4")
                m_activo = self.config_actual.get("motor_activo", "Fabric")
                
                ctk.CTkLabel(tab1, text="Current Active Profile:", text_color="gray60", font=ctk.CTkFont(weight="bold")).pack(anchor="w", padx=10, pady=(10, 0))
                card_perfil = ctk.CTkFrame(tab1, fg_color="#2A2A2A", corner_radius=8)
                card_perfil.pack(fill="x", padx=10, pady=10)
                ctk.CTkLabel(card_perfil, text=f"Minecraft {v_activa}", font=ctk.CTkFont(size=16, weight="bold")).pack(side="left", padx=15, pady=15)
                ctk.CTkLabel(card_perfil, text=f"🌙 {m_activo}", text_color="gray").pack(side="right", padx=15)

                lbl_status = ctk.CTkLabel(tab1, text="", font=ctk.CTkFont(weight="bold"))
                lbl_status.pack(pady=5)

                def ejecutar_descarga(version_obj, motor_obj):
                    lbl_status.configure(text="⏳ Downloading...", text_color="#3498DB")
                    self.update()
                    import threading
                    def _hilo_dl():
                        import requests, shutil, os
                        try:
                            loader = "fabric" if "Fabric" in motor_obj or "Paraguacraft" in motor_obj else "forge"
                            if tipo_proy in ["shader", "resourcepack", "datapack"]: params = {"game_versions": f'["{version_obj}"]'}
                            else: params = {"loaders": f'["{loader}"]', "game_versions": f'["{version_obj}"]'}

                            r_ver = requests.get(f"https://api.modrinth.com/v2/project/{project_id}/version", params=params, headers={"User-Agent": "Paraguacraft/2.0"})
                            
                            if r_ver.status_code == 200 and len(r_ver.json()) > 0:
                                archivo_data = r_ver.json()[0]["files"][0]
                                download_url, filename = archivo_data["url"], archivo_data["filename"]

                                import minecraft_launcher_lib
                                try: mine_dir = minecraft_launcher_lib.utils.get_minecraft_directory()
                                except: mine_dir = os.path.join(os.getenv('APPDATA'), ".minecraft")
                                
                                nombre_limpio_motor = motor_obj.replace(" ", "_").replace("+", "Plus")
                                folder_name = f"Paraguacraft_{version_obj}_{nombre_limpio_motor}"
                                
                                sub_folder = "shaderpacks" if tipo_proy == "shader" else "resourcepacks" if tipo_proy == "resourcepack" else "mods"
                                dest_dir = os.path.join(mine_dir, "instancias", folder_name, sub_folder)
                                os.makedirs(dest_dir, exist_ok=True)
                                filepath = os.path.join(dest_dir, filename)

                                r_dl = requests.get(download_url, stream=True, headers={"User-Agent": "Paraguacraft/2.0"})
                                if r_dl.status_code == 200:
                                    with open(filepath, "wb") as f: shutil.copyfileobj(r_dl.raw, f)
                                    self.after(0, lambda: lbl_status.configure(text="✅ Successfully Installed!", text_color="#2ECC71"))
                                    self.after(1500, popup.destroy)
                            else:
                                self.after(0, lambda: lbl_status.configure(text="❌ No compatible file found", text_color="#E74C3C"))
                        except Exception as e:
                            self.after(0, lambda: lbl_status.configure(text="❌ Error during install", text_color="#E74C3C"))
                    threading.Thread(target=_hilo_dl, daemon=True).start()

                ctk.CTkButton(tab1, text="Install to Current Profile", fg_color="#2ECC71", hover_color="#27AE60", text_color="black", font=ctk.CTkFont(weight="bold"), command=lambda: ejecutar_descarga(v_activa, m_activo)).pack(pady=10)

                # TAB 2: Crear Nuevo Perfil
                tab2 = tabview.tab("+ Create New Pack")
                ctk.CTkLabel(tab2, text="Name").pack(anchor="w", padx=10, pady=(5,0))
                entry_pack = ctk.CTkEntry(tab2, placeholder_text="My Custom Pack...", fg_color="#2A2A2A")
                entry_pack.pack(fill="x", padx=10, pady=(0, 10))

                frame_cols = ctk.CTkFrame(tab2, fg_color="transparent")
                frame_cols.pack(fill="x", padx=10)
                
                col1 = ctk.CTkFrame(frame_cols, fg_color="transparent")
                col1.pack(side="left", fill="x", expand=True, padx=(0, 5))
                ctk.CTkLabel(col1, text="Game Version").pack(anchor="w")
                combo_v = ctk.CTkOptionMenu(col1, values=["1.21.1", "1.20.4", "1.19.4", "1.8.9"], fg_color="#2A2A2A")
                combo_v.pack(fill="x")

                col2 = ctk.CTkFrame(frame_cols, fg_color="transparent")
                col2.pack(side="left", fill="x", expand=True, padx=(5, 0))
                ctk.CTkLabel(col2, text="Loader").pack(anchor="w")
                combo_l = ctk.CTkOptionMenu(col2, values=["Fabric", "Forge", "Quilt", "Neoforge"], fg_color="#2A2A2A")
                combo_l.pack(fill="x")

                def crear_y_descargar():
                    self.config_actual["version_activa"] = combo_v.get()
                    self.config_actual["motor_activo"] = combo_l.get()
                    self.guardar_config()
                    ejecutar_descarga(combo_v.get(), combo_l.get())

                ctk.CTkButton(tab2, text="Create & Install", fg_color="#2ECC71", hover_color="#27AE60", text_color="black", font=ctk.CTkFont(weight="bold"), command=crear_y_descargar).pack(pady=20)

            def crear_item_tienda(nombre, autor, desc, descargas, project_id=None, tipo_proy="mod"):
                item = ctk.CTkFrame(frame_lista_tienda, fg_color="#181818", corner_radius=10)
                item.pack(fill="x", pady=5)
                
                icono = "✨" if tipo_proy == "shader" else "📦" if tipo_proy in ["resourcepack", "modpack", "datapack"] else "🧩"
                ctk.CTkLabel(item, text=icono, font=ctk.CTkFont(size=30)).pack(side="left", padx=15, pady=15)
                info = ctk.CTkFrame(item, fg_color="transparent")
                info.pack(side="left", fill="x", expand=True)
                
                ctk.CTkLabel(info, text=nombre, font=ctk.CTkFont(size=16, weight="bold")).pack(anchor="w")
                ctk.CTkLabel(info, text=f"Por {autor}", text_color="gray", font=ctk.CTkFont(size=11)).pack(anchor="w")
                
                desc_cortada = desc[:80] + "..." if len(desc) > 80 else desc
                ctk.CTkLabel(info, text=desc_cortada, text_color="gray60", font=ctk.CTkFont(size=12)).pack(anchor="w", pady=(5,0))
                
                ctk.CTkLabel(item, text=f"⬇️ {descargas}", text_color="gray").pack(side="left", padx=20)
                
                btn_install = ctk.CTkButton(item, text="Install", fg_color="transparent", hover_color="#2A2A2A", border_width=1, border_color="#2ECC71", text_color="#2ECC71", width=80)
                btn_install.pack(side="right", padx=20)
                
                if project_id:
                    btn_install.configure(command=lambda p=project_id, n=nombre, t=tipo_proy: abrir_popup_instalacion(p, n, t))

            def mostrar_resultados(hits, tipo_proy):
                for widget in frame_lista_tienda.winfo_children(): widget.destroy()
                if not hits: ctk.CTkLabel(frame_lista_tienda, text=f"No se encontraron resultados buscando '{entry_busqueda.get()}'").pack(pady=20)
                for mod in hits:
                    crear_item_tienda(mod["title"], mod["author"], mod["description"], f"{mod.get('downloads', 0):,}", mod["project_id"], tipo_proy)

            def buscar_online():
                import threading
                import requests
                query = entry_busqueda.get()
                if not query: return
                tipo_proyecto = combo_tipo.get()
                
                for widget in frame_lista_tienda.winfo_children(): widget.destroy()
                ctk.CTkLabel(frame_lista_tienda, text=f"Buscando '{query}' ({tipo_proyecto}) en Modrinth...", font=ctk.CTkFont(slant="italic")).pack(pady=40)
                
                def _hilo_buscar():
                    try:
                        facets_str = f'[["project_type:{tipo_proyecto}"]]'
                        # Fix de parámetros en requests para evitar bugs en la URL
                        params = {'query': query, 'limit': 15, 'facets': facets_str}
                        r = requests.get("https://api.modrinth.com/v2/search", params=params, headers={"User-Agent": "Paraguacraft/2.0"})
                        if r.status_code == 200:
                            hits = r.json().get("hits", [])
                            self.after(0, lambda: mostrar_resultados(hits, tipo_proyecto))
                        else:
                            self.after(0, lambda: ctk.CTkLabel(frame_lista_tienda, text=f"Error API: {r.status_code}").pack())
                    except Exception as e:
                        self.after(0, lambda: ctk.CTkLabel(frame_lista_tienda, text=f"Error de red: {e}").pack())
                threading.Thread(target=_hilo_buscar, daemon=True).start()

            # BOTONES DIBUJADOS AL FINAL (Ya no tiran NameError)
            ctk.CTkButton(frame_top_tienda, text="🔍 Search", fg_color="#2A2A2A", hover_color="#3A3A3A", width=80, command=buscar_online).pack(side="left", padx=10)
            
            # FIX TRANSPARENCIA: Puesto en un gris re oscuro (#1E1E1E) para que no rompa el atributo
            combo_api = ctk.CTkOptionMenu(frame_top_tienda, values=["🟢 Modrinth", "🔴 CurseForge (WIP)"], width=130, fg_color="#1E1E1E", button_color="#2A2A2A")
            combo_api.pack(side="right", padx=5)

            # Ejemplos iniciales
            crear_item_tienda("Sodium", "jellysquid3", "A modern rendering engine that vastly improves frame rates.", "25,000,000", "AANobbMI", "mod")
            crear_item_tienda("Iris Shaders", "coderbot", "A modern shaders mod intended to be compatible with OptiFine.", "12,000,000", "YL57xq9U", "mod")

        elif nombre_vista == "servidores":
            self.vista_actual.grid_columnconfigure(0, weight=1)
            self.vista_actual.grid_rowconfigure(1, weight=1)

            frame_top_serv = ctk.CTkFrame(self.vista_actual, fg_color="transparent")
            frame_top_serv.grid(row=0, column=0, sticky="ew", padx=20, pady=(20, 10))
            ctk.CTkLabel(frame_top_serv, text="Multijugador", font=ctk.CTkFont(size=24, weight="bold")).pack(side="left")
            
            def crear_server_local():
                from tkinter import filedialog, messagebox
                carpeta = filedialog.askdirectory(title="Elegí una carpeta VACÍA para tu servidor")
                if carpeta: messagebox.showinfo("Playit.gg", f"Preparando servidor local en: {carpeta}\n(Lógica de CreadorServidor.py conectada)")
            
            ctk.CTkButton(frame_top_serv, text="🖥️ Crear Servidor Local (Playit.gg)", fg_color="#8E44AD", hover_color="#732D91", font=ctk.CTkFont(weight="bold"), command=crear_server_local).pack(side="right")

            frame_scroll = ctk.CTkScrollableFrame(self.vista_actual, fg_color=("gray85", "gray15"), corner_radius=10)
            frame_scroll.grid(row=1, column=0, padx=20, sticky="nsew")

            def crear_tarjeta_server(parent, logo, titulo, description, ip, jugadores):
                card = ctk.CTkFrame(parent, fg_color="#1A1A1A", corner_radius=10)
                card.pack(fill="x", pady=10, padx=10)
                ctk.CTkLabel(card, text=logo, font=ctk.CTkFont(size=30)).pack(side="left", padx=20, pady=15)
                info = ctk.CTkFrame(card, fg_color="transparent")
                info.pack(side="left")
                ctk.CTkLabel(info, text=titulo, font=ctk.CTkFont(size=16, weight="bold")).pack(anchor="w")
                ctk.CTkLabel(info, text=description, text_color="gray", font=ctk.CTkFont(size=12)).pack(anchor="w")
                ctk.CTkLabel(info, text=ip, text_color="#3498DB", font=ctk.CTkFont(size=11)).pack(anchor="w")
                ctk.CTkLabel(card, text=f"🟢 {jugadores} Online", text_color="#2ECC71", font=ctk.CTkFont(size=12, weight="bold")).pack(side="left", padx=20)
                ctk.CTkButton(card, text="Jugar", width=100, fg_color="#27AE60", hover_color="#2ECC71").pack(side="right", padx=20)

            crear_tarjeta_server(frame_scroll, "🔷", "Hypixel", "Skyblock, Bedwars & more.", "mc.hypixel.net", 78051)
            crear_tarjeta_server(frame_scroll, "🌿", "Paraguacraft", "Official Paraguay Server.", "play.paraguacraft.com", 584)
            crear_tarjeta_server(frame_scroll, "🟥", "Mineplex", "A classic Minecraft Server.", "us.mineplex.com", 2150)
            crear_tarjeta_server(frame_scroll, "🐝", "The Hive", "Mini-games and fun.", "geo.hivemc.com", 14200)
            crear_tarjeta_server(frame_scroll, "⚔️", "CubeCraft", "PvP & Parkour.", "play.cubecraft.net", 9800)

        elif nombre_vista == "skins":
            self.vista_actual.grid_columnconfigure(0, weight=1)
            self.vista_actual.grid_rowconfigure(2, weight=1)
            ctk.CTkLabel(self.vista_actual, text="Buscador de Skins Premium (Minotar API)", font=ctk.CTkFont(size=24, weight="bold")).grid(row=0, column=0, padx=20, pady=(20, 10), sticky="w")
            
            frame_buscador = ctk.CTkFrame(self.vista_actual, fg_color="transparent")
            frame_buscador.grid(row=1, column=0, padx=20, pady=10, sticky="ew")
            entry_skin = ctk.CTkEntry(frame_buscador, placeholder_text="Buscá un usuario de Minecraft (ej: Notch)...", width=300, fg_color="#1E1E1E")
            entry_skin.pack(side="left", padx=(0, 10))

            frame_grid_skins = ctk.CTkScrollableFrame(self.vista_actual, fg_color=("gray85", "gray15"), corner_radius=10)
            frame_grid_skins.grid(row=2, column=0, padx=20, sticky="nsew")
            frame_grid_skins.grid_columnconfigure((0, 1, 2, 3), weight=1)

            def crear_mini_skin_card(parent, fila, columna, username):
                card = ctk.CTkFrame(parent, fg_color="#1A1A1A", corner_radius=8)
                card.grid(row=fila, column=columna, padx=10, pady=10, sticky="nsew")
                
                lbl_img = ctk.CTkLabel(card, text="Cargando...", text_color="gray50", width=100, height=100)
                lbl_img.pack(pady=(15, 5))
                ctk.CTkLabel(card, text=username, font=ctk.CTkFont(size=12, weight="bold")).pack()
                ctk.CTkButton(card, text="Usar esta Skin", fg_color="#27AE60", hover_color="#2ECC71", height=25, width=100).pack(pady=10)

                def _bajar_skin():
                    import requests
                    from PIL import Image
                    from io import BytesIO
                    try:
                        r = requests.get(f"https://minotar.net/armor/bust/{username}/100.png", timeout=5)
                        if r.status_code == 200:
                            img_data = Image.open(BytesIO(r.content))
                            skin_img = ctk.CTkImage(light_image=img_data, dark_image=img_data, size=(80, 80))
                            self.after(0, lambda: lbl_img.configure(image=skin_img, text=""))
                            lbl_img.image = skin_img
                        else: self.after(0, lambda: lbl_img.configure(text="No existe"))
                    except: self.after(0, lambda: lbl_img.configure(text="Error"))
                import threading
                threading.Thread(target=_bajar_skin, daemon=True).start()

            def buscar_skin_real():
                query = entry_skin.get().strip()
                if query:
                    for widget in frame_grid_skins.winfo_children(): widget.destroy()
                    crear_mini_skin_card(frame_grid_skins, 0, 0, query)

            ctk.CTkButton(frame_buscador, text="🔍 Buscar", fg_color="#27AE60", hover_color="#2ECC71", command=buscar_skin_real).pack(side="left")

            # Skins populares iniciales
            crear_mini_skin_card(frame_grid_skins, 0, 0, "Notch")
            crear_mini_skin_card(frame_grid_skins, 0, 1, "Technoblade")
            crear_mini_skin_card(frame_grid_skins, 0, 2, "Dream")
            crear_mini_skin_card(frame_grid_skins, 0, 3, "DanTDM")
            crear_mini_skin_card(frame_grid_skins, 1, 0, "Grian")
            crear_mini_skin_card(frame_grid_skins, 1, 1, "MumboJumbo")

        elif nombre_vista == "cuenta":
            self.vista_actual.grid_columnconfigure(0, weight=1)
            ctk.CTkLabel(self.vista_actual, text="Perfil y Cuenta", font=ctk.CTkFont(size=24, weight="bold")).grid(row=0, column=0, padx=20, pady=(20, 30), sticky="w")
            ctk.CTkLabel(self.vista_actual, text=f"Sesión activa: {self.nombre_display}", text_color="#27AE60", font=ctk.CTkFont(size=16)).grid(row=1, column=0)

        elif nombre_vista == "discord":
            self.vista_actual.grid_columnconfigure(0, weight=1)
            ctk.CTkLabel(self.vista_actual, text="Discord RPC", font=ctk.CTkFont(size=24, weight="bold")).grid(row=0, column=0, padx=20, pady=(20, 30), sticky="w")
            ctk.CTkLabel(self.vista_actual, text="(Enlazado internamente a Discord).", text_color="gray").grid(row=1, column=0)

        elif nombre_vista == "almacenamiento":
            self.vista_actual.grid_columnconfigure(0, weight=1)
            ctk.CTkLabel(self.vista_actual, text="Almacenamiento", font=ctk.CTkFont(size=24, weight="bold")).grid(row=0, column=0, padx=20, pady=(20, 30), sticky="w")
            ctk.CTkLabel(self.vista_actual, text="(En construcción para limpieza de caché)", text_color="gray").grid(row=1, column=0)

if __name__ == "__main__":
    app = NuevoParaguacraft()
    app.mainloop()