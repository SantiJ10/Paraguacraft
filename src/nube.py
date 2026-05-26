import os
import re
import pymongo
from pymongo.errors import ConnectionFailure

MONGO_URI = "mongodb+srv://cluster0.y5grb.mongodb.net/?appName=Cluster0"
try:
    import credentials as _cred
    MONGO_USER = _cred.MONGO_USER
    MONGO_PASS = _cred.MONGO_PASS
except (ImportError, AttributeError):
    MONGO_USER = ""
    MONGO_PASS = ""

def _extraer_resource_packs_options(texto):
    """Devuelve lista ordenada de entradas entre comillas en resourcePacks:."""
    if not texto:
        return []
    packs, vistos = [], set()
    for linea in texto.splitlines():
        if not linea.startswith("resourcePacks:"):
            continue
        for m in re.finditer(r'"((?:[^"\\]|\\.)*)"', linea):
            tok = m.group(1)
            if tok not in vistos:
                vistos.add(tok)
                packs.append(tok)
    return packs


def _fusionar_options_txt(local_txt, cloud_txt):
    """Une resourcePacks de local + nube sin pisar selección del jugador en el juego."""
    if not cloud_txt:
        return local_txt
    if not local_txt:
        return cloud_txt
    local_lines = local_txt.splitlines(keepends=True)
    cloud_lines = cloud_txt.splitlines(keepends=True)
    local_packs = _extraer_resource_packs_options(local_txt)
    cloud_packs = _extraer_resource_packs_options(cloud_txt)
    merged = []
    vistos = set()
    for p in local_packs + cloud_packs:
        if p not in vistos:
            vistos.add(p)
            merged.append(p)
    if not merged:
        return local_txt
    out_lines = [l for l in local_lines if not l.startswith("resourcePacks:")]
    packs_json = ",".join(f'"{p}"' for p in merged)
    out_lines.append(f"resourcePacks:[{packs_json}]\n")
    return "".join(out_lines)


class GestorNube:
    def __init__(self):
        try:
            # Conexión rápida (3 segundos) para que si no hay internet, el launcher no se trabe
            self.cliente = pymongo.MongoClient(
                MONGO_URI,
                username=MONGO_USER,
                password=MONGO_PASS,
                authSource="admin",
                serverSelectionTimeoutMS=3000,
            )
            self.db = self.cliente["paraguacraft_db"]
            self.coleccion_perfiles = self.db["perfiles_usuarios"]
        except ConnectionFailure:
            self.db = None

    def subir_datos(self, usuario, options_path, servers_path):
        """Sube la configuración (opciones y servidores) del usuario a la base de datos."""
        if getattr(self, 'db', None) is None: return False
        
        datos = {"_id": usuario}
        
        # Leemos el archivo de controles y gráficos
        if os.path.exists(options_path):
            with open(options_path, "r", encoding="utf-8") as f:
                datos["options_txt"] = f.read()
                
        # Leemos el archivo de servidores multijugador (es binario, por eso "rb")
        if os.path.exists(servers_path):
            with open(servers_path, "rb") as f:
                datos["servers_dat"] = f.read()
                
        # Guardamos en la nube (si el usuario ya existe, lo pisa con lo más nuevo)
        try:
            self.coleccion_perfiles.update_one({"_id": usuario}, {"$set": datos}, upsert=True)
            print(f"[{usuario}] Datos guardados en la nube exitosamente.")
            return True
        except Exception as e:
            print(f"Error subiendo a la nube: {e}")
            return False

    def descargar_datos(self, usuario, options_path, servers_path):
        """Descarga la configuración desde la nube y la aplica antes de abrir el juego."""
        if getattr(self, 'db', None) is None: return False
        
        try:
            perfil = self.coleccion_perfiles.find_one({"_id": usuario})
            if not perfil: 
                print(f"[{usuario}] Perfil nuevo, no hay datos en la nube todavía.")
                return False 
            
            # Escribimos options fusionando con local (no pisar resource packs del juego)
            if "options_txt" in perfil:
                local_txt = ""
                if os.path.exists(options_path):
                    try:
                        with open(options_path, "r", encoding="utf-8", errors="ignore") as f:
                            local_txt = f.read()
                    except OSError:
                        pass
                merged = _fusionar_options_txt(local_txt, perfil["options_txt"])
                with open(options_path, "w", encoding="utf-8") as f:
                    f.write(merged)
                    
            if "servers_dat" in perfil:
                with open(servers_path, "wb") as f:
                    f.write(perfil["servers_dat"])
                    
            print(f"[{usuario}] Datos sincronizados desde la nube.")
            return True
        except Exception as e:
            print(f"Error descargando de la nube: {e}")
            return False