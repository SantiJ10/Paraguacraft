f = r'c:\Users\Amin\Documents\Amin\Proyectos\Paraguacraft\web\index.html'
with open(f, 'r', encoding='utf-8') as fh:
    lines = fh.readlines()
# Lines 1305-1308 (1-indexed) = indices 1304-1307 (0-indexed)
lines[1304] = '                            <p class="font-bold text-gray-400">\u00bfC\u00f3mo usarlo?</p>\n'
lines[1305] = '                            <p>1. Hac\u00e9 clic en \u00abAbrir Overlay\u00bb</p>\n'
lines[1306] = '                            <p>2. Se abre una ventana siempre encima de Minecraft</p>\n'
lines[1307] = '                            <p>3. Cerrala con el bot\u00f3n de cierre del overlay</p>\n'
with open(f, 'w', encoding='utf-8') as fh:
    fh.writelines(lines)
print('Overlay text fixed.')
