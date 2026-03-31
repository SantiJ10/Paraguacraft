import psutil
import os

class GameBooster:
    @staticmethod
    def activar_modo_gamer(callback_progreso):
        """
        Asesino de Stuttering: Baja la prioridad de procesos de fondo
        y prepara el entorno para máxima fluidez y cero tirones.
        """
        callback_progreso("Iniciando Game Booster (Limpiando procesos)...")
        
        # Lista de programas que suelen hacer saltar la CPU de fondo
        procesos_tragones = ["chrome.exe", "msedge.exe", "onedrive.exe", "discord.exe", "spotify.exe"]
        procesos_limitados = 0

        for proc in psutil.process_iter(['name']):
            try:
                nombre = proc.info['name'].lower()
                if nombre in procesos_tragones:
                    # Le sacamos prioridad en Windows para que no interrumpan al juego
                    if os.name == 'nt':
                        proc.nice(psutil.BELOW_NORMAL_PRIORITY_CLASS)
                        procesos_limitados += 1
            except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
                pass
        
        if procesos_limitados > 0:
            callback_progreso(f"Game Booster activo: {procesos_limitados} procesos limitados.")
        else:
            callback_progreso("Game Booster: El sistema ya está óptimo.")