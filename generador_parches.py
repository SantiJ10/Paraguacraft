import bsdiff4

viejo = "Paraguacraft_v1.0.5.exe" # El .exe viejo que ya publicaste
nuevo = "Paraguacraft_v1.0.6.exe" # El .exe nuevo que acabás de compilar
parche = "Paraguacraft_update.patch" # El nombre del parche que se va a generar

print("Comparando binarios... (Esto puede tardar un minuto)")
bsdiff4.file_diff(viejo, nuevo, parche)
print("¡Parche generado con éxito!")