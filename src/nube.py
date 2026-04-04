import os
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
            
            # Escribimos los archivos en la compu
            if "options_txt" in perfil:
                with open(options_path, "w", encoding="utf-8") as f:
                    f.write(perfil["options_txt"])
                    
            if "servers_dat" in perfil:
                with open(servers_path, "wb") as f:
                    f.write(perfil["servers_dat"])
                    
            print(f"[{usuario}] Datos sincronizados desde la nube.")
            return True
        except Exception as e:
            print(f"Error descargando de la nube: {e}")
            return False